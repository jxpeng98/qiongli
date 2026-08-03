#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Prepare the accepted macOS App's isolated manual home for live host acceptance.

Usage:
  pnpm desktop:macos:acceptance:host-prepare
  pnpm desktop:macos:acceptance:host-prepare -- --acceptance-root /absolute/path

This command does not rebuild the App and never uses the real user home. It
validates the accepted product receipt and binary, installs both managed Plugin
sources and registrations into the accepted App's manual-home, then prepares
the same three-project continuity fixture there. It does not authenticate or
activate either model Host.
EOF
}

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
acceptance_root="$repo_root/dist/macos-acceptance/current"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --acceptance-root)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      acceptance_root="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$(/usr/bin/uname -s)" != "Darwin" ]]; then
  printf 'Host acceptance preparation can only run on macOS.\n' >&2
  exit 1
fi
if [[ "$acceptance_root" != /* ]]; then
  printf 'The acceptance root must be absolute.\n' >&2
  exit 1
fi
if [[ -n "$(git -C "$repo_root" status --short)" ]]; then
  printf 'Commit the host-preparation implementation before producing formal evidence.\n' >&2
  exit 1
fi

cargo run \
  --locked \
  --manifest-path "$repo_root/packages/qiongli-native/Cargo.toml" \
  --package qiongli \
  --example native_packaged_product_acceptance \
  -- \
  --prepare-host-only \
  --acceptance-root "$acceptance_root"

canonical_cli="$acceptance_root/extracted/Qiongli.app/Contents/MacOS/qiongli-cli"
manual_home="$acceptance_root/manual-home"
codex_registration="$manual_home/.qiongli/plugins/codex/.qiongli-next-codex-registration.json"
claude_registration="$manual_home/.qiongli/v2/integrations/claude-code/.qiongli-next-claude-registration.json"
integration_plan="$(/usr/bin/mktemp "${TMPDIR:-/tmp}/qiongli-c5-integrations.XXXXXX")"
integration_snapshot="$(/usr/bin/mktemp "${TMPDIR:-/tmp}/qiongli-c5-integration-snapshot.XXXXXX")"
cleanup() {
  /bin/rm -f "$integration_plan" "$integration_snapshot"
}
trap cleanup EXIT

if [[ ! -x "$canonical_cli" ]]; then
  printf 'The accepted Qiongli CLI is missing or not executable.\n' >&2
  exit 1
fi

/bin/mkdir -p "$manual_home/.codex" "$manual_home/.claude"
isolated_env=(
  /usr/bin/env
  "HOME=$manual_home"
  "QIONGLI_CONFIG_HOME=$manual_home/.config/qiongli"
  "CODEX_HOME=$manual_home/.codex"
  "CLAUDE_CONFIG_DIR=$manual_home/.claude"
)

if [[ ! -f "$codex_registration" || ! -f "$claude_registration" ]]; then
  "${isolated_env[@]}" "$canonical_cli" app plan integrations-install --target all \
    > "$integration_plan"
  plan_digest="$(/usr/bin/plutil -extract plan_digest_sha256 raw -o - "$integration_plan")"
  "${isolated_env[@]}" "$canonical_cli" app apply \
    --plan "$integration_plan" \
    --expected-plan-digest "$plan_digest" \
    --approve-filesystem-write \
    --approve-client-config-change \
    --approve-host-trust
fi

"${isolated_env[@]}" "$canonical_cli" app verify-integrations --target all \
  > "$integration_snapshot"
for target_index in 0 1; do
  source_state="$(/usr/bin/plutil -extract "snapshot.integrations.$target_index.managedContent.source" raw -o - "$integration_snapshot")"
  registration_state="$(/usr/bin/plutil -extract "snapshot.integrations.$target_index.managedContent.registration" raw -o - "$integration_snapshot")"
  if [[ "$source_state" != "ready" || "$registration_state" != "ready" ]]; then
    printf 'The isolated managed integration is not current.\n' >&2
    exit 1
  fi
done

printf '\nPrepared isolated host fixture:\n  %s\n' \
  "$acceptance_root/qiongli-packaged-host-fixture.receipt.json"
printf 'Isolated home (install and fixture proof only; do not authenticate):\n  %s\n' \
  "$acceptance_root/manual-home"
printf 'Isolated Codex and Claude Code registrations: ready (Hosts not activated)\n'
