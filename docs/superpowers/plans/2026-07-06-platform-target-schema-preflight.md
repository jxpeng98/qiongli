# Platform Target Schema Preflight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Stage 12 release preflight fail early when a platform target registry entry lacks positive required-path checks, negative forbidden-path checks, or release-download metadata.

**Architecture:** Keep `content/distribution/platform-targets.yaml` as the canonical registry and strengthen the existing `qiongli.platform_targets` loader/validator. Add a small validation script that release preflight can run against the materialized staging root before the standard validator and unit tests.

**Tech Stack:** Python stdlib, PyYAML-backed `qiongli.platform_targets`, existing Bash release preflight, existing `unittest` contract tests.

---

## Files

- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`
  - Reject targets with empty `required_paths` or `forbidden_paths`.
  - Require `release_download.guide_label`, `release_download.recommended_key`, and list-valued `release_download.asset_groups`.
  - Add a small `main()` entrypoint for script reuse if needed.
- Create: `tooling/scripts/validate_platform_targets.py`
  - Load the registry for a selected root and print one `[FAIL]` line per schema issue.
- Create: `scripts/validate_platform_targets.py`
  - Keep the existing root-level script shim pattern.
- Modify: `tooling/scripts/release_preflight.sh`
  - Run the platform target registry schema gate after materialization and before skill docs/standard validation.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Add fail-closed tests for missing positive/negative check policy and missing release-download metadata.
- Modify: `tests/test_release_automation.py`
  - Assert release preflight invokes the schema gate before the standard validator.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 12 now has an explicit schema preflight gate for target registry entries.

## Task 1: Add Failing Schema And Preflight Tests

- [x] **Step 1: Add fail-closed registry schema tests**

Add tests that build temporary `content/distribution/platform-targets.yaml`
fixtures and assert `validate_platform_target_registry(root)` reports:

```python
"missing positive required_path checks"
"missing negative forbidden_path checks"
"release_download"
```

- [x] **Step 2: Add release preflight wiring test**

Add a test that asserts `tooling/scripts/release_preflight.sh` contains:

```bash
python3 scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"
```

and that this command appears before:

```bash
run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"
```

- [x] **Step 3: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_targets_without_positive_or_negative_checks tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_missing_release_download_metadata tests.test_release_automation.ReleaseAutomationTests.test_release_preflight_validates_platform_target_registry_before_standard_validator -q
```

Expected: FAIL because the stricter schema checks and preflight command are not present yet.

## Task 2: Implement Schema Gate

- [x] **Step 1: Strengthen `qiongli.platform_targets` validation**

Update `_parse_target(...)` so:

- `required_paths` must contain at least one string.
- `forbidden_paths` must contain at least one string.
- `release_download` must be an object containing non-empty string
  `guide_label`, non-empty string `recommended_key`, and list-valued
  `asset_groups`.

- [x] **Step 2: Add validation script**

Create `tooling/scripts/validate_platform_targets.py` that imports
`validate_platform_target_registry`, prints `[OK] platform target registry schema
valid` on success, prints `[FAIL] platform target registry: ...` for each
failure, and exits non-zero when failures exist.

Create `scripts/validate_platform_targets.py` as the same wrapper pattern used
by root-level tooling shims.

- [x] **Step 3: Wire release preflight**

Add:

```bash
echo "[preflight] platform target registry schema"
python3 scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"
```

after the skill package self-contained check and before skill docs generation.

- [x] **Step 4: Run GREEN**

Run the focused tests from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run targeted regression tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_automation -q
```

- [x] **Step 2: Run the new script against the current repo**

Run:

```bash
.venv/bin/python scripts/validate_platform_targets.py --root .
```

Expected: `[OK] platform target registry schema valid`.

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog wording so the schema preflight gate is no
longer listed as missing.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add packages/python-qiongli/src/qiongli/platform_targets.py tooling/scripts/validate_platform_targets.py scripts/validate_platform_targets.py tooling/scripts/release_preflight.sh tests/test_plugin_distribution_contract.py tests/test_release_automation.py
git commit -m "feat(release): validate platform target schema"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-platform-target-schema-preflight.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record platform target schema preflight"
```

## Self-Review

- Spec coverage: Covers the Stage 12 optimization backlog item for a schema validator and release preflight failure when target entries lack positive and negative checks.
- Placeholder scan: No open placeholders remain.
- Type consistency: New schema requirements map onto existing `PlatformTarget` fields and `release_download` metadata already used by release download generation.
