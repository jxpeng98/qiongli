#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Prepare the accepted macOS App's isolated manual home for live host acceptance.

Usage:
  pnpm desktop:macos:acceptance:host-prepare
  pnpm desktop:macos:acceptance:host-prepare -- --acceptance-root /absolute/path

This command does not rebuild the App and never uses the real user home. It
validates the accepted product receipt and binary, then prepares the same
three-project continuity fixture inside the accepted App's manual-home.
The isolated home proves installation and fixture state only; do not log a
model Host into it.
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

printf '\nPrepared isolated host fixture:\n  %s\n' \
  "$acceptance_root/qiongli-packaged-host-fixture.receipt.json"
printf 'Isolated home (install and fixture proof only; do not authenticate):\n  %s\n' \
  "$acceptance_root/manual-home"
