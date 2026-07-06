# Accounting Runtime Promotion Design

## Goal

Promote accounting from `activation_status: eval_ready` to
`activation_status: runtime_enabled` only after the merged accounting fixture
pack, subject gate, and full-cycle workflow harness evidence support the
activation.

This is a narrow activation slice. It should make accounting available to the
adaptive runtime for clear accounting research requests while preserving the
core router rule that method-only evidence borrows accounting lenses instead
of switching the primary subject.

## Current Baseline

On `dev` as of July 5, 2026:

- The full-cycle workflow harness and manuscript-first journal-fit recommender
  from the July 4 plan are present.
- The accounting eval-ready fixture pack and manifest-backed signals are
  merged.
- `qiongli_lifecycle_plan` and `qiongli_journal_fit_recommend` are exposed by
  the Python MCP handler layer.
- `tooling/scripts/run_full_cycle_workflow_harness.py` runs the deterministic
  clean empirical fixture successfully.
- `accounting` has `activation_status: eval_ready`.

Current gate evidence:

```text
accounting --gate eval-ready:
  activation_status: eval_ready
  eligible_for_eval_ready: true
  blocking_failures: []
  metrics:
    decision_accuracy: 1.0
    primary_subject_accuracy: 1.0
    suggest_subject_precision: 1.0
    near_miss_false_positives: 0
    forbidden_subject_accuracy: 1.0
    method_lens_accuracy: 1.0
    all_case_checks_passed: 1.0

accounting --gate runtime-enabled:
  activation_status: eval_ready
  eligible_for_runtime_enabled: false
  blocking_failures:
    - activation_status is eval_ready
  metrics:
    decision_accuracy: 1.0
    primary_subject_accuracy: 1.0
    suggest_subject_precision: 1.0
    near_miss_false_positives: 0
    forbidden_subject_accuracy: 1.0
    method_lens_accuracy: 1.0
    all_case_checks_passed: 1.0
```

The runtime gate is blocked only because the manifest has not been promoted.
The fixture metrics already satisfy the accounting activation thresholds.

## Decision

Run a guarded accounting activation slice now, before adding business,
political economy, geoeconomics, or the economics-accounting bridge.

Why this order:

- The full-cycle harness is already merged, so repeating that work would waste
  the next roadmap slot.
- Accounting is already eval-ready and has zero measured false positives in
  the current fixture pack.
- The activation diff can be small and reversible.
- Activating one adjacent subject tests the subject-gate contract before
  broader, less bounded subjects are added.

## Scope

In scope:

- Add one extra accounting method-only auto-mode fixture to guard against
  runtime over-activation after promotion.
- Promote `content/subjects/accounting/runtime-subject.yaml` to
  `activation_status: runtime_enabled`.
- Update tests that currently assert accounting is eval-ready.
- Update CLI, release, and roadmap docs so accounting runtime activation is
  described accurately.
- Re-run subject gate checks for accounting, finance, and economics.
- Re-run deterministic full-cycle harness checks.

Out of scope:

- Do not activate business, political economy, geoeconomics, or
  economics-accounting.
- Do not change provider configuration, Zotero lookup behavior, literature
  search defaults, or full-text retrieval.
- Do not add network-dependent tests.
- Do not launch local agents in default verification.
- Do not broaden accounting signals unless a fixture proves the need.

## Runtime Safety Rules

Accounting can become the primary subject only when evidence includes at least
two independent accounting dimensions, such as:

- method plus data/outcome,
- method plus theory/construct,
- venue plus accounting construct,
- data/outcome plus accounting construct.

Accounting must not become the primary subject when the evidence is limited to:

- accrual quality or discretionary accruals as a single method-only phrase,
- generic bookkeeping or budget reporting,
- generic "accounting for heterogeneity" language,
- operational financial reporting dashboards,
- management forecast staffing or project operations language.

Method-only accounting evidence may still borrow:

- `accrual-quality`,
- `construct-proxy-audit`.

Borrowed accounting lenses should use `resource_level:
method_pack_only`/method-level activation and must not load full accounting
subject resources unless accounting is the active or suggested subject.

## Required Fixture Addition

Add a new auto-mode method-only guard:

```text
tests/fixtures/subject_router_eval/accounting/method_only_auto_accrual_controls.json
```

Expected behavior after accounting is runtime-enabled:

- Decision: `borrow_lens`.
- Primary subject: `auto`.
- Suggested subjects: none.
- Forbidden subjects: `accounting`.
- Method lenses: `accrual-quality`.

This fixture is intentionally different from the existing locked-finance
method-only case. It proves that runtime-enabled accounting still does not
activate from a single accounting method signal in a neutral auto project.

## Expected Code Changes

### Accounting manifest

Change:

```yaml
activation_status: eval_ready
```

to:

```yaml
activation_status: runtime_enabled
```

No signal weights should change unless the new method-only guard fails.

### Subject contract tests

Update repository contract expectations:

- `subject_activation_status("accounting", contracts)` becomes
  `runtime_enabled`.
- The accounting manifest test should be renamed from eval-ready wording to
  runtime-enabled wording.
- Accounting must still declare signal groups, method lenses, evaluation pack,
  near-miss policy, and gate metrics.

### Subject router gate tests

Update real accounting gate tests:

- `--subject accounting --gate runtime-enabled` must pass.
- The old real-fixture assertion that accounting runtime gate blocks on
  `activation_status is eval_ready` should be replaced with a patched unit test
  that still proves an eval-ready contract cannot pass the runtime gate.
- Existing patched eval-ready gate tests should remain, because future subject
  candidates still use that path.

### Subject refinement tests

Update runtime behavior expectations:

- Clear accounting evidence in auto mode can suggest accounting.
- Method-only accounting evidence in auto mode borrows the accounting lens
  without suggesting accounting.
- Confirmed accounting projects can load accounting resources now that
  accounting is runtime-enabled.
- Locked finance/economics projects still keep the lock and may only borrow
  accounting method lenses.

## Verification

Required focused checks:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py \
  -q
```

Required gate checks:

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

Required regression checks:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json

uv run python tooling/scripts/run_full_cycle_workflow_harness.py \
  --fixture tests/fixtures/full_cycle_harness/clean_empirical \
  --json-report /tmp/qiongli-full-cycle-harness.json

uv run python -m pytest \
  tests/test_full_cycle_harness_script.py \
  tests/test_lifecycle_harness.py \
  tests/test_journal_fit.py \
  tests/test_mcp_tool_handlers.py \
  -q

git diff --check
```

## Acceptance Criteria

- Accounting runtime-enabled gate exits 0.
- Accounting fixture metrics still meet all manifest thresholds.
- Accounting near-miss false positives remain 0.
- The new auto-mode method-only fixture does not suggest accounting.
- Finance and economics runtime-enabled gates still pass.
- Full-cycle workflow harness still passes the clean empirical fixture.
- Docs no longer describe full-cycle harness as the next unimplemented
  priority.
- Business, political economy, geoeconomics, and economics-accounting remain
  unactivated.

## Risks

- Over-activation: accounting may become suggested from weak method-only
  language. Mitigation: add the auto-mode method-only fixture before promotion.
- Adjacent-subject drift: accounting could override finance/economics when it
  should only contribute a measurement lens. Mitigation: keep locked and mixed
  subject fixtures in the runtime gate.
- Resource loading mismatch: confirmed accounting may begin loading subject
  resources that were previously withheld. Mitigation: update tests to assert
  the exact resource activation plan.
- Roadmap drift: full-cycle status may remain stale in docs. Mitigation:
  update the roadmap in the same PR as the activation plan or implementation.

## Rollback

If the activation causes routing drift:

1. Set `content/subjects/accounting/runtime-subject.yaml` back to
   `activation_status: eval_ready`.
2. Keep the new auto-mode method-only fixture as a regression guard.
3. Re-run the accounting eval-ready gate to confirm the fixture pack remains
   healthy.
4. Defer activation until the failing subject refinement rule is narrowed.
