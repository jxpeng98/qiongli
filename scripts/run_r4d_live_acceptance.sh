#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
acceptance_script="$script_dir/r4d_live_acceptance.mjs"

if [[ "${1:-}" == "--" ]]; then
  shift
fi

for argument in "$@"; do
  case "$argument" in
    -h|--help)
      exec node "$acceptance_script" --help
      ;;
  esac
done

if [[ "$(/usr/bin/uname -s)" != "Darwin" ]]; then
  printf 'desktop:macos:r4d-acceptance can only run on macOS.\n' >&2
  exit 1
fi

bash "$script_dir/build_macos_app.sh"
exec node "$acceptance_script" "$@"
