# Economics-Accounting Eval-Ready Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the economics-accounting bridge from a deferred candidate shell to `eval_ready` with bridge-owned fixture coverage, manifest-backed signals, method-only borrowing, mixed-adjacent guards, and near-miss protection.

**Architecture:** Keep economics-accounting inactive in default runtime until a later promotion review. Use `evaluation_subjects=["economics-accounting"]` under the eval-ready gate to measure bridge behavior. Avoid built-in economics trigger phrases such as `DID`, `causal identification`, `policy shock`, and `identification strategy`, and avoid broad accounting subject phrases such as `financial reporting`, `earnings quality`, and `disclosure quality` in clear-positive bridge fixtures so the bridge tests measure the composite manifest rather than pre-existing economics/accounting subjects.

**Tech Stack:** Python 3, unittest, JSON router fixtures, YAML runtime subject manifests, Qiongli subject router evaluation tooling.

---

## File Map

- Modify: `tests/test_subject_router_eval.py`
  - Add economics-accounting fixture inventory coverage.
  - Add a real fixture-pack eval-ready gate test.
  - Add a runtime-enabled guard proving checked-in `eval_ready` is not default activation.
  - Remove economics-accounting from the deferred-shell candidate loop.
- Modify: `tests/test_subject_contracts.py`
  - Classify economics-accounting as `eval_ready` instead of a deferred candidate.
  - Add manifest structure checks for bridge signal groups, method lenses, blank subject skill, evaluation pack, and metrics.
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`
  - Set `activation_status: eval_ready`.
  - Keep `subject_skill: ""` because there is no subject-owned skill yet and eval-ready allows blank optional subject resources.
  - Add method, data/outcome, venue, and theory/construct signal groups.
  - Add method lenses and `evaluation_pack: tests/fixtures/subject_router_eval/economics-accounting`.
- Create: `tests/fixtures/subject_router_eval/economics-accounting/*.json`
  - Add clear positive, method-only borrow, confirmed-subject, mixed-adjacent, and near-miss fixtures.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record economics-accounting as eval-ready.
  - Leave runtime activation as a separate follow-up.

## Execution Notes

- Do not promote economics-accounting to `runtime_enabled` in this slice.
- Do not change accounting, economics, finance, business, political-economy, or geoeconomics routing behavior.
- Do not modify `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py` unless the measured RED/GREEN path proves the existing runtime-subject contract cannot express the bridge.
- Keep venue entries context-only and avoid venue-only method lenses so course/syllabus references to accounting venues do not borrow bridge lenses.
- Keep commits grouped by behavior and docs:
  - `feat(subjects): mark economics-accounting eval ready`
  - `docs(roadmap): record economics-accounting eval readiness`

## Task 1: Add Economics-Accounting Gate Tests

**Files:**
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Add fixture inventory expectations**

Add this required fixture ID block next to the other subject inventory blocks:

```python
        required_economics_accounting_ids = {
            "economics_accounting_clear_identification_measurement_bridge",
            "economics_accounting_method_only_alignment_borrow",
            "economics_accounting_confirmed_bridge_positioning",
            "economics_accounting_mixed_accounting_economics_bridge",
            "economics_accounting_near_miss_course_comparison",
            "economics_accounting_near_miss_internal_reporting_workflow",
        }
        self.assertTrue(required_economics_accounting_ids.issubset(set(ids)))
        economics_accounting_tags = {
            tag
            for case_id in required_economics_accounting_ids
            for tag in list(cases_by_id[case_id].tags or [])
        }
        self.assertTrue(
            {
                "clear_positive",
                "method_only_borrow",
                "mixed_subject",
                "near_miss",
                "confirmed_subject",
            }.issubset(economics_accounting_tags)
        )
```

- [ ] **Step 2: Add eval-ready gate test**

Add:

```python
    def test_economics_accounting_eval_ready_gate_passes_real_fixture_pack(
        self,
    ) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("economics-accounting", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "economics-accounting")
        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["decision_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["primary_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["suggest_subject_precision"], 1.0)
        self.assertEqual(report["metrics"]["forbidden_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["method_lens_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["all_case_checks_passed"], 1.0)
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)
```

- [ ] **Step 3: Add runtime-enabled guard**

Add:

```python
    def test_economics_accounting_runtime_enabled_gate_blocks_eval_ready_manifest(
        self,
    ) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report(
            "economics-accounting",
            cases,
            gate="runtime-enabled",
        )

        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])
```

- [ ] **Step 4: Update deferred candidate loop**

Change the deferred subject tuple in `test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons` to:

```python
        deferred_subjects = ()
```

- [ ] **Step 5: Update contract classification**

In `tests/test_subject_contracts.py`, classify economics-accounting as `eval_ready` and remove it from the deferred candidate shell set. Add a manifest test checking:

```python
contract.activation_status == "eval_ready"
contract.evaluation_pack == "tests/fixtures/subject_router_eval/economics-accounting"
contract.subject_skill == ""
```

The manifest test must also assert all four signal dimensions are populated, method lenses include `identification-measurement-alignment`, `fiscal-window-alignment`, and `economics-accounting-positioning`, and gate metrics remain `0.95`, `0.95`, and `0`.

- [ ] **Step 6: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_economics_accounting_eval_ready_gate_passes_real_fixture_pack -q
```

Expected before implementation:

```text
FAIL: activation_status is candidate
```

## Task 2: Add Economics-Accounting Runtime Manifest

**Files:**
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`

- [ ] **Step 1: Promote only to eval-ready**

Set:

```yaml
activation_status: eval_ready
subject_skill: ""
evaluation_pack: tests/fixtures/subject_router_eval/economics-accounting
```

- [ ] **Step 2: Add method-only signals and lenses**

Use method-only signals for:

- `identification-measurement-alignment`
- `fiscal-window-alignment`

Method lenses:

- `identification-measurement-alignment` -> `content/subjects/economics-accounting/overlays/skills/stats-engine.md`
- `fiscal-window-alignment` -> `content/subjects/economics-accounting/overlays/skills/stats-engine.md`
- `economics-accounting-positioning` -> `content/subjects/economics-accounting/overlays/skills/manuscript-architect.md`

- [ ] **Step 3: Add subject-level signals**

Subject-level signals must include:

- data/outcome: accounting disclosure rule, disclosure institution, fiscal-year timing, capital-market outcome windows, archival firm-year panel.
- theory/construct: economics and accounting contribution, accounting measurement and economic variation, cross-disciplinary contribution, composite reviewer risk.

Venue signals should include `Journal of Accounting Research`, `The Accounting Review`, and `Review of Accounting Studies`, but should not attach method lenses.

## Task 3: Add Economics-Accounting Fixtures

**Files:**
- Create: `tests/fixtures/subject_router_eval/economics-accounting/clear_identification_measurement_bridge.json`
- Create: `tests/fixtures/subject_router_eval/economics-accounting/method_only_alignment_borrow.json`
- Create: `tests/fixtures/subject_router_eval/economics-accounting/confirmed_bridge_positioning.json`
- Create: `tests/fixtures/subject_router_eval/economics-accounting/mixed_accounting_economics_bridge.json`
- Create: `tests/fixtures/subject_router_eval/economics-accounting/near_miss_course_comparison.json`
- Create: `tests/fixtures/subject_router_eval/economics-accounting/near_miss_internal_reporting_workflow.json`

- [ ] **Step 1: Add clear positive fixture**

Use Journal of Accounting Research, economics-and-accounting contribution, identification-measurement alignment, estimand measurement proxy map, identifying variation, accounting disclosure rule, disclosure institution, fiscal-year timing, and capital-market outcome windows. Under `eval-ready`, expect:

```json
{
  "decision": "recommend",
  "primary_subject": "economics-accounting",
  "suggest_subjects": ["economics-accounting"],
  "forbidden_subjects": [],
  "method_lenses": [
    "identification-measurement-alignment",
    "economics-accounting-positioning"
  ]
}
```

Default `expected` should suppress subject suggestion and only borrow bridge lenses until runtime activation.

- [ ] **Step 2: Add method-only borrow fixture**

Use only identification-measurement alignment and estimand measurement proxy map wording without subject-level data, venue, or theory signals. Expect primary `auto`, no economics-accounting suggestion, and method lens `identification-measurement-alignment`.

- [ ] **Step 3: Add confirmed-subject fixture**

Use `active_subject: "economics-accounting"` and `subject_mode: "confirmed"`. Expect primary `economics-accounting` and no inferred suggestion requirement.

- [ ] **Step 4: Add mixed-adjacent fixture**

Use an accounting venue and bridge-specific subject evidence, but avoid built-in economics method trigger phrases. Under `eval-ready`, expect economics-accounting primary. Allow accounting and economics only as neighbors if they appear.

- [ ] **Step 5: Add near-miss fixtures**

Course comparison and internal reporting workflow near misses must remain `core_only`, with `forbidden_subjects: ["economics-accounting"]`.

## Task 4: Verify And Update Roadmap

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Run GREEN checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_economics_accounting_eval_ready_gate_passes_real_fixture_pack -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject economics-accounting --gate eval-ready --json
```

Expected:

```text
eligible_for_eval_ready: true
blocking_failures: []
near_miss_false_positives: 0
```

- [ ] **Step 2: Run regressions**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval tests.test_subject_contracts -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --json
git diff --check
```

Expected: tests and full fixture evaluation exit 0; whitespace check exits 0.

- [ ] **Step 3: Commit by content**

Run:

```bash
git add content/subjects/economics-accounting/runtime-subject.yaml tests/test_subject_router_eval.py tests/test_subject_contracts.py tests/fixtures/subject_router_eval/economics-accounting
git commit -m "feat(subjects): mark economics-accounting eval ready"
git add docs/superpowers/plans/2026-07-06-economics-accounting-eval-ready.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record economics-accounting eval readiness"
```
