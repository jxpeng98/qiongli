#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PKG_DIR="$ROOT_DIR/packages/npm-qiongli"
NPM_CACHE="${NPM_CONFIG_CACHE:-${TMPDIR:-/tmp}/qiongli-npm-cache}"

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
