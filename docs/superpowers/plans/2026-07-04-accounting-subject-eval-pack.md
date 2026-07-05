# Accounting Subject Eval Pack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make accounting the first new subject that can pass an `eval-ready` subject gate without enabling accounting as a runtime-suggested subject.

**Architecture:** Add gate-specific fixture expectations so default router evaluation can stay runtime-safe while `--gate eval-ready` can measure accounting routing in evaluation mode. Add accounting signal metadata to the manifest, load manifest-backed non-core signals in `subject_refinement.py`, and keep normal subject suggestions gated by `runtime_enabled`.

**Tech Stack:** Python 3, stdlib `argparse`/`dataclasses`/`json`/`pathlib`/`re`, PyYAML runtime subject manifests, unittest, Qiongli bridge modules.

---

## File Map

- Modify: `tooling/scripts/evaluate_subject_router.py`
  - Add `eval-ready` gate support.
  - Add gate-specific expected fixture handling.
  - Pass evaluation-only subjects into the router only when running `--gate eval-ready`.
  - Keep `runtime-enabled` gate behavior fail-closed for non-runtime subjects.
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Load manifest-backed signals for accounting and future non-core subjects.
  - Add an evaluation-only subject allowlist parameter.
  - Keep normal suggestions tied to `activation_status: runtime_enabled`.
- Modify: `content/subjects/accounting/runtime-subject.yaml`
  - Promote accounting from `candidate` to `eval_ready`.
  - Add accounting signal groups and method lenses.
- Create: `tests/fixtures/subject_router_eval/accounting/*.json`
  - Add accounting clear-positive, method-only, mixed, near-miss, locked, and confirmed fixtures.
- Modify: `tests/test_subject_router_eval.py`
  - Cover `eval-ready` gate behavior and gate-specific expected fixtures.
- Modify: `tests/test_subject_refinement.py`
  - Cover manifest-backed accounting signals, evaluation-mode suggestion, and runtime suppression.
- Modify: `tests/test_subject_contracts.py`
  - Cover accounting `eval_ready` manifest metadata and update candidate-status expectations.
- Modify: `docs/reference/cli.md`
  - Document `--gate eval-ready`.
- Modify: `docs/advanced/publish-pypi.md`
  - Add accounting eval-ready report to optional subject expansion checks.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark accounting as the active Stage 4 expansion slice.

## Execution Notes

- Work on branch `feature/accounting-subject-eval-pack`.
- Commit after each task so subagents can review narrow diffs.
- Run the focused test command listed in each task before committing that task.
- Do not promote accounting to `runtime_enabled` in this plan.
- Do not edit business, political economy, geoeconomics, or economics-accounting manifests except if a test assertion needs to keep them explicitly candidate.

## Task 1: Add Eval-Ready Gate Plumbing

**Files:**
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tooling/scripts/evaluate_subject_router.py`

- [ ] **Step 1: Write failing tests for gate-specific expected outputs and eval-ready eligibility**

Add these helpers near `_finance_contract()` in `tests/test_subject_router_eval.py`:

```python
def _accounting_contract(
    *,
    activation_status: str = "eval_ready",
    source: str | Path | None = None,
    signal_groups: Mapping[str, list[Mapping[str, Any]]] | None = None,
    method_lenses: Mapping[str, Mapping[str, Any]] | None = None,
    required_metrics: Mapping[str, float] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="accounting",
        display_name="Accounting",
        activation_status=activation_status,
        extends="core",
        source=str(
            source or Path("content/subjects/accounting/runtime-subject.yaml").resolve()
        ),
        domain_profile="content/skills/domain-profiles/accounting.yaml",
        overlay="",
        subject_skill="content/subjects/accounting/skills/accounting-measurement-auditor.md",
        signal_groups={
            key: [dict(item) for item in value]
            for key, value in (
                signal_groups
                or {
                    "method": [{"id": "accounting.method.accrual-quality"}],
                    "data_or_outcome": [{"id": "accounting.data.audit-analytics"}],
                    "venue": [{"id": "accounting.venue.accounting-review"}],
                    "theory_or_construct": [{"id": "accounting.construct.reporting-quality"}],
                }
            ).items()
        },
        method_lenses={
            key: dict(value)
            for key, value in (
                method_lenses
                or {
                    "accrual-quality": {
                        "resource": "content/subjects/accounting/skills/accounting-measurement-auditor.md",
                        "activation": "method_only",
                    }
                }
            ).items()
        },
        evaluation_pack="tests/fixtures/subject_router_eval/accounting",
        near_miss_policy={"forbidden_subjects": ["finance", "economics"]},
        activation_gate={
            "required_metrics": dict(
                required_metrics
                or {
                    "primary_subject_accuracy": 0.95,
                    "suggest_subject_precision": 0.95,
                    "near_miss_false_positives": 0,
                }
            )
        },
    )


def _successful_eval_report() -> dict[str, Any]:
    return {
        "case_count": 3,
        "metrics": {
            "decision_accuracy": 1.0,
            "primary_subject_accuracy": 1.0,
            "suggest_subject_precision": 1.0,
            "near_miss_false_positives": 0,
            "forbidden_subject_accuracy": 1.0,
            "method_lens_accuracy": 1.0,
            "all_case_checks_passed": 1.0,
        },
        "cases": [],
        "threshold_failures": [],
    }


def _gate_case(case_id: str, tags: list[str]) -> EvalCase:
    return EvalCase(
        id=case_id,
        description=case_id,
        request="accounting fixture",
        manifest={
            "active_subject": "auto",
            "subject_mode": "auto",
            "secondary_subjects": [],
            "venue_profiles": [],
            "method_lenses": [],
            "strictness": "standard",
        },
        expected={
            "decision": "recommend",
            "primary_subject": "auto",
            "suggest_subjects": [],
            "forbidden_subjects": [],
            "method_lenses": ["accrual-quality"],
        },
        source=f"tests/fixtures/subject_router_eval/accounting/{case_id}.json",
        subject_under_test="accounting",
        tags=["accounting", *tags],
    )
```

Add these tests inside `SubjectRouterEvalTests`:

```python
    def test_eval_ready_gate_accepts_eval_ready_subject_without_runtime_activation(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={"accounting": _accounting_contract()},
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "accounting")
        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])

    def test_runtime_enabled_gate_still_blocks_eval_ready_subject(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={"accounting": _accounting_contract()},
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="runtime-enabled")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])

    def test_gate_specific_expected_overrides_default_expected(self) -> None:
        case = EvalCase(
            id="accounting_gate_specific",
            description="gate-specific accounting expectation",
            request="Design an archival accounting study of discretionary accruals.",
            manifest={
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            expected={
                "decision": "recommend",
                "primary_subject": "auto",
                "suggest_subjects": [],
                "forbidden_subjects": [],
                "method_lenses": ["accrual-quality"],
            },
            source="inline.json",
            subject_under_test="accounting",
            tags=["accounting", "clear_positive"],
            gate_expected={
                "eval-ready": {
                    "decision": "recommend",
                    "primary_subject": "accounting",
                    "suggest_subjects": ["accounting"],
                    "forbidden_subjects": [],
                    "method_lenses": ["accrual-quality"],
                }
            },
        )

        self.assertEqual(
            case.expected_for_gate("")["primary_subject"],
            "auto",
        )
        self.assertEqual(
            case.expected_for_gate("eval-ready")["primary_subject"],
            "accounting",
        )

    def test_main_subject_eval_ready_gate_uses_eval_ready_eligibility(self) -> None:
        stdout = io.StringIO()

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={"accounting": _accounting_contract()},
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ), patch(
            "tooling.scripts.evaluate_subject_router.load_eval_cases",
            return_value=[
                _gate_case("accounting_clear", ["clear_positive"]),
                _gate_case("accounting_method", ["method_only_borrow"]),
                _gate_case("accounting_near_miss", ["near_miss"]),
            ],
        ), contextlib.redirect_stdout(stdout):
            exit_code = main(["--subject", "accounting", "--gate", "eval-ready", "--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertTrue(report["subject_gate"]["eligible_for_eval_ready"])
```

- [ ] **Step 2: Run the focused tests and verify they fail for missing behavior**

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected: FAIL because `EvalCase` has no `gate_expected`, `subject_gate_report()` has no `gate` parameter, and argparse rejects `--gate eval-ready`.

- [ ] **Step 3: Implement gate-specific expected fixture support**

Modify `tooling/scripts/evaluate_subject_router.py`:

```python
@dataclass(frozen=True)
class EvalCase:
    id: str
    description: str
    request: str
    manifest: dict[str, Any]
    expected: dict[str, Any]
    source: str
    subject_under_test: str = ""
    tags: list[str] | None = None
    gate_expected: dict[str, dict[str, Any]] | None = None

    def expected_for_gate(self, gate: str) -> dict[str, Any]:
        gate_expectations = self.gate_expected or {}
        selected = gate_expectations.get(gate)
        return dict(selected) if isinstance(selected, Mapping) else dict(self.expected)
```

In `load_eval_cases()`, read `gate_expected`:

```python
        gate_expected_raw = payload.get("gate_expected", {})
        gate_expected = {
            str(gate_name): dict(expected)
            for gate_name, expected in dict(gate_expected_raw).items()
            if isinstance(gate_name, str) and isinstance(expected, Mapping)
        } if isinstance(gate_expected_raw, Mapping) else {}
        cases.append(
            EvalCase(
                id=case_id,
                description=str(payload.get("description", "")),
                request=str(payload["request"]),
                manifest=dict(payload["manifest"]),
                expected=dict(payload["expected"]),
                source=_repo_relative(path),
                subject_under_test=subject_under_test,
                tags=tags,
                gate_expected=gate_expected,
            )
        )
```

Change `run_eval_case()` and `evaluate_cases()` signatures:

```python
def run_eval_case(
    case: EvalCase,
    *,
    gate: str = "",
    evaluation_subjects: set[str] | None = None,
) -> dict[str, Any]:
    manifest = ProjectManifest(**case.manifest).normalized()
    packet = infer_subject_refinement(
        {"topic": case.request, "context": case.request},
        manifest_state=manifest,
        evaluation_subjects=evaluation_subjects,
    )
    refinement = packet.to_packet()
    actual = _actual_eval_result(manifest, refinement)
    expected = _normalized_expected(case.expected_for_gate(gate))
    passed = {
        "decision": actual["decision"] == expected["decision"],
        "primary_subject": actual["primary_subject"] == expected["primary_subject"],
        "suggest_subjects": set(expected["suggest_subjects"]).issubset(
            set(actual["suggest_subjects"])
        ),
        "forbidden_subjects": not (
            set(expected["forbidden_subjects"]) & set(actual["suggest_subjects"])
        ),
        "method_lenses": set(actual["method_lenses"]) == set(expected["method_lenses"]),
    }
    return {
        "id": case.id,
        "description": case.description,
        "source": case.source,
        "expected": expected,
        "actual": actual,
        "passed": passed,
    }


def evaluate_cases(
    cases: list[EvalCase],
    thresholds: Mapping[str, float] = DEFAULT_THRESHOLDS,
    *,
    gate: str = "",
    evaluation_subjects: set[str] | None = None,
) -> dict[str, Any]:
    if not cases:
        raise ValueError("cannot evaluate an empty case list")
    case_results = [
        run_eval_case(case, gate=gate, evaluation_subjects=evaluation_subjects)
        for case in cases
    ]
    metrics = _metrics(case_results, cases)
    return {
        "case_count": len(case_results),
        "metrics": metrics,
        "cases": case_results,
        "threshold_failures": threshold_failures(metrics, thresholds),
    }
```

- [ ] **Step 4: Implement `eval-ready` and `runtime-enabled` gate semantics**

Add constants near `REQUIRED_GATE_TAGS`:

```python
GATE_CHOICES = ("eval-ready", "runtime-enabled")
GATE_ELIGIBILITY_KEYS = {
    "eval-ready": "eligible_for_eval_ready",
    "runtime-enabled": "eligible_for_runtime_enabled",
}
REQUIRED_SIGNAL_DIMENSIONS = {
    "accounting": {"method", "data_or_outcome", "venue", "theory_or_construct"},
}
```

Replace `subject_gate_report()` with this gate-aware version:

```python
def subject_gate_report(
    subject: str,
    cases: list[EvalCase],
    *,
    gate: str = "runtime-enabled",
) -> dict[str, Any]:
    if gate not in GATE_CHOICES:
        raise ValueError(f"unsupported subject gate: {gate}")
    contracts = load_runtime_subject_contracts()
    contract = contracts.get(subject)
    activation_status = contract.activation_status if contract else "candidate"
    thresholds = _contract_thresholds(contract)
    subject_cases = [
        case
        for case in cases
        if case.subject_under_test == subject or subject in list(case.tags or [])
    ]
    evaluation_subjects = {subject} if gate == "eval-ready" else None
    report = (
        evaluate_cases(
            subject_cases,
            thresholds=thresholds,
            gate=gate,
            evaluation_subjects=evaluation_subjects,
        )
        if subject_cases
        else _empty_eval_report()
    )
    subject_tags = {
        tag
        for case in subject_cases
        for tag in list(case.tags or [])
    }
    blocking_failures: list[str] = []
    if contract is None:
        blocking_failures.append("missing runtime subject contract")
    if gate == "eval-ready":
        if activation_status != "eval_ready":
            blocking_failures.append(f"activation_status is {activation_status}")
        if contract is not None:
            blocking_failures.extend(_missing_eval_ready_resource_failures(contract))
            blocking_failures.extend(_missing_signal_dimension_failures(contract))
    if gate == "runtime-enabled":
        if activation_status != "runtime_enabled":
            blocking_failures.append(f"activation_status is {activation_status}")
        if contract is not None and activation_status == "runtime_enabled":
            blocking_failures.extend(_missing_resource_failures(contract))
    missing_tags = sorted(REQUIRED_GATE_TAGS - subject_tags)
    for tag in missing_tags:
        blocking_failures.append(f"missing {tag} fixtures")
    for failure in report["threshold_failures"]:
        metric = failure.get("metric", "unknown")
        blocking_failures.append(f"threshold failure: {metric}")
    eligible_for_eval_ready = (
        gate == "eval-ready"
        and not blocking_failures
        and activation_status == "eval_ready"
    )
    eligible_for_runtime_enabled = (
        gate == "runtime-enabled"
        and not blocking_failures
        and activation_status == "runtime_enabled"
    )
    return {
        "subject": subject,
        "gate": gate,
        "activation_status": activation_status,
        "eligible_for_eval_ready": eligible_for_eval_ready,
        "eligible_for_runtime_enabled": eligible_for_runtime_enabled,
        "case_count": len(subject_cases),
        "required_tags": sorted(REQUIRED_GATE_TAGS),
        "present_tags": sorted(subject_tags),
        "metrics": report["metrics"],
        "blocking_failures": blocking_failures,
    }
```

Add the eval-ready resource helpers:

```python
def _missing_eval_ready_resource_failures(contract: Any) -> list[str]:
    resource_root = _contract_resource_root(getattr(contract, "source", ""))
    failures: list[str] = []
    for field in ("domain_profile", "evaluation_pack"):
        resource = getattr(contract, field, "")
        if _resource_is_missing(resource_root, resource):
            failures.append(f"missing resource: {field} {resource}")
    for field in ("overlay", "subject_skill"):
        resource = getattr(contract, field, "")
        if isinstance(resource, str) and resource.strip() and _resource_is_missing(resource_root, resource):
            failures.append(f"missing resource: {field} {resource}")
    method_lenses = getattr(contract, "method_lenses", {})
    if isinstance(method_lenses, Mapping):
        for lens, config in method_lenses.items():
            if not isinstance(config, Mapping):
                continue
            resource = config.get("resource", "")
            if _resource_is_missing(resource_root, resource):
                failures.append(
                    f"missing resource: method_lenses[{lens}].resource {resource}"
                )
    return failures


def _missing_signal_dimension_failures(contract: Any) -> list[str]:
    required = REQUIRED_SIGNAL_DIMENSIONS.get(getattr(contract, "subject", ""), set())
    signal_groups = getattr(contract, "signal_groups", {})
    if not isinstance(signal_groups, Mapping):
        return [f"missing signal dimension: {dimension}" for dimension in sorted(required)]
    failures: list[str] = []
    for dimension in sorted(required):
        values = signal_groups.get(dimension, [])
        if not isinstance(values, list) or not values:
            failures.append(f"missing signal dimension: {dimension}")
    return failures
```

Update argparse and exit handling in `main()`:

```python
    parser.add_argument("--gate", choices=GATE_CHOICES, default="")
```

```python
        if args.subject and args.gate:
            report["subject_gate"] = subject_gate_report(
                args.subject,
                cases,
                gate=args.gate,
            )
```

```python
    gate = report.get("subject_gate")
    if isinstance(gate, Mapping):
        eligibility_key = GATE_ELIGIBILITY_KEYS.get(str(gate.get("gate", "")))
        if eligibility_key and not gate.get(eligibility_key):
            return 1
```

- [ ] **Step 5: Run focused gate tests**

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected: PASS for the new gate-plumbing tests, with possible later accounting fixture failures absent because fixture files have not been added yet.

- [ ] **Step 6: Commit gate plumbing**

```bash
git add tooling/scripts/evaluate_subject_router.py tests/test_subject_router_eval.py
git commit -m "feat(subjects): add eval-ready router gate"
```

## Task 2: Populate Accounting Manifest And Contract Tests

**Files:**
- Modify: `content/subjects/accounting/runtime-subject.yaml`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Write failing contract tests for accounting eval-ready metadata**

Update `test_default_repository_contracts_classify_enabled_and_candidates()`:

```python
    def test_default_repository_contracts_classify_enabled_candidates_and_eval_ready(self) -> None:
        contracts = load_runtime_subject_contracts()

        self.assertEqual(
            subject_activation_status("economics", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("finance", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "eval_ready",
        )
        self.assertIn("accounting", contracts)
        for subject in {
            "business",
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }:
            self.assertEqual(subject_activation_status(subject, contracts), "candidate")
            self.assertIn(subject, contracts)
```

Add a new test below it:

```python
    def test_accounting_eval_ready_manifest_declares_signals_and_method_lenses(self) -> None:
        contracts = load_runtime_subject_contracts()
        contract = contracts["accounting"]

        self.assertEqual(contract.activation_status, "eval_ready")
        self.assertEqual(
            set(contract.signal_groups),
            {"method", "data_or_outcome", "venue", "theory_or_construct"},
        )
        for dimension in ("method", "data_or_outcome", "venue", "theory_or_construct"):
            self.assertTrue(contract.signal_groups[dimension], dimension)
            for entry in contract.signal_groups[dimension]:
                self.assertIsInstance(entry["id"], str)
                self.assertIsInstance(entry["value"], str)
                self.assertIsInstance(entry["patterns"], list)
                self.assertTrue(entry["patterns"])
        self.assertIn("accrual-quality", contract.method_lenses)
        self.assertIn("construct-proxy-audit", contract.method_lenses)
        self.assertEqual(
            contract.method_lenses["accrual-quality"]["activation"],
            "method_only",
        )
        self.assertEqual(
            contract.activation_gate["required_metrics"]["primary_subject_accuracy"],
            0.95,
        )
```

- [ ] **Step 2: Run contract tests and verify they fail against candidate accounting**

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py -q
```

Expected: FAIL because accounting is still `candidate` and signal groups are empty.

- [ ] **Step 3: Replace the accounting runtime manifest**

Replace `content/subjects/accounting/runtime-subject.yaml` with:

```yaml
schema_version: 1.0
subject: accounting
display_name: Accounting
activation_status: eval_ready
extends: core
domain_profile: content/skills/domain-profiles/accounting.yaml
overlay: ""
subject_skill: content/subjects/accounting/skills/accounting-measurement-auditor.md
signal_groups:
  method:
    - id: accounting.method.accrual-quality
      value: accrual-quality
      weight: 0.35
      activation: method_only
      patterns:
        - "\\baccrual quality\\b"
        - "\\bdiscretionary accruals?\\b"
        - "\\bearnings management\\b"
        - "\\bmodified Jones\\b"
      examples:
        - "Measure discretionary accruals with a modified Jones model."
      near_misses:
        - "Accrual accounting entry for a small business ledger."
    - id: accounting.method.construct-proxy-audit
      value: construct-proxy-audit
      weight: 0.30
      activation: method_only
      patterns:
        - "\\bconstruct[- ]proxy\\b"
        - "\\bmeasurement validity\\b"
        - "\\barchival accounting\\b"
        - "\\bfiscal timing\\b"
      examples:
        - "Audit the construct-proxy mapping for archival accounting variables."
      near_misses:
        - "Account for missing covariates in a regression."
  data_or_outcome:
    - id: accounting.data.audit-analytics
      value: audit-analytics
      weight: 0.25
      activation: subject
      patterns:
        - "\\bAudit Analytics\\b"
        - "\\brestatements?\\b"
        - "\\binternal[- ]control weaknesses?\\b"
        - "\\bSOX 404\\b"
      examples:
        - "Use Audit Analytics restatement and internal-control weakness data."
      near_misses:
        - "Audit a software analytics dashboard."
    - id: accounting.data.financial-reporting-quality
      value: financial-reporting-quality
      weight: 0.25
      activation: subject
      patterns:
        - "\\bfinancial reporting quality\\b"
        - "\\breporting quality\\b"
        - "\\bearnings quality\\b"
        - "\\bmanagement forecasts?\\b"
      examples:
        - "Study earnings quality and management forecast accuracy."
      near_misses:
        - "Improve the quality of a report document."
  venue:
    - id: accounting.venue.accounting-review
      value: accounting-review
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bThe Accounting Review\\b"
        - "\\bAccounting Review\\b"
      examples:
        - "Target The Accounting Review."
      near_misses:
        - "Review the accounting section of a budget."
    - id: accounting.venue.journal-of-accounting-research
      value: journal-of-accounting-research
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bJournal of Accounting Research\\b"
        - "\\bJAR\\b"
      examples:
        - "Position the study for Journal of Accounting Research."
      near_misses:
        - "Jar file accounting logs."
    - id: accounting.venue.review-of-accounting-studies
      value: review-of-accounting-studies
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bReview of Accounting Studies\\b"
        - "\\bRAST\\b"
      examples:
        - "Frame the paper for Review of Accounting Studies."
      near_misses:
        - "Review accounting studies in a course syllabus."
  theory_or_construct:
    - id: accounting.construct.reporting-mechanism
      value: reporting-mechanism
      weight: 0.25
      activation: subject
      patterns:
        - "\\bfinancial reporting\\b"
        - "\\bdisclosure quality\\b"
        - "\\breporting incentives?\\b"
        - "\\breporting mechanism\\b"
      examples:
        - "Explain the financial reporting mechanism behind disclosure quality."
      near_misses:
        - "Report project status to the sponsor."
    - id: accounting.construct.audit-setting
      value: audit-setting
      weight: 0.25
      activation: subject
      patterns:
        - "\\baudit fees?\\b"
        - "\\bauditing setting\\b"
        - "\\baudit committees?\\b"
        - "\\bgoing concern\\b"
      examples:
        - "Use audit fees and going-concern opinions as accounting outcomes."
      near_misses:
        - "Audit a data pipeline."
method_lenses:
  accrual-quality:
    resource: content/subjects/accounting/skills/accounting-measurement-auditor.md
    activation: method_only
  construct-proxy-audit:
    resource: content/subjects/accounting/overlays/skills/variable-constructor.md
    activation: method_only
evaluation_pack: tests/fixtures/subject_router_eval/accounting
near_miss_policy:
  forbidden_subjects:
    - finance
    - economics
activation_gate:
  required_metrics:
    primary_subject_accuracy: 0.95
    suggest_subject_precision: 0.95
    near_miss_false_positives: 0
```

- [ ] **Step 4: Run contract tests**

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit accounting manifest metadata**

```bash
git add content/subjects/accounting/runtime-subject.yaml tests/test_subject_contracts.py
git commit -m "feat(subjects): mark accounting eval ready metadata"
```

## Task 3: Add Manifest-Backed Accounting Signal Routing

**Files:**
- Modify: `tests/test_subject_refinement.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`

- [ ] **Step 1: Write failing accounting signal tests**

Add these tests to `tests/test_subject_refinement.py`:

```python
    def test_eval_ready_accounting_signals_borrow_lens_without_runtime_suggestion(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": (
                    "Design a study of discretionary accruals, Audit Analytics "
                    "restatements, internal-control weaknesses, financial reporting "
                    "quality, and Journal of Accounting Research positioning."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("accrual-quality", [lens["lens"] for lens in packet["borrowed_lenses"]])
        signal_ids = {signal["id"] for signal in packet["signals"]}
        self.assertIn("accounting.method.accrual-quality", signal_ids)
        self.assertIn("accounting.data.audit-analytics", signal_ids)
        self.assertIn("accounting.venue.journal-of-accounting-research", signal_ids)

    def test_eval_ready_gate_mode_can_measure_accounting_primary_subject(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": (
                    "Design a study of discretionary accruals, Audit Analytics "
                    "restatements, financial reporting quality, and Journal of "
                    "Accounting Research positioning."
                ),
            },
            manifest_state=ProjectManifest(),
            evaluation_subjects={"accounting"},
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("accrual-quality", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["overlays"], [])
        self.assertEqual(packet["loaded_resources"]["subject_skills"], [])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])

    def test_accounting_near_miss_account_for_heterogeneity_keeps_core(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "regression diagnostics",
                "context": "Explain how to account for heterogeneity in a generic regression model.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertNotIn(
            "accounting",
            [signal["subject"] for signal in packet["signals"]],
        )
        self.assertEqual(packet["decision"], "no_subject")
        self.assertNotEqual(packet["primary_subject"], "accounting")
```

- [ ] **Step 2: Run refinement tests and verify they fail**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected: FAIL because `infer_subject_refinement()` does not accept `evaluation_subjects` and no accounting manifest signals are detected.

- [ ] **Step 3: Add runtime subject contract loading and evaluation allowlist**

Change the import in `subject_refinement.py`:

```python
from .subject_contracts import (
    RuntimeSubjectContract,
    load_runtime_subject_contracts,
    subject_activation_status,
)
```

Add this dataclass after `SubjectSignals`:

```python
@dataclass(frozen=True)
class RuntimeSubjectMatch:
    subject: str
    dimensions: tuple[str, ...]
    method_lenses: tuple[str, ...]
    evidence: tuple[str, ...]
    signal_ids: tuple[str, ...]

    @property
    def has_subject_strength(self) -> bool:
        return len(self.dimensions) >= 2
```

Add `runtime_subject_matches` to `SubjectSignals`:

```python
    runtime_subject_matches: dict[str, RuntimeSubjectMatch]
```

Update `SubjectSignals.has_any`:

```python
            or self.economics_venues
            or self.runtime_subject_matches
```

Change `infer_subject_refinement()` signature:

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
) -> SubjectRefinementPacket:
```

Add near the current runtime-enabled checks:

```python
    evaluation_subjects = set(evaluation_subjects or set())
    finance_runtime_enabled = _subject_can_be_suggested(
        "finance",
        evaluation_subjects=evaluation_subjects,
    )
    economics_runtime_enabled = _subject_can_be_suggested(
        "economics",
        evaluation_subjects=evaluation_subjects,
    )
```

Update `_subject_can_be_suggested()`:

```python
def _subject_can_be_suggested(
    subject: str,
    *,
    evaluation_subjects: set[str] | None = None,
) -> bool:
    if evaluation_subjects and subject in evaluation_subjects:
        return True
    return subject_activation_status(subject) == "runtime_enabled"
```

- [ ] **Step 4: Implement manifest-backed signal detection**

Add these helpers after `_detect_signal_records()`:

```python
def _detect_manifest_signal_records(
    text: str,
) -> tuple[list[dict[str, Any]], dict[str, RuntimeSubjectMatch]]:
    records: list[dict[str, Any]] = []
    matches: dict[str, RuntimeSubjectMatch] = {}
    for subject, contract in load_runtime_subject_contracts().items():
        if subject in {"economics", "finance"}:
            continue
        subject_records = _manifest_records_for_contract(contract, text)
        if not subject_records:
            continue
        records.extend(subject_records)
        dimensions = _unique(
            [str(record["dimension"]) for record in subject_records]
        )
        method_lenses = _unique(
            [
                str(record["value"])
                for record in subject_records
                if record["dimension"] == "method"
                and str(record["value"]) in contract.method_lenses
            ]
        )
        matches[subject] = RuntimeSubjectMatch(
            subject=subject,
            dimensions=tuple(dimensions),
            method_lenses=tuple(method_lenses),
            evidence=tuple(_unique([str(record["snippet"]) for record in subject_records])),
            signal_ids=tuple(_unique([str(record["id"]) for record in subject_records])),
        )
    return records, matches


def _manifest_records_for_contract(
    contract: RuntimeSubjectContract,
    text: str,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for dimension, entries in contract.signal_groups.items():
        for entry in entries:
            if not isinstance(entry, Mapping):
                continue
            entry_id = entry.get("id")
            value = entry.get("value")
            patterns = entry.get("patterns", [])
            if not isinstance(entry_id, str) or not isinstance(value, str):
                continue
            if not isinstance(patterns, list):
                continue
            for pattern_text in patterns:
                if not isinstance(pattern_text, str):
                    continue
                try:
                    pattern = re.compile(pattern_text, re.I)
                except re.error:
                    continue
                match = pattern.search(text)
                if not match:
                    continue
                records.append(
                    {
                        "id": entry_id,
                        "subject": contract.subject,
                        "dimension": str(dimension),
                        "value": value,
                        "weight": float(entry.get("weight", 0.0) or 0.0),
                        "source": "task_text",
                        "snippet": _snippet_for_match(text, match),
                    }
                )
                break
    return _unique_records(records, key="id")
```

Update `_detect_signals()`:

```python
    manifest_records, runtime_subject_matches = _detect_manifest_signal_records(text)
    return SubjectSignals(
        finance_method_lenses=finance_method_lenses,
        finance_data_outcomes=finance_data_outcomes,
        finance_venues=finance_venues,
        economics_method_lenses=economics_method_lenses,
        economics_venues=economics_venues,
        evidence=_evidence(text, extra_records=manifest_records),
        signals=_unique_records([*_detect_signal_records(text), *manifest_records], key="id"),
        runtime_subject_matches=runtime_subject_matches,
    )
```

Change `_evidence()`:

```python
def _evidence(text: str, *, extra_records: list[dict[str, Any]] | None = None) -> list[str]:
    patterns = [
        *FINANCE_METHOD_PATTERNS.values(),
        *FINANCE_DATA_OUTCOME_PATTERNS,
        *FINANCE_VENUE_PATTERNS,
        *ECONOMICS_METHOD_PATTERNS.values(),
        *ECONOMICS_VENUE_PATTERNS,
    ]
    snippets: list[str] = []
    for pattern in patterns:
        match = pattern.search(text)
        if not match:
            continue
        snippet = _snippet_for_match(text, match)
        if snippet not in snippets:
            snippets.append(snippet)
    for record in extra_records or []:
        snippet = record.get("snippet")
        if isinstance(snippet, str) and snippet not in snippets:
            snippets.append(snippet)
    return snippets[:5]
```

- [ ] **Step 5: Add accounting suggestion and borrow-lens branches**

After the economics suggestion branch and before finance/economics borrow-lens branches, add:

```python
    accounting_match = signals.runtime_subject_matches.get("accounting")
    accounting_runtime_enabled = _subject_can_be_suggested(
        "accounting",
        evaluation_subjects=evaluation_subjects,
    )
    if (
        accounting_match is not None
        and accounting_match.has_subject_strength
        and accounting_runtime_enabled
    ):
        method_lenses = _unique(list(accounting_match.method_lenses))
        borrowed_lenses = _borrowed_lenses("accounting", signals)
        return _packet(
            decision="suggest_subject",
            mode="suggested",
            active_subject=manifest.active_subject,
            primary_subject="accounting",
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="accounting",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill", "method_pack"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject="accounting",
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_result.warnings,
            ),
            persistence={"status": "proposed"},
            summary="Accounting subject measured from archival method, construct, data, and venue signals.",
            domain="accounting",
            confidence=0.75,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if (
        accounting_match is not None
        and accounting_match.method_lenses
        and manifest.active_subject != "accounting"
    ):
        borrowed_lenses = [
            _borrowed_lens_record(
                "accounting",
                lens,
                reason="accounting method-only signal; keep active subject",
            )
            for lens in accounting_match.method_lenses
        ]
        return _packet(
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="accounting",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=_unique(list(manifest.method_lenses or [])),
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["method_pack_only"],
                primary_subject=manifest.active_subject,
                method_lenses=[],
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_result.warnings,
            ),
            persistence={"status": "temporary"},
            summary=_summary(
                "Borrowing accounting method lens without changing the project subject.",
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=0.45,
            evidence=signals.evidence,
            signals=signals.signals,
        )
```

Update `_borrowed_lenses()` before its final return:

```python
    for subject, match in signals.runtime_subject_matches.items():
        if active_subject == subject:
            continue
        lenses.extend(
            _borrowed_lens_record(
                subject,
                lens,
                reason=f"{subject} method-only signal; keep active subject",
            )
            for lens in match.method_lenses
        )
```

Update `_candidate_subjects()` signature and body:

```python
def _candidate_subjects(
    signals: SubjectSignals,
    *,
    preferred: str | None = None,
    evaluation_subjects: set[str] | None = None,
) -> list[dict[str, Any]]:
    subjects: list[str] = []
    if preferred:
        subjects.append(preferred)
    if (
        signals.finance_method_lenses
        or signals.finance_data_outcomes
        or signals.finance_venues
    ):
        subjects.append("finance")
    if signals.economics_method_lenses or signals.economics_venues:
        subjects.append("economics")
    subjects.extend(signals.runtime_subject_matches)
    return [
        _candidate_subject_record(subject, signals)
        for subject in _unique(subjects)
        if _subject_can_be_suggested(subject, evaluation_subjects=evaluation_subjects)
    ]
```

Update `_candidate_subject_record()` for manifest matches before the fallback return:

```python
    runtime_match = signals.runtime_subject_matches.get(subject)
    if runtime_match is not None:
        return {
            "subject": subject,
            "confidence": min(0.85, 0.35 + 0.15 * len(runtime_match.dimensions)),
            "evidence": list(runtime_match.evidence),
            "matched_dimensions": list(runtime_match.dimensions),
            "method_lenses": list(runtime_match.method_lenses),
            "signal_ids": list(runtime_match.signal_ids),
        }
```

Update all existing `_candidate_subjects(signals, ...)` calls to pass `evaluation_subjects=evaluation_subjects` where the local variable exists.

- [ ] **Step 6: Run refinement tests**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected: PASS.

- [ ] **Step 7: Commit accounting signal routing**

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_refinement.py tests/test_subject_refinement.py
git commit -m "feat(subjects): route eval-ready accounting signals"
```

## Task 4: Add Accounting Router Fixture Pack

**Files:**
- Create: `tests/fixtures/subject_router_eval/accounting/clear_discretionary_accruals.json`
- Create: `tests/fixtures/subject_router_eval/accounting/method_only_borrow_accrual_quality.json`
- Create: `tests/fixtures/subject_router_eval/accounting/mixed_accounting_finance_reporting_returns.json`
- Create: `tests/fixtures/subject_router_eval/accounting/near_miss_account_for_heterogeneity.json`
- Create: `tests/fixtures/subject_router_eval/accounting/near_miss_bookkeeping_budget.json`
- Create: `tests/fixtures/subject_router_eval/accounting/locked_finance_borrow_accounting_measurement.json`
- Create: `tests/fixtures/subject_router_eval/accounting/confirmed_accounting_construct_audit.json`
- Modify: `tests/test_subject_router_eval.py`

- [ ] **Step 1: Update fixture count and id tests**

In `test_load_eval_cases_reads_all_fixtures()`, change the strict id list into required subset assertions so future subject packs do not make the test brittle:

```python
        ids = [case.id for case in cases]
        self.assertTrue(
            {
                "clear_economics",
                "clear_finance",
                "economics_method_only_borrow",
                "finance_method_only_borrow",
                "locked_subject_neighbor_lens",
                "mixed_econ_finance",
                "near_miss_finance",
                "weak_core_only",
            }.issubset(set(ids))
        )
        self.assertGreaterEqual(len(cases), 15)
```

Add this test:

```python
    def test_accounting_eval_ready_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)
```

- [ ] **Step 2: Add clear-positive fixture with gate-specific expected output**

Create `tests/fixtures/subject_router_eval/accounting/clear_discretionary_accruals.json`:

```json
{
  "id": "accounting_clear_discretionary_accruals",
  "subject_under_test": "accounting",
  "tags": ["accounting", "clear_positive"],
  "description": "Archival accounting request with method, data, construct, and venue signals.",
  "request": "Design an archival accounting study of discretionary accruals, Audit Analytics restatements, internal-control weaknesses, financial reporting quality, and Journal of Accounting Research positioning.",
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
    "forbidden_subjects": [],
    "method_lenses": ["accrual-quality"]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "accounting",
      "suggest_subjects": ["accounting"],
      "forbidden_subjects": [],
      "method_lenses": ["accrual-quality"]
    }
  }
}
```

- [ ] **Step 3: Add method-only borrow fixture**

Create `tests/fixtures/subject_router_eval/accounting/method_only_borrow_accrual_quality.json`:

```json
{
  "id": "accounting_method_only_borrow_accrual_quality",
  "subject_under_test": "accounting",
  "tags": ["accounting", "method_only_borrow"],
  "description": "Locked finance project may borrow an accounting measurement lens without switching subject.",
  "request": "Within my finance paper, add discretionary accruals and accrual quality as reporting-quality controls, but keep the main framing on abnormal returns and asset pricing.",
  "manifest": {
    "active_subject": "finance",
    "subject_mode": "locked",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": ["asset-pricing"],
    "strictness": "standard"
  },
  "expected": {
    "decision": "keep_locked",
    "primary_subject": "finance",
    "suggest_subjects": [],
    "forbidden_subjects": ["accounting"],
    "method_lenses": ["asset-pricing", "accrual-quality"]
  }
}
```

- [ ] **Step 4: Add mixed accounting-finance fixture**

Create `tests/fixtures/subject_router_eval/accounting/mixed_accounting_finance_reporting_returns.json`:

```json
{
  "id": "accounting_mixed_reporting_returns",
  "subject_under_test": "accounting",
  "tags": ["accounting", "mixed_subject"],
  "description": "Mixed accounting-finance request with explicit accounting primary expectation under the eval-ready gate.",
  "request": "Frame a paper on earnings management, discretionary accruals, financial reporting quality, and abnormal returns around disclosure announcements using Compustat and CRSP.",
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
    "allowed_neighbor_subjects": ["finance"],
    "forbidden_subjects": [],
    "method_lenses": ["accrual-quality"]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "accounting",
      "suggest_subjects": ["accounting"],
      "allowed_neighbor_subjects": ["finance"],
      "forbidden_subjects": [],
      "method_lenses": ["accrual-quality"]
    }
  }
}
```

- [ ] **Step 5: Add near-miss fixtures**

Create `tests/fixtures/subject_router_eval/accounting/near_miss_account_for_heterogeneity.json`:

```json
{
  "id": "accounting_near_miss_account_for_heterogeneity",
  "subject_under_test": "accounting",
  "tags": ["accounting", "near_miss"],
  "description": "Generic phrase 'account for heterogeneity' must not activate accounting.",
  "request": "Explain how to account for heterogeneity in a generic regression model with treatment effect variation.",
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
    "forbidden_subjects": ["accounting"],
    "method_lenses": []
  }
}
```

Create `tests/fixtures/subject_router_eval/accounting/near_miss_bookkeeping_budget.json`:

```json
{
  "id": "accounting_near_miss_bookkeeping_budget",
  "subject_under_test": "accounting",
  "tags": ["accounting", "near_miss"],
  "description": "Bookkeeping and project budget wording must stay core.",
  "request": "Prepare a bookkeeping-style budget tracker for staff costs, equipment categories, and monthly reporting milestones.",
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
    "forbidden_subjects": ["accounting", "finance", "economics"],
    "method_lenses": []
  }
}
```

- [ ] **Step 6: Add locked and confirmed fixtures**

Create `tests/fixtures/subject_router_eval/accounting/locked_finance_borrow_accounting_measurement.json`:

```json
{
  "id": "accounting_locked_finance_borrow_measurement",
  "subject_under_test": "accounting",
  "tags": ["accounting", "locked_subject"],
  "description": "Locked finance subject can borrow accounting measurement without switching primary subject.",
  "request": "Keep this as a finance paper on event-study abnormal returns, but add construct-proxy checks for discretionary accruals and earnings quality.",
  "manifest": {
    "active_subject": "finance",
    "subject_mode": "locked",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": ["event-study"],
    "strictness": "standard"
  },
  "expected": {
    "decision": "keep_locked",
    "primary_subject": "finance",
    "suggest_subjects": [],
    "forbidden_subjects": ["accounting"],
    "method_lenses": ["event-study", "accrual-quality", "construct-proxy-audit"]
  }
}
```

Create `tests/fixtures/subject_router_eval/accounting/confirmed_accounting_construct_audit.json`:

```json
{
  "id": "accounting_confirmed_construct_audit",
  "subject_under_test": "accounting",
  "tags": ["accounting", "confirmed_subject"],
  "description": "Confirmed accounting project remains accounting even while subject resources are withheld before runtime activation.",
  "request": "Audit the construct-proxy mapping for discretionary accruals, fiscal timing, and financial reporting quality.",
  "manifest": {
    "active_subject": "accounting",
    "subject_mode": "confirmed",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": ["construct-proxy-audit"],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "accounting",
    "suggest_subjects": [],
    "forbidden_subjects": [],
    "method_lenses": ["construct-proxy-audit"]
  }
}
```

- [ ] **Step 7: Run fixture and gate tests**

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate eval-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
```

Expected:

- `tests/test_subject_router_eval.py` passes.
- Default `--json` exits 0.
- `--gate eval-ready` exits 0 with `eligible_for_eval_ready: true`.
- `--gate runtime-enabled` exits 1 with `activation_status is eval_ready`.

- [ ] **Step 8: Commit accounting fixtures**

```bash
git add tests/fixtures/subject_router_eval/accounting tests/test_subject_router_eval.py
git commit -m "test(subjects): add accounting eval-ready fixture pack"
```

## Task 5: Update Docs And Roadmap

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update CLI subject expansion gate docs**

In `docs/reference/cli.md`, update the Subject Expansion Gate section to include:

````markdown
For readiness checks before runtime activation, use the eval-ready gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate eval-ready \
  --json
```

`eligible_for_eval_ready: true` means the subject has a passing fixture pack and
metadata that maintainers can review. It does not allow adaptive runtime
suggestions.

For final activation checks, use the runtime-enabled gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

`eligible_for_runtime_enabled: false` means the subject can remain packaged as
candidate or eval-ready content, but adaptive runtime must not suggest it as a
primary subject.
````

- [ ] **Step 2: Update release-readiness docs**

In `docs/advanced/publish-pypi.md`, replace the optional subject expansion gate block with:

````markdown
Optional subject expansion gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate eval-ready \
  --json
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

The first command must pass for release. The eval-ready command should pass for
the accounting expansion slice. The runtime-enabled command should fail closed
until accounting is intentionally promoted to `runtime_enabled`.
````

- [ ] **Step 3: Update roadmap Stage 4 status**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`, update Stage 4 status:

```markdown
Status: accounting expansion slice in progress. Accounting is the first
eval-ready candidate subject; business, political economy, geoeconomics, and
the economics-accounting bridge remain deferred until the accounting gate is
reviewed.
```

Update the Recommended Immediate Plan tail:

```markdown
Current Stage 4 execution sequence:

- Finish the accounting eval-ready fixture pack and manifest-backed signal
  routing.
- Review the accounting eval-ready gate report before considering a separate
  runtime-enabled activation change.
- Keep business, political economy, geoeconomics, and economics-accounting as
  separate follow-up specs.
```

- [ ] **Step 4: Run documentation grep checks**

Run:

```bash
rg -n "eval-ready|runtime-enabled|eligible_for_eval_ready|eligible_for_runtime_enabled" docs/reference/cli.md docs/advanced/publish-pypi.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
```

Expected: output includes both gate names and both eligibility fields.

- [ ] **Step 5: Commit docs**

```bash
git add docs/reference/cli.md docs/advanced/publish-pypi.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(subjects): document accounting eval-ready gate"
```

## Task 6: Full Verification And Review Prep

**Files:**
- No new files expected.
- Verify all files changed by Tasks 1-5.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py
```

Expected: PASS.

- [ ] **Step 2: Run subject router evaluation commands**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate eval-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject finance --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject economics --gate runtime-enabled --json
```

Expected:

- Default `--json` exits 0.
- Accounting `eval-ready` exits 0.
- Accounting `runtime-enabled` exits 1 with `activation_status is eval_ready`.
- Finance and economics `runtime-enabled` gates exit 0.

- [ ] **Step 3: Run whitespace and status checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected:

- `git diff --check` exits 0.
- `git status --short --branch` shows the feature branch and no unstaged changes.

- [ ] **Step 4: Prepare review summary**

Collect these facts for the final handoff:

```text
Accounting activation_status:
Accounting eval-ready gate exit:
Accounting runtime-enabled gate exit:
Default router eval exit:
Finance runtime-enabled gate exit:
Economics runtime-enabled gate exit:
Focused pytest command:
```

Use actual command results. Do not claim a gate passed unless the command exited 0.

## Deferred Items Explicitly Outside This Plan

- Do not add a fake dismissed-subject router fixture. The router fixture schema currently has `manifest` but no subject evidence memory payload, and dismissed-subject behavior is covered in guidance runtime and lifecycle tests. Add dismissed router fixtures only after the eval harness accepts evidence-memory setup.
- Do not promote accounting to `runtime_enabled`.
- Do not activate other candidate subjects.
- Do not change provider, Zotero, literature-search, or full-cycle workflow code in this accounting slice.
