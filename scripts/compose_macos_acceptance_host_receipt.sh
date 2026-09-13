#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Compose a canonical package-bound C5 receipt from one live-host observation.

Usage:
  bash scripts/compose_macos_acceptance_host_receipt.sh \
    --observation /absolute/path/to/observation.json \
    --system-registration /absolute/path/to/qiongli-next-registration.json

Options:
  --observation PATH          Canonical path-redacted live-host observation.
  --system-registration PATH  Current 2.x registration used by the live system Host.
  --acceptance-root PATH      Accepted product root (defaults to dist current).
  --fixture PATH              Fixed C5 fixture path.
  -h, --help                  Show this help.

The observation must conform to
tooling/release/acceptance/fixtures/r5c-c5-host-observation.schema.json.
The command binds the isolated installation to the current system registration.
It does not read or copy Host authentication state.
It never accepts credentials, prompts, responses, conversations, project IDs,
paths, or tool bodies in the observation.
EOF
}

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
acceptance_root="$repo_root/dist/macos-acceptance/current"
fixture="$repo_root/tooling/release/acceptance/fixtures/r5c-c5-host-driven-v1.json"
observation=""
system_registration=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --observation)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      observation="$2"
      shift 2
      ;;
    --system-registration)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      system_registration="$2"
      shift 2
      ;;
    --acceptance-root)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      acceptance_root="$2"
      shift 2
      ;;
    --fixture)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      fixture="$2"
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

if [[ -z "$observation" ]]; then
  printf 'A live-host observation is required.\n\n' >&2
  usage >&2
  exit 2
fi
if [[ -z "$system_registration" ]]; then
  printf 'The live Host system registration is required.\n\n' >&2
  usage >&2
  exit 2
fi
if [[ -n "$(git -C "$repo_root" status --short)" ]]; then
  printf 'Commit the receipt composer before producing formal evidence.\n' >&2
  exit 1
fi

cargo run \
  --locked \
  --manifest-path "$repo_root/packages/qiongli-native/Cargo.toml" \
  --package qiongli \
  --example native_host_acceptance_contract \
  -- \
  compose-packaged-receipt \
  "$fixture" \
  "$acceptance_root" \
  "$observation" \
  "$system_registration"

printf '\nThe canonical host receipt is stored inside the accepted product root.\n'
