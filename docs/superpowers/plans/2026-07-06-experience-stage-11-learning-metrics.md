# Experience Stage 11 Learning Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the missing Stage 11 experience metrics for guidance acceptance, subject routing correction, review blockers, and literature diagnostic failures.

**Architecture:** Extend the existing local `experience_metrics()` summary without changing the experience index location or CLI command surface. Keep new metrics derived from already versioned experience records, and add small optional fields to newly built records so future task runs preserve guidance-update evidence.

**Tech Stack:** Python stdlib, existing `bridges.experience_runtime`, existing `unittest` coverage.

---

## Files

- Modify: `tests/test_experience_runtime.py`
  - Extend the fixture helper with optional record sections.
  - Add a failing metrics test for Stage 11 learning-rate groups.
- Modify: `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py`
  - Store guidance update evidence in new experience records.
  - Summarize guidance acceptance, subject routing lifecycle actions, review blockers, and literature diagnostics in `experience_metrics()`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 11 metrics now cover the full named metric set from the scope.

## Task 1: Add Failing Metrics Contract Test

- [x] **Step 1: Extend the fixture helper**

In `tests/test_experience_runtime.py`, add optional parameters to
`_write_experience_fixture()`:

```python
inputs: dict[str, object] | None = None,
outputs: dict[str, object] | None = None,
quality: dict[str, object] | None = None,
experience: dict[str, object] | None = None,
execution: dict[str, object] | None = None,
```

Merge them into the generated record after the default sections are created.

- [x] **Step 2: Add Stage 11 metrics test**

Add a test that writes three B1 records:

- one accepted guidance update with subject confirm action and no diagnostic failure
- one proposed but not applied guidance update with subject dismiss action, blocking review issue, and missing `search_diagnostics.md`
- one non-literature F3 passing run

Assert:

```python
self.assertEqual(metrics["guidance"]["proposal_runs"], 2)
self.assertEqual(metrics["guidance"]["accepted_runs"], 1)
self.assertEqual(metrics["guidance"]["acceptance_rate"], 0.5)
self.assertEqual(metrics["subject_routing"]["confirmation_count"], 1)
self.assertEqual(metrics["subject_routing"]["dismissal_count"], 1)
self.assertEqual(metrics["subject_routing"]["correction_count"], 1)
self.assertEqual(metrics["subject_routing"]["correction_rate"], 0.5)
self.assertEqual(metrics["review"]["blocker_count"], 1)
self.assertEqual(metrics["literature_diagnostics"]["checked_runs"], 2)
self.assertEqual(metrics["literature_diagnostics"]["failure_count"], 1)
self.assertEqual(metrics["literature_diagnostics"]["failure_rate"], 0.5)
```

- [x] **Step 3: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime.ExperienceRuntimeTests.test_experience_metrics_summarize_stage_11_learning_rates -q
```

Expected: FAIL because `experience_metrics()` does not yet return the new groups.

## Task 2: Implement Metrics

- [x] **Step 1: Preserve guidance update evidence**

In `build_experience_record()`, add `experience["guidance_update"]` with:

```python
{
    "proposal_path": str(guidance_trace.get("guidance_proposal", "")),
    "applied": bool(guidance_trace.get("applied_guidance_update", False)),
    "mode": str(guidance_trace.get("guidance_mode", "")),
}
```

- [x] **Step 2: Extend `experience_metrics()`**

Add metric groups:

- `guidance`: proposal count, accepted count, acceptance rate.
- `subject_routing`: lifecycle action count, confirmation count, dismissal count, correction count, and rates.
- `review`: blocking issue count and blocked review run count.
- `literature_diagnostics`: checked literature runs, failure count, failure rate.

Use helper functions to keep `experience_metrics()` readable.

- [x] **Step 3: Run GREEN**

Run the focused test from Task 1. Expected: PASS.

## Task 3: Verify And Document

- [x] **Step 1: Run related tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime tests.test_orchestrator_workflows -q
```

- [x] **Step 2: Run schema compatibility smoke**

Run:

```bash
.venv/bin/python scripts/check_experience_schema_compatibility.py --root /private/tmp/qiongli-empty-experience-metrics-smoke --json
```

- [x] **Step 3: Update roadmap**

Update Stage 11 status to name guidance acceptance, subject routing correction,
review blocker, and literature diagnostic failure metrics.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add packages/python-qiongli/src/qiongli/bridges/experience_runtime.py tests/test_experience_runtime.py
git commit -m "feat(experience): add stage 11 learning metrics"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-experience-stage-11-learning-metrics.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record stage 11 learning metrics"
```

## Self-Review

- Spec coverage: Covers the Stage 11 named metrics not yet present in the status summary.
- Placeholder scan: No placeholders remain.
- Type consistency: New metric names are stable JSON keys under existing `experience metrics` output.
