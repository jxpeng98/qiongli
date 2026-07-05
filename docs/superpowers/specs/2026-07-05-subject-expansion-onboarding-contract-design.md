# Subject Expansion Onboarding Contract Design

## Goal

Add an explicit onboarding contract for deferred subject expansion so business,
political economy, geoeconomics, and economics-accounting cannot look
eval-ready by accidentally reusing accounting fixtures or incomplete manifest
metadata.

This is a gate-hardening slice. It does not add a new subject, does not promote
any candidate subject, and does not add new business or political economy
signals. It makes the next subject expansion review measurable before any
subject-specific fixture pack is written.

## Current Baseline

On `dev` as of July 5, 2026:

- Accounting is `runtime_enabled` and passes the runtime-enabled gate.
- Economics and finance remain runtime-enabled and continue to pass their
  existing gate checks.
- Business, political economy, geoeconomics, and economics-accounting are
  present as `candidate` contracts.
- The four deferred candidate contracts currently have empty `signal_groups`
  and `method_lenses`.
- Those same candidate contracts still point `evaluation_pack` at
  `tests/fixtures/subject_router_eval/accounting`, even though they do not have
  subject-owned fixture packs.

Current candidate gate behavior is safe but under-explained:

```text
business --gate eval-ready:
  activation_status: candidate
  eligible_for_eval_ready: false
  blocking_failures:
    - activation_status is candidate
    - missing clear_positive fixtures
    - missing method_only_borrow fixtures
    - missing near_miss fixtures
```

This fails closed, but it does not say that the business manifest is still a
deferred shell or that its `evaluation_pack` points at another subject. The
next subject expansion worker could misread the manifest as partially wired to
an accounting-quality fixture pack.

## Decision

Create a subject expansion onboarding contract before adding the next subject
fixture pack.

The next subject expansion can still be business, because business has existing
subject resources and venue profiles. Business should not be implemented until
the onboarding contract makes these preconditions explicit:

- Candidate subjects may be represented as deferred shells.
- Deferred shells must not point at another subject-specific fixture pack.
- Eval-ready subjects must have subject-owned fixture coverage before gate
  eligibility is possible.
- Runtime-enabled subjects must keep their existing stricter resource checks.

## Scope

In scope:

- Add tests that describe deferred subject contract hygiene for business,
  political economy, geoeconomics, and economics-accounting.
- Update deferred candidate manifests so their `evaluation_pack` field is
  blank until a reviewed subject-owned pack exists.
- Add gate diagnostics that explicitly report missing subject-owned evaluation
  packs and missing manifest signal dimensions for candidate/eval-ready
  subjects.
- Block eval-ready eligibility when a subject-specific `evaluation_pack` path
  points at a different subject.
- Keep accounting, economics, and finance runtime-enabled gate behavior
  unchanged.
- Update the roadmap to mark this onboarding contract as the current Stage 4
  prerequisite before the first post-accounting subject pack.

Out of scope:

- Do not add business, political economy, geoeconomics, or economics-accounting
  signals.
- Do not add new fixture packs for those subjects.
- Do not change accounting activation, signal weights, or fixture expectations.
- Do not change provider configuration, literature search defaults, full-text
  retrieval, Zotero behavior, or release automation.
- Do not launch local agents in default verification.

## Contract Rules

### Deferred candidate manifests

The following candidate subjects are deferred shells until a separate reviewed
spec adds their first fixture pack:

- `business`
- `political-economy`
- `geoeconomics`
- `economics-accounting`

For these subjects:

- `activation_status` remains `candidate`.
- `signal_groups` remains empty.
- `method_lenses` remains empty.
- `evaluation_pack` must be blank.
- `subject_skill` may remain blank if no subject-owned skill should load at
  eval-ready.
- Venue profiles and subject skills may remain on disk as resources, but they
  must not imply gate eligibility without fixtures.

### Subject-owned fixture packs

An eval-ready candidate must have one of these fixture layouts:

- A subject-specific pack at
  `tests/fixtures/subject_router_eval/<subject>/`, where `<subject>` exactly
  matches the contract subject.
- A shared aggregate pack at `tests/fixtures/subject_router_eval/`, if and only
  if the gate selects cases by `subject_under_test` or tags for that subject.

A contract must fail the eval-ready gate if `evaluation_pack` points at a
different subject-specific pack. For example, business must not point at
`tests/fixtures/subject_router_eval/accounting`.

### Required fixture tags

Every subject seeking eval-ready must have subject-scoped fixture coverage for:

- `clear_positive`
- `method_only_borrow`
- `near_miss`

The existing `mixed_subject`, `locked_subject`, and `confirmed_subject` tags
remain recommended but not required for the first eval-ready gate.

### Required signal dimensions

The onboarding diagnostics should report missing manifest dimensions for any
subject that is not already runtime-enabled and has no reviewed signal groups.
For the first version of this contract, a subject is considered incomplete when
all of these dimensions are empty:

- `method`
- `data_or_outcome`
- `venue`
- `theory_or_construct`

Accounting keeps its stricter existing dimension check. Finance and economics
must not gain new failures from this candidate-only diagnostic.

## Expected Code Changes

### Candidate manifests

Update these files so `evaluation_pack` is blank:

```text
content/subjects/business/runtime-subject.yaml
content/subjects/political-economy/runtime-subject.yaml
content/subjects/geoeconomics/runtime-subject.yaml
content/subjects/economics-accounting/runtime-subject.yaml
```

No activation status or signal groups change in this slice.

### Subject contract tests

Extend `tests/test_subject_contracts.py` with default repository contract
coverage:

- The four deferred candidate subjects load successfully.
- Each deferred candidate has `activation_status: candidate`.
- Each deferred candidate has blank `evaluation_pack`.
- Each deferred candidate has no signal groups and no method lenses.
- Accounting remains `runtime_enabled`.
- Finance and economics remain `runtime_enabled`.

The test should protect against future accidental fixture-pack reuse by
explicitly asserting that none of the deferred candidates points at
`tests/fixtures/subject_router_eval/accounting`.

### Subject gate report

Extend `tooling/scripts/evaluate_subject_router.py` with small helper logic
rather than hard-coded per-subject branches:

- Detect whether `evaluation_pack` is blank.
- Detect whether `evaluation_pack` points at a subject-specific directory whose
  name differs from the contract subject.
- Report an explicit blocking failure for blank evaluation packs when a subject
  is evaluated for `eval-ready`.
- Report an explicit blocking failure for subject-pack mismatches.
- Report missing candidate signal dimensions for non-runtime subjects whose
  signal groups are empty.

Expected failure strings:

```text
missing evaluation_pack for deferred subject
evaluation_pack subject mismatch: expected business, found accounting
missing signal dimension: method
missing signal dimension: data_or_outcome
missing signal dimension: venue
missing signal dimension: theory_or_construct
```

The existing strings remain unchanged where they already apply:

```text
activation_status is candidate
missing clear_positive fixtures
missing method_only_borrow fixtures
missing near_miss fixtures
```

### Subject gate tests

Extend `tests/test_subject_router_eval.py` with tests that prove:

- `subject_gate_report("business", cases, gate="eval-ready")` fails closed and
  reports the deferred-shell reasons.
- The same behavior applies to political economy, geoeconomics, and
  economics-accounting.
- A patched business contract that points at the accounting pack reports
  `evaluation_pack subject mismatch: expected business, found accounting`.
- Accounting runtime-enabled gate still passes with no blocking failures.
- Economics and finance runtime-enabled gates still pass with no blocking
  failures.

### Roadmap

Update
`docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md` so
the current Stage 4 step is:

1. Merge the subject expansion onboarding contract.
2. Then prepare the first post-accounting subject spec, with business as the
   recommended next candidate unless review chooses otherwise.

## Verification

Required focused tests:

```bash
uv run python -m pytest \
  tests/test_subject_contracts.py \
  tests/test_subject_router_eval.py \
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

Expected results:

- Accounting, finance, and economics runtime-enabled gate commands exit `0`.
- Candidate subject eval-ready commands exit non-zero.
- Candidate subject reports include the explicit deferred-shell failures.
- No runtime-enabled subject gains new blocking failures.

Final hygiene:

```bash
git diff --check
git status --short
```

## Rollback

This slice is reversible by restoring the four candidate `evaluation_pack`
fields and reverting the gate diagnostic helpers/tests. Runtime behavior for
accounting, finance, and economics should not need rollback because this slice
does not change their manifests or routing logic.

## Next Step After This Slice

After this onboarding contract merges, write the business eval-ready design
spec. That later spec should add business-owned fixture cases, business signal
groups, method-lens rules, and near-miss guards before business can move from
`candidate` to `eval_ready`.
