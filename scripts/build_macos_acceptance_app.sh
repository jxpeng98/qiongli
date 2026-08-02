#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Build a local-installable macOS Qiongli acceptance App.

Usage:
  pnpm desktop:macos:acceptance -- [--open] [--diagnostics]

Options:
  --open        Launch the accepted App inside its isolated test home.
  --diagnostics Show bounded isolated child output when acceptance fails.
  -h, --help    Show this help.

The generated App has ephemeral development-only product control, is ad-hoc
signed, and writes client integrations only inside dist/macos-acceptance/current/
automated-home and control-plane-automated-home. The latter also proves the
schema-2 installed CLI authority plus digest-bound plugin and standalone Skills
plans. Manual UI testing opens in a separate clean manual-home. It is
non-publishing test evidence and must not be distributed.
Its install grants expire one hour after the build starts; rebuild when expired.
EOF
}

open_after_build="false"
diagnostics="false"
for argument in "$@"; do
  case "$argument" in
    --)
      ;;
    --open)
      open_after_build="true"
      ;;
    --diagnostics)
      diagnostics="true"
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
signing_script="$repo_root/tooling/scripts/macos_native_sign_notarize.sh"
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
  printf 'The product-controlled acceptance App requires a clean Git worktree.\n' >&2
  printf 'Commit the intended source first so its displayed build identity is exact.\n' >&2
  printf 'Use pnpm desktop:macos:open for an uncommitted source build.\n' >&2
  exit 1
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
acceptance_command=(
  cargo run
  --manifest-path "$native_manifest"
  --package qiongli
  --example native_packaged_product_acceptance
  --locked
  --
  --output "$stage_root"
  --source-commit "$source_commit"
  --signing-script "$signing_script"
)
if [[ "$diagnostics" == "true" ]]; then
  QIONGLI_ACCEPTANCE_DIAGNOSTICS=1 "${acceptance_command[@]}"
else
  "${acceptance_command[@]}"
fi

receipt="$stage_root/qiongli-packaged-product-acceptance.receipt.json"
app="$stage_root/extracted/Qiongli.app"
automated_home="$stage_root/automated-home"
control_plane_home="$stage_root/control-plane-automated-home"
manual_home="$stage_root/manual-home"
launcher="$app/Contents/MacOS/Qiongli"
if [[ ! -f "$receipt" || ! -x "$launcher" || ! -d "$automated_home" \
  || ! -d "$control_plane_home" || ! -d "$manual_home" ]]; then
  printf 'The acceptance build did not produce the expected receipt, App, and isolated homes.\n' >&2
  exit 1
fi
if [[ "$(/usr/bin/plutil -extract schema_version raw -expect integer "$receipt")" != "3" \
  || "$(/usr/bin/plutil -extract status raw -expect string "$receipt")" != \
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
  zotero_companion_artifact_bound \
  inventory_discovered \
  skills_materialize_verify_refresh \
  lite_mcp_self_test \
  project_three_project_restart \
  project_app_cli_library_full_mcp_parity \
  project_artifact_internal_projection \
  continuity_delivery_restart_replay \
  continuity_assignment_resolution \
  continuity_archive_restore_rebuild \
  continuity_catalog_query_timeline \
  continuity_path_redacted \
  provider_keychain_save_replace_restart_remove \
  cli_schema3_app_authority \
  managed_operation_plan_apply \
  standalone_skills_all_targets \
  cli_plugin_reconcile_remove \
  codex_install_verify_remove \
  claude_install_verify_remove \
  registration_repair \
  packaged_restart_verification \
  legacy_migration_fixture_isolated \
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
automated_home="$accepted_root/automated-home"
control_plane_home="$accepted_root/control-plane-automated-home"
home="$accepted_root/manual-home"
log="$accepted_root/qiongli-acceptance-app.log"
zotero_acceptance_receipt="$accepted_root/qiongli-r5d-zotero-acceptance.receipt.json"

printf 'Running R5D Zotero automated acceptance against the accepted App...\n'
zotero_acceptance_command=(
  node
  "$repo_root/scripts/r5d_zotero_acceptance.mjs"
  --app "$app"
  --receipt "$zotero_acceptance_receipt"
)
if [[ "$diagnostics" == "true" ]]; then
  QIONGLI_ACCEPTANCE_DIAGNOSTICS=1 "${zotero_acceptance_command[@]}"
else
  "${zotero_acceptance_command[@]}"
fi
if [[ "$(/usr/bin/plutil -extract status raw -expect string "$zotero_acceptance_receipt")" != \
  "accepted-automated-nonpublishing" \
  || "$(/usr/bin/plutil -extract publicationAllowed raw -expect bool "$zotero_acceptance_receipt")" != \
  "false" ]]; then
  printf 'The R5D Zotero automated acceptance receipt is invalid.\n' >&2
  exit 1
fi

printf '\nBuilt and accepted local-installable macOS App:\n  %s\n' "$app"
printf 'Automated lifecycle home:\n  %s\n' "$automated_home"
printf 'CLI control-plane lifecycle home:\n  %s\n' "$control_plane_home"
printf 'Clean manual UI home:\n  %s\n' "$home"
printf 'Acceptance receipt:\n  %s\n' "$accepted_root/qiongli-packaged-product-acceptance.receipt.json"
printf 'Zotero automated acceptance receipt:\n  %s\n' "$zotero_acceptance_receipt"
printf 'Manual Zotero gate identifiers:\n  pnpm acceptance:zotero:manual-record -- --list-gates\n'
printf 'Manual R5F gate identifiers:\n  pnpm acceptance:r5f:manual-record -- --list-gates\n'
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
