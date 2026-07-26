#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Validate one live Codex or Claude Code receipt against the accepted macOS App.

Usage:
  pnpm acceptance:host:c5:receipt -- --receipt /absolute/path/to/receipt.json

Options:
  --receipt PATH          Canonical live-host receipt (required).
  --acceptance-root PATH  Accepted product root (defaults to dist current).
  --fixture PATH          Fixed host fixture (defaults to the R5C C5 fixture).
  -h, --help              Show this help.

The validator checks the fixture, accepted product receipt, exact App binary,
prepared isolated project revision, and host-specific installed plugin digest.
Its output is path-redacted and remains non-publishing evidence.
EOF
}

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
acceptance_root="$repo_root/dist/macos-acceptance/current"
fixture="$repo_root/tooling/release/acceptance/fixtures/r5c-c5-host-driven-v1.json"
receipt=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --receipt)
      if [[ $# -lt 2 ]]; then
        usage >&2
        exit 2
      fi
      receipt="$2"
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

if [[ -z "$receipt" ]]; then
  printf 'A live-host receipt is required.\n\n' >&2
  usage >&2
  exit 2
fi
if [[ -n "$(git -C "$repo_root" status --short)" ]]; then
  printf 'Commit the validator implementation before producing formal evidence.\n' >&2
  exit 1
fi

cargo run \
  --locked \
  --manifest-path "$repo_root/packages/qiongli-native/Cargo.toml" \
  --package qiongli \
  --example native_host_acceptance_contract \
  -- \
  packaged-receipt \
  "$fixture" \
  "$acceptance_root" \
  "$receipt"
