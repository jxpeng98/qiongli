# Accounting Runtime Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote accounting from eval-ready to runtime-enabled while preserving method-only borrowed-lens safety and adjacent-subject precision.

**Architecture:** Treat promotion as a small gated activation diff: add one auto-mode method-only guard fixture, update accounting activation status, update tests/docs that encode the old eval-ready state, and verify accounting, finance, economics, and full-cycle harness regressions together.

**Tech Stack:** Python 3, unittest/pytest, JSON router fixtures, YAML runtime subject manifests, Qiongli bridge modules, deterministic harness scripts.

---

## File Map

- Create: `tests/fixtures/subject_router_eval/accounting/method_only_auto_accrual_controls.json`
  - Guards against accounting runtime over-activation from one method-only
    signal in auto mode.
- Modify: `tests/test_subject_router_eval.py`
  - Add the new fixture id to the accounting fixture inventory test.
  - Replace real-fixture eval-ready/runtime-block expectations with
    runtime-enabled expectations.
  - Keep patched eval-ready gate tests for future candidate subjects.
- Modify: `tests/test_subject_contracts.py`
  - Expect accounting to be runtime-enabled in default repository contracts.
  - Keep accounting signal, method lens, evaluation pack, and activation gate
    assertions.
- Modify: `tests/test_subject_refinement.py`
  - Expect clear accounting evidence to suggest accounting after promotion.
  - Expect method-only accounting evidence to remain borrowed-lens only.
  - Expect confirmed accounting projects to load accounting resources after
    promotion.
- Modify: `content/subjects/accounting/runtime-subject.yaml`
  - Change `activation_status` from `eval_ready` to `runtime_enabled`.
- Modify: `docs/reference/cli.md`
  - Update accounting gate examples and wording.
- Modify: `docs/advanced/publish-pypi.md`
  - Move accounting from optional eval-ready check to runtime-enabled release
    verification.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark full-cycle harness and accounting eval-ready as completed.
  - Set accounting runtime promotion as the current Stage 4 next step.

## Execution Notes

- Execute on a fresh branch from updated `dev`, for example
  `feature/accounting-runtime-enabled`.
- Use subagent-driven development if possible:
  - Task 1: fixture and failing expectations.
  - Task 2: manifest/runtime behavior.
  - Task 3: docs and roadmap.
  - Task 4: verification and PR prep.
- Commit after each task with narrow Conventional Commit messages.
- Do not activate any subject other than accounting.
- Do not change provider, literature search, Zotero, or full-text behavior in
  this branch.

## Task 1: Add Runtime Promotion Guard Tests

**Files:**
- Create: `tests/fixtures/subject_router_eval/accounting/method_only_auto_accrual_controls.json`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/test_subject_contracts.py`
- Modify: `tests/test_subject_refinement.py`

- [ ] **Step 1: Add the auto-mode method-only guard fixture**

Create `tests/fixtures/subject_router_eval/accounting/method_only_auto_accrual_controls.json`:

```json
{
  "id": "accounting_method_only_auto_accrual_controls",
  "subject_under_test": "accounting",
  "tags": ["accounting", "method_only_borrow"],
  "description": "Auto-mode accrual-quality controls borrow an accounting lens without activating accounting.",
  "request": "Add accrual quality and discretionary accrual controls to the empirical appendix, but keep the project framing on general robustness checks.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "borrow_lens",
    "primary_subject": "auto",
    "suggest_subjects": [],
    "forbidden_subjects": ["accounting"],
    "method_lenses": ["accrual-quality"]
  }
}
```

- [ ] **Step 2: Register the fixture in the inventory assertion**

In `tests/test_subject_router_eval.py`, add the new id to
`required_accounting_ids` in the accounting fixture inventory test:

```python
            "accounting_method_only_auto_accrual_controls",
```

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected before activation: the file loads and router fixture checks still pass
or the output identifies a method-only over-activation that must be fixed
before Task 2.

- [ ] **Step 3: Change repository contract expectations to runtime-enabled**

In `tests/test_subject_contracts.py`, update the default repository contract
test so accounting is expected to be runtime-enabled:

```python
        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "runtime_enabled",
        )
```

Rename the accounting manifest test from eval-ready wording to
runtime-enabled wording:

```python
    def test_accounting_runtime_enabled_manifest_declares_signals_and_method_lenses(self) -> None:
```

Inside that test, update:

```python
        self.assertEqual(contract.activation_status, "runtime_enabled")
```

Run:

```bash
uv run python -m pytest tests/test_subject_contracts.py -q
```

Expected before Task 2: FAIL because the manifest still says `eval_ready`.

- [ ] **Step 4: Update real accounting gate expectations**

In `tests/test_subject_router_eval.py`, replace the real-fixture runtime gate
blocking test with a runtime-enabled success test:

```python
    def test_accounting_runtime_enabled_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases()

        report = subject_gate_report("accounting", cases, gate="runtime-enabled")

        self.assertEqual(report["subject"], "accounting")
        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertTrue(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)
```

Keep a patched unit test that proves eval-ready contracts still fail the
runtime gate:

```python
    def test_runtime_enabled_gate_still_blocks_eval_ready_subject_contract(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    activation_status="eval_ready"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="runtime-enabled")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])
```

If `test_accounting_eval_ready_gate_passes_real_fixture_pack` exists, replace
it with the runtime-enabled real-fixture test above. Keep patched eval-ready
tests such as `test_eval_ready_gate_accepts_eval_ready_subject_without_runtime_activation`.

Run:

```bash
uv run python -m pytest tests/test_subject_router_eval.py -q
```

Expected before Task 2: FAIL where real fixtures still report
`activation_status: eval_ready`.

- [ ] **Step 5: Update subject refinement expectations for promoted accounting**

In `tests/test_subject_refinement.py`, update clear accounting evidence so it
expects a runtime suggestion after promotion:

```python
    def test_runtime_enabled_accounting_signals_suggest_accounting(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": (
                    "Use discretionary accruals, Audit Analytics restatements, "
                    "internal-control weaknesses, financial reporting quality, "
                    "and Journal of Accounting Research positioning."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("accrual-quality", packet["method_lenses"])
        self.assertIn("construct-proxy-audit", packet["method_lenses"])
```

Add or keep a method-only guard:

```python
    def test_runtime_enabled_accounting_method_only_auto_borrows_lens(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "robustness controls",
                "context": (
                    "Add accrual quality and discretionary accrual controls "
                    "to the empirical appendix."
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
        self.assertIn("accrual-quality", packet["method_lenses"])
```

Run:

```bash
uv run python -m pytest tests/test_subject_refinement.py -q
```

Expected before Task 2: clear-evidence suggestion expectations fail because
accounting is still eval-ready.

- [ ] **Step 6: Commit Task 1**

```bash
git add \
  tests/fixtures/subject_router_eval/accounting/method_only_auto_accrual_controls.json \
  tests/test_subject_router_eval.py \
  tests/test_subject_contracts.py \
  tests/test_subject_refinement.py
git commit -m "test(subjects): add accounting runtime promotion guards"
```

## Task 2: Promote Accounting Manifest And Runtime Behavior

**Files:**
- Modify: `content/subjects/accounting/runtime-subject.yaml`
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py` only if Task 1 exposes method-only or context-only over-activation

- [ ] **Step 1: Promote accounting activation status**

In `content/subjects/accounting/runtime-subject.yaml`, change:

```yaml
activation_status: eval_ready
```

to:

```yaml
activation_status: runtime_enabled
```

- [ ] **Step 2: Run focused tests**

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected after the manifest change: PASS. If only the new method-only guard
fails, continue to Step 3. If unrelated finance/economics tests fail, stop and
inspect the router diff before changing signal weights.

- [ ] **Step 3: Narrow method-only or context-only activation only if needed**

The current `RuntimeSubjectMatch.has_subject_strength` requires at least two
matched dimensions, so the new method-only fixture should pass without a router
change. If it fails, or if a local check shows that method-only plus
context-only venue evidence can suggest accounting, update
`packages/python-qiongli/src/qiongli/bridges/subject_refinement.py` with this
explicit activation-aware strength rule.

Extend `RuntimeSubjectMatch`:

```python
@dataclass(frozen=True)
class RuntimeSubjectMatch:
    subject: str
    dimensions: tuple[str, ...]
    subject_level_dimensions: tuple[str, ...]
    method_lenses: tuple[str, ...]
    evidence: tuple[str, ...]
    signal_ids: tuple[str, ...]

    @property
    def has_subject_strength(self) -> bool:
        return bool(self.subject_level_dimensions) and len(self.dimensions) >= 2
```

In `_manifest_records_for_contract`, preserve each manifest signal activation:

```python
                records.append(
                    {
                        "id": entry_id,
                        "subject": contract.subject,
                        "dimension": str(dimension),
                        "value": value,
                        "activation": str(entry.get("activation", "subject") or "subject"),
                        "weight": _coerce_signal_weight(entry.get("weight", 0.0)),
                        "source": "task_text",
                        "snippet": _snippet_for_match(text, match),
                    }
                )
```

In `_detect_manifest_signal_records`, compute subject-level dimensions before
constructing `RuntimeSubjectMatch`:

```python
        subject_level_dimensions = _unique(
            [
                str(record["dimension"])
                for record in subject_records
                if str(record.get("activation", "subject"))
                not in {"method_only", "context_only"}
            ]
        )
        matches[subject] = RuntimeSubjectMatch(
            subject=subject,
            dimensions=tuple(dimensions),
            subject_level_dimensions=tuple(subject_level_dimensions),
            method_lenses=tuple(method_lenses),
            evidence=tuple(_unique([str(record["snippet"]) for record in subject_records])),
            signal_ids=tuple(_unique([str(record["id"]) for record in subject_records])),
        )
```

- [ ] **Step 4: Re-run focused tests**

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected: PASS.

- [ ] **Step 5: Run subject gate checks**

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

Expected: each command exits 0. In the accounting report,
`eligible_for_runtime_enabled` must be true and `blocking_failures` must be
empty.

- [ ] **Step 6: Commit Task 2**

```bash
git add \
  content/subjects/accounting/runtime-subject.yaml \
  packages/python-qiongli/src/qiongli/bridges/subject_refinement.py
git commit -m "feat(subjects): enable accounting runtime routing"
```

If `subject_refinement.py` was not modified, omit it from `git add`.

## Task 3: Update Documentation And Roadmap

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update CLI reference**

In `docs/reference/cli.md`, update the accounting example so the runtime gate
is the primary check:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

Keep `--gate eval-ready` documented as the gate for future candidate subjects.

- [ ] **Step 2: Update release guidance**

In `docs/advanced/publish-pypi.md`, move accounting from optional eval-ready
checks into the runtime-enabled subject verification block:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

State that business, political economy, geoeconomics, and
economics-accounting remain future eval-ready candidates.

- [ ] **Step 3: Update roadmap status**

In `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`:

- Mark the full-cycle harness and manuscript-first journal fit as completed on
  `dev`.
- Mark accounting eval-ready as completed.
- Mark accounting runtime promotion as the current Stage 4 slice.
- Keep other subjects deferred.

Use this current-priority wording:

```markdown
## Priority Update: Accounting Runtime Promotion

Status: current Stage 4 priority after the full-cycle workflow harness and
accounting eval-ready pack merged.
```

- [ ] **Step 4: Run docs consistency searches**

```bash
rg -n "still lacks an end-to-end|Status: next priority|accounting eval-ready slice in progress" \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md
```

Expected: no stale roadmap or release text remains. It is acceptable for
historical specs/plans to retain old wording.

- [ ] **Step 5: Commit Task 3**

```bash
git add \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(subjects): document accounting runtime activation"
```

## Task 4: Final Verification And PR Preparation

**Files:**
- No new files expected.

- [ ] **Step 1: Run the default subject router evaluation**

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit 0.

- [ ] **Step 2: Run full-cycle harness regression**

```bash
uv run python tooling/scripts/run_full_cycle_workflow_harness.py \
  --fixture tests/fixtures/full_cycle_harness/clean_empirical \
  --json-report /tmp/qiongli-full-cycle-harness.json
```

Expected: exit 0.

- [ ] **Step 3: Run full-cycle and MCP focused tests**

```bash
uv run python -m pytest \
  tests/test_full_cycle_harness_script.py \
  tests/test_lifecycle_harness.py \
  tests/test_journal_fit.py \
  tests/test_mcp_tool_handlers.py \
  -q
```

Expected: PASS.

- [ ] **Step 4: Run focused subject tests again**

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Expected: PASS.

- [ ] **Step 5: Check formatting-sensitive diffs**

```bash
git diff --check
git status --short
```

Expected: `git diff --check` has no output. `git status --short` should show
only intentional branch changes before final commit or PR.

- [ ] **Step 6: Prepare PR summary**

Use this PR focus:

```markdown
## Summary

- Promotes accounting from eval-ready to runtime-enabled routing.
- Adds an auto-mode method-only accounting guard to prevent over-activation.
- Updates subject gate tests, contract expectations, and release documentation.
- Verifies accounting, finance, economics, and full-cycle harness regressions.

## Testing

- uv run python -m pytest tests/test_subject_contracts.py tests/test_subject_router_eval.py tests/test_subject_refinement.py -q
- uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
- uv run python tooling/scripts/evaluate_subject_router.py --subject finance --gate runtime-enabled --json
- uv run python tooling/scripts/evaluate_subject_router.py --subject economics --gate runtime-enabled --json
- uv run python tooling/scripts/evaluate_subject_router.py --json
- uv run python tooling/scripts/run_full_cycle_workflow_harness.py --fixture tests/fixtures/full_cycle_harness/clean_empirical --json-report /tmp/qiongli-full-cycle-harness.json
- uv run python -m pytest tests/test_full_cycle_harness_script.py tests/test_lifecycle_harness.py tests/test_journal_fit.py tests/test_mcp_tool_handlers.py -q
- git diff --check
```

- [ ] **Step 7: Open PR to dev**

```bash
git push -u origin feature/accounting-runtime-enabled
gh pr create --base dev --head feature/accounting-runtime-enabled
```

Expected: PR is ready for review, not draft, if all verification commands pass.

## Self-Review Checklist

- [ ] The new method-only guard prevents accounting suggestions from one
  method phrase in auto mode.
- [ ] Accounting runtime-enabled gate passes with no blocking failures.
- [ ] Finance and economics runtime gates still pass.
- [ ] Full-cycle harness remains green.
- [ ] No other subject is promoted.
- [ ] Roadmap no longer points to an already completed full-cycle harness as
  the next unimplemented priority.
