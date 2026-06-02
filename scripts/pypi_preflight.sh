#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUN_BUILD=1
RUN_INSTALL_SMOKE=1
KEEP_DIST=0

require_python_module() {
  local module="$1"
  local package="$2"

  if python3 -c "import ${module}" >/dev/null 2>&1; then
    return 0
  fi

  echo "[pypi-preflight] missing Python dependency: ${package} (module: ${module})" >&2
  echo "[pypi-preflight] install release dependencies before publishing, for example:" >&2
  echo "  python3 -m pip install -e . build twine" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/pypi_preflight.sh [options]

Description:
  Pre-publish checks for PyPI/TestPyPI release safety.

Checks:
  1) Clean dist/ (optional)
  2) Build sdist + wheel
  3) twine metadata validation
  4) Install latest wheel in a temporary virtualenv
  5) CLI smoke checks (qiongli / ql / research-skills / rsk / rsw)

Options:
  --root <dir>      Repository root to check. Defaults to this script's checkout.
  --no-build         Skip build step (expects artifacts in dist/)
  --no-install-smoke Skip temporary venv install + CLI smoke checks
  --keep-dist        Do not delete existing dist/ before build
  -h, --help         Show this message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { echo "[pypi-preflight] missing value for --root" >&2; exit 2; }
      ROOT_DIR="$(cd "$2" && pwd)"
      shift 2
      ;;
    --no-build)
      RUN_BUILD=0
      shift
      ;;
    --no-install-smoke)
      RUN_INSTALL_SMOKE=0
      shift
      ;;
    --keep-dist)
      KEEP_DIST=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[pypi-preflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"

if [[ "$RUN_BUILD" -eq 1 ]]; then
  require_python_module build build
fi
require_python_module twine twine

if [[ "$RUN_BUILD" -eq 1 ]]; then
  if [[ "$KEEP_DIST" -eq 0 ]]; then
    echo "[pypi-preflight] cleaning dist/"
    rm -rf dist
  fi

  echo "[pypi-preflight] materialize distribution payloads"
  python3 scripts/materialize_distribution_payloads.py --target all --in-place

  echo "[pypi-preflight] building package"
  python3 -m build
fi

echo "[pypi-preflight] checking package metadata"
python3 -m twine check dist/*

if [[ "$RUN_INSTALL_SMOKE" -eq 1 ]]; then
  shopt -s nullglob
  wheels=(dist/*.whl)
  shopt -u nullglob
  if [[ ${#wheels[@]} -eq 0 ]]; then
    echo "[pypi-preflight] no wheel found under dist/" >&2
    exit 1
  fi

  latest_wheel="$(ls -1t dist/*.whl | head -n1)"
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-pypi-preflight.XXXXXX")"
  trap 'rm -rf "$tmp_dir"' EXIT

  echo "[pypi-preflight] creating temp venv: $tmp_dir"
  python3 -m venv "$tmp_dir/venv"
  venv_python="$tmp_dir/venv/bin/python"
  venv_qiongli="$tmp_dir/venv/bin/qiongli"
  venv_ql="$tmp_dir/venv/bin/ql"
  venv_research_skills="$tmp_dir/venv/bin/research-skills"
  venv_rsk="$tmp_dir/venv/bin/rsk"
  venv_rsw="$tmp_dir/venv/bin/rsw"

  "$venv_python" -m pip install --upgrade pip >/dev/null
  "$venv_python" -m pip install --ignore-installed --force-reinstall "$latest_wheel"

  echo "[pypi-preflight] smoke: qiongli --help"
  "$venv_qiongli" --help >/dev/null

  echo "[pypi-preflight] smoke: ql --help"
  "$venv_ql" --help >/dev/null

  echo "[pypi-preflight] smoke: research-skills --help"
  "$venv_research_skills" --help >/dev/null

  echo "[pypi-preflight] smoke: rsk --help"
  "$venv_rsk" --help >/dev/null

  echo "[pypi-preflight] smoke: rsw --help"
  "$venv_rsw" --help >/dev/null

  echo "[pypi-preflight] smoke: subcommand help"
  "$venv_rsk" check --help >/dev/null
  "$venv_rsk" upgrade --help >/dev/null
fi

echo "[pypi-preflight] all checks passed"
echo "[pypi-preflight] package preflight completed; publish mode owns tag/release flow"
