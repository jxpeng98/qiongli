# Political Economy Runtime Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote political economy from `eval_ready` to `runtime_enabled` after the promotion-ready gate has proved its subject-owned fixture pack, resources, method-only borrowing, mixed-adjacent behavior, and near-miss guards.

**Architecture:** Treat activation as a narrow manifest-status change backed by fixture expectation migration. Political economy already passes `promotion-ready`; after activation, default and `runtime-enabled` evaluations should use the same positive expectations that were previously scoped to eval/promotion gates, while method-only and near-miss cases remain suppressed.

**Tech Stack:** Python 3, unittest, JSON router fixtures, YAML runtime subject manifests, Qiongli subject router evaluation tooling.

---

## File Map

- Modify: `tests/fixtures/subject_router_eval/political-economy/clear_actor_institution_outcome.json`
  - Make the default expectation political-economy positive after runtime activation.
  - Add or keep `runtime-enabled` expectation matching the positive gate behavior.
- Modify: `tests/fixtures/subject_router_eval/political-economy/mixed_capital_market_distributional_conflict.json`
  - Make the default expectation political-economy positive after runtime activation.
  - Add or keep `runtime-enabled` expectation matching the positive gate behavior.
- Modify: `tests/test_subject_router_eval.py`
  - Replace the real-manifest eval-ready success test with a runtime-enabled success test.
  - Add real-manifest rejection tests for eval-ready and promotion-ready gates after activation.
- Modify: `tests/test_subject_contracts.py`
  - Expect political economy to be `runtime_enabled` while preserving resource, signal, lens, and gate metric checks.
- Modify: `content/subjects/political-economy/runtime-subject.yaml`
  - Change only `activation_status: eval_ready` to `activation_status: runtime_enabled`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record political economy as runtime-enabled.
  - Leave geoeconomics and economics-accounting as deferred candidates.

## Execution Notes

- Do not change political-economy signal weights, regexes, skill paths, method lenses, or thresholds unless a measured runtime-enabled gate failure requires it.
- Do not activate geoeconomics or economics-accounting.
- Do not change provider configuration, literature search, local-agent execution, packaging, or release automation.
- Keep commits grouped by behavior and docs:
  - `feat(subjects): activate political economy runtime`
  - `docs(roadmap): record political economy runtime activation`

## Task 1: Update Runtime Activation Tests And Fixtures

**Files:**
- Modify: `tests/fixtures/subject_router_eval/political-economy/clear_actor_institution_outcome.json`
- Modify: `tests/fixtures/subject_router_eval/political-economy/mixed_capital_market_distributional_conflict.json`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Update clear-positive default expectation**

In `clear_actor_institution_outcome.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "political-economy",
  "suggest_subjects": ["political-economy"],
  "forbidden_subjects": [],
  "method_lenses": [
    "political-mechanism-audit",
    "political-economy-positioning"
  ]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 2: Update mixed-adjacent default expectation**

In `mixed_capital_market_distributional_conflict.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "political-economy",
  "suggest_subjects": ["political-economy"],
  "allowed_neighbor_subjects": ["finance"],
  "forbidden_subjects": [],
  "method_lenses": ["political-economy-positioning"]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 3: Replace real eval-ready success with runtime-enabled success**

In `tests/test_subject_router_eval.py`, replace `test_political_economy_eval_ready_gate_passes_real_fixture_pack` with:

```python
    def test_political_economy_runtime_enabled_gate_passes_real_fixture_pack(
        self,
    ) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report(
            "political-economy",
            cases,
            gate="runtime-enabled",
        )

        self.assertEqual(report["subject"], "political-economy")
        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_promotion"])
        self.assertTrue(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["decision_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["primary_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["suggest_subject_precision"], 1.0)
        self.assertEqual(report["metrics"]["forbidden_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["method_lens_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["all_case_checks_passed"], 1.0)
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)
```

- [ ] **Step 4: Add real-manifest eval-ready and promotion-ready blockers**

Replace `test_political_economy_runtime_enabled_gate_blocks_eval_ready_manifest` with:

```python
    def test_political_economy_eval_ready_gate_blocks_runtime_enabled_manifest(
        self,
    ) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("political-economy", cases, gate="eval-ready")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])

    def test_political_economy_promotion_ready_gate_blocks_runtime_enabled_manifest(
        self,
    ) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report(
            "political-economy",
            cases,
            gate="promotion-ready",
        )

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_runtime_promotion"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])
```

- [ ] **Step 5: Update contract expectations**

In `tests/test_subject_contracts.py`, rename the political-economy manifest test to:

```python
    def test_political_economy_runtime_enabled_manifest_declares_signals_and_lenses(
        self,
    ) -> None:
```

Change its status assertion to:

```python
        self.assertEqual(contract.activation_status, "runtime_enabled")
```

In the default repository classification test, expect:

```python
        self.assertEqual(
            subject_activation_status("political-economy", contracts),
            "runtime_enabled",
        )
```

- [ ] **Step 6: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_political_economy_runtime_enabled_gate_passes_real_fixture_pack -q
```

Expected before manifest activation:

```text
FAIL: activation_status is eval_ready
```

## Task 2: Activate Manifest And Verify

**Files:**
- Modify: `content/subjects/political-economy/runtime-subject.yaml`

- [ ] **Step 1: Change only activation status**

Change:

```yaml
activation_status: eval_ready
```

to:

```yaml
activation_status: runtime_enabled
```

- [ ] **Step 2: Run targeted GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_political_economy_runtime_enabled_gate_passes_real_fixture_pack -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject political-economy --gate runtime-enabled --json
```

Expected:

```text
eligible_for_runtime_enabled: true
blocking_failures: []
near_miss_false_positives: 0
```

- [ ] **Step 3: Run related regressions**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval tests.test_subject_contracts -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject political-economy --gate eval-ready --json
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject political-economy --gate promotion-ready --json
git diff --check
```

Expected:

- router/contract tests exit 0,
- runtime-enabled gate exits 0,
- eval-ready and promotion-ready commands exit 1 because the checked-in manifest is already `runtime_enabled`,
- `git diff --check` exits 0.

## Task 3: Update Roadmap And Commit

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update Stage 4 status**

Record that political economy is now runtime-enabled and remove it from the eval-ready list.

- [ ] **Step 2: Update remaining gaps**

Keep the remaining subject expansion gap focused on geoeconomics and economics-accounting.

- [ ] **Step 3: Commit by content**

Run:

```bash
git add content/subjects/political-economy/runtime-subject.yaml tests/test_subject_router_eval.py tests/test_subject_contracts.py tests/fixtures/subject_router_eval/political-economy
git commit -m "feat(subjects): activate political economy runtime"

git add docs/superpowers/plans/2026-07-06-political-economy-runtime-activation.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record political economy runtime activation"
```
