#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PREFLIGHT_ROOT=""
PREFLIGHT_IN_PLACE=0
NPM_CACHE="${NPM_CONFIG_CACHE:-${TMPDIR:-/tmp}/qiongli-npm-cache}"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/npm_preflight.sh [options]

Options:
  --root <dir>  Repository root to check. Defaults to this script's checkout.
  --in-place    Materialize generated payloads in --root. Use only for
                explicit release maintenance.
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
    --in-place)
      PREFLIGHT_IN_PLACE=1
      shift
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

if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then
  PREFLIGHT_ROOT="$ROOT_DIR"
else
  PREFLIGHT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-npm-preflight-root.XXXXXX")"
fi

PKG_DIR="$ROOT_DIR/packages/npm-qiongli"

cd "$ROOT_DIR"
mkdir -p "$NPM_CACHE"

echo "[npm-preflight] materialize distribution payloads"
if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then
  echo "[npm-preflight] in-place materialization requires explicit --in-place"
  python3 scripts/materialize_distribution_payloads.py --target all --in-place
else
  python3 scripts/materialize_distribution_payloads.py --target all --out "$PREFLIGHT_ROOT" --force
fi

cd "$PREFLIGHT_ROOT"
PKG_DIR="$PREFLIGHT_ROOT/packages/npm-qiongli"

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
