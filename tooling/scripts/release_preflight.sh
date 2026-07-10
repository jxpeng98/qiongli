#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PREFLIGHT_ROOT="$ROOT_DIR"
QUICK_MODE=0
RUN_SMOKE=1
RUN_UNIT_TESTS=1
RUN_CONTROLLER_EVALS=1
MAINTAINER_SMOKE=0
STRICT_MODE=1
TAG=""
SKIP_NOTE_GEN=0
NOTE_OVERWRITE=0
FROM_TAG=""
MATERIALIZE_OUT=""
MATERIALIZE_IN_PLACE=0
FAILED_STAGE=""
FAILED_LOG=""
FAILED_STATUS=""

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

require_python_module() {
  local module="$1"
  local package="$2"

  if python3 -c "import ${module}" >/dev/null 2>&1; then
    return 0
  fi

  echo "[preflight] missing Python dependency: ${package} (module: ${module})" >&2
  echo "[preflight] install release dependencies before publishing, for example:" >&2
  echo "  python3 -m pip install -e ." >&2
  exit 1
}

run_logged_stage() {
  local label="$1"
  local log_file="$2"
  local -a statuses
  shift 2

  echo "[preflight] ${label}"
  set +e
  "$@" 2>&1 | tee "$log_file"
  statuses=("${PIPESTATUS[@]}")
  set -e

  local command_status="${statuses[0]}"
  local tee_status="${statuses[1]}"
  if [[ "$tee_status" -ne 0 ]]; then
    echo "[preflight] FAIL: ${label} log capture failed with exit code ${tee_status}" >&2
    exit "$tee_status"
  fi
  if [[ "$command_status" -ne 0 ]]; then
    FAILED_STAGE="$label"
    FAILED_LOG="$log_file"
    FAILED_STATUS="$command_status"
    echo "[preflight] FAIL: ${label} failed with exit code ${command_status}" >&2
    echo "[preflight] log: ${log_file}" >&2
    exit "$command_status"
  fi
}

run_warning_stage() {
  local label="$1"
  local log_file="$2"
  local -a statuses
  shift 2

  echo "[preflight] ${label}"
  set +e
  "$@" 2>&1 | tee "$log_file"
  statuses=("${PIPESTATUS[@]}")
  set -e

  local command_status="${statuses[0]}"
  local tee_status="${statuses[1]}"
  if [[ "$tee_status" -ne 0 ]]; then
    echo "[preflight] WARN: ${label} log capture failed with exit code ${tee_status}" >&2
    return 0
  fi
  if [[ "$command_status" -ne 0 ]]; then
    echo "[preflight] WARN: ${label} failed with exit code ${command_status}" >&2
    echo "[preflight] warning log: ${log_file}" >&2
  fi
}

cleanup_logs() {
  local status="$?"
  if [[ "$status" -eq 0 ]]; then
    rm -f "$validator_log" "$unit_log" "$smoke_log" "$eval_log" \
      "$rust_fmt_log" "$rust_clippy_log" "$rust_test_log" "$rust_build_log"
    rm -rf "$rust_artifact_dir"
  else
    echo "[preflight] retained logs for failed run:" >&2
    echo "  Rust fmt: $rust_fmt_log" >&2
    echo "  Rust clippy: $rust_clippy_log" >&2
    echo "  Rust tests: $rust_test_log" >&2
    echo "  Rust release build: $rust_build_log" >&2
    echo "  Rust current-host artifact: $rust_artifact_dir" >&2
    echo "  validator: $validator_log" >&2
    echo "  unit tests: $unit_log" >&2
    echo "  smoke: $smoke_log" >&2
    echo "  controller-mode evals: $eval_log" >&2
    if [[ -n "$FAILED_LOG" && -f "$FAILED_LOG" ]]; then
      echo "[preflight] failure summary: ${FAILED_STAGE} exited with ${FAILED_STATUS}" >&2
      tail -n 120 "$FAILED_LOG" >&2
    fi
  fi
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_preflight.sh [--tag <tag>] [options]

Description:
  Run standardized pre-release gates:
    0) prerelease: auto-generate tooling/release/<tag>.md draft
       stable: verify matching CHANGELOG.md section exists
    1) Rust Lite fmt, clippy, tests, and current-host release build
    2) strict standard validator
    3) repository unit tests
    4) release smoke tier (literature pipeline + doctor)
    5) optional maintainer smoke tier (parallel + task-run profile paths)

Options:
  --tag <tag>     Optional release tag to pre-check. If provided, script verifies
                  the tag does not already exist locally.
  --from-tag <t>  Optional baseline tag passed to prerelease note generator.
  --skip-note-gen Skip auto generation of tooling/release/<tag>.md draft for prerelease tags.
  --note-overwrite  Overwrite tooling/release/<tag>.md when auto-generating prerelease draft.
  --skip-smoke    Skip smoke test stage.
  --skip-unit-tests  Skip repository unit tests.
  --skip-controller-evals  Skip controller-mode eval warning stage.
  --quick         Run the CI gate: Rust Lite gates + validator + package checks,
                  while skipping the broad Python unit, smoke, and controller-eval stages.
  --materialize-out <dir>  Materialize generated payloads into a staging
                  directory and run package validation against that tree.
  --in-place      Materialize generated payloads in the source checkout.
                  This is for explicit release maintenance only.
  --maintainer-smoke  Run maintainer smoke tier instead of release smoke tier.
  --no-strict     Run validator without --strict.
  -h, --help      Show this message.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[preflight] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --from-tag)
      [[ $# -ge 2 ]] || { echo "[preflight] missing value for --from-tag" >&2; exit 2; }
      FROM_TAG="$2"
      shift 2
      ;;
    --skip-note-gen)
      SKIP_NOTE_GEN=1
      shift
      ;;
    --note-overwrite)
      NOTE_OVERWRITE=1
      shift
      ;;
    --skip-smoke)
      RUN_SMOKE=0
      shift
      ;;
    --skip-unit-tests)
      RUN_UNIT_TESTS=0
      shift
      ;;
    --skip-controller-evals)
      RUN_CONTROLLER_EVALS=0
      shift
      ;;
    --quick)
      QUICK_MODE=1
      RUN_SMOKE=0
      RUN_UNIT_TESTS=0
      RUN_CONTROLLER_EVALS=0
      shift
      ;;
    --materialize-out)
      [[ $# -ge 2 ]] || { echo "[preflight] missing value for --materialize-out" >&2; exit 2; }
      MATERIALIZE_OUT="$2"
      shift 2
      ;;
    --in-place)
      MATERIALIZE_IN_PLACE=1
      shift
      ;;
    --maintainer-smoke)
      MAINTAINER_SMOKE=1
      shift
      ;;
    --no-strict)
      STRICT_MODE=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[preflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"

if [[ "$MATERIALIZE_IN_PLACE" -eq 1 && -n "$MATERIALIZE_OUT" ]]; then
  echo "[preflight] --in-place and --materialize-out are mutually exclusive" >&2
  exit 2
fi

require_python_module yaml PyYAML

if [[ "$QUICK_MODE" -eq 1 ]]; then
  echo "[preflight] quick CI gate enabled"
fi

if [[ -n "$TAG" ]]; then
  if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "[preflight] tag already exists locally: $TAG" >&2
    exit 1
  fi
  echo "[preflight] tag pre-check passed: $TAG is available"

  if is_prerelease_tag "$TAG"; then
    if [[ "$SKIP_NOTE_GEN" -eq 0 ]]; then
      note_cmd=(./scripts/generate_release_notes.sh --tag "$TAG")
      if [[ -n "$FROM_TAG" ]]; then
        note_cmd+=(--from-tag "$FROM_TAG")
      fi
      if [[ "$NOTE_OVERWRITE" -eq 1 ]]; then
        note_cmd+=(--overwrite)
      fi
      echo "[preflight] prerelease note draft"
      "${note_cmd[@]}"
    else
      echo "[preflight] prerelease note generation skipped"
    fi

    if [[ ! -f "tooling/release/${TAG}.md" ]]; then
      echo "[preflight] missing prerelease notes file: tooling/release/${TAG}.md" >&2
      exit 1
    fi
  else
    echo "[preflight] validating stable release changelog entry"
    python3 scripts/changelog_section.py --version "${TAG#v}" --check
  fi
fi

validator_log="$(mktemp -t qiongli-validator.XXXXXX.log)"
unit_log="$(mktemp -t qiongli-unittest.XXXXXX.log)"
smoke_log="$(mktemp -t qiongli-smoke.XXXXXX.log)"
eval_log="$(mktemp -t qiongli-controller-evals.XXXXXX.log)"
rust_fmt_log="$(mktemp -t qiongli-rust-fmt.XXXXXX.log)"
rust_clippy_log="$(mktemp -t qiongli-rust-clippy.XXXXXX.log)"
rust_test_log="$(mktemp -t qiongli-rust-test.XXXXXX.log)"
rust_build_log="$(mktemp -t qiongli-rust-build.XXXXXX.log)"
rust_artifact_dir="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-rust-lite-artifact.XXXXXX")"
trap cleanup_logs EXIT

run_logged_stage \
  "Rust Lite MCP format" \
  "$rust_fmt_log" \
  cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml -- --check
run_logged_stage \
  "Rust Lite MCP clippy" \
  "$rust_clippy_log" \
  cargo clippy --locked --manifest-path packages/qiongli-lite-mcp/Cargo.toml --all-targets -- -D warnings
run_logged_stage \
  "Rust Lite MCP tests" \
  "$rust_test_log" \
  cargo test --locked --manifest-path packages/qiongli-lite-mcp/Cargo.toml --all-targets
run_logged_stage \
  "Rust Lite MCP current-host release build" \
  "$rust_build_log" \
  python3 tooling/scripts/build_lite_mcp.py --target current --out-dir "$rust_artifact_dir"

echo "[preflight] materialize distribution payloads"
if [[ "$MATERIALIZE_IN_PLACE" -eq 1 ]]; then
  echo "[preflight] in-place materialization requires explicit --in-place"
  python3 scripts/materialize_distribution_payloads.py --target all --in-place
else
  if [[ -z "$MATERIALIZE_OUT" ]]; then
    MATERIALIZE_OUT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-release-preflight.XXXXXX")"
  fi
  python3 scripts/materialize_distribution_payloads.py --target all --out "$MATERIALIZE_OUT" --force
  PREFLIGHT_ROOT="$MATERIALIZE_OUT"
fi

pkg_dir="$PREFLIGHT_ROOT/qiongli-workflow"
sync_ok=1
for check_dir in skills templates standards roles; do
  if [[ ! -d "$pkg_dir/$check_dir" ]]; then
    echo "[preflight] missing bundled directory: $check_dir" >&2
    sync_ok=0
  fi
done
if [[ ! -f "$pkg_dir/skills-core.md" ]]; then
  echo "[preflight] missing bundled file: skills-core.md" >&2
  sync_ok=0
fi
if [[ "$sync_ok" -eq 0 ]]; then
  echo "[preflight] FAIL: skill package is not self-contained" >&2
  exit 1
fi
echo "[preflight] skill package verified self-contained"

echo "[preflight] release target registries schema"
python3 scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"

echo "[preflight] capability contract v2"
python3 scripts/validate_capability_contract.py --root "$PREFLIGHT_ROOT"

echo "[preflight] sync skill reference docs"
(
  cd "$PREFLIGHT_ROOT"
  python3 scripts/generate_skill_docs.py
)

validate_cmd=(python3 scripts/validate_research_standard.py --root "$PREFLIGHT_ROOT")
if [[ "$STRICT_MODE" -eq 1 ]]; then
  validate_cmd+=(--strict)
fi

run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"
validator_summary="$(grep '^Summary:' "$validator_log" | tail -n1 || true)"
if [[ -z "$validator_summary" ]]; then
  validator_summary="completed"
fi

if [[ "$RUN_UNIT_TESTS" -eq 1 ]]; then
  run_logged_stage "unit tests" "$unit_log" python3 -m unittest discover -s tests -v
  unit_ran_line="$(grep -E '^Ran [0-9]+ tests? in ' "$unit_log" | tail -n1 || true)"
  if grep -q '^OK$' "$unit_log"; then
    if [[ -n "$unit_ran_line" ]]; then
      unittest_summary="${unit_ran_line} ... OK"
    else
      unittest_summary="OK"
    fi
  else
    unittest_summary="FAILED"
  fi
else
  echo "[preflight] unit tests skipped"
  unittest_summary="skipped"
fi

if [[ "$RUN_CONTROLLER_EVALS" -eq 1 ]]; then
  run_warning_stage "controller-mode evals" "$eval_log" python3 scripts/run_controller_mode_evals.py evals/controller_modes
else
  echo "[preflight] controller-mode evals skipped"
fi

if [[ "$RUN_SMOKE" -eq 1 ]]; then
  smoke_tier="release"
  if [[ "$MAINTAINER_SMOKE" -eq 1 ]]; then
    smoke_tier="maintainer"
  fi
  run_logged_stage "smoke (${smoke_tier} tier)" "$smoke_log" ./scripts/run_beta_smoke.sh --tier "$smoke_tier"
  if grep -q '\[smoke\] passed' "$smoke_log"; then
    smoke_summary="passed (${smoke_tier}-tier)"
  else
    smoke_summary="completed (${smoke_tier}-tier)"
  fi
else
  echo "[preflight] smoke skipped"
  smoke_summary="skipped"
fi

if [[ -n "$TAG" && "$SKIP_NOTE_GEN" -eq 0 ]] && is_prerelease_tag "$TAG"; then
  update_note_cmd=(
    ./scripts/generate_release_notes.sh
    --tag "$TAG"
    --update-existing
    --validator-result "$validator_summary"
    --unittest-result "$unittest_summary"
    --smoke-result "$smoke_summary"
  )
  if [[ -n "$FROM_TAG" ]]; then
    update_note_cmd+=(--from-tag "$FROM_TAG")
  fi
  echo "[preflight] release note evidence update"
  "${update_note_cmd[@]}"
fi

echo "[preflight] all checks passed"
echo "[preflight] preflight completed; publish mode owns tag/push"
