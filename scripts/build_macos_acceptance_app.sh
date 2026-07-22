#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Build a local-installable macOS Qiongli acceptance App.

Usage:
  pnpm desktop:macos:acceptance [-- --open]

Options:
  --open     Launch the accepted App inside its isolated test home.
  -h, --help Show this help.

The generated App has ephemeral development-only product control, is ad-hoc
signed, and writes client integrations only inside dist/macos-acceptance/current/
isolated-home. It is non-publishing test evidence and must not be distributed.
Its install grants expire one hour after the build starts; rebuild when expired.
EOF
}

open_after_build="false"
for argument in "$@"; do
  case "$argument" in
    --open)
      open_after_build="true"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$argument" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$(/usr/bin/uname -s)" != "Darwin" ]]; then
  printf 'desktop:macos:acceptance can only build on macOS.\n' >&2
  exit 1
fi

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
native_manifest="$repo_root/packages/qiongli-native/Cargo.toml"
signing_script="$repo_root/tooling/scripts/macos_alpha1_sign_notarize.sh"
frontend_root="$repo_root/packages/qiongli-desktop"
vite_entry="$frontend_root/node_modules/vite/bin/vite.js"
output_parent="$repo_root/dist/macos-acceptance"
accepted_root="$output_parent/current"

if [[ ! -f "$vite_entry" ]]; then
  printf 'Desktop dependencies are missing; run pnpm install --frozen-lockfile first.\n' >&2
  exit 1
fi

source_commit="$(git -C "$repo_root" rev-parse --verify HEAD)"
if [[ ! "$source_commit" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'Could not resolve the current Git commit.\n' >&2
  exit 1
fi
if [[ -n "$(git -C "$repo_root" status --short)" ]]; then
  printf 'Note: building a non-publishing acceptance App from a dirty worktree.\n'
  printf 'The embedded development authority is still bound to HEAD %s.\n\n' "$source_commit"
fi

mkdir -p "$output_parent"
# The release composer deliberately rejects outputs inside the Git checkout.
# Complete the whole non-publishing acceptance in a private system temporary
# directory, then publish the accepted result into the ignored dist directory.
stage_parent="$(/usr/bin/mktemp -d -t Qiongli.acceptance)"
# macOS reports the temporary root through /var, which is a system symlink to
# /private/var. The composer rejects symlink ancestors, so pass its physical
# owner-private path instead.
stage_parent="$(CDPATH= cd -- "$stage_parent" && pwd -P)"
stage_root="$stage_parent/product"
cleanup() {
  rm -rf -- "$stage_parent"
}
trap cleanup EXIT

printf 'Building the embedded Svelte desktop assets...\n'
(
  cd "$frontend_root"
  node "$vite_entry" build
)

printf 'Building and exercising the isolated product-controlled App...\n'
cargo run \
  --manifest-path "$native_manifest" \
  --package qiongli \
  --example native_packaged_product_acceptance \
  --locked \
  -- \
  --output "$stage_root" \
  --source-commit "$source_commit" \
  --signing-script "$signing_script"

receipt="$stage_root/qiongli-packaged-product-acceptance.receipt.json"
app="$stage_root/extracted/Qiongli.app"
home="$stage_root/isolated-home"
launcher="$app/Contents/MacOS/Qiongli"
if [[ ! -f "$receipt" || ! -x "$launcher" || ! -d "$home" ]]; then
  printf 'The acceptance build did not produce the expected receipt, App, and test home.\n' >&2
  exit 1
fi
if [[ "$(/usr/bin/plutil -extract status raw -expect string "$receipt")" != \
  "accepted-ad-hoc-nonpublishing" \
  || "$(/usr/bin/plutil -extract publication_allowed raw -expect bool "$receipt")" != \
  "false" ]]; then
  printf 'The generated package did not pass non-publishing acceptance.\n' >&2
  exit 1
fi

for check in \
  embedded_authority \
  canonical_signature_preserved \
  product_control_verified \
  skills_materialize_verify_refresh \
  codex_install_verify_remove \
  claude_install_verify_remove \
  registration_repair \
  packaged_restart_verification \
  legacy_content_preserved \
  empty_path_startup; do
  if [[ "$(/usr/bin/plutil -extract "checks.$check" raw -expect bool "$receipt")" != \
    "true" ]]; then
    printf 'Acceptance check failed: %s\n' "$check" >&2
    exit 1
  fi
done

previous="$output_parent/.previous.$$"
if [[ -e "$accepted_root" || -L "$accepted_root" ]]; then
  mv "$accepted_root" "$previous"
fi
if mv "$stage_root" "$accepted_root"; then
  rm -rf -- "$previous"
else
  if [[ -e "$previous" || -L "$previous" ]]; then
    mv "$previous" "$accepted_root"
  fi
  exit 1
fi

app="$accepted_root/extracted/Qiongli.app"
home="$accepted_root/isolated-home"
log="$accepted_root/qiongli-acceptance-app.log"

printf '\nBuilt and accepted local-installable macOS App:\n  %s\n' "$app"
printf 'Isolated test home:\n  %s\n' "$home"
printf 'Acceptance receipt:\n  %s\n' "$accepted_root/qiongli-packaged-product-acceptance.receipt.json"
printf 'Authority: ephemeral development-only; signing: ad-hoc; publishing: forbidden\n'
printf 'Rebuild after one hour to refresh the temporary install grants.\n'

if [[ "$open_after_build" == "true" ]]; then
  printf 'Launching with HOME isolated from your real Codex and Claude configuration...\n'
  /usr/bin/touch "$log"
  /bin/chmod 600 "$log"
  /usr/bin/open \
    --new \
    --fresh \
    --env "HOME=$home" \
    --env "CODEX_HOME=$home/.codex" \
    --env "CLAUDE_CONFIG_DIR=$home/.claude" \
    --stdout "$log" \
    --stderr "$log" \
    "$app"
  printf 'App log:\n  %s\n' "$log"
fi
