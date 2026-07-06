# Economics-Accounting Runtime Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the economics-accounting bridge from `eval_ready` to `runtime_enabled` after its fixture pack has proved identification/measurement bridge signals, method-only borrowing, mixed adjacent-subject behavior, confirmed-subject behavior, and near-miss guards.

**Architecture:** Treat runtime activation as a narrow subject-runtime promotion. The bridge already passes eval-ready expectations through gate-specific fixture overrides; runtime activation moves the clear and mixed bridge cases into default runtime behavior, adds a subject-owned bridge auditor skill, and changes the manifest status only after the runtime-enabled gate has explicit coverage.

**Tech Stack:** Python 3, unittest, JSON router fixtures, YAML runtime subject manifests, Qiongli subject router evaluation tooling.

---

## File Map

- Modify: `tests/fixtures/subject_router_eval/economics-accounting/clear_identification_measurement_bridge.json`
  - Make the default expectation economics-accounting positive after runtime activation.
  - Add `gate_expected["runtime-enabled"]` matching the positive gate behavior.
- Modify: `tests/fixtures/subject_router_eval/economics-accounting/mixed_accounting_economics_bridge.json`
  - Make the default expectation economics-accounting positive after runtime activation.
  - Add `gate_expected["runtime-enabled"]` matching the positive gate behavior.
- Modify: `tests/test_subject_router_eval.py`
  - Replace the real-manifest eval-ready success test with a runtime-enabled success test.
  - Add real-manifest rejection tests for eval-ready and promotion-ready gates after activation.
- Modify: `tests/test_subject_contracts.py`
  - Expect economics-accounting to be `runtime_enabled`.
  - Require a subject-owned bridge auditor skill resource.
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`
  - Change `activation_status: eval_ready` to `activation_status: runtime_enabled`.
  - Set `subject_skill` to the bridge auditor resource.
- Add: `content/subjects/economics-accounting/skills/economics-accounting-bridge-auditor.md`
  - Define the runtime skill used when the subject is active or confirmed.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record economics-accounting as runtime-enabled.
  - Remove it from eval-ready and immediate follow-up status.

## Execution Notes

- Do not change economics-accounting signal weights, regexes, method lenses, venue profiles, or thresholds unless a measured runtime-enabled gate failure requires it.
- Do not change broader local-agent execution, experience promotion, packaging, release automation, or platform target behavior.
- Keep the bridge auditor as a subject runtime resource under `content/subjects/economics-accounting/skills/`; it is not a global Codex skill or marketplace artifact.
- Keep commits grouped by behavior and docs:
  - `feat(subjects): activate economics-accounting runtime`
  - `docs(roadmap): record economics-accounting runtime activation`

## Task 1: Update Runtime Activation Tests And Fixtures

**Files:**
- Modify: `tests/fixtures/subject_router_eval/economics-accounting/clear_identification_measurement_bridge.json`
- Modify: `tests/fixtures/subject_router_eval/economics-accounting/mixed_accounting_economics_bridge.json`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Update clear-positive default expectation**

In `clear_identification_measurement_bridge.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "economics-accounting",
  "suggest_subjects": ["economics-accounting"],
  "forbidden_subjects": [],
  "method_lenses": [
    "identification-measurement-alignment",
    "fiscal-window-alignment",
    "economics-accounting-positioning"
  ]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 2: Update mixed-adjacent default expectation**

In `mixed_accounting_economics_bridge.json`, change `expected` to:

```json
{
  "decision": "recommend",
  "primary_subject": "economics-accounting",
  "suggest_subjects": ["economics-accounting"],
  "allowed_neighbor_subjects": ["accounting", "economics"],
  "forbidden_subjects": [],
  "method_lenses": [
    "fiscal-window-alignment",
    "economics-accounting-positioning"
  ]
}
```

Add `gate_expected["runtime-enabled"]` with the same fields.

- [ ] **Step 3: Replace real eval-ready success with runtime-enabled success**

In `tests/test_subject_router_eval.py`, replace the eval-ready success test with a runtime-enabled success test that asserts:

- `activation_status == "runtime_enabled"`
- `eligible_for_eval_ready == False`
- `eligible_for_runtime_promotion == False`
- `eligible_for_runtime_enabled == True`
- all fixture metrics stay at 1.0 and `near_miss_false_positives == 0`

- [ ] **Step 4: Add real-manifest eval-ready and promotion-ready blockers**

After activation, eval-ready and promotion-ready gates should block with:

```text
activation_status is runtime_enabled
```

- [ ] **Step 5: Update contract expectations**

In `tests/test_subject_contracts.py`, expect economics-accounting to be runtime-enabled and require:

```text
content/subjects/economics-accounting/skills/economics-accounting-bridge-auditor.md
```

- [ ] **Step 6: Run RED**

Run the targeted runtime-enabled gate test and contract test before changing the manifest:

```bash
.venv/bin/python -m unittest \
  tests.test_subject_router_eval.SubjectRouterEvalTests.test_economics_accounting_runtime_enabled_gate_passes_real_fixture_pack \
  tests.test_subject_contracts.RuntimeSubjectContractTests.test_economics_accounting_runtime_enabled_manifest_declares_signals_and_lenses \
  -q
```

Expected before manifest activation: failure on `activation_status is eval_ready` or the missing subject skill expectation.

## Task 2: Activate Runtime Resources

**Files:**
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`
- Add: `content/subjects/economics-accounting/skills/economics-accounting-bridge-auditor.md`

- [ ] **Step 1: Add subject-owned skill resource**

Create a focused bridge auditor that checks:

- economics-style estimand and identifying variation,
- accounting construct/proxy validity,
- disclosure or reporting institution,
- fiscal timing and capital-market outcome window,
- sample filters and source-item mapping,
- composite reviewer risk.

- [ ] **Step 2: Promote the manifest**

Change:

```yaml
activation_status: runtime_enabled
subject_skill: content/subjects/economics-accounting/skills/economics-accounting-bridge-auditor.md
```

- [ ] **Step 3: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest \
  tests.test_subject_router_eval.SubjectRouterEvalTests.test_economics_accounting_runtime_enabled_gate_passes_real_fixture_pack \
  tests.test_subject_contracts.RuntimeSubjectContractTests.test_economics_accounting_runtime_enabled_manifest_declares_signals_and_lenses \
  -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject economics-accounting --gate runtime-enabled --json
```

## Task 3: Update Roadmap And Verify Scope

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record activation**

Update the baseline, remaining gaps, Stage 4 status, runtime-enabled list, and immediate plan so economics-accounting is no longer the remaining eval-ready subject.

- [ ] **Step 2: Run regression checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval tests.test_subject_contracts -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --json
git diff --check
```

- [ ] **Step 3: Commit by category**

Feature commit:

```text
feat(subjects): activate economics-accounting runtime
```

Docs commit:

```text
docs(roadmap): record economics-accounting runtime activation
```
