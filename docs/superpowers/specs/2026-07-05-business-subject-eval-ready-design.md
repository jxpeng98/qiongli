# Business Subject Eval-Ready Design

## Goal

Prepare business as the first post-accounting subject expansion by moving it
from a deferred `candidate` shell to `eval_ready` only after a business-owned
fixture pack, manifest-backed signals, and near-miss guards are in place.

This is not a runtime activation slice. Business must remain unavailable for
default runtime subject switching until a later runtime-enabled promotion
review proves precision after eval-ready.

## Current Baseline

On `dev` as of July 5, 2026:

- Accounting, finance, and economics are `runtime_enabled`.
- The subject expansion onboarding contract is merged.
- Business is a deferred `candidate` with:
  - empty `signal_groups`,
  - empty `method_lenses`,
  - blank `evaluation_pack`.
- Business resources already exist:
  - `content/skills/domain-profiles/business-management.yaml`,
  - `content/subjects/business/skills/business-journal-positioning-auditor.md`,
  - venue profiles for Academy of Management Journal, Organization Science,
    Journal of Management, Journal of Marketing, and Strategic Management
    Journal,
  - overlays for manuscript architecture, study design, and statistics.

Current business eval-ready gate should fail closed with explicit onboarding
blockers:

```text
business --gate eval-ready:
  activation_status: candidate
  eligible_for_eval_ready: false
  blocking_failures:
    - activation_status is candidate
    - missing evaluation_pack for deferred subject
    - missing signal dimension: method
    - missing signal dimension: data_or_outcome
    - missing signal dimension: venue
    - missing signal dimension: theory_or_construct
    - missing clear_positive fixtures
    - missing method_only_borrow fixtures
    - missing near_miss fixtures
```

## Decision

Add a business eval-ready pack as a guarded subject expansion slice.

Why business is the next candidate:

- Business resources and venue profiles already exist.
- Business has a well-defined quality bar in the local domain profile:
  management theory contribution, construct clarity, method transparency, and
  venue positioning.
- It is adjacent enough to finance/economics/accounting that near-miss guards
  are useful, but the intended scope can stay bounded to scholarly business,
  management, organization, strategy, marketing, and operations research.

## Scope

In scope:

- Add a business-owned fixture pack under
  `tests/fixtures/subject_router_eval/business/`.
- Add business manifest signal groups for method, data/outcome, venue, and
  theory/construct dimensions.
- Add business method lenses that borrow business review guidance without
  switching the primary subject from one method phrase alone.
- Move business from `candidate` to `eval_ready`.
- Point business `evaluation_pack` at
  `tests/fixtures/subject_router_eval/business`.
- Add contract and router-gate tests proving business passes `eval-ready` and
  fails `runtime-enabled`.
- Re-run accounting, finance, and economics runtime gates to guard against
  adjacent-subject regressions.
- Update the roadmap to mark business eval-ready as the current Stage 4 slice.

Out of scope:

- Do not promote business to `runtime_enabled`.
- Do not add political economy, geoeconomics, or economics-accounting fixtures.
- Do not broaden accounting, finance, or economics signals.
- Do not change provider configuration, literature search, full-text retrieval,
  Zotero behavior, local-agent execution, or release automation.
- Do not add network-dependent verification.

## Business Subject Boundary

Business can become the primary subject in eval-ready measurement only when a
request has at least two business-specific dimensions such as:

- business method plus management theory construct,
- venue plus construct or theory contribution,
- organizational/firm/team/platform setting plus mechanism or construct,
- marketing/strategy/organization venue plus empirical design.

Business must not activate from:

- generic "business plan", "small business", or startup operations wording,
- course case assignments or teaching cases without research contribution,
- consulting, market analysis, sales enablement, or product marketing copy,
- project management or workflow operations,
- generic firm, customer, or market mentions that are actually finance,
  economics, accounting, or core planning tasks.

Method-only business evidence may still borrow business lenses without
suggesting business.

## Proposed Manifest Signals

The implementation plan should encode these signal groups in
`content/subjects/business/runtime-subject.yaml`.

### Method signals

Suggested method-only lenses:

- `business.method.gioia`
  - value: `gioia-method`
  - activation: `method_only`
  - patterns: `Gioia`, `first-order concepts`, `second-order themes`,
    `aggregate dimensions`
  - near misses: generic "concept map" or non-research theme sorting
- `business.method.case-study`
  - value: `case-study`
  - activation: `method_only`
  - patterns: `multiple case study`, `Eisenhardt`, `Yin case study`,
    `within-case`, `cross-case`
  - near misses: teaching case, business case, case interview
- `business.method.process-research`
  - value: `process-research`
  - activation: `method_only`
  - patterns: `process research`, `temporal bracketing`, `event timeline`,
    `turning points`
  - near misses: process improvement workflow

### Data or outcome signals

Suggested subject-level data/outcome signals:

- `business.data.organization-panel`
  - value: `organization-panel`
  - activation: `subject`
  - patterns: `firm-level panel`, `organization-level data`, `team-level data`,
    `manager survey`, `employee survey`
- `business.data.qualitative-fieldwork`
  - value: `qualitative-fieldwork`
  - activation: `subject`
  - patterns: `interviews with managers`, `fieldnotes`,
    `organizational ethnography`, `case evidence database`,
    `archival documents`
- `business.data.market-platform`
  - value: `market-platform`
  - activation: `subject`
  - patterns: `platform marketplace`, `customer journey`, `marketing channel`,
    `firm-customer interaction`

### Venue signals

Suggested context-only venue signals:

- `business.venue.amj`
  - value: `academy-of-management-journal`
  - patterns: `Academy of Management Journal`, `AMJ`
- `business.venue.organization-science`
  - value: `organization-science`
  - patterns: `Organization Science`
- `business.venue.journal-of-management`
  - value: `journal-of-management`
  - patterns: `Journal of Management`
- `business.venue.journal-of-marketing`
  - value: `journal-of-marketing`
  - patterns: `Journal of Marketing`
- `business.venue.strategic-management-journal`
  - value: `strategic-management-journal`
  - patterns: `Strategic Management Journal`, `SMJ`

### Theory or construct signals

Suggested subject-level theory/construct signals:

- `business.construct.theory-contribution`
  - value: `theory-contribution`
  - activation: `subject`
  - patterns: `management theory`, `theory contribution`, `literature stream`,
    `construct clarity`, `boundary conditions`
- `business.construct.organization-mechanism`
  - value: `organization-mechanism`
  - activation: `subject`
  - patterns: `organizational mechanism`, `strategy mechanism`, `capability`,
    `dynamic capabilities`, `organizational routines`
- `business.construct.managerial-implication`
  - value: `managerial-implication`
  - activation: `subject`
  - patterns: `managerial implication`, `business phenomenon`,
    `strategic management`, `competitive advantage`

## Method Lenses

Business should add method-level lenses that can be borrowed by other subjects:

```yaml
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
```

These lenses should not by themselves make business the primary subject.

## Required Fixture Pack

Create `tests/fixtures/subject_router_eval/business/` with at least these
fixtures:

- `clear_management_theory_case_study.json`
  - clear positive
  - expected primary subject: `business`
  - expected method lenses: `business-positioning`, `qualitative-transparency`
- `clear_marketing_platform_experiment.json`
  - clear positive
  - expected primary subject: `business`
  - expected method lenses: `business-positioning`
- `method_only_gioia_borrow.json`
  - method-only borrow
  - expected primary subject remains `auto`
  - forbidden subject: `business`
  - expected method lens: `qualitative-transparency`
- `mixed_finance_strategy_returns.json`
  - mixed business-finance case
  - expected primary subject should stay with the dominant framing in the
    fixture; business may appear only when business theory contribution is
    explicit
- `locked_economics_borrow_business_positioning.json`
  - locked economics subject
  - expected decision: `keep_locked`
  - may borrow business positioning lens
- `confirmed_business_journal_positioning.json`
  - confirmed business subject
  - expected primary subject: `business`
- `near_miss_small_business_plan.json`
  - near miss
  - forbidden subject: `business`
- `near_miss_consulting_market_analysis.json`
  - near miss
  - forbidden subject: `business`
- `near_miss_project_management_workflow.json`
  - near miss
  - forbidden subject: `business`
- `near_miss_teaching_case_assignment.json`
  - near miss
  - forbidden subject: `business`

Each fixture must use `subject_under_test: "business"` and include tags that
support the eval-ready gate:

- `clear_positive`,
- `method_only_borrow`,
- `near_miss`.

## Expected Gate Behavior

After this slice:

```text
business --gate eval-ready:
  activation_status: eval_ready
  eligible_for_eval_ready: true
  eligible_for_runtime_enabled: false
  blocking_failures: []
```

```text
business --gate runtime-enabled:
  activation_status: eval_ready
  eligible_for_runtime_enabled: false
  blocking_failures:
    - activation_status is eval_ready
```

Accounting, finance, and economics runtime-enabled gates must remain green.
Political economy, geoeconomics, and economics-accounting must remain deferred
candidates and keep the onboarding blockers.

## Expected Tests

Add or update tests in:

- `tests/test_subject_contracts.py`
  - business is `eval_ready`,
  - business has non-empty signal groups in all four dimensions,
  - business has method lenses,
  - business `evaluation_pack` points to the business fixture pack,
  - remaining deferred subjects still have blank `evaluation_pack`.
- `tests/test_subject_router_eval.py`
  - business fixture inventory includes required clear, method-only, mixed,
    locked, confirmed, and near-miss cases,
  - business eval-ready gate passes the real fixture pack,
  - business runtime-enabled gate fails on `activation_status is eval_ready`,
  - other deferred subjects still fail closed with onboarding blockers.
- `tests/test_subject_refinement.py`
  - clear business evidence can be measured under `evaluation_subjects`,
  - method-only business evidence borrows a lens without suggesting business,
  - runtime default does not activate business while business is only
    eval-ready.

## Verification

Required focused tests:

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
  --subject business \
  --gate eval-ready \
  --json

uv run python tooling/scripts/evaluate_subject_router.py \
  --subject business \
  --gate runtime-enabled \
  --json

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

Required full eval:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected:

- Business eval-ready exits `0` with empty blockers.
- Business runtime-enabled exits non-zero because activation is still
  `eval_ready`.
- Accounting, finance, and economics runtime-enabled gates exit `0`.
- Full eval metrics remain at required thresholds with no new near-miss false
  positives.

Final hygiene:

```bash
git diff --check
git status --short
```

## Rollback

Rollback should restore business to:

```yaml
activation_status: candidate
signal_groups:
  method: []
  data_or_outcome: []
  venue: []
  theory_or_construct: []
method_lenses: {}
evaluation_pack: ""
```

Business fixtures and tests can remain as candidate planning artifacts only if
the gate keeps failing closed; otherwise revert the fixture pack with the
manifest change.

## Next Step After This Slice

If business eval-ready passes and review finds no precision issues, prepare a
separate business runtime promotion design. That later design must add at least
one auto-mode method-only guard and rerun accounting, finance, economics, and
business gates together before changing business to `runtime_enabled`.
