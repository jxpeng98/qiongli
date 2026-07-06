# Business Runtime Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote business from eval-ready to runtime-enabled while preserving method-only borrowed-lens behavior, mixed-subject precision, and practitioner near-miss safety.

**Architecture:** Treat activation as a small gated manifest-status change. First update tests and fixture gate expectations that encode the new runtime-enabled behavior, then flip the business manifest, update docs, and verify business/accounting/finance/economics gates together.

**Tech Stack:** Python 3, unittest/pytest, JSON router fixtures, YAML runtime subject manifests, Qiongli bridge modules, subject router evaluation tooling.

---

## File Map

- Modify: `content/subjects/business/runtime-subject.yaml`
  - Change business from `activation_status: eval_ready` to
    `activation_status: runtime_enabled`.
- Modify: `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`
  - Replace the old runtime-enabled suppressed expectation with the
    promotion-ready business-positive expectation.
- Modify: `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`
  - Replace the old runtime-enabled suppressed expectation with the
    promotion-ready business-positive expectation.
- Modify: `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`
  - Add a runtime-enabled expectation matching the existing promotion-ready
    business-positive expectation.
- Modify: `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`
  - Add a runtime-enabled expectation matching the existing promotion-ready
    business-positive expectation.
- Modify: `tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json`
  - Add a runtime-enabled expectation that keeps economics locked and allows
    business only as an accepted neighbor when the router suggests it.
- Modify: `tests/test_subject_contracts.py`
  - Expect business to be runtime-enabled and keep all resource/signal checks.
- Modify: `tests/test_subject_router_eval.py`
  - Replace real business eval-ready success assertions with runtime-enabled
    success assertions.
  - Add real-manifest assertions that eval-ready and promotion-ready gates now
    reject business because business is already runtime-enabled.
  - Keep patched eval-ready and promotion-ready tests for future candidates.
- Modify: `tests/test_subject_refinement.py`
  - Expect real business manifest default runtime to suggest business for clear
    scholarly business evidence.
  - Add or update method-only real-manifest business tests so Gioia/business
    method wording still borrows lenses without suggesting business.
- Modify: `docs/reference/cli.md`
  - Describe business as runtime-enabled after the activation PR and keep
    promotion-ready as the pre-activation gate for future subjects.
- Modify: `docs/advanced/publish-pypi.md`
  - Move business from eval-ready/promotion-ready review to runtime-enabled
    release verification.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark business runtime activation completed and move the next roadmap slot
    to Stage 5 explainability or the next subject spec.

## Execution Notes

- Execute from updated `dev` on a fresh branch, for example
  `feature/business-runtime-enabled`.
- Use subagent-driven development:
  - Task 1: fixture expectations and failing tests.
  - Task 2: manifest activation and runtime behavior.
  - Task 3: docs and roadmap.
  - Task 4: final verification and PR prep.
- Commit after each task with narrow Conventional Commit messages.
- Do not activate political economy, geoeconomics, or economics-accounting.
- Do not change provider, literature search, Zotero, full-text, or local-agent
  behavior.
- No router code changes are expected. If the runtime-enabled gate fails after
  manifest and fixture expectation updates, stop and report the measured
  failure before changing business signal weights or router thresholds.

## Task 1: Update Runtime Activation Tests And Fixtures

**Files:**
- Modify: `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`
- Modify: `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`
- Modify: `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`
- Modify: `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`
- Modify: `tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json`
- Modify: `tests/test_subject_contracts.py`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_refinement.py`

- [ ] **Step 1: Update clear management runtime-enabled fixture expectation**

In `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`,
replace `gate_expected["runtime-enabled"]` with:

```json
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning",
        "qualitative-transparency",
        "construct-level-fit"
      ]
    }
```

This block should match the existing `promotion-ready` expectation.

- [ ] **Step 2: Update clear marketing runtime-enabled fixture expectation**

In `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`,
replace `gate_expected["runtime-enabled"]` with:

```json
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning"
      ]
    }
```

This block should match the existing `promotion-ready` expectation.

- [ ] **Step 3: Add organization-panel runtime-enabled fixture expectation**

In `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`,
add this sibling of `promotion-ready` inside `gate_expected`:

```json
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning",
        "construct-level-fit"
      ]
    }
```

- [ ] **Step 4: Add strategic-management runtime-enabled fixture expectation**

In `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`,
add this sibling of `promotion-ready` inside `gate_expected`:

```json
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning"
      ]
    }
```

- [ ] **Step 5: Add locked economics runtime-enabled neighbor expectation**

In `tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json`,
add this sibling of `promotion-ready` inside `gate_expected`:

```json
    "runtime-enabled": {
      "decision": "keep_locked",
      "primary_subject": "economics",
      "suggest_subjects": [],
      "allowed_neighbor_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning"
      ]
    }
```

The locked primary subject must remain `economics`.

- [ ] **Step 6: Update business contract expectations**

In `tests/test_subject_contracts.py`, rename:

```python
    def test_business_eval_ready_manifest_declares_signals_and_method_lenses(self) -> None:
```

to:

```python
    def test_business_runtime_enabled_manifest_declares_signals_and_method_lenses(self) -> None:
```

Inside that test, update:

```python
        self.assertEqual(contract.activation_status, "runtime_enabled")
```

In `test_default_repository_contracts_classify_runtime_enabled_and_candidates`,
update the business assertion:

```python
        self.assertEqual(subject_activation_status("business", contracts), "runtime_enabled")
```

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py -q
```

Expected before Task 2: FAIL because `content/subjects/business/runtime-subject.yaml`
still says `eval_ready`.

- [ ] **Step 7: Update real business subject gate tests**

In `tests/test_subject_router_eval.py`, replace
`test_business_eval_ready_gate_passes_real_fixture_pack` with:

```python
    def test_business_runtime_enabled_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="runtime-enabled")

        self.assertEqual(report["subject"], "business")
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

Replace `test_business_runtime_enabled_gate_blocks_eval_ready_manifest` with two
real-manifest gate rejection tests:

```python
    def test_business_eval_ready_gate_blocks_runtime_enabled_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="eval-ready")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])

    def test_business_promotion_ready_gate_blocks_runtime_enabled_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="promotion-ready")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_runtime_promotion"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is runtime_enabled", report["blocking_failures"])
```

Keep the patched tests that prove `eval-ready` and `promotion-ready` gate
semantics for future eval-ready subjects.

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected before Task 2: FAIL because the real business manifest still reports
`activation_status: eval_ready`.

- [ ] **Step 8: Update business CLI gate test**

In `tests/test_subject_router_eval.py`, replace
`test_main_business_eval_ready_gate_json_has_consistent_thresholds` with:

```python
    def test_main_business_runtime_enabled_gate_json_has_consistent_thresholds(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "business", "--gate", "runtime-enabled", "--json"]
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertEqual(report["threshold_failures"], [])
        self.assertEqual(report["case_count"], report["subject_gate"]["case_count"])
        self.assertTrue(report["subject_gate"]["eligible_for_runtime_enabled"])
        self.assertFalse(report["subject_gate"]["eligible_for_runtime_promotion"])
```

Expected before Task 2: FAIL because business has not been activated.

- [ ] **Step 9: Update real business subject refinement expectations**

In `tests/test_subject_refinement.py`, replace
`test_business_eval_ready_real_manifest_does_not_activate_in_default_runtime`
with:

```python
    def test_runtime_enabled_business_real_manifest_suggests_business(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "management theory case study",
                "context": (
                    "Design a multiple case study using interviews with managers "
                    "to develop a management theory contribution for AMJ."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
```

Add this method-only real-manifest guard near the business runtime tests:

```python
    def test_runtime_enabled_business_method_only_real_manifest_borrows_lens(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "qualitative coding",
                "context": (
                    "Use the Gioia method with first-order concepts, "
                    "second-order themes, and aggregate dimensions to organize "
                    "qualitative coding."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("qualitative-transparency", packet["method_lenses"])
```

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected before Task 2: the clear-evidence runtime suggestion test fails
because business remains eval-ready.

- [ ] **Step 10: Commit Task 1**

```bash
git add \
  tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json \
  tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json \
  tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json \
  tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json \
  tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py
git commit -m "test(subjects): prepare business runtime activation expectations"
```

## Task 2: Activate Business Manifest

**Files:**
- Modify: `content/subjects/business/runtime-subject.yaml`

- [ ] **Step 1: Promote business activation status**

In `content/subjects/business/runtime-subject.yaml`, change:

```yaml
activation_status: eval_ready
```

to:

```yaml
activation_status: runtime_enabled
```

Do not change `overlay`, `subject_skill`, `signal_groups`, `method_lenses`, or
`activation_gate`.

- [ ] **Step 2: Run focused tests**

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected after the manifest change: PASS. If this command fails, inspect the
first failing business case and report whether the failure is a fixture
expectation mismatch or measured router over-activation. Do not change signal
weights in this task.

- [ ] **Step 3: Run business gate checks**

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json
```

Expected: exit 0, `eligible_for_runtime_enabled: true`, and
`near_miss_false_positives: 0`.

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate promotion-ready \
  --json
```

Expected: exit 1 with `activation_status is runtime_enabled`, because
promotion-ready is only for checked-in eval-ready subjects.

- [ ] **Step 4: Commit Task 2**

```bash
git add content/subjects/business/runtime-subject.yaml
git commit -m "feat(subjects): activate business runtime subject"
```

## Task 3: Update Documentation For Business Runtime Activation

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update CLI subject gate docs**

In `docs/reference/cli.md`, update the subject expansion gate section so:

````markdown
For the current business runtime check, use the runtime-enabled gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json
```
````

Keep the promotion-ready explanation, but describe it as the pre-activation
review gate for future eval-ready subjects.

- [ ] **Step 2: Update PyPI publish checklist**

In `docs/advanced/publish-pypi.md`, ensure the subject runtime gate checks
include:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json
```

Remove wording that says business should remain below runtime activation after
this activation PR.

- [ ] **Step 3: Update roadmap state**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`:

- add business to the runtime-enabled subjects list,
- remove business from the eval-ready subjects list,
- mark business runtime activation completed,
- set the next immediate plan to Stage 5 feedback-aware explainability unless
  the reviewer chooses the next subject expansion spec.

Use this replacement shape for the immediate plan:

```markdown
1. Continue Stage 5 feedback-aware explainability so router outputs separate
   task-text, manifest, trace-memory, and user-action evidence more clearly.
2. Keep political economy, geoeconomics, and economics-accounting as separate
   follow-up subject specs with their own fixture packs and gate criteria.
3. Keep local-agent runtime execution opt-in until maintainer smoke
   environments are stable.
```

- [ ] **Step 4: Run documentation checks**

```bash
rg -n "business|promotion-ready|runtime-enabled|eval-ready|eligible_for_runtime" \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md

git diff --check
```

Expected: references distinguish runtime-enabled business from promotion-ready
future-subject review, and whitespace check exits 0.

- [ ] **Step 5: Commit Task 3**

```bash
git add \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(subjects): document business runtime activation"
```

## Task 4: Final Verification And PR Prep

**Files:**
- No source edits expected.

- [ ] **Step 1: Run focused test suite**

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected: all tests pass.

- [ ] **Step 2: Run default router eval**

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit 0, `threshold_failures: []`, and `near_miss_false_positives: 0`.

- [ ] **Step 3: Run runtime-enabled subject gates**

```bash
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject finance --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject economics --gate runtime-enabled --json
```

Expected: each command exits 0 and reports `eligible_for_runtime_enabled: true`.

- [ ] **Step 4: Verify promotion-ready now blocks business**

```bash
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate promotion-ready --json
```

Expected: exit 1 with `activation_status is runtime_enabled`.

- [ ] **Step 5: Run repository hygiene checks**

```bash
git diff --check
git status --short --branch
```

Expected: whitespace check exits 0 and status shows a clean feature branch.

- [ ] **Step 6: Summarize PR readiness**

Prepare a summary with:

- commits created,
- test commands and results,
- business runtime-enabled gate result,
- confirmation that promotion-ready no longer applies to business after
  activation,
- residual risk that business vocabulary is broader than accounting and should
  be monitored through Stage 5 explainability.
