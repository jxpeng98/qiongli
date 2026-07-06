# Geoeconomics Eval-Ready Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move geoeconomics from a deferred candidate shell to `eval_ready` with a subject-owned fixture pack, manifest-backed signals, method-only borrowing, mixed-adjacent coverage, and near-miss guards.

**Architecture:** Reuse the existing runtime subject contract and subject router evaluation gate. Geoeconomics should be measurable only under the `eval-ready` gate through `evaluation_subjects=["geoeconomics"]`; default runtime behavior remains suppressed until a later promotion-ready/runtime-enabled review.

**Tech Stack:** Python 3, unittest, JSON subject router fixtures, YAML runtime subject manifests, Qiongli subject router evaluation tooling.

---

## File Map

- Modify: `tests/test_subject_router_eval.py`
  - Add geoeconomics fixture inventory coverage.
  - Add a real fixture-pack eval-ready gate test.
  - Add a runtime-enabled guard proving checked-in `eval_ready` is not default activation.
  - Remove geoeconomics from the deferred-shell candidate loop.
- Modify: `tests/test_subject_contracts.py`
  - Classify geoeconomics as `eval_ready` instead of a deferred candidate.
  - Add manifest structure checks for signal groups, method lenses, subject skill, evaluation pack, and metrics.
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`
  - Set `activation_status: eval_ready`.
  - Add geoeconomics `subject_skill`.
  - Add method, data/outcome, venue, and theory/construct signal groups.
  - Add method lenses and `evaluation_pack: tests/fixtures/subject_router_eval/geoeconomics`.
- Create: `tests/fixtures/subject_router_eval/geoeconomics/*.json`
  - Add clear positive, method-only borrow, confirmed-subject, mixed-adjacent, and near-miss fixtures.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record geoeconomics as eval-ready.
  - Leave economics-accounting as the remaining deferred candidate.

## Execution Notes

- Do not promote geoeconomics to `runtime_enabled` in this slice.
- Do not change finance, economics, accounting, business, or political-economy routing behavior.
- Avoid triggering built-in finance/economics priority in geoeconomics fixtures unless the fixture explicitly treats that as an accepted adjacent subject.
- Do not change provider configuration, local-agent execution, packaging, or release automation.
- Group commits by content:
  - `feat(subjects): mark geoeconomics eval ready`
  - `docs(roadmap): record geoeconomics eval readiness`

## Task 1: Add Geoeconomics Gate Tests

**Files:**
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Add fixture inventory expectations**

Add this `required_geoeconomics_ids` block next to the existing political-economy inventory block:

```python
        required_geoeconomics_ids = {
            "geoeconomics_clear_sanctions_statecraft",
            "geoeconomics_method_only_statecraft_borrow",
            "geoeconomics_confirmed_statecraft_audit",
            "geoeconomics_mixed_finance_supply_chain_exposure",
            "geoeconomics_near_miss_supply_chain_ops",
            "geoeconomics_near_miss_geopolitics_brief",
        }
        self.assertTrue(required_geoeconomics_ids.issubset(set(ids)))
        geoeconomics_tags = {
            tag
            for case_id in required_geoeconomics_ids
            for tag in list(cases_by_id[case_id].tags or [])
        }
        self.assertTrue(
            {
                "clear_positive",
                "method_only_borrow",
                "mixed_subject",
                "near_miss",
                "confirmed_subject",
            }.issubset(geoeconomics_tags)
        )
```

- [ ] **Step 2: Add eval-ready gate test**

Add near the political-economy gate tests:

```python
    def test_geoeconomics_eval_ready_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("geoeconomics", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "geoeconomics")
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
    def test_geoeconomics_runtime_enabled_gate_blocks_eval_ready_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("geoeconomics", cases, gate="runtime-enabled")

        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])
```

- [ ] **Step 4: Update deferred candidate loop**

Change the deferred subject tuple in `test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons` to contain only:

```python
        deferred_subjects = (
            "economics-accounting",
        )
```

- [ ] **Step 5: Update contract classification**

In `tests/test_subject_contracts.py`, classify geoeconomics as `eval_ready` and remove it from the deferred candidate shell set. Add a geoeconomics manifest test equivalent to the political-economy manifest test but checking:

```python
contract.activation_status == "eval_ready"
contract.evaluation_pack == "tests/fixtures/subject_router_eval/geoeconomics"
contract.subject_skill == "content/subjects/geoeconomics/skills/geoeconomic-statecraft-auditor.md"
```

- [ ] **Step 6: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_geoeconomics_eval_ready_gate_passes_real_fixture_pack -q
```

Expected before implementation:

```text
FAIL: activation_status is candidate
```

## Task 2: Add Geoeconomics Runtime Manifest

**Files:**
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`

- [ ] **Step 1: Promote only to eval-ready**

Set:

```yaml
activation_status: eval_ready
subject_skill: content/subjects/geoeconomics/skills/geoeconomic-statecraft-auditor.md
evaluation_pack: tests/fixtures/subject_router_eval/geoeconomics
```

- [ ] **Step 2: Add method-only signals and lenses**

Use method-only signals for:

- `statecraft-instrument-audit`
- `supply-chain-exposure-design`

Method lenses:

- `geoeconomic-statecraft-audit` -> `content/subjects/geoeconomics/skills/geoeconomic-statecraft-auditor.md`
- `supply-chain-exposure-design` -> `content/subjects/geoeconomics/overlays/skills/study-designer.md`
- `geoeconomic-positioning` -> `content/subjects/geoeconomics/overlays/skills/manuscript-architect.md`

- [ ] **Step 3: Add subject-level signals**

Subject-level signals must include at least:

- data/outcome: sanctions/export-control response, supply-chain exposure, target response, substitution/evasion.
- theory/construct: sender-target-instrument logic, strategic competition, economic statecraft mechanism.

Venue signals should include `International Security`, `International Organization`, `Review of International Political Economy`, and `World Politics`.

## Task 3: Add Geoeconomics Fixtures

**Files:**
- Create: `tests/fixtures/subject_router_eval/geoeconomics/clear_sanctions_statecraft.json`
- Create: `tests/fixtures/subject_router_eval/geoeconomics/method_only_statecraft_borrow.json`
- Create: `tests/fixtures/subject_router_eval/geoeconomics/confirmed_statecraft_audit.json`
- Create: `tests/fixtures/subject_router_eval/geoeconomics/mixed_finance_supply_chain_exposure.json`
- Create: `tests/fixtures/subject_router_eval/geoeconomics/near_miss_supply_chain_ops.json`
- Create: `tests/fixtures/subject_router_eval/geoeconomics/near_miss_geopolitics_brief.json`

- [ ] **Step 1: Add clear positive fixture**

Use sender-target-instrument, sanctions/export controls, strategic objective, target response, and an international security venue. Under `eval-ready`, expect:

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

Default `expected` should suppress subject suggestion and only borrow method lenses until runtime activation.

- [ ] **Step 2: Add method-only borrow fixture**

Use economic statecraft instrument-audit wording without subject-level sanctions, target-response, or strategic-competition signals. Expect primary `auto`, no geoeconomics suggestion, and method lens `geoeconomic-statecraft-audit`.

- [ ] **Step 3: Add confirmed-subject fixture**

Use `active_subject: "geoeconomics"` and `subject_mode: "confirmed"`. Expect primary `geoeconomics` and no inferred suggestion requirement.

- [ ] **Step 4: Add mixed-adjacent fixture**

Mention finance adjacency only as market reaction or exposure, not enough to make finance primary. Under `eval-ready`, expect geoeconomics primary and allow finance as an adjacent subject if it appears.

- [ ] **Step 5: Add near-miss fixtures**

Supply-chain operations and geopolitics briefing near misses must remain `core_only`, with `forbidden_subjects: ["geoeconomics"]`.

## Task 4: Verify And Update Roadmap

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Run GREEN checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_geoeconomics_eval_ready_gate_passes_real_fixture_pack -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject geoeconomics --gate eval-ready --json
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
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 3: Update roadmap**

Record geoeconomics as eval-ready and leave economics-accounting as the only deferred candidate subject.

- [ ] **Step 4: Commit by content**

Run:

```bash
git add content/subjects/geoeconomics/runtime-subject.yaml tests/test_subject_router_eval.py tests/test_subject_contracts.py tests/fixtures/subject_router_eval/geoeconomics
git commit -m "feat(subjects): mark geoeconomics eval ready"

git add docs/superpowers/plans/2026-07-06-geoeconomics-eval-ready.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record geoeconomics eval readiness"
```
