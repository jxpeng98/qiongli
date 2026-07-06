# Business Subject Eval-Ready Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move business from a deferred candidate shell to `eval_ready` with a business-owned fixture pack, manifest-backed signals, and gate coverage, while keeping business unavailable for default runtime subject activation.

**Architecture:** Reuse the runtime subject contract and evaluation gate machinery already built for accounting. Generalize the manifest-backed router path so eval-ready subjects beyond accounting can be measured through `evaluation_subjects`, then add a business manifest, fixtures, tests, and roadmap/docs updates.

**Tech Stack:** Python 3, unittest/pytest, PyYAML runtime manifests, JSON fixture packs, Qiongli bridge modules, local subject router evaluation script.

---

## File Map

- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Generalize accounting-only manifest-backed suggestion and borrow-lens paths to any runtime subject contract.
  - Support an optional `method_lenses` list on manifest signal entries so a business signal value can map to one or more business lenses.
  - Keep default suggestions gated by `activation_status: runtime_enabled`; allow eval-only suggestion only when `evaluation_subjects` contains the subject.
- Modify: `tooling/scripts/evaluate_subject_router.py`
  - Add business to subject-scoped onboarding signal dimensions for eval-ready gate checks.
- Modify: `content/subjects/business/runtime-subject.yaml`
  - Promote business from `candidate` to `eval_ready`.
  - Add method, data/outcome, venue, and theory/construct signal groups.
  - Add method lenses and point `evaluation_pack` at the business fixture pack.
- Create: `tests/fixtures/subject_router_eval/business/*.json`
  - Add clear-positive, method-only, mixed, locked, confirmed, and near-miss business fixtures.
- Modify: `tests/test_subject_refinement.py`
  - Cover business eval-only measurement, method-only lens borrowing, and runtime suppression while eval-ready.
- Modify: `tests/test_subject_contracts.py`
  - Cover business eval-ready manifest metadata and keep other deferred subjects as blank candidate shells.
- Modify: `tests/test_subject_router_eval.py`
  - Cover business fixture inventory, business eval-ready gate, business runtime-enabled gate failure, and remaining deferred candidate blockers.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark the onboarding contract complete and business eval-ready as the active Stage 4 slice.
- Modify: `docs/reference/cli.md`
  - Document business eval-ready as the current candidate readiness check.
- Modify: `docs/advanced/publish-pypi.md`
  - Add business eval-ready to optional subject runtime gate checks.

## Execution Notes

- Work on branch `feature/business-subject-eval-ready-plan`.
- Commit after each task. The controller will squash commits by content before opening the PR.
- Do not promote business to `runtime_enabled`.
- Do not change provider configuration, Zotero behavior, full-text retrieval, local-agent execution, or release automation.
- Do not broaden accounting, finance, or economics signals.
- The final PR target is `dev`.

## Task 1: Generalize Manifest-Backed Eval Subject Routing

**Files:**
- Modify: `tests/test_subject_refinement.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `tooling/scripts/evaluate_subject_router.py`

- [ ] **Step 1: Add failing business refinement tests**

In `tests/test_subject_refinement.py`, add this helper below `_runtime_subject_contract()`:

```python
def _business_runtime_subject_contract(
    *,
    activation_status: str = "eval_ready",
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="business",
        display_name="Business",
        activation_status=activation_status,
        extends="core",
        source="content/subjects/business/runtime-subject.yaml",
        domain_profile="content/skills/domain-profiles/business-management.yaml",
        overlay="",
        subject_skill="",
        signal_groups={
            "method": [
                {
                    "id": "business.method.case-study",
                    "value": "case-study",
                    "weight": 0.30,
                    "activation": "method_only",
                    "patterns": [r"\bmultiple case study\b"],
                    "method_lenses": [
                        "business-positioning",
                        "qualitative-transparency",
                    ],
                },
                {
                    "id": "business.method.gioia",
                    "value": "gioia-method",
                    "weight": 0.30,
                    "activation": "method_only",
                    "patterns": [r"\bGioia\b", r"\bfirst-order concepts\b"],
                    "method_lenses": ["qualitative-transparency"],
                },
            ],
            "data_or_outcome": [
                {
                    "id": "business.data.qualitative-fieldwork",
                    "value": "qualitative-fieldwork",
                    "weight": 0.25,
                    "activation": "subject",
                    "patterns": [r"\binterviews with managers\b"],
                }
            ],
            "venue": [
                {
                    "id": "business.venue.amj",
                    "value": "academy-of-management-journal",
                    "weight": 0.20,
                    "activation": "context_only",
                    "patterns": [r"\bAcademy of Management Journal\b", r"\bAMJ\b"],
                    "method_lenses": ["business-positioning"],
                }
            ],
            "theory_or_construct": [
                {
                    "id": "business.construct.theory-contribution",
                    "value": "theory-contribution",
                    "weight": 0.25,
                    "activation": "subject",
                    "patterns": [r"\bmanagement theory\b", r"\btheory contribution\b"],
                }
            ],
        },
        method_lenses={
            "business-positioning": {
                "resource": (
                    "content/subjects/business/skills/"
                    "business-journal-positioning-auditor.md"
                ),
                "activation": "method_only",
            },
            "qualitative-transparency": {
                "resource": "content/subjects/business/overlays/skills/study-designer.md",
                "activation": "method_only",
            },
        },
        evaluation_pack="tests/fixtures/subject_router_eval/business",
        near_miss_policy={"forbidden_subjects": ["finance", "economics"]},
        activation_gate={
            "required_metrics": {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            }
        },
    )
```

Add these tests inside `SubjectRefinementTests` near the accounting eval-ready tests:

```python
    def test_eval_ready_business_signals_can_be_measured_under_evaluation_subjects(self) -> None:
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
                evaluation_subjects={"business"},
            ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["overlays"], [])
        self.assertEqual(packet["loaded_resources"]["subject_skills"], [])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn(
            "activation_status=eval_ready",
            packet["loaded_resources"]["contract_warnings"][0],
        )

    def test_eval_ready_business_method_only_borrows_lens_without_subject_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "qualitative coding appendix",
                    "context": (
                        "Use the Gioia method with first-order concepts and "
                        "second-order themes for a qualitative coding appendix."
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
        self.assertIn(
            ("business", "qualitative-transparency"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )

    def test_eval_ready_business_default_runtime_does_not_suggest_business(self) -> None:
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

- [ ] **Step 2: Run the new tests and verify they fail**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_refinement.py::SubjectRefinementTests::test_eval_ready_business_signals_can_be_measured_under_evaluation_subjects \
  tests/test_subject_refinement.py::SubjectRefinementTests::test_eval_ready_business_method_only_borrows_lens_without_subject_suggestion \
  tests/test_subject_refinement.py::SubjectRefinementTests::test_eval_ready_business_default_runtime_does_not_suggest_business \
  -q
```

Expected: at least the eval-only business suggestion test fails because `subject_refinement.py` only suggests accounting from manifest-backed runtime subject matches.

- [ ] **Step 3: Add method-lens extraction for manifest signal records**

In `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`, update `_manifest_records_for_contract()` so records include declared `method_lenses` when the manifest entry defines a list of lens names:

```python
                method_lenses = [
                    str(lens)
                    for lens in list(entry.get("method_lenses", []) or [])
                    if isinstance(lens, str) and lens.strip()
                ]
                records.append(
                    {
                        "id": entry_id,
                        "subject": contract.subject,
                        "dimension": str(dimension),
                        "value": value,
                        "weight": _coerce_signal_weight(entry.get("weight", 0.0)),
                        "activation": str(entry.get("activation", "subject") or "subject"),
                        "source": "task_text",
                        "snippet": _snippet_for_match(text, match),
                        "method_lenses": method_lenses,
                    }
                )
```

In `_detect_manifest_signal_records()`, replace the current manifest method-lens collection:

```python
        method_lenses = _unique(
            [
                str(record["value"])
                for record in subject_records
                if record["dimension"] == "method"
                and str(record["value"]) in contract.method_lenses
            ]
        )
```

with this helper-based collection:

```python
        method_lenses = _runtime_subject_method_lenses(contract, subject_records)
```

Add this helper below `_detect_manifest_signal_records()`:

```python
def _runtime_subject_method_lenses(
    contract: RuntimeSubjectContract,
    subject_records: list[dict[str, Any]],
) -> list[str]:
    lenses: list[str] = []
    for record in subject_records:
        declared_lenses = record.get("method_lenses", [])
        if isinstance(declared_lenses, list):
            lenses.extend(
                str(lens)
                for lens in declared_lenses
                if isinstance(lens, str) and str(lens) in contract.method_lenses
            )
        if (
            str(record.get("dimension", "")) == "method"
            and str(record.get("value", "")) in contract.method_lenses
        ):
            lenses.append(str(record["value"]))
    return _unique(lenses)
```

- [ ] **Step 4: Replace accounting-specific manifest routing with generic runtime-subject routing**

In `infer_subject_refinement()`, replace the `accounting_match = ...` suggestion branch and the following accounting borrow branch with generic branches that work for accounting and business.

Add these helpers near `_candidate_subjects()`:

```python
def _runtime_subject_suggestion_match(
    signals: SubjectSignals,
    *,
    evaluation_subjects: set[str],
) -> RuntimeSubjectMatch | None:
    for subject, match in signals.runtime_subject_matches.items():
        if (
            match.has_subject_strength
            and _subject_can_be_suggested(
                subject,
                evaluation_subjects=evaluation_subjects,
            )
        ):
            return match
    return None


def _runtime_subject_borrow_match(
    signals: SubjectSignals,
    *,
    active_subject: str,
) -> RuntimeSubjectMatch | None:
    for subject, match in signals.runtime_subject_matches.items():
        if subject != active_subject and match.method_lenses:
            return match
    return None
```

Use the helpers in `infer_subject_refinement()` after the economics branch:

```python
    runtime_subject_match = _runtime_subject_suggestion_match(
        signals,
        evaluation_subjects=evaluation_subjects,
    )
    if runtime_subject_match is not None:
        subject = runtime_subject_match.subject
        method_lenses = _unique(list(runtime_subject_match.method_lenses))
        borrowed_lenses = _borrowed_lenses(subject, signals)
        return _packet(
            decision="suggest_subject",
            mode="suggested",
            active_subject=manifest.active_subject,
            primary_subject=subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill", "method_pack"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject=subject,
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "proposed"},
            summary=f"{subject.title()} subject measured from manifest-backed signals.",
            domain=_domain_for_subject(subject),
            confidence=0.75,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    runtime_borrow_match = _runtime_subject_borrow_match(
        signals,
        active_subject=manifest.active_subject,
    )
    if runtime_borrow_match is not None:
        borrowed_lenses = _borrowed_lenses(manifest.active_subject, signals)
        return _packet(
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
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
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "temporary"},
            summary=_summary(
                (
                    f"Borrowing {runtime_borrow_match.subject} method lens "
                    "without changing the project subject."
                ),
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=0.45,
            evidence=signals.evidence,
            signals=signals.signals,
        )
```

Keep the hard-coded finance and economics branches before this generic branch so existing finance/economics priority remains stable.

- [ ] **Step 5: Add business to eval-ready signal dimension requirements**

In `tooling/scripts/evaluate_subject_router.py`, update `REQUIRED_SIGNAL_DIMENSIONS_BY_SUBJECT`:

```python
REQUIRED_SIGNAL_DIMENSIONS_BY_SUBJECT = {
    "accounting": (
        "method",
        "data_or_outcome",
        "venue",
        "theory_or_construct",
    ),
    "business": (
        "method",
        "data_or_outcome",
        "venue",
        "theory_or_construct",
    ),
}
```

- [ ] **Step 6: Run focused routing tests**

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py tests/test_subject_router_eval.py -q
```

Expected: PASS.

- [ ] **Step 7: Commit generic routing**

Run:

```bash
git add \
  packages/python-qiongli/src/qiongli/bridges/subject_refinement.py \
  tooling/scripts/evaluate_subject_router.py \
  tests/test_subject_refinement.py
git commit -m "feat(subjects): generalize eval-ready manifest routing"
```

## Task 2: Add Business Manifest, Fixture Pack, And Gate Tests

**Files:**
- Modify: `content/subjects/business/runtime-subject.yaml`
- Create: `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`
- Create: `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`
- Create: `tests/fixtures/subject_router_eval/business/method_only_gioia_borrow.json`
- Create: `tests/fixtures/subject_router_eval/business/mixed_finance_strategy_returns.json`
- Create: `tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json`
- Create: `tests/fixtures/subject_router_eval/business/confirmed_business_journal_positioning.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_small_business_plan.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_consulting_market_analysis.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_project_management_workflow.json`
- Create: `tests/fixtures/subject_router_eval/business/near_miss_teaching_case_assignment.json`
- Modify: `tests/test_subject_contracts.py`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_refinement.py`

- [ ] **Step 1: Update contract tests for business eval-ready**

In `tests/test_subject_contracts.py`, update `test_default_repository_contracts_classify_runtime_enabled_and_candidates()` so business is eval-ready and the remaining deferred subjects stay candidates:

```python
        self.assertEqual(subject_activation_status("business", contracts), "eval_ready")
        self.assertIn("business", contracts)
        for subject in {
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }:
            self.assertEqual(subject_activation_status(subject, contracts), "candidate")
            self.assertIn(subject, contracts)
```

Update `test_default_deferred_candidate_subjects_are_manifest_shells()` so `deferred_subjects` excludes business:

```python
        deferred_subjects = {
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }
```

Add this test near the accounting manifest metadata test:

```python
    def test_business_eval_ready_manifest_declares_signals_and_method_lenses(self) -> None:
        contracts = load_runtime_subject_contracts()
        contract = contracts["business"]

        self.assertEqual(contract.activation_status, "eval_ready")
        self.assertEqual(
            contract.evaluation_pack,
            "tests/fixtures/subject_router_eval/business",
        )
        self.assertEqual(
            set(contract.signal_groups),
            {"method", "data_or_outcome", "venue", "theory_or_construct"},
        )
        valid_activations = {"subject", "method_only", "context_only"}
        for dimension in ("method", "data_or_outcome", "venue", "theory_or_construct"):
            self.assertTrue(contract.signal_groups[dimension], dimension)
            for entry in contract.signal_groups[dimension]:
                with self.subTest(dimension=dimension, signal_id=entry.get("id")):
                    self.assertIsInstance(entry["id"], str)
                    self.assertTrue(entry["id"].strip())
                    self.assertIsInstance(entry["value"], str)
                    self.assertTrue(entry["value"].strip())
                    self.assertIsInstance(entry["weight"], (int, float))
                    self.assertGreater(entry["weight"], 0)
                    self.assertIn(entry["activation"], valid_activations)
                    for field in ("patterns", "examples", "near_misses"):
                        self.assertIsInstance(entry[field], list)
                        self.assertTrue(entry[field], field)
                        for value in entry[field]:
                            self.assertIsInstance(value, str)
                            self.assertTrue(value.strip(), field)
                    for pattern in entry["patterns"]:
                        re.compile(pattern, re.I)
        self.assertIn("business-positioning", contract.method_lenses)
        self.assertIn("qualitative-transparency", contract.method_lenses)
        self.assertIn("construct-level-fit", contract.method_lenses)
        for lens in contract.method_lenses.values():
            self.assertEqual(lens["activation"], "method_only")
        self.assertEqual(
            contract.activation_gate["required_metrics"],
            {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            },
        )
```

- [ ] **Step 2: Run contract tests and verify they fail**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py::RuntimeSubjectContractTests::test_default_repository_contracts_classify_runtime_enabled_and_candidates \
  tests/test_subject_contracts.py::RuntimeSubjectContractTests::test_default_deferred_candidate_subjects_are_manifest_shells \
  tests/test_subject_contracts.py::RuntimeSubjectContractTests::test_business_eval_ready_manifest_declares_signals_and_method_lenses \
  -q
```

Expected: FAIL because business is still `candidate` with blank signals and no fixture pack.

- [ ] **Step 3: Replace the business runtime subject manifest**

Replace `content/subjects/business/runtime-subject.yaml` with:

```yaml
schema_version: 1.0
subject: business
display_name: Business
activation_status: eval_ready
extends: core
domain_profile: content/skills/domain-profiles/business-management.yaml
overlay: ""
subject_skill: ""
signal_groups:
  method:
    - id: business.method.gioia
      value: gioia-method
      weight: 0.30
      activation: method_only
      patterns:
        - "\\bGioia\\b"
        - "\\bfirst[- ]order concepts\\b"
        - "\\bsecond[- ]order themes\\b"
        - "\\baggregate dimensions\\b"
      method_lenses:
        - qualitative-transparency
      examples:
        - "Use the Gioia method with first-order concepts and aggregate dimensions."
      near_misses:
        - "Sort generic brainstormed themes for a workshop."
    - id: business.method.case-study
      value: case-study
      weight: 0.30
      activation: method_only
      patterns:
        - "\\bmultiple case stud(?:y|ies)\\b"
        - "\\bEisenhardt\\b"
        - "\\bYin case study\\b"
        - "\\bwithin[- ]case\\b"
        - "\\bcross[- ]case\\b"
      method_lenses:
        - qualitative-transparency
      examples:
        - "Build a multiple case study with within-case and cross-case analysis."
      near_misses:
        - "Write a teaching case or business case for a course."
    - id: business.method.process-research
      value: process-research
      weight: 0.25
      activation: method_only
      patterns:
        - "\\bprocess research\\b"
        - "\\btemporal bracketing\\b"
        - "\\bevent timeline\\b"
        - "\\bturning points\\b"
      method_lenses:
        - qualitative-transparency
      examples:
        - "Use temporal bracketing to explain organizational process change."
      near_misses:
        - "Improve a workflow process for an operations checklist."
    - id: business.method.journal-positioning
      value: journal-positioning
      weight: 0.20
      activation: method_only
      patterns:
        - "\\bbusiness journal positioning\\b"
        - "\\bjournal positioning\\b"
        - "\\bdoctoral[- ]level journal contribution\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Audit business journal positioning for a doctoral-level contribution."
      near_misses:
        - "Choose a trade magazine for practitioner visibility."
    - id: business.method.construct-fit
      value: construct-fit
      weight: 0.20
      activation: method_only
      patterns:
        - "\\bconstruct clarity\\b"
        - "\\blevel of analysis\\b"
        - "\\bconstruct[- ]level fit\\b"
      method_lenses:
        - construct-level-fit
      examples:
        - "Check construct clarity and level-of-analysis fit."
      near_misses:
        - "Clarify a product feature label."
  data_or_outcome:
    - id: business.data.organization-panel
      value: organization-panel
      weight: 0.25
      activation: subject
      patterns:
        - "\\bfirm[- ]level panel\\b"
        - "\\borganization[- ]level data\\b"
        - "\\bteam[- ]level data\\b"
        - "\\bmanager survey\\b"
        - "\\bemployee survey\\b"
      examples:
        - "Use firm-level panel data and manager survey measures."
      near_misses:
        - "Create a team schedule or employee rota."
    - id: business.data.qualitative-fieldwork
      value: qualitative-fieldwork
      weight: 0.25
      activation: subject
      patterns:
        - "\\binterviews with managers\\b"
        - "\\bfieldnotes\\b"
        - "\\borganizational ethnography\\b"
        - "\\bcase evidence database\\b"
        - "\\barchival documents\\b"
      examples:
        - "Use interviews with managers and fieldnotes as qualitative evidence."
      near_misses:
        - "Summarize meeting notes from a consulting project."
    - id: business.data.market-platform
      value: market-platform
      weight: 0.25
      activation: subject
      patterns:
        - "\\bplatform marketplace\\b"
        - "\\bcustomer journey\\b"
        - "\\bmarketing channel\\b"
        - "\\bfirm[- ]customer interaction\\b"
      examples:
        - "Study platform marketplace design and firm-customer interaction."
      near_misses:
        - "Write sales enablement copy for a channel campaign."
  venue:
    - id: business.venue.amj
      value: academy-of-management-journal
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bAcademy of Management Journal\\b"
        - "\\bAMJ\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Position the study for Academy of Management Journal."
      near_misses:
        - "Manage a journal club schedule."
    - id: business.venue.organization-science
      value: organization-science
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bOrganization Science\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Frame the paper for Organization Science."
      near_misses:
        - "Organize a science fair project."
    - id: business.venue.journal-of-management
      value: journal-of-management
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bJournal of Management\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Target Journal of Management."
      near_misses:
        - "Create a management journal for personal notes."
    - id: business.venue.journal-of-marketing
      value: journal-of-marketing
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bJournal of Marketing\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Position the manuscript for Journal of Marketing."
      near_misses:
        - "Draft a marketing journal entry for a campaign review."
    - id: business.venue.strategic-management-journal
      value: strategic-management-journal
      weight: 0.20
      activation: context_only
      patterns:
        - "\\bStrategic Management Journal\\b"
        - "\\bSMJ\\b"
      method_lenses:
        - business-positioning
      examples:
        - "Position the strategy contribution for Strategic Management Journal."
      near_misses:
        - "Write a strategic management memo."
  theory_or_construct:
    - id: business.construct.theory-contribution
      value: theory-contribution
      weight: 0.25
      activation: subject
      patterns:
        - "\\bmanagement theory\\b"
        - "\\btheory contribution\\b"
        - "\\bliterature stream\\b"
        - "\\bconstruct clarity\\b"
        - "\\bboundary conditions\\b"
      examples:
        - "Name the management theory contribution and boundary conditions."
      near_misses:
        - "Describe a practical business idea without a theory contribution."
    - id: business.construct.organization-mechanism
      value: organization-mechanism
      weight: 0.25
      activation: subject
      patterns:
        - "\\borganizational mechanism\\b"
        - "\\bstrategy mechanism\\b"
        - "\\bcapabilit(?:y|ies)\\b"
        - "\\bdynamic capabilities\\b"
        - "\\borganizational routines\\b"
      examples:
        - "Explain the organizational mechanism and dynamic capabilities."
      near_misses:
        - "List operational capabilities for a product launch."
    - id: business.construct.managerial-implication
      value: managerial-implication
      weight: 0.20
      activation: subject
      patterns:
        - "\\bmanagerial implication\\b"
        - "\\bbusiness phenomenon\\b"
        - "\\bstrategic management\\b"
        - "\\bcompetitive advantage\\b"
      examples:
        - "Connect the business phenomenon to strategic management theory."
      near_misses:
        - "Write a consulting recommendation deck."
method_lenses:
  business-positioning:
    resource: content/subjects/business/skills/business-journal-positioning-auditor.md
    activation: method_only
  qualitative-transparency:
    resource: content/subjects/business/overlays/skills/study-designer.md
    activation: method_only
  construct-level-fit:
    resource: content/subjects/business/overlays/skills/manuscript-architect.md
    activation: method_only
evaluation_pack: tests/fixtures/subject_router_eval/business
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

- [ ] **Step 4: Add router eval inventory and business gate tests**

In `tests/test_subject_router_eval.py`, update `test_load_eval_cases_reads_all_fixtures()` to require these business fixture ids:

```python
        required_business_ids = {
            "business_clear_management_theory_case_study",
            "business_clear_marketing_platform_experiment",
            "business_method_only_gioia_borrow",
            "business_mixed_finance_strategy_returns",
            "business_locked_economics_borrow_positioning",
            "business_confirmed_journal_positioning",
            "business_near_miss_small_business_plan",
            "business_near_miss_consulting_market_analysis",
            "business_near_miss_project_management_workflow",
            "business_near_miss_teaching_case_assignment",
        }
        self.assertTrue(required_business_ids.issubset(set(ids)))
        business_tags = {
            tag
            for case_id in required_business_ids
            for tag in list(cases_by_id[case_id].tags or [])
        }
        self.assertTrue(
            {
                "clear_positive",
                "method_only_borrow",
                "mixed_subject",
                "near_miss",
                "locked_subject",
                "confirmed_subject",
            }.issubset(business_tags)
        )
```

Update `test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons()` so `deferred_subjects` excludes business:

```python
        deferred_subjects = (
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        )
```

Add these tests near the accounting runtime gate tests:

```python
    def test_business_eval_ready_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "business")
        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)

    def test_business_runtime_enabled_gate_blocks_eval_ready_manifest(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("business", cases, gate="runtime-enabled")

        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])
```

- [ ] **Step 5: Create business fixture directory**

Run:

```bash
mkdir -p tests/fixtures/subject_router_eval/business
```

- [ ] **Step 6: Create business clear-positive fixtures**

Create `tests/fixtures/subject_router_eval/business/clear_management_theory_case_study.json`:

```json
{
  "id": "business_clear_management_theory_case_study",
  "subject_under_test": "business",
  "tags": ["business", "clear_positive"],
  "description": "Management theory case-study request with method, fieldwork, venue, and construct signals.",
  "request": "Design a multiple case study using interviews with managers, fieldnotes, and archival documents to develop a management theory contribution about organizational routines, construct clarity, and boundary conditions for Academy of Management Journal business journal positioning.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": ["business-positioning", "qualitative-transparency", "construct-level-fit"]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": ["business"],
      "forbidden_subjects": [],
      "method_lenses": ["business-positioning", "qualitative-transparency", "construct-level-fit"]
    },
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "auto",
      "suggest_subjects": [],
      "forbidden_subjects": ["business"],
      "method_lenses": ["business-positioning", "qualitative-transparency", "construct-level-fit"]
    }
  }
}
```

Create `tests/fixtures/subject_router_eval/business/clear_marketing_platform_experiment.json`:

```json
{
  "id": "business_clear_marketing_platform_experiment",
  "subject_under_test": "business",
  "tags": ["business", "clear_positive"],
  "description": "Marketing platform request with market-platform data, venue, and theory contribution signals.",
  "request": "Frame a Journal of Marketing manuscript on platform marketplace design, customer journey mechanisms, marketing channel strategy, firm-customer interaction, theory contribution, and managerial implication with explicit journal positioning.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": ["business-positioning"]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "business",
      "suggest_subjects": ["business"],
      "forbidden_subjects": [],
      "method_lenses": ["business-positioning"]
    },
    "runtime-enabled": {
      "decision": "recommend",
      "primary_subject": "auto",
      "suggest_subjects": [],
      "forbidden_subjects": ["business"],
      "method_lenses": ["business-positioning"]
    }
  }
}
```

- [ ] **Step 7: Create method-only, mixed, locked, and confirmed fixtures**

Create `tests/fixtures/subject_router_eval/business/method_only_gioia_borrow.json`:

```json
{
  "id": "business_method_only_gioia_borrow",
  "subject_under_test": "business",
  "tags": ["business", "method_only_borrow"],
  "description": "Gioia method wording borrows a business qualitative lens without suggesting business.",
  "request": "Use the Gioia method with first-order concepts, second-order themes, and aggregate dimensions to organize qualitative coding, but do not change the project subject.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": ["qualitative-transparency"]
  }
}
```

Create `tests/fixtures/subject_router_eval/business/mixed_finance_strategy_returns.json`:

```json
{
  "id": "business_mixed_finance_strategy_returns",
  "subject_under_test": "business",
  "tags": ["business", "mixed_subject"],
  "description": "Finance-dominant request with adjacent business strategy language should keep finance primary.",
  "request": "Design an event study of abnormal returns around alliance announcements using CRSP data, while discussing competitive advantage and strategy mechanism as secondary framing.",
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
    "primary_subject": "finance",
    "suggest_subjects": ["finance"],
    "allowed_neighbor_subjects": ["business"],
    "forbidden_subjects": [],
    "method_lenses": ["event-study"]
  },
  "gate_expected": {
    "eval-ready": {
      "decision": "recommend",
      "primary_subject": "finance",
      "suggest_subjects": ["finance"],
      "allowed_neighbor_subjects": ["business"],
      "forbidden_subjects": [],
      "method_lenses": ["event-study"]
    }
  }
}
```

Create `tests/fixtures/subject_router_eval/business/locked_economics_borrow_business_positioning.json`:

```json
{
  "id": "business_locked_economics_borrow_positioning",
  "subject_under_test": "business",
  "tags": ["business", "locked_subject"],
  "description": "Locked economics project may borrow business journal positioning without switching subject.",
  "request": "Keep the paper locked to economics, but borrow business journal positioning for AMJ and clarify the managerial implication without changing the primary subject.",
  "manifest": {
    "active_subject": "economics",
    "subject_mode": "locked",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "keep_locked",
    "primary_subject": "economics",
    "suggest_subjects": [],
    "forbidden_subjects": ["business"],
    "method_lenses": ["business-positioning"]
  }
}
```

Create `tests/fixtures/subject_router_eval/business/confirmed_business_journal_positioning.json`:

```json
{
  "id": "business_confirmed_journal_positioning",
  "subject_under_test": "business",
  "tags": ["business", "confirmed_subject"],
  "description": "Confirmed business project remains business while eval-ready subject resources are withheld from runtime loading.",
  "request": "Tighten the Academy of Management Journal positioning, management theory contribution, construct clarity, and boundary conditions for this business manuscript.",
  "manifest": {
    "active_subject": "business",
    "subject_mode": "confirmed",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": ["business-positioning"],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "business",
    "suggest_subjects": [],
    "forbidden_subjects": [],
    "method_lenses": ["business-positioning"]
  }
}
```

- [ ] **Step 8: Create business near-miss fixtures**

Create `tests/fixtures/subject_router_eval/business/near_miss_small_business_plan.json`:

```json
{
  "id": "business_near_miss_small_business_plan",
  "subject_under_test": "business",
  "tags": ["business", "near_miss"],
  "description": "Small business plan wording must not activate scholarly business.",
  "request": "Help me draft a small business plan with pricing, hiring, inventory, and a launch checklist for a local service company.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": []
  }
}
```

Create `tests/fixtures/subject_router_eval/business/near_miss_consulting_market_analysis.json`:

```json
{
  "id": "business_near_miss_consulting_market_analysis",
  "subject_under_test": "business",
  "tags": ["business", "near_miss"],
  "description": "Consulting market analysis wording must stay core.",
  "request": "Prepare a consulting market analysis with competitor bullets, sales enablement messages, and product positioning copy for a client workshop.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": []
  }
}
```

Create `tests/fixtures/subject_router_eval/business/near_miss_project_management_workflow.json`:

```json
{
  "id": "business_near_miss_project_management_workflow",
  "subject_under_test": "business",
  "tags": ["business", "near_miss"],
  "description": "Project management operations wording must not activate business.",
  "request": "Create a project management workflow with milestones, task owners, delivery risks, and sprint review notes.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": []
  }
}
```

Create `tests/fixtures/subject_router_eval/business/near_miss_teaching_case_assignment.json`:

```json
{
  "id": "business_near_miss_teaching_case_assignment",
  "subject_under_test": "business",
  "tags": ["business", "near_miss"],
  "description": "Teaching case assignment wording must not activate scholarly business routing.",
  "request": "Write discussion questions for a teaching case assignment about a startup founder and classroom participation.",
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
    "forbidden_subjects": ["business"],
    "method_lenses": []
  }
}
```

- [ ] **Step 9: Add real business behavior tests in subject refinement**

In `tests/test_subject_refinement.py`, add these tests near the generic business tests from Task 1:

```python
    def test_business_eval_ready_real_manifest_can_be_measured_under_evaluation_subjects(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "management theory case study",
                "context": (
                    "Design a multiple case study using interviews with managers "
                    "to develop a management theory contribution about organizational "
                    "routines for Academy of Management Journal business journal positioning."
                ),
            },
            manifest_state=ProjectManifest(),
            evaluation_subjects={"business"},
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])

    def test_business_eval_ready_real_manifest_does_not_activate_in_default_runtime(self) -> None:
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

        self.assertNotEqual(packet["decision"], "suggest_subject")
        self.assertNotEqual(packet["primary_subject"], "business")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
```

- [ ] **Step 10: Run focused tests and business gates**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected: PASS.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate eval-ready \
  --json
```

Expected: exit `0`, `subject_gate.eligible_for_eval_ready: true`, and `subject_gate.blocking_failures: []`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json
```

Expected: exit `1`, `subject_gate.eligible_for_runtime_enabled: false`, and `subject_gate.blocking_failures` includes `activation_status is eval_ready`.

- [ ] **Step 11: Commit business manifest and fixtures**

Run:

```bash
git add \
  content/subjects/business/runtime-subject.yaml \
  tests/fixtures/subject_router_eval/business \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py
git commit -m "feat(subjects): add business eval-ready gate pack"
```

## Task 3: Update Roadmap, Docs, And Final Gate Coverage

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/advanced/publish-pypi.md`

- [ ] **Step 1: Update roadmap priority section**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`, replace the `Priority Update: Subject Expansion Onboarding Contract` section with a `Priority Update: Business Eval-Ready Slice` section:

```markdown
## Priority Update: Business Eval-Ready Slice

Status: the subject expansion onboarding contract is complete, and business is
the current Stage 4 subject expansion slice.

The accounting runtime-enabled gate has been reviewed and remains green.
Business now has a dedicated eval-ready design and execution plan. This slice
keeps business below runtime activation while adding a business-owned fixture
pack, manifest-backed method/data/venue/construct signals, method-lens
borrowing, near-miss guards, and eval-ready gate coverage.

Business must remain `eval_ready`, not `runtime_enabled`, until a separate
runtime promotion review proves default precision after the eval-ready pack is
merged.

Formal design and execution plan:

- `docs/superpowers/specs/2026-07-05-accounting-runtime-promotion-design.md`
- `docs/superpowers/plans/2026-07-05-accounting-runtime-promotion.md`
- `docs/superpowers/specs/2026-07-05-subject-expansion-onboarding-contract-design.md`
- `docs/superpowers/plans/2026-07-05-subject-expansion-onboarding-contract.md`
- `docs/superpowers/specs/2026-07-05-business-subject-eval-ready-design.md`
- `docs/superpowers/plans/2026-07-05-business-subject-eval-ready.md`
```

In the Stage 4 status paragraph, change it to:

```markdown
Status: accounting eval-ready and runtime promotion are completed as of
July 5, 2026. The subject expansion onboarding contract is complete. Business
is the current eval-ready slice; political economy, geoeconomics, and the
economics-accounting bridge remain deferred candidates.
```

Update the subject lists:

```markdown
Runtime-enabled subjects:

- Accounting.
- Economics.
- Finance.

Eval-ready subjects:

- Business and management.

Deferred candidate subjects:

- Political economy.
- Geoeconomics.
- Economics-accounting bridge.
```

- [ ] **Step 2: Update CLI gate docs**

In `docs/reference/cli.md`, update the final sentence of the Subject Expansion Gate section:

```markdown
`eligible_for_eval_ready: true` means the subject has a passing fixture pack and
metadata that maintainers can review. It does not allow adaptive runtime
suggestions. Business is the current eval-ready subject. Political economy,
geoeconomics, and economics-accounting remain future eval-ready candidates.
```

- [ ] **Step 3: Update release gate docs**

In `docs/advanced/publish-pypi.md`, update the subject runtime gate checks block:

````markdown
Subject runtime gate checks:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate eval-ready \
  --json
```

The default router evaluation and accounting runtime-enabled gate must pass for
release. Business eval-ready should also pass while business remains below
runtime activation.
````

Update the candidate paragraph:

```markdown
Business is eval-ready. Political economy, geoeconomics, and
economics-accounting remain future eval-ready candidates.
```

- [ ] **Step 4: Run required final verification**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected: PASS.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate eval-ready \
  --json
```

Expected: exit `0`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json
```

Expected: exit `1` with `activation_status is eval_ready`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

Expected: exit `0`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject finance \
  --gate runtime-enabled \
  --json
```

Expected: exit `0`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject economics \
  --gate runtime-enabled \
  --json
```

Expected: exit `0`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit `0`, no threshold failures, and no new near-miss false positives.

Run:

```bash
git diff --check
git status --short
```

Expected: `git diff --check` exits `0`; status shows only intended files before commit.

- [ ] **Step 5: Commit docs and roadmap**

Run:

```bash
git add \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md
git commit -m "docs(subjects): document business eval-ready gate"
```

## Final Controller Steps

The controller performs these steps after all subagent tasks pass review:

- [ ] Run the full required verification commands from Task 3 Step 4.
- [ ] Run `git log --oneline dev..HEAD` and inspect the commit stack.
- [ ] Squash the branch into structured commits:
  - `docs(subjects): plan business eval-ready slice`
  - `feat(subjects): add business eval-ready routing and fixtures`
  - `docs(subjects): document business eval-ready gate`
- [ ] Run the final verification commands again after the squash.
- [ ] Push the branch to origin.
- [ ] Create a pull request targeting `dev`.
