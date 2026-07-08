# Political Economy Eval-Ready Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move political economy from a deferred candidate shell to `eval_ready` with a subject-owned fixture pack, manifest-backed signals, and gate coverage, while keeping it unavailable for default runtime activation.

**Architecture:** Reuse the existing runtime subject contract, manifest-backed subject router, and `eval-ready` gate. Political economy becomes measurable only when the evaluation runner passes `evaluation_subjects=["political-economy"]`; default runtime suggestions still require a later `runtime_enabled` promotion.

**Tech Stack:** Python 3, unittest, PyYAML runtime manifests, JSON subject router fixtures, Qiongli bridge modules, local subject router evaluation script.

---

## File Map

- Modify: `tests/test_subject_router_eval.py`
  - Add a real political-economy eval-ready gate test.
  - Keep runtime-enabled blocked while the subject is only `eval_ready`.
  - Remove political economy from the deferred-shell blocker loop.
- Modify: `content/subjects/political-economy/runtime-subject.yaml`
  - Set `activation_status: eval_ready`.
  - Add subject-owned `subject_skill`.
  - Add method, data/outcome, venue, and theory/construct signal groups.
  - Add method lenses and a subject-specific `evaluation_pack`.
- Create: `tests/fixtures/subject_router_eval/political-economy/*.json`
  - Add clear positive, method-only borrow, confirmed-subject, mixed-adjacent, and near-miss fixtures.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record political economy as the current eval-ready Stage 4 subject.
  - Keep geoeconomics and economics-accounting as deferred candidates.

## Execution Notes

- Do not promote political economy to `runtime_enabled`.
- Do not change finance, economics, accounting, or business routing behavior.
- Do not change provider configuration, local-agent execution, release automation, or plugin packaging.
- Group commits by content:
  - `test(subject-router): cover political economy eval readiness`
  - `feat(subjects): mark political economy eval ready`
  - `docs(roadmap): record political economy eval readiness`

## Task 1: Add Political Economy Gate Tests

**Files:**
- Modify: `tests/test_subject_router_eval.py`

- [ ] **Step 1: Write the failing eval-ready gate test**

Add this test next to the existing accounting, business, and economics real fixture pack gate tests:

```python
    def test_political_economy_eval_ready_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("political-economy", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "political-economy")
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

- [ ] **Step 2: Write the failing runtime guard test**

Add this test near the business eval-ready/runtime blocking tests:

```python
    def test_political_economy_runtime_enabled_gate_blocks_eval_ready_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report(
            "political-economy",
            cases,
            gate="runtime-enabled",
        )

        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])
```

- [ ] **Step 3: Keep only true deferred shells in the candidate blocker test**

Change the `deferred_subjects` tuple in `test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons` to:

```python
        deferred_subjects = (
            "geoeconomics",
            "economics-accounting",
        )
```

- [ ] **Step 4: Run the targeted RED check**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_political_economy_eval_ready_gate_passes_real_fixture_pack -q
```

Expected before implementation:

```text
FAIL: activation_status is candidate
```

## Task 2: Add Political Economy Runtime Manifest Resources

**Files:**
- Modify: `content/subjects/political-economy/runtime-subject.yaml`

- [ ] **Step 1: Promote only to eval-ready**

Set:

```yaml
activation_status: eval_ready
subject_skill: content/subjects/political-economy/skills/political-economy-mechanism-auditor.md
evaluation_pack: tests/fixtures/subject_router_eval/political-economy
```

- [ ] **Step 2: Add method-only signals and lenses**

Add method signals whose activation is `method_only`:

```yaml
method:
  - id: political-economy.method.process-tracing
    value: process-tracing
    activation: method_only
    method_lenses:
      - political-mechanism-audit
  - id: political-economy.method.comparative-case
    value: comparative-case
    activation: method_only
    method_lenses:
      - comparative-institutional-design
```

Add method lenses:

```yaml
method_lenses:
  political-mechanism-audit:
    resource: content/subjects/political-economy/skills/political-economy-mechanism-auditor.md
    activation: method_only
  comparative-institutional-design:
    resource: content/subjects/political-economy/overlays/skills/study-designer.md
    activation: method_only
  political-economy-positioning:
    resource: content/subjects/political-economy/overlays/skills/manuscript-architect.md
    activation: method_only
```

- [ ] **Step 3: Add subject-level signals**

Add at least one `subject` signal in `data_or_outcome` and at least one in `theory_or_construct`, plus venue `context_only` signals. Positive fixtures must hit at least two dimensions and one subject-level dimension.

- [ ] **Step 4: Keep runtime disabled**

Do not set `activation_status: runtime_enabled`.

## Task 3: Add Subject-Owned Fixtures

**Files:**
- Create: `tests/fixtures/subject_router_eval/political-economy/clear_actor_institution_outcome.json`
- Create: `tests/fixtures/subject_router_eval/political-economy/method_only_process_tracing_borrow.json`
- Create: `tests/fixtures/subject_router_eval/political-economy/confirmed_political_economy_mechanism_audit.json`
- Create: `tests/fixtures/subject_router_eval/political-economy/mixed_finance_distributional_conflict.json`
- Create: `tests/fixtures/subject_router_eval/political-economy/near_miss_policy_brief.json`
- Create: `tests/fixtures/subject_router_eval/political-economy/near_miss_campaign_strategy.json`

- [ ] **Step 1: Add clear positive fixture**

Expected:

```json
{
  "decision": "recommend",
  "primary_subject": "political-economy",
  "suggest_subjects": ["political-economy"],
  "forbidden_subjects": [],
  "method_lenses": ["political-mechanism-audit", "political-economy-positioning"]
}
```

- [ ] **Step 2: Add method-only borrow fixture**

Expected:

```json
{
  "decision": "recommend",
  "primary_subject": "auto",
  "suggest_subjects": [],
  "forbidden_subjects": ["political-economy"],
  "method_lenses": ["political-mechanism-audit"]
}
```

- [ ] **Step 3: Add confirmed-subject fixture**

Use `active_subject: "political-economy"` and `subject_mode: "confirmed"`.

Expected:

```json
{
  "decision": "confirm_subject",
  "primary_subject": "political-economy",
  "suggest_subjects": [],
  "forbidden_subjects": [],
  "method_lenses": []
}
```

- [ ] **Step 4: Add adjacent and near-miss guards**

Near misses must stay `core_only` with `forbidden_subjects: ["political-economy"]`.
The mixed finance case may mention market returns, but it must still include enough political economy subject-level signal to keep the expected primary subject as `political-economy` under the eval-ready gate.

## Task 4: Verify Gate And Update Roadmap

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Run targeted GREEN checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval.SubjectRouterEvalTests.test_political_economy_eval_ready_gate_passes_real_fixture_pack -q
.venv/bin/python tooling/scripts/evaluate_subject_router.py --subject political-economy --gate eval-ready --json
```

Expected:

```text
eligible_for_eval_ready: true
blocking_failures: []
near_miss_false_positives: 0
```

- [ ] **Step 2: Run related regression checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_router_eval tests.test_subject_contracts -q
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 3: Update Stage 4 roadmap**

Record:

- political economy is now `eval_ready`.
- runtime activation remains a later promotion-ready/runtime-enabled follow-up.
- geoeconomics and economics-accounting remain deferred candidate shells.

- [ ] **Step 4: Commit by content**

Run:

```bash
git add tests/test_subject_router_eval.py tests/fixtures/subject_router_eval/political-economy
git commit -m "test(subject-router): cover political economy eval readiness"

git add content/subjects/political-economy/runtime-subject.yaml
git commit -m "feat(subjects): mark political economy eval ready"

git add docs/superpowers/plans/2026-07-06-political-economy-eval-ready.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record political economy eval readiness"
```
