# Subject Expansion Onboarding Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make deferred subject expansion explicit and measurable before adding the first post-accounting subject pack.

**Architecture:** Keep runtime subjects unchanged and harden only the onboarding/gate layer. Candidate manifests become honest deferred shells with blank fixture packs, while `evaluate_subject_router.py` reports explicit readiness blockers for blank packs, mismatched subject packs, and empty candidate signal dimensions.

**Tech Stack:** Python 3, unittest/pytest, YAML runtime subject manifests, JSON subject-router fixtures, deterministic gate scripts.

---

## File Map

- Modify: `content/subjects/business/runtime-subject.yaml`
  - Clear `evaluation_pack` for the deferred business shell.
- Modify: `content/subjects/political-economy/runtime-subject.yaml`
  - Clear `evaluation_pack` for the deferred political economy shell.
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`
  - Clear `evaluation_pack` for the deferred geoeconomics shell.
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`
  - Clear `evaluation_pack` for the deferred economics-accounting shell.
- Modify: `tests/test_subject_contracts.py`
  - Add default repository contract coverage for deferred candidate manifest
    hygiene.
- Modify: `tests/test_subject_router_eval.py`
  - Add gate tests for deferred-shell diagnostics and subject-pack mismatch.
- Modify: `tooling/scripts/evaluate_subject_router.py`
  - Add onboarding diagnostics used by the eval-ready gate.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark this onboarding contract as the current Stage 4 prerequisite before
    the first post-accounting subject pack.

## Execution Notes

- Execute from branch `feature/subject-expansion-onboarding-contract-plan`.
- Follow TDD for Tasks 1 and 2: write tests, run them to observe the expected
  failure, implement the minimum change, then rerun.
- Commit after each task with a narrow Conventional Commit message.
- Do not promote any subject in this slice.
- Do not add business, political economy, geoeconomics, or economics-accounting
  signal groups.
- Do not change accounting, finance, or economics runtime behavior.
- Do not change provider, Zotero, literature-search, full-text, local-agent, or
  release automation code.

## Task 1: Guard Deferred Candidate Manifest Hygiene

**Files:**
- Modify: `tests/test_subject_contracts.py`
- Modify: `content/subjects/business/runtime-subject.yaml`
- Modify: `content/subjects/political-economy/runtime-subject.yaml`
- Modify: `content/subjects/geoeconomics/runtime-subject.yaml`
- Modify: `content/subjects/economics-accounting/runtime-subject.yaml`

- [ ] **Step 1: Add the failing contract hygiene test**

In `tests/test_subject_contracts.py`, add this test inside
`RuntimeSubjectContractTests` near the default repository contract tests:

```python
    def test_default_deferred_candidate_subjects_are_manifest_shells(self) -> None:
        contracts = load_runtime_subject_contracts()
        deferred_subjects = {
            "business",
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }

        for subject in sorted(deferred_subjects):
            with self.subTest(subject=subject):
                contract = contracts[subject]
                self.assertEqual(contract.activation_status, "candidate")
                self.assertEqual(contract.evaluation_pack, "")
                self.assertNotEqual(
                    contract.evaluation_pack,
                    "tests/fixtures/subject_router_eval/accounting",
                )
                self.assertEqual(contract.method_lenses, {})
                self.assertTrue(
                    all(
                        isinstance(entries, list) and not entries
                        for entries in contract.signal_groups.values()
                    )
                )

        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("finance", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("economics", contracts),
            "runtime_enabled",
        )
```

- [ ] **Step 2: Run the new test and verify RED**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py::RuntimeSubjectContractTests::test_default_deferred_candidate_subjects_are_manifest_shells \
  -q
```

Expected before manifest changes: FAIL because at least one deferred candidate
has `evaluation_pack == "tests/fixtures/subject_router_eval/accounting"`.

- [ ] **Step 3: Clear the deferred candidate evaluation packs**

In each candidate manifest listed below, replace:

```yaml
evaluation_pack: tests/fixtures/subject_router_eval/accounting
```

with:

```yaml
evaluation_pack: ""
```

Files:

```text
content/subjects/business/runtime-subject.yaml
content/subjects/political-economy/runtime-subject.yaml
content/subjects/geoeconomics/runtime-subject.yaml
content/subjects/economics-accounting/runtime-subject.yaml
```

Do not change `activation_status`, `signal_groups`, `method_lenses`,
`subject_skill`, or venue-profile resources in this task.

- [ ] **Step 4: Run focused contract tests and verify GREEN**

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py -q
```

Expected after manifest changes: PASS.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add \
  tests/test_subject_contracts.py \
  content/subjects/business/runtime-subject.yaml \
  content/subjects/political-economy/runtime-subject.yaml \
  content/subjects/geoeconomics/runtime-subject.yaml \
  content/subjects/economics-accounting/runtime-subject.yaml
git commit -m "test(subjects): guard deferred subject manifests"
```

## Task 2: Add Explicit Eval-Ready Onboarding Diagnostics

**Files:**
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tooling/scripts/evaluate_subject_router.py`

- [ ] **Step 1: Add a subject-gate case helper for non-accounting subjects**

In `tests/test_subject_router_eval.py`, add this helper below `_gate_case`:

```python
def _subject_gate_case(subject: str, case_id: str, tags: list[str]) -> EvalCase:
    base = _gate_case(case_id, tags)
    return replace(
        base,
        id=case_id,
        description=case_id,
        source=f"tests/fixtures/subject_router_eval/{subject}/{case_id}.json",
        subject_under_test=subject,
        tags=[subject, *tags],
    )
```

- [ ] **Step 2: Add a patched business contract helper**

In `tests/test_subject_router_eval.py`, add this helper below
`_accounting_contract`:

```python
def _business_contract(
    *,
    activation_status: str = "candidate",
    source: str | Path | None = None,
    evaluation_pack: str = "",
    signal_groups: Mapping[str, list[Mapping[str, Any]]] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="business",
        display_name="Business",
        activation_status=activation_status,
        extends="core",
        source=str(
            source or Path("content/subjects/business/runtime-subject.yaml").resolve()
        ),
        domain_profile="content/skills/domain-profiles/business-management.yaml",
        overlay="",
        subject_skill="",
        signal_groups={
            key: [dict(item) for item in value]
            for key, value in (
                signal_groups
                or {
                    "method": [],
                    "data_or_outcome": [],
                    "venue": [],
                    "theory_or_construct": [],
                }
            ).items()
        },
        method_lenses={},
        evaluation_pack=evaluation_pack,
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

- [ ] **Step 3: Add failing deferred-shell gate tests**

In `tests/test_subject_router_eval.py`, add these tests near the existing gate
tests:

```python
    def test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)
        deferred_subjects = (
            "business",
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        )

        for subject in deferred_subjects:
            with self.subTest(subject=subject):
                report = subject_gate_report(subject, cases, gate="eval-ready")

                self.assertEqual(report["subject"], subject)
                self.assertEqual(report["activation_status"], "candidate")
                self.assertFalse(report["eligible_for_eval_ready"])
                self.assertFalse(report["eligible_for_runtime_enabled"])
                self.assertEqual(report["case_count"], 0)
                self.assertIn(
                    "activation_status is candidate",
                    report["blocking_failures"],
                )
                self.assertIn(
                    "missing evaluation_pack for deferred subject",
                    report["blocking_failures"],
                )
                for dimension in (
                    "method",
                    "data_or_outcome",
                    "venue",
                    "theory_or_construct",
                ):
                    self.assertIn(
                        f"missing signal dimension: {dimension}",
                        report["blocking_failures"],
                    )
                for tag in (
                    "clear_positive",
                    "method_only_borrow",
                    "near_miss",
                ):
                    self.assertIn(
                        f"missing {tag} fixtures",
                        report["blocking_failures"],
                    )

    def test_eval_ready_gate_reports_subject_specific_pack_mismatch(self) -> None:
        cases = [
            _subject_gate_case("business", "business_clear", ["clear_positive"]),
            _subject_gate_case(
                "business",
                "business_method",
                ["method_only_borrow"],
            ),
            _subject_gate_case("business", "business_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "business": _business_contract(
                    activation_status="eval_ready",
                    evaluation_pack="tests/fixtures/subject_router_eval/accounting",
                    signal_groups={
                        "method": [{"id": "business.method.case-study"}],
                        "data_or_outcome": [
                            {"id": "business.data.organization-panel"}
                        ],
                        "venue": [{"id": "business.venue.amj"}],
                        "theory_or_construct": [
                            {"id": "business.construct.capability"}
                        ],
                    },
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("business", cases, gate="eval-ready")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertIn(
            "evaluation_pack subject mismatch: expected business, found accounting",
            report["blocking_failures"],
        )
```

- [ ] **Step 4: Run new gate tests and verify RED**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_router_eval.py::SubjectRouterEvalTests::test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons \
  tests/test_subject_router_eval.py::SubjectRouterEvalTests::test_eval_ready_gate_reports_subject_specific_pack_mismatch \
  -q
```

Expected before implementation: FAIL because the new blocking failure strings
are not produced yet.

- [ ] **Step 5: Add onboarding constants and eval-ready diagnostics**

In `tooling/scripts/evaluate_subject_router.py`, add this constant near
`REQUIRED_SIGNAL_DIMENSIONS_BY_SUBJECT`:

```python
ONBOARDING_SIGNAL_DIMENSIONS = (
    "method",
    "data_or_outcome",
    "venue",
    "theory_or_construct",
)
```

Inside `subject_gate_report()`, in the `if gate == "eval-ready":` block,
change the logic to include onboarding diagnostics before resource checks:

```python
    if gate == "eval-ready":
        if activation_status != "eval_ready":
            blocking_failures.append(f"activation_status is {activation_status}")
        if contract is not None:
            blocking_failures.extend(_evaluation_pack_onboarding_failures(contract))
            blocking_failures.extend(
                _missing_onboarding_signal_dimension_failures(contract)
            )
        if contract is not None and activation_status == "eval_ready":
            blocking_failures.extend(_missing_eval_ready_resource_failures(contract))
            blocking_failures.extend(_missing_signal_dimension_failures(contract))
```

Add these helper functions above `_missing_eval_ready_resource_failures()`:

```python
def _evaluation_pack_onboarding_failures(contract: Any) -> list[str]:
    evaluation_pack = getattr(contract, "evaluation_pack", "")
    if not isinstance(evaluation_pack, str) or not evaluation_pack.strip():
        return ["missing evaluation_pack for deferred subject"]

    actual_subject = _subject_specific_evaluation_pack_subject(evaluation_pack)
    expected_subject = str(getattr(contract, "subject", "") or "")
    if actual_subject and expected_subject and actual_subject != expected_subject:
        return [
            "evaluation_pack subject mismatch: "
            f"expected {expected_subject}, found {actual_subject}"
        ]
    return []


def _subject_specific_evaluation_pack_subject(evaluation_pack: str) -> str:
    parts = Path(evaluation_pack.strip()).parts
    marker = ("tests", "fixtures", "subject_router_eval")
    for index in range(0, len(parts) - len(marker) + 1):
        if tuple(parts[index : index + len(marker)]) != marker:
            continue
        tail = parts[index + len(marker) :]
        return tail[0] if tail else ""
    return ""


def _missing_onboarding_signal_dimension_failures(contract: Any) -> list[str]:
    activation_status = str(getattr(contract, "activation_status", "") or "")
    if activation_status == "runtime_enabled":
        return []

    signal_groups = getattr(contract, "signal_groups", {})
    if _has_any_signal_group_entry(signal_groups):
        return []
    return [
        f"missing signal dimension: {dimension}"
        for dimension in ONBOARDING_SIGNAL_DIMENSIONS
    ]


def _has_any_signal_group_entry(signal_groups: Any) -> bool:
    if not isinstance(signal_groups, Mapping):
        return False
    return any(
        isinstance(entries, list) and any(isinstance(entry, Mapping) for entry in entries)
        for entries in signal_groups.values()
    )
```

- [ ] **Step 6: Run focused gate tests and verify GREEN**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_router_eval.py::SubjectRouterEvalTests::test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons \
  tests/test_subject_router_eval.py::SubjectRouterEvalTests::test_eval_ready_gate_reports_subject_specific_pack_mismatch \
  -q
```

Expected after implementation: PASS.

- [ ] **Step 7: Run the full subject router eval tests**

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected: PASS. If patched eval-ready tests now fail because a test helper
represents an eval-ready subject with empty `signal_groups`, update only that
test helper data to include minimal reviewed signal groups. Do not weaken the
production onboarding diagnostics.

- [ ] **Step 8: Check candidate gate JSON output**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate eval-ready \
  --json
```

Expected: command exits non-zero because business is still a deferred
candidate. The JSON `subject_gate.blocking_failures` must include:

```text
activation_status is candidate
missing evaluation_pack for deferred subject
missing signal dimension: method
missing signal dimension: data_or_outcome
missing signal dimension: venue
missing signal dimension: theory_or_construct
missing clear_positive fixtures
missing method_only_borrow fixtures
missing near_miss fixtures
```

- [ ] **Step 9: Commit Task 2**

Run:

```bash
git add tests/test_subject_router_eval.py tooling/scripts/evaluate_subject_router.py
git commit -m "feat(subjects): report deferred subject readiness"
```

## Task 3: Update Roadmap For Onboarding Contract Priority

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update the priority section**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`,
replace the `## Priority Update: Accounting Gate Review And Next Subject Spec`
section heading with:

```markdown
## Priority Update: Subject Expansion Onboarding Contract
```

Update the section body so it says:

```markdown
Status: accounting runtime promotion completed by the accounting runtime
promotion change after the full-cycle workflow harness, manuscript-first
journal fit, and accounting eval-ready pack were completed.

The accounting runtime-enabled gate has been reviewed and remains green. The
next Stage 4 prerequisite is the subject expansion onboarding contract, which
prevents deferred candidate subjects from reusing another subject's fixture
pack and makes missing fixture/signal readiness explicit in gate reports.

After this onboarding contract merges, prepare the first post-accounting
subject spec. Business is the recommended next candidate because business
resources and venue profiles already exist, but it must remain `candidate`
until a business-owned fixture pack, near-miss guards, and signal groups are
reviewed.
```

Keep the existing accounting runtime promotion spec and plan links. Add the
new onboarding spec and plan links to the formal design list:

```markdown
- `docs/superpowers/specs/2026-07-05-subject-expansion-onboarding-contract-design.md`
- `docs/superpowers/plans/2026-07-05-subject-expansion-onboarding-contract.md`
```

- [ ] **Step 2: Update Stage 4 status and recommended immediate plan**

In the Stage 4 status paragraph, replace wording that says the next step is to
review the accounting runtime-enabled gate report with wording that says the
current prerequisite is the onboarding contract.

In `## Recommended Immediate Plan`, use this ordered list:

```markdown
1. Merge the subject expansion onboarding contract so deferred candidate
   subjects cannot reuse another subject's fixture pack and gate reports expose
   missing readiness explicitly.
2. Prepare the first post-accounting subject spec after the onboarding contract
   is merged. Business is the recommended next candidate unless review chooses
   political economy, geoeconomics, or the economics-accounting bridge.
3. Keep business, political economy, geoeconomics, and economics-accounting as
   deferred specs until their fixture packs and activation criteria are
   reviewed.
4. If subject expansion is deferred after onboarding, continue Stage 5
   feedback-aware explainability work so router outputs separate task-text,
   manifest, trace-memory, and user-action evidence more clearly.
```

Update `Current Stage 4 execution sequence` to:

```markdown
- Merge the subject expansion onboarding contract.
- Prepare a separate business eval-ready spec unless review selects another
  candidate.
- Keep political economy, geoeconomics, and economics-accounting as separate
  follow-up specs.
```

- [ ] **Step 3: Search for stale roadmap wording**

Run:

```bash
rg -n "Review the accounting runtime-enabled gate report before starting|choose the next reviewed subject expansion spec|Accounting Gate Review And Next Subject Spec" \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
```

Expected: no matches. Historical specs and older plans may still contain old
wording; this search is limited to the active roadmap.

- [ ] **Step 4: Commit Task 3**

Run:

```bash
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(subjects): prioritize subject onboarding contract"
```

## Task 4: Final Verification And Review

**Files:**
- No new files expected.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  -q
```

Expected: PASS.

- [ ] **Step 2: Run runtime-enabled gate checks**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject finance \
  --gate runtime-enabled \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject economics \
  --gate runtime-enabled \
  --json
```

Expected: all three commands exit `0`. Each `subject_gate.blocking_failures`
array must be empty.

- [ ] **Step 3: Run deferred candidate gate checks**

Run each command separately:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate eval-ready \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject political-economy \
  --gate eval-ready \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject geoeconomics \
  --gate eval-ready \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject economics-accounting \
  --gate eval-ready \
  --json
```

Expected: each command exits non-zero because each subject remains
`candidate`. Each JSON report must include:

```text
activation_status is candidate
missing evaluation_pack for deferred subject
missing clear_positive fixtures
missing method_only_borrow fixtures
missing near_miss fixtures
```

Each JSON report must also include all four missing onboarding signal
dimensions:

```text
missing signal dimension: method
missing signal dimension: data_or_outcome
missing signal dimension: venue
missing signal dimension: theory_or_construct
```

- [ ] **Step 4: Run full subject router evaluation**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit `0`; existing accounting, finance, economics, and core fixtures
remain green.

- [ ] **Step 5: Check diff hygiene and branch status**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: `git diff --check` has no output. `git status --short --branch`
shows a clean `feature/subject-expansion-onboarding-contract-plan` branch after
all task commits.

- [ ] **Step 6: Final review**

Dispatch one final independent reviewer over the full branch diff from
`84158866` to `HEAD`. The reviewer must check:

- Deferred candidate manifests no longer point at the accounting fixture pack.
- Eval-ready gate diagnostics are explicit and not hard-coded to only business.
- Accounting, finance, and economics runtime gates remain compatible.
- Roadmap priority matches the implemented onboarding contract.
- No subject was promoted and no provider/Zotero/full-text code changed.

If the reviewer finds issues, fix them with a narrow follow-up commit and rerun
the affected tests before reporting completion.

## Self-Review Checklist

- [ ] Plan and spec are both committed to git.
- [ ] Candidate subject manifests are honest deferred shells.
- [ ] Eval-ready gate reports blank pack and subject-pack mismatch explicitly.
- [ ] Missing signal dimension diagnostics appear for deferred shells.
- [ ] Accounting runtime-enabled gate still passes.
- [ ] Finance and economics runtime-enabled gates still pass.
- [ ] Business, political economy, geoeconomics, and economics-accounting remain
      `candidate`.
- [ ] Roadmap points to onboarding before business eval-ready work.
