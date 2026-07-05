# Business Runtime Promotion Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a promotion-ready gate that proves business default runtime routing would be precise under simulated runtime activation, while keeping the checked-in business manifest at `eval_ready`.

**Architecture:** Extend the subject router evaluation harness with a `promotion-ready` gate and a test-only activation-status override that flows through subject refinement. Then expand business runtime resources and fixtures so the gate can pass without changing `activation_status`. Documentation updates explain eval-ready, promotion-ready, and runtime-enabled as separate states.

**Tech Stack:** Python 3, pytest/unittest, dataclass runtime subject contracts, JSON subject-router fixtures, YAML subject manifests, Qiongli bridge modules.

---

## File Map

- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Add optional activation-status override plumbing for evaluation tooling.
  - Keep normal runtime behavior unchanged when no override is passed.
- Modify: `tooling/scripts/evaluate_subject_router.py`
  - Add `promotion-ready` as a gate.
  - Evaluate subject-scoped reports with runtime activation overrides for promotion-ready only.
  - Report `eligible_for_runtime_promotion`.
- Modify: `tests/test_subject_refinement.py`
  - Add override tests proving business can be measured as runtime-enabled without changing default behavior.
- Modify: `tests/test_subject_router_eval.py`
  - Add promotion-ready gate tests and business fixture inventory coverage.
- Modify: `content/subjects/business/runtime-subject.yaml`
  - Keep `activation_status: eval_ready`.
  - Set `subject_skill` to the existing business journal positioning auditor.
- Add: `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`
- Add: `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`
- Add: `tests/fixtures/subject_router_eval/business/near_miss_customer_segmentation_sales_forecast.json`
- Add: `tests/fixtures/subject_router_eval/business/near_miss_strategy_competitive_advantage_memo.json`
- Modify: existing files under `tests/fixtures/subject_router_eval/business/*.json`
  - Add `promotion-ready` gate expectations where default expectations differ.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark business eval-ready complete and promotion readiness as the active Stage 4 work.
- Modify: `docs/reference/cli.md`
  - Document `promotion-ready`.
- Modify: `docs/advanced/publish-pypi.md`
  - Add optional business promotion-ready gate verification.

## Execution Notes

- Work on branch `feature/business-runtime-promotion-readiness`.
- Commit after each task.
- Do not change `content/subjects/business/runtime-subject.yaml` to `runtime_enabled`.
- Do not change provider configuration, Zotero behavior, full-text retrieval, local-agent execution, or release automation.
- Do not broaden accounting, finance, or economics signals.
- Keep all new fixtures deterministic and local.

## Task 1: Add Promotion-Ready Gate Plumbing

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `tooling/scripts/evaluate_subject_router.py`
- Modify: `tests/test_subject_refinement.py`
- Modify: `tests/test_subject_router_eval.py`

- [ ] **Step 1: Add failing subject refinement override tests**

In `tests/test_subject_refinement.py`, add these tests near the existing business eval-ready tests:

```python
    def test_business_activation_override_measures_default_runtime_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
                activation_status_overrides={"business": "runtime_enabled"},
            ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertEqual(packet["domain"], "business-management")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_business_without_activation_override_remains_suppressed_by_default(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertNotEqual(packet["decision"], "suggest_subject")
        self.assertNotEqual(packet["primary_subject"], "business")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
```

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected: the first new test fails because `infer_subject_refinement()` does not accept `activation_status_overrides`.

- [ ] **Step 2: Implement activation override plumbing**

In `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`, update the public function signature:

```python
def infer_subject_refinement(
    task_packet: Mapping[str, Any],
    *,
    manifest_state: ProjectManifestState | ProjectManifest | Mapping[str, Any],
    draft_content: str = "",
    review_content: str = "",
    merged_analysis: str = "",
    standards_dir: str | Path | None = None,
    evaluation_subjects: set[str] | None = None,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> SubjectRefinementPacket:
```

At the start of the function after `text` is collected, normalize overrides and pass them into signal detection:

```python
    activation_status_overrides = {
        str(subject): str(status)
        for subject, status in dict(activation_status_overrides or {}).items()
        if isinstance(subject, str) and isinstance(status, str)
    }
    signals = _detect_signals(
        text,
        activation_status_overrides=activation_status_overrides,
    )
```

Update `_detect_signals()` and `_detect_manifest_signal_records()` signatures:

```python
def _detect_signals(
    text: str,
    *,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> SubjectSignals:
```

```python
def _detect_manifest_signal_records(
    text: str,
    *,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> tuple[list[dict[str, Any]], dict[str, RuntimeSubjectMatch], list[str]]:
```

Inside `_detect_manifest_signal_records()`, compute the match status with the override:

```python
        activation_status = (
            activation_status_overrides[subject]
            if activation_status_overrides is not None
            and subject in activation_status_overrides
            else contract.activation_status
        )
```

Use that value when creating `RuntimeSubjectMatch`:

```python
            activation_status=activation_status,
```

Update `_subject_can_be_suggested()`:

```python
def _subject_can_be_suggested(
    subject: str,
    *,
    evaluation_subjects: set[str] | None = None,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> bool:
    if evaluation_subjects and subject in evaluation_subjects:
        return True
    return (
        _runtime_activation_status(
            subject,
            activation_status_overrides=activation_status_overrides,
        )
        == "runtime_enabled"
    )
```

Update `_runtime_activation_status()`:

```python
def _runtime_activation_status(
    subject: str,
    *,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> str:
    if activation_status_overrides and subject in activation_status_overrides:
        return activation_status_overrides[subject]
    try:
        return subject_activation_status(subject)
    except Exception:
        if subject in {"economics", "finance"}:
            return "runtime_enabled"
        return "candidate"
```

Thread `activation_status_overrides=activation_status_overrides` through calls to:

```python
_subject_can_be_suggested(...)
_candidate_subjects(...)
_runtime_subject_suggestion_match(...)
```

Update those helper signatures and their internal `_subject_can_be_suggested()` calls. The important behavior is:

```python
_candidate_subjects(
    signals,
    preferred=subject,
    evaluation_subjects=evaluation_subjects,
    activation_status_overrides=activation_status_overrides,
)
```

and:

```python
runtime_subject_match = _runtime_subject_suggestion_match(
    signals,
    evaluation_subjects=evaluation_subjects,
    activation_status_overrides=activation_status_overrides,
)
```

- [ ] **Step 3: Run subject refinement tests**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected: all tests in `tests/test_subject_refinement.py` pass.

- [ ] **Step 4: Add failing promotion-ready gate tests**

In `tests/test_subject_router_eval.py`, add a promotion-ready CLI test near the existing subject gate CLI tests:

```python
    def test_main_business_promotion_ready_gate_json_has_consistent_thresholds(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "business", "--gate", "promotion-ready", "--json"]
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertEqual(report["threshold_failures"], [])
        self.assertEqual(report["case_count"], report["subject_gate"]["case_count"])
        self.assertFalse(report["subject_gate"]["eligible_for_eval_ready"])
        self.assertTrue(report["subject_gate"]["eligible_for_runtime_promotion"])
        self.assertFalse(report["subject_gate"]["eligible_for_runtime_enabled"])
```

Add this real gate report test:

```python
    def test_business_promotion_ready_gate_requires_eval_ready_status(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="promotion-ready")

        self.assertEqual(report["subject"], "business")
        self.assertEqual(report["gate"], "promotion-ready")
        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_runtime_promotion"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
```

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected: tests fail because `promotion-ready` is not an accepted gate.

- [ ] **Step 5: Implement promotion-ready gate in evaluation runner**

In `tooling/scripts/evaluate_subject_router.py`, update gate constants:

```python
GATE_CHOICES = ("runtime-enabled", "eval-ready", "promotion-ready")
GATE_ELIGIBILITY_KEYS = {
    "runtime-enabled": "eligible_for_runtime_enabled",
    "eval-ready": "eligible_for_eval_ready",
    "promotion-ready": "eligible_for_runtime_promotion",
}
```

Add a helper:

```python
def _activation_status_overrides_for_gate(
    subject: str,
    gate: str,
) -> dict[str, str] | None:
    if gate == "promotion-ready":
        return {subject: "runtime_enabled"}
    return None
```

Update `run_eval_case()`, `evaluate_cases()`, and `_infer_subject_refinement()` to accept and pass `activation_status_overrides`.

Use this shape in `_infer_subject_refinement()`:

```python
def _infer_subject_refinement(
    task_packet: Mapping[str, Any],
    *,
    manifest_state: ProjectManifest,
    evaluation_subjects: list[str] | None = None,
    activation_status_overrides: Mapping[str, str] | None = None,
) -> Any:
    kwargs: dict[str, Any] = {"manifest_state": manifest_state}
    if evaluation_subjects and _router_accepts_evaluation_subjects():
        kwargs["evaluation_subjects"] = list(evaluation_subjects)
    if activation_status_overrides and _router_accepts_activation_status_overrides():
        kwargs["activation_status_overrides"] = dict(activation_status_overrides)
    return infer_subject_refinement(task_packet, **kwargs)
```

Add `_router_accepts_activation_status_overrides()` mirroring `_router_accepts_evaluation_subjects()`:

```python
def _router_accepts_activation_status_overrides() -> bool:
    try:
        parameters = inspect.signature(infer_subject_refinement).parameters.values()
    except (TypeError, ValueError):
        return False
    return any(
        parameter.kind == inspect.Parameter.VAR_KEYWORD
        or parameter.name == "activation_status_overrides"
        for parameter in parameters
    )
```

Update `_evaluation_subjects_for_gate()` so only eval-ready uses eval subjects:

```python
def _evaluation_subjects_for_gate(subject: str, gate: str) -> list[str] | None:
    return [subject] if gate == "eval-ready" else None
```

In `subject_gate_report()`, compute:

```python
    activation_status_overrides = _activation_status_overrides_for_gate(subject, gate)
```

Pass it into `evaluate_cases()`.

Add promotion-ready blockers:

```python
    if gate == "promotion-ready":
        if activation_status != "eval_ready":
            blocking_failures.append(f"activation_status is {activation_status}")
        if contract is not None:
            blocking_failures.extend(_missing_resource_failures(contract))
            blocking_failures.extend(_missing_signal_dimension_failures(contract))
```

Return `eligible_for_runtime_promotion`:

```python
        "eligible_for_eval_ready": gate == "eval-ready" and eligible,
        "eligible_for_runtime_promotion": gate == "promotion-ready" and eligible,
        "eligible_for_runtime_enabled": gate == "runtime-enabled" and eligible,
```

In `main()`, pass both gate helpers into top-level subject-scoped `evaluate_cases()`.

- [ ] **Step 6: Run gate plumbing tests**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py tests/test_subject_router_eval.py -q
```

Expected: tests that rely on real business fixtures may still fail until Task 2 adds promotion-ready fixture expectations and business subject skill. Override unit tests should pass.

- [ ] **Step 7: Commit Task 1**

Stage only Task 1 files:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_refinement.py tooling/scripts/evaluate_subject_router.py tests/test_subject_refinement.py tests/test_subject_router_eval.py
git commit -m "feat(subjects): add promotion-ready gate plumbing"
```

## Task 2: Expand Business Promotion Readiness Fixtures

**Files:**
- Modify: `content/subjects/business/runtime-subject.yaml`
- Modify: `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`
- Modify: `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`
- Modify: `tests/fixtures/subject_router_eval/business/confirmed_business_journal_positioning.json`
- Create: `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`
- Create: `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_customer_segmentation_sales_forecast.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_strategy_competitive_advantage_memo.json`
- Modify: `tests/test_subject_contracts.py`
- Modify: `tests/test_subject_router_eval.py`

- [ ] **Step 1: Add runtime resource metadata without activation**

In `content/subjects/business/runtime-subject.yaml`, keep:

```yaml
activation_status: eval_ready
```

Change:

```yaml
subject_skill: ""
```

to:

```yaml
subject_skill: content/subjects/business/skills/business-journal-positioning-auditor.md
```

Keep:

```yaml
overlay: ""
```

- [ ] **Step 2: Add promotion-ready expectations to existing clear fixtures**

In `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`, add a `promotion-ready` block inside `gate_expected` with the same expected outcome as `eval-ready`:

```json
    "promotion-ready": {
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

In `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`, add:

```json
    "promotion-ready": {
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

In `tests/fixtures/subject_router_eval/business/confirmed_business_journal_positioning.json`, add:

```json
    "promotion-ready": {
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

- [ ] **Step 3: Add non-qualitative positive fixture**

Create `tests/fixtures/subject_router_eval/business/clear_organization_panel_manager_survey.json`:

```json
{
  "id": "business_clear_organization_panel_manager_survey",
  "subject_under_test": "business",
  "tags": [
    "business",
    "clear_positive"
  ],
  "description": "Quantitative organization-level business study with manager survey, construct, and venue signals.",
  "request": "Design a Journal of Management manuscript using organization-level data, firm-level panel measures, manager survey evidence, construct clarity, and boundary conditions to build a management theory contribution.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "auto",
    "suggest_subjects": [],
    "forbidden_subjects": [
      "business"
    ],
    "method_lenses": [
      "business-positioning",
      "construct-level-fit"
    ]
  },
  "gate_expected": {
    "eval-ready": {
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
    },
    "promotion-ready": {
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
  }
}
```

- [ ] **Step 4: Add strategic management positive fixture**

Create `tests/fixtures/subject_router_eval/business/clear_strategic_management_capabilities.json`:

```json
{
  "id": "business_clear_strategic_management_capabilities",
  "subject_under_test": "business",
  "tags": [
    "business",
    "clear_positive"
  ],
  "description": "Strategic management manuscript with dynamic capabilities, sustained competitive advantage, venue, and theory contribution signals.",
  "request": "Frame a Strategic Management Journal manuscript on dynamic capabilities, sustained competitive advantage, organizational routines, and strategy mechanism as a management theory contribution with explicit boundary conditions.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "auto",
    "suggest_subjects": [],
    "forbidden_subjects": [
      "business"
    ],
    "method_lenses": [
      "business-positioning"
    ]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": [
        "business"
      ],
      "forbidden_subjects": [],
      "method_lenses": [
        "business-positioning"
      ]
    },
    "promotion-ready": {
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
  }
}
```

- [ ] **Step 5: Add practitioner customer and sales near-miss fixture**

Create `tests/fixtures/subject_router_eval/business/near_miss_customer_segmentation_sales_forecast.json`:

```json
{
  "id": "business_near_miss_customer_segmentation_sales_forecast",
  "subject_under_test": "business",
  "tags": [
    "business",
    "near_miss"
  ],
  "description": "Practitioner customer segmentation, journey, and sales forecast wording must stay core.",
  "request": "Create customer segmentation bullets, improve the customer journey, forecast sales, and write launch messaging for a product marketing team.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "core_only",
    "primary_subject": "core",
    "suggest_subjects": [],
    "forbidden_subjects": [
      "business"
    ],
    "method_lenses": []
  }
}
```

- [ ] **Step 6: Add practitioner strategy near-miss fixture**

Create `tests/fixtures/subject_router_eval/business/near_miss_strategy_competitive_advantage_memo.json`:

```json
{
  "id": "business_near_miss_strategy_competitive_advantage_memo",
  "subject_under_test": "business",
  "tags": [
    "business",
    "near_miss"
  ],
  "description": "Practitioner strategy memo with competitive advantage and capabilities wording must stay core.",
  "request": "Write a strategic management memo for executives about competitive advantage, operational capabilities, market positioning, and a quarterly action plan.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "core_only",
    "primary_subject": "core",
    "suggest_subjects": [],
    "forbidden_subjects": [
      "business"
    ],
    "method_lenses": []
  }
}
```

- [ ] **Step 7: Register fixture inventory and subject skill tests**

In `tests/test_subject_router_eval.py`, extend `required_business_ids` with:

```python
            "business_clear_organization_panel_manager_survey",
            "business_clear_strategic_management_capabilities",
            "business_near_miss_customer_segmentation_sales_forecast",
            "business_near_miss_strategy_competitive_advantage_memo",
```

In `tests/test_subject_contracts.py`, add or update the business eval-ready assertions so they include:

```python
self.assertEqual(
    business.subject_skill,
    "content/subjects/business/skills/business-journal-positioning-auditor.md",
)
self.assertEqual(business.activation_status, "eval_ready")
```

- [ ] **Step 8: Run promotion-ready gate and fixture tests**

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py tests/test_subject_router_eval.py tests/test_subject_refinement.py -q
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate eval-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate promotion-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate runtime-enabled --json
```

Expected:

- pytest passes,
- business eval-ready exits 0,
- business promotion-ready exits 0,
- business runtime-enabled exits 1 with `activation_status is eval_ready`,
- business runtime-enabled JSON still has subject-scoped metrics at 1.0.

- [ ] **Step 9: Commit Task 2**

Stage only Task 2 files:

```bash
git add content/subjects/business/runtime-subject.yaml tests/fixtures/subject_router_eval/business tests/test_subject_contracts.py tests/test_subject_router_eval.py
git commit -m "test(subjects): expand business promotion readiness pack"
```

## Task 3: Document Promotion-Ready Workflow

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/advanced/publish-pypi.md`

- [ ] **Step 1: Update roadmap priority status**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`, change the priority update so it states:

```markdown
Status: the subject expansion onboarding contract is complete, business is
eval-ready, and the current Stage 4 slice is business runtime promotion
readiness.
```

Add a short paragraph:

```markdown
The readiness slice adds a `promotion-ready` gate. It simulates business as
runtime-enabled for subject-router evaluation only, keeps the manifest at
`eval_ready`, and blocks actual runtime activation until a separate promotion
PR changes `activation_status`.
```

In the Stage 4 section, add `promotion-ready` to the business status:

```markdown
Eval-ready subjects:

- Business and management, with promotion-ready gate coverage.
```

Update the recommended immediate plan so completed items are no longer listed
as next actions:

```markdown
Recommended immediate plan:

1. Complete business promotion-readiness review.
2. If promotion-ready remains green after review, prepare a separate business
   runtime activation PR.
3. If activation is deferred, move to Stage 5 feedback-aware explainability.
4. Keep political economy, geoeconomics, and economics-accounting as separate
   follow-up specs.
```

- [ ] **Step 2: Update CLI docs**

In `docs/reference/cli.md`, add a promotion-ready example near the eval-ready
and runtime-enabled gate examples:

```markdown
For a pre-activation promotion review, use the promotion-ready gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate promotion-ready \
  --json
```

`eligible_for_runtime_promotion: true` means the subject is still eval-ready,
but its fixture pack passes default-routing checks under a test-only
runtime-enabled simulation. It does not activate the subject.
```
```

- [ ] **Step 3: Update release checklist docs**

In `docs/advanced/publish-pypi.md`, add the promotion-ready command after the
business eval-ready command:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate promotion-ready \
  --json
```

Add:

```markdown
Business promotion-ready is an optional pre-activation review gate. It should
pass before a business runtime activation PR, but business remains eval-ready
until that separate PR changes the manifest status.
```

- [ ] **Step 4: Run doc scans**

Run:

```bash
rg -n "promotion-ready|eligible_for_runtime_promotion|runtime-enabled|eval-ready" docs/reference/cli.md docs/advanced/publish-pypi.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git diff --check
```

Expected: `rg` shows the new promotion-ready language and `git diff --check`
exits 0.

- [ ] **Step 5: Commit Task 3**

Stage only docs:

```bash
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md docs/reference/cli.md docs/advanced/publish-pypi.md
git commit -m "docs(subjects): document business promotion readiness"
```

## Task 4: Final Verification

**Files:**
- No file edits unless verification exposes a defect.

- [ ] **Step 1: Run focused tests**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py tests/test_subject_router_eval.py -q
```

Expected: all tests pass.

- [ ] **Step 2: Run full subject-router regression tests**

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py tests/test_subject_router_eval.py tests/test_subject_refinement.py -q
```

Expected: all tests pass.

- [ ] **Step 3: Run gate matrix**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate eval-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate promotion-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject finance --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject economics --gate runtime-enabled --json
```

Expected:

- default eval exits 0,
- business eval-ready exits 0,
- business promotion-ready exits 0,
- business runtime-enabled exits 1 with `activation_status is eval_ready`,
- accounting runtime-enabled exits 0,
- finance runtime-enabled exits 0,
- economics runtime-enabled exits 0.

- [ ] **Step 4: Run workspace checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: whitespace check exits 0 and git status is clean after committed work.

- [ ] **Step 5: Summarize remaining activation decision**

In the final handoff, state:

```text
Business promotion-ready is green, but business is still eval_ready. A separate
activation PR is required to change activation_status to runtime_enabled.
```

## Self-Review Checklist

- Spec coverage:
  - Activation override is implemented in Task 1.
  - Promotion-ready gate is implemented in Task 1.
  - Business resource metadata and fixtures are implemented in Task 2.
  - Documentation is implemented in Task 3.
  - Verification matrix is implemented in Task 4.
- Red-flag scan:
  - This plan contains no deferred implementation markers.
- Type consistency:
  - `activation_status_overrides` is consistently a mapping of subject id to activation status.
  - `promotion-ready` uses `eligible_for_runtime_promotion`.
  - Business manifest remains `activation_status: eval_ready`.
