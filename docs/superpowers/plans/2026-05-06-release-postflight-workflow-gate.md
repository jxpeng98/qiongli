# Release Postflight Workflow Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `release_automation.sh publish` complete the postflight release flow without waiting on a non-existent GitHub Actions workflow name.

**Architecture:** Keep publish as the single end-to-end release entrypoint. Fix the postflight CI gate to require the workflow names that GitHub Actions actually reports, and improve pending diagnostics when required workflows are missing.

**Tech Stack:** Bash release scripts, Python `unittest` assertions over release automation scripts, GitHub Actions workflow names.

---

### Task 1: Lock The Required Workflow Name

**Files:**
- Modify: `tests/test_release_automation.py`
- Modify: `scripts/release_postflight.sh`

- [ ] **Step 1: Write the failing test**

Update `test_release_postflight_waits_for_required_workflows` so it expects:

```python
self.assertIn('REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
self.assertNotIn('REQUIRED_WORKFLOWS=("CI" "Install Check")', content)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m unittest tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_waits_for_required_workflows -v`

Expected: FAIL because `scripts/release_postflight.sh` still contains `Install Check`.

- [ ] **Step 3: Write minimal implementation**

Change `scripts/release_postflight.sh`:

```bash
REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python3 -m unittest tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_waits_for_required_workflows -v`

Expected: PASS.

### Task 2: Add Missing-Workflow Diagnostics

**Files:**
- Modify: `tests/test_release_automation.py`
- Modify: `scripts/release_postflight.sh`

- [ ] **Step 1: Write the failing test**

Add assertions that `query_ci_status` reports observed workflow names when required workflows are missing:

```python
self.assertIn('observed = sorted({r.get("name") or "unknown" for r in runs if r.get("head_sha") == commit})', content)
self.assertIn('labels.append("observed=" + ",".join(observed))', content)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m unittest tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_waits_for_required_workflows -v`

Expected: FAIL because missing diagnostics do not include observed workflow names yet.

- [ ] **Step 3: Write minimal implementation**

In the embedded Python inside `query_ci_status`, compute observed workflow names for the release commit and add them to pending diagnostics when `missing` is non-empty.

- [ ] **Step 4: Run test to verify it passes**

Run: `python3 -m unittest tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_waits_for_required_workflows -v`

Expected: PASS.

### Task 3: Verify Release Automation Flow

**Files:**
- Test only: `tests/test_release_automation.py`
- Syntax only: `scripts/release_postflight.sh`, `scripts/release_automation.sh`

- [ ] **Step 1: Run targeted release automation tests**

Run: `python3 -m unittest tests.test_release_automation -v`

Expected: all tests OK.

- [ ] **Step 2: Run shell syntax checks**

Run:

```bash
bash -n scripts/release_postflight.sh
bash -n scripts/release_automation.sh
```

Expected: both commands exit 0.

- [ ] **Step 3: Commit**

Run:

```bash
git add scripts/release_postflight.sh tests/test_release_automation.py docs/superpowers/plans/2026-05-06-release-postflight-workflow-gate.md
git commit -m "fix: align postflight workflow gate"
```
