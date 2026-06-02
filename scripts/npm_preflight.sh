#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
NPM_CACHE="${NPM_CONFIG_CACHE:-${TMPDIR:-/tmp}/qiongli-npm-cache}"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/npm_preflight.sh [options]

Options:
  --root <dir>  Repository root to check. Defaults to this script's checkout.
  -h, --help    Show this message.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { echo "[npm-preflight] missing value for --root" >&2; exit 2; }
      ROOT_DIR="$(cd "$2" && pwd)"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[npm-preflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

PKG_DIR="$ROOT_DIR/packages/npm-qiongli"

cd "$ROOT_DIR"
mkdir -p "$NPM_CACHE"

echo "[npm-preflight] materialize distribution payloads"
python3 scripts/materialize_distribution_payloads.py --target all --in-place

echo "[npm-preflight] running node tests"
NPM_CONFIG_CACHE="$NPM_CACHE" npm --prefix "$PKG_DIR" test

echo "[npm-preflight] checking npm pack"
(
  cd "$PKG_DIR"
  NPM_CONFIG_CACHE="$NPM_CACHE" npm pack --dry-run
) >/tmp/qiongli-npm-pack.txt
cat /tmp/qiongli-npm-pack.txt

echo "[npm-preflight] install dry-run"
node "$PKG_DIR/bin/qiongli.mjs" install --target all --dry-run

echo "[npm-preflight] check json"
node "$PKG_DIR/bin/qiongli.mjs" check --json >/tmp/qiongli-npm-check.json
python3 -m json.tool /tmp/qiongli-npm-check.json >/dev/null

echo "[npm-preflight] runtime doctor"
node "$PKG_DIR/bin/qiongli.mjs" runtime doctor

echo "[npm-preflight] all checks passed"
