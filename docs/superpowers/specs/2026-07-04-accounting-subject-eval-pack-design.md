# Accounting Subject Eval Pack Design

## Goal

Make accounting the first new subject expansion slice after the full-cycle
workflow harness. This slice should make accounting gate-verifiable and ready
for explicit review, but it should not silently activate accounting in the
adaptive runtime.

The target milestone is `activation_status: eval_ready` for accounting, with a
passing subject evaluation pack and conservative router support. A later,
separate activation change may promote accounting to `runtime_enabled` only
after the gate output is reviewed.

## Current Baseline

The Stage 4 subject-expansion foundation already exists:

- Runtime manifests are loaded from `content/subjects/*/runtime-subject.yaml`.
- `candidate`, `eval_ready`, `runtime_enabled`, and `disabled` activation
  statuses are recognized by the subject contract layer.
- The evaluation script can run router fixtures and enforce the
  `runtime-enabled` gate.
- Economics and finance are runtime-enabled and covered by legacy fixtures.
- Accounting, business, political economy, geoeconomics, and the
  economics-accounting bridge are still candidates.

Accounting has useful content assets, but no real evaluation pack yet:

- `content/skills/domain-profiles/accounting.yaml`
- `content/subjects/accounting/skills/accounting-measurement-auditor.md`
- `content/subjects/accounting/overlays/skills/*.md`
- `content/subjects/accounting/venue-profiles/*.yaml`
- `content/subjects/accounting/runtime-subject.yaml`

The current accounting runtime manifest points to
`tests/fixtures/subject_router_eval/accounting`, but that directory does not
contain the required accounting fixture set. The manifest also has empty signal
groups and method lenses, so the router cannot explain or evaluate accounting
evidence independently.

## Design Decision

Use an accounting-first subject expansion slice.

This is the lowest-risk Stage 4 continuation because accounting is adjacent to
the current economics and finance router, has existing content resources, and
has clear near-miss boundaries. Business, political economy, and geoeconomics
are broader and should wait until the gate pattern is proven on one adjacent
subject.

Accounting will move through two explicit states:

1. `candidate`: content exists, but the adaptive runtime cannot suggest it.
2. `eval_ready`: fixtures, signals, method lenses, and resource references are
   complete enough for deterministic evaluation, but runtime suggestion is
   still blocked.

`runtime_enabled` is not part of this slice.

## Scope

This slice will add a real accounting evaluation pack, accounting signal
metadata, and the minimum router support needed to measure accounting routing
without changing default runtime activation.

In scope:

- Create `tests/fixtures/subject_router_eval/accounting/`.
- Add accounting fixture categories for clear positives, method-only borrowing,
  mixed adjacent-subject cases, near misses, locked subjects, confirmed
  subjects, and dismissed subjects where the existing fixture harness supports
  those states.
- Populate accounting signal groups in
  `content/subjects/accounting/runtime-subject.yaml`.
- Add accounting method lenses for archival accounting measurement and
  construct-proxy audit.
- Add or adapt an `eval-ready` gate so maintainers can validate accounting
  readiness separately from final runtime activation.
- Add conservative accounting router diagnostics while preserving the existing
  rule that only `runtime_enabled` subjects can be suggested.
- Update CLI/reference documentation for the accounting eval-ready gate.
- Update tests for fixture loading, gate behavior, router diagnostics, and
  economics/finance non-regression.

Out of scope:

- Do not promote accounting to `runtime_enabled`.
- Do not activate business, political economy, geoeconomics, or
  economics-accounting.
- Do not change the default install subject selection flow.
- Do not require network access, provider credentials, or LLM calls for the
  gate.
- Do not rewrite economics and finance routing beyond compatibility changes
  required by shared fixture or manifest loaders.

## Accounting Signal Model

Accounting should be suggested only when the evidence contains at least two
independent accounting dimensions. A single method phrase should be treated as a
borrowed lens or diagnostic, not as a subject switch.

Required dimensions:

- `method`: accrual quality, discretionary accruals, earnings management,
  restatements, internal controls, audit fees, going concern, abnormal audit
  fees, tax avoidance, book-tax differences.
- `data_or_outcome`: Compustat accounting items, Audit Analytics,
  restatement data, internal-control weakness disclosures, management
  forecasts, analyst following, earnings quality, financial reporting quality.
- `venue`: `The Accounting Review`, `Journal of Accounting Research`,
  `Review of Accounting Studies`, `Contemporary Accounting Research`, and
  `Journal of Accounting and Economics` when paired with accounting constructs.
- `theory_or_construct`: financial reporting, auditing, disclosure, tax,
  governance through reporting mechanisms, ESG reporting, construct-proxy
  validity, fiscal timing, archival sample filters.

Conservative rule:

- Strong accounting suggestion requires two dimensions, for example
  `method + data_or_outcome`, `venue + construct`, or `method + construct`.
- Method-only evidence can produce an accounting lens but must not switch the
  primary subject.
- Venue-only evidence must not switch the primary subject.
- Generic uses of "accounting" as bookkeeping, categorization, or "accounting
  for heterogeneity" must remain core or the active non-accounting subject.

## Fixture Pack

The accounting fixture pack should be split into small files under:

```text
tests/fixtures/subject_router_eval/accounting/
```

Required cases:

- `clear_positive`: accounting should be selected when the request combines an
  accounting construct, archival proxy, and reporting or audit setting.
- `method_only_borrow`: accounting methods should be borrowed without changing
  the primary subject when the user's active topic is economics or finance.
- `mixed_subject`: accounting can coexist with finance or economics, but the
  expected primary subject must be explicit in the fixture.
- `near_miss`: phrases such as budgeting, administrative accounting,
  management accounting as internal operations, and "accounting for
  heterogeneity" must not activate accounting.
- `locked_subject`: a locked economics or finance subject should not be
  replaced by accounting evidence.
- `confirmed_subject`: confirmed accounting state may load accounting as the
  applied subject, but only through explicit lifecycle state.
- `dismissed_subject`: prior dismissal should suppress repeated accounting
  promotion until materially stronger new evidence exists.
- `legacy_regression`: economics and finance cases continue to meet their
  existing thresholds.

Example clear-positive fixture intent:

```text
Design an archival accounting study on discretionary accruals, internal-control
weaknesses, Audit Analytics restatement data, and reporting-quality mechanisms.
```

Expected result:

- Primary subject can be accounting only in gate evaluation mode.
- Runtime suggestion remains blocked unless accounting is
  `activation_status: runtime_enabled`.
- The report explains the triggering dimensions.

Example near-miss fixture intent:

```text
Explain how to account for heterogeneity in a difference-in-differences model.
```

Expected result:

- No accounting subject suggestion.
- Economics or core behavior remains unchanged.

## Gate Behavior

Add a separate readiness gate for subjects that are not yet runtime-enabled:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate eval-ready \
  --json
```

`eval-ready` gate checks:

- The accounting manifest has `activation_status: eval_ready`.
- Required fixture categories are present.
- Required accounting signal dimensions are non-empty.
- Fixture metrics meet accounting thresholds.
- Accounting near-miss false positives are zero.
- Economics and finance legacy fixtures still pass.
- Required content resource paths resolve inside the packaged payload.

`runtime-enabled` gate keeps its stricter behavior:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

For this slice, `runtime-enabled` should still fail because accounting is not
promoted to `runtime_enabled`.

## Router Behavior

The implementation should prefer data-backed accounting signal definitions from
the runtime manifest instead of adding another large hard-coded block to
`subject_refinement.py`.

Acceptable implementation shape:

- Add a small manifest-backed signal loader for non-core subjects.
- Keep economics and finance hard-coded compatibility behavior unchanged.
- Emit accounting candidates and diagnostics in evaluation mode.
- Continue suppressing adaptive subject suggestions unless
  `_subject_can_be_suggested("accounting")` returns true.
- Keep accounting resources withheld unless
  `subject_activation_status("accounting") == "runtime_enabled"`.

The report should expose enough information for review:

- Matched accounting dimensions.
- Matched signal ids.
- Whether the candidate was blocked by activation status.
- Whether only a method lens was borrowed.
- Which near-miss guard prevented activation, when applicable.

## Documentation

Update documentation only after the tests and gate behavior exist.

Required docs:

- `docs/reference/cli.md`: document `--gate eval-ready` and show the accounting
  command.
- `docs/advanced/publish-pypi.md`: include accounting eval-ready report in the
  release-readiness checklist once implemented.
- Stage 4 roadmap status: mark accounting as the first active expansion slice
  and keep the remaining candidate subjects deferred.

Documentation must make clear that `eval_ready` does not mean runtime-enabled.

## Testing

Minimum test coverage:

- Contract tests for accounting manifest fields and activation status.
- Fixture loader tests for nested accounting fixture packs.
- Gate tests for `eval-ready` success and `runtime-enabled` failure while
  accounting remains eval-ready.
- Router tests showing accounting clear-positive evidence is measured but not
  suggested at runtime before activation.
- Router tests showing method-only accounting evidence borrows a lens without
  switching subject.
- Near-miss tests showing accounting false positives remain zero.
- Regression tests showing economics and finance fixture metrics do not drop.

Recommended verification commands:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
  tests/test_subject_refinement.py

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate eval-ready \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

Expected final state for this slice:

- `eval-ready` exits successfully.
- `runtime-enabled` exits with an intentional activation-status blocker.
- Economics and finance regression fixtures still pass.

## Risks And Mitigations

- Over-activation risk: require two independent accounting dimensions and zero
  near-miss false positives.
- Adjacent-subject drift: include mixed accounting-finance and
  accounting-economics cases with explicit primary-subject expectations.
- Manifest/resource mismatch: gate all required paths before evaluating router
  behavior.
- Hidden runtime activation: keep `_subject_can_be_suggested` and resource
  loading tied to `runtime_enabled`.
- Evaluation overfitting: include both positive and near-miss accounting cases,
  and keep legacy economics/finance fixtures in the gate report.

## Rollback

Rollback is straightforward because this slice is not supposed to activate
accounting at runtime:

- Set `content/subjects/accounting/runtime-subject.yaml` back to
  `activation_status: candidate`.
- Remove or ignore `tests/fixtures/subject_router_eval/accounting/`.
- Revert manifest-backed accounting signal loading if it affects router
  behavior.
- Keep accounting content assets unchanged unless a resource-path issue is
  discovered.

## Acceptance Criteria

The implementation for this spec is complete when:

- Accounting has a dedicated fixture pack with clear-positive, method-only,
  mixed, and near-miss coverage.
- Accounting manifest signal groups and method lenses are populated.
- `--gate eval-ready` exists and passes for accounting.
- `--gate runtime-enabled` still blocks accounting because it is not activated.
- Existing economics and finance subject evaluation still passes.
- Documentation explains the difference between `eval_ready` and
  `runtime_enabled`.
- No other candidate subject is promoted or activated.
