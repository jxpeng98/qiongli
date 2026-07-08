# Experience Schema Release Check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the Stage 11 release-readiness follow-up by adding an experience schema compatibility gate to release preparation.

**Architecture:** Add schema compatibility validation to `experience_runtime.py` so runtime code and release scripts share one definition of a compatible experience record. Add a small tooling script and root wrapper, then call it from `release_ready.sh` against the staged release root.

**Tech Stack:** Python 3.12, Bash release scripts, `unittest`, JSONL experience indexes.

---

## File Map

- Modify: `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py`
  - Add compatibility checks for experience records and indexes.
- Modify: `tests/test_experience_runtime.py`
  - Add tests for valid records, malformed JSONL, missing required objects, and mismatched record files.
- Create: `tooling/scripts/check_experience_schema_compatibility.py`
  - CLI entrypoint that prints a compact report and exits non-zero on incompatible records.
- Create: `scripts/check_experience_schema_compatibility.py`
  - Standard root wrapper for the tooling script.
- Modify: `tooling/scripts/release_ready.sh`
  - Run the checker after staged version verification and before package preflights.
- Modify: `tests/test_release_automation.py`
  - Static contract test that release readiness runs the checker in the intended order.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Stage 11 release-readiness compatibility checks implemented.

## Task 1: Add Failing Runtime Schema Tests

**Files:**
- Modify: `tests/test_experience_runtime.py`

- [x] **Step 1: Add compatibility test expectations**

Add tests for:

```python
report = experience_schema_compatibility(root)
self.assertTrue(report["ok"])
self.assertEqual(report["checked_records"], 1)
self.assertEqual(report["errors"], [])
```

and for an invalid record:

```python
report = experience_schema_compatibility(root)
self.assertFalse(report["ok"])
self.assertTrue(any("missing required object: task" in error for error in report["errors"]))
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime.ExperienceRuntimeTests.test_experience_schema_compatibility_accepts_current_records tests.test_experience_runtime.ExperienceRuntimeTests.test_experience_schema_compatibility_reports_malformed_records -q
```

Expected: import or attribute failure because the function does not exist.

## Task 2: Implement Runtime Compatibility Check

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py`

- [x] **Step 1: Add `experience_schema_compatibility(project_root)`**

Validate all rows from `.qiongli/trace/experience.jsonl`; if a row points to `.qiongli/trace/runs/<run_id>/experience_record.json`, validate that file too. Required top-level fields:

- `schema_version == SCHEMA_VERSION`
- `run_id` non-empty string
- object fields: `task`, `execution`, `inputs`, `outputs`, `quality`, `experience`, `privacy`

Return:

```python
{
    "project_dir": str(root),
    "ok": not errors,
    "checked_records": checked_records,
    "malformed_count": malformed_count,
    "errors": errors,
}
```

- [x] **Step 2: Run GREEN**

Run the two new runtime tests. Expected: `OK`.

## Task 3: Add CLI Checker And Release-Ready Gate

**Files:**
- Create: `tooling/scripts/check_experience_schema_compatibility.py`
- Create: `scripts/check_experience_schema_compatibility.py`
- Modify: `tooling/scripts/release_ready.sh`
- Modify: `tests/test_release_automation.py`

- [x] **Step 1: Add failing release automation test**

Assert `release_ready.sh` includes:

```python
checker = 'python3 scripts/check_experience_schema_compatibility.py --root "$RELEASE_STAGING_DIR"'
self.assertIn(checker, content)
self.assertLess(content.index(verify), content.index(checker))
self.assertLess(content.index(checker), content.index(local_install))
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_automation.ReleaseAutomationTests.test_release_ready_checks_experience_schema_compatibility -q
```

Expected: fails because the checker is not wired.

- [x] **Step 3: Implement CLI script and wrapper**

The tooling script should call `experience_schema_compatibility(root)`, print JSON with `--json`, print a compact text summary by default, and exit `1` if `ok` is false.

- [x] **Step 4: Wire `release_ready.sh`**

Add:

```bash
echo "[release-ready] experience schema compatibility"
python3 scripts/check_experience_schema_compatibility.py --root "$RELEASE_STAGING_DIR"
```

after `verify_release_tag_version.sh` and before local install/package checks.

- [x] **Step 5: Run GREEN**

Run the release automation test. Expected: `OK`.

## Task 4: Verify And Update Roadmap

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-06-experience-schema-release-check.md`

- [x] **Step 1: Mark Stage 11 status**

Update Stage 11 status to include release-readiness compatibility checks.

- [x] **Step 2: Run verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime tests.test_release_automation -q
git diff --check
```

Expected: tests pass and whitespace check exits 0.
