# Geoeconomics Runtime Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote geoeconomics from `eval_ready` to `runtime_enabled` after its eval-ready fixture pack has proved statecraft signals, method-only borrowing, mixed-adjacent behavior, and near-miss guards.

**Architecture:** Treat activation as a narrow manifest-status change backed by fixture expectation migration. Geoeconomics already passes `eval-ready`; after activation, default and `runtime-enabled` evaluations should use the same positive expectations that were previously scoped to eval/promotion gates, while method-only and near-miss cases remain suppressed.

**Tech Stack:** Python 3, unittest, JSON router fixtures, YAML runtime subject manifests, Qiongli subject router evaluation tooling.

---

## File Map

- Modify: `tests/fixtures/subject_router_eval/geoeconomics/clear_sanctions_statecraft.json`
  - Make the default expectation geoeconomics positive after runtime activation.
  - Add `gate_expected["runtime-enabled"]` matching the positive gate behavior.
- Modify: `tests/fixtures/subject_router_eval/geoeconomics/mixed_finance_supply_chain_exposure.json`
  - Make the default expectation geoeconomics positive after runtime activation.
  - Add `gate_expected["runtime-enabled"]` matching the positive gate behavior.
- Modify: `tests/test_subject_router_eval.py`
  - Replace the real-manifest eval-ready success test with a runtime-enabled success test.
  - Add real-manifest rejection tests for eval-ready and promotion-ready gates after activation.
- Modify: `tests/test_subject_contracts.py`
  - Expect geoeconomics to be `runtime_enabled` while preserving resource, signal, lens, and gate metric checks.
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`
  - Change only `activation_status: eval_ready` to `activation_status: runtime_enabled`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record geoeconomics as runtime-enabled.
  - Leave economics-accounting as the remaining deferred candidate.

## Execution Notes

- Do not change geoeconomics signal weights, regexes, skill paths, method lenses, or thresholds unless a measured runtime-enabled gate failure requires it.
- Do not activate economics-accounting.
- Do not change provider configuration, literature search, local-agent execution, packaging, or release automation.
- Keep commits grouped by behavior and docs:
  - `feat(subjects): activate geoeconomics runtime`
  - `docs(roadmap): record geoeconomics runtime activation`

## Task 1: Update Runtime Activation Tests And Fixtures

**Files:**
- Modify: `tests/fixtures/subject_router_eval/geoeconomics/clear_sanctions_statecraft.json`
- Modify: `tests/fixtures/subject_router_eval/geoeconomics/mixed_finance_supply_chain_exposure.json`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Update clear-positive default expectation**

In `clear_sanctions_statecraft.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "geoeconomics",
  "suggest_subjects": ["geoeconomics"],
  "forbidden_subjects": [],
  "method_lenses": [
    "geoeconomic-statecraft-audit",
    "geoeconomic-positioning"
  ]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 2: Update mixed-adjacent default expectation**

In `mixed_finance_supply_chain_exposure.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "geoeconomics",
  "suggest_subjects": ["geoeconomics"],
  "allowed_neighbor_subjects": ["finance"],
  "forbidden_subjects": [],
  "method_lenses": [
    "supply-chain-exposure-design",
    "geoeconomic-positioning"
  ]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 3: Replace real eval-ready success with runtime-enabled success**

In `tests/test_subject_router_eval.py`, replace `test_geoeconomics_eval_ready_gate_passes_real_fixture_pack` with:

```python
    def test_geoeconomics_runtime_enabled_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("geoeconomics", cases, gate="runtime-enabled")

        self.assertEqual(report["subject"], "geoeconomics")
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

Replace `test_geoeconomics_runtime_enabled_gate_blocks_eval_ready_manifest` with:

```python
    def test_geoeconomics_eval_ready_gate_blocks_runtime_enabled_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("geoeconomics", cases, gate="eval-ready")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])

    def test_geoeconomics_promotion_ready_gate_blocks_runtime_enabled_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("geoeconomics", cases, gate="promotion-ready")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_runtime_promotion"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])
```

- [ ] **Step 5: Update contract expectations**

In `tests/test_subject_contracts.py`, rename:

```python
    def test_geoeconomics_eval_ready_manifest_declares_signals_and_lenses(
```

to:

```python
    def test_geoeconomics_runtime_enabled_manifest_declares_signals_and_lenses(
```

Inside that test, update:

```python
        self.assertEqual(contract.activation_status, "runtime_enabled")
```

In the default repository classification test, expect:

```python
        self.assertEqual(
            subject_activation_status("geoeconomics", contracts),
            "runtime_enabled",
        )
```

- [ ] **Step 6: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_geoeconomics_runtime_enabled_gate_passes_real_fixture_pack -q
```

Expected before manifest activation:

```text
FAIL: activation_status is eval_ready
```

## Task 2: Activate Manifest And Verify

**Files:**
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`

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
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_geoeconomics_runtime_enabled_gate_passes_real_fixture_pack -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject geoeconomics --gate runtime-enabled --json
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
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject geoeconomics --gate eval-ready --json
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject geoeconomics --gate promotion-ready --json
.venv/bin/python tooling/scripts/evaluate_subject_router.py --json
git diff --check
```

Expected:

- router/contract tests exit 0.
- runtime-enabled gate exits 0.
- eval-ready and promotion-ready commands exit 1 because the checked-in manifest is already `runtime_enabled`.
- all-fixture evaluation exits 0.
- `git diff --check` exits 0.

## Task 3: Update Roadmap And Commit

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update Stage 4 status**

Record that geoeconomics is now runtime-enabled and remove it from the eval-ready list.

- [ ] **Step 2: Update remaining gaps**

Keep the remaining subject expansion gap focused on the economics-accounting bridge.

- [ ] **Step 3: Commit by content**

Run:

```bash
git add content/subjects/geoeconomics/runtime-subject.yaml tests/test_subject_router_eval.py tests/test_subject_contracts.py tests/fixtures/subject_router_eval/geoeconomics
git commit -m "feat(subjects): activate geoeconomics runtime"
git add docs/superpowers/plans/2026-07-06-geoeconomics-runtime-activation.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record geoeconomics runtime activation"
```
