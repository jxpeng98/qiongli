# Business Runtime Activation Design

## Goal

Promote business from `activation_status: eval_ready` to
`activation_status: runtime_enabled` after the merged promotion-ready gate has
proved default routing precision under a harness-only runtime simulation.

This is a narrow activation slice. It should make business available to the
adaptive runtime for clear scholarly business, management, strategy,
organization, marketing, and operations research requests while preserving the
rule that method-only or practitioner wording does not switch the primary
subject.

## Current Baseline

On `dev` as of July 5, 2026:

- Accounting, economics, and finance are `runtime_enabled`.
- Business is `eval_ready`.
- Business has a business-owned fixture pack, runtime resources, method lenses,
  near-miss guards, and promotion-ready gate coverage.
- Business `eval-ready` gate exits 0.
- Business `promotion-ready` gate exits 0 with
  `eligible_for_runtime_promotion: true`.
- Business `runtime-enabled` gate exits 1 only because the manifest is still
  `activation_status: eval_ready`.

Current gate evidence:

```text
business --gate eval-ready:
  activation_status: eval_ready
  eligible_for_eval_ready: true
  eligible_for_runtime_promotion: false
  eligible_for_runtime_enabled: false
  blocking_failures: []
  metrics:
    decision_accuracy: 1.0
    primary_subject_accuracy: 1.0
    suggest_subject_precision: 1.0
    near_miss_false_positives: 0
    forbidden_subject_accuracy: 1.0
    method_lens_accuracy: 1.0
    all_case_checks_passed: 1.0

business --gate promotion-ready:
  activation_status: eval_ready
  eligible_for_eval_ready: false
  eligible_for_runtime_promotion: true
  eligible_for_runtime_enabled: false
  blocking_failures: []
  metrics:
    decision_accuracy: 1.0
    primary_subject_accuracy: 1.0
    suggest_subject_precision: 1.0
    near_miss_false_positives: 0
    forbidden_subject_accuracy: 1.0
    method_lens_accuracy: 1.0
    all_case_checks_passed: 1.0

business --gate runtime-enabled:
  activation_status: eval_ready
  eligible_for_eval_ready: false
  eligible_for_runtime_promotion: false
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

## Decision

Run the business activation slice now, before starting political economy,
geoeconomics, or the economics-accounting bridge.

Why this order:

- The promotion-ready gate is green and was created specifically as the
  pre-activation review path.
- The activation diff can be small: manifest status, gate expectations, tests,
  and docs.
- Business has broader vocabulary than accounting, so activating it before the
  next subject expansion gives the harness a chance to catch runtime
  over-activation while the scope is still bounded.
- A separate activation PR preserves the review line between "ready to promote"
  and "actually runtime-enabled."

## Scope

In scope:

- Promote `content/subjects/business/runtime-subject.yaml` to
  `activation_status: runtime_enabled`.
- Update business runtime-enabled fixture expectations where promotion-ready
  expectations now become normal runtime behavior.
- Update tests that still assert business is eval-ready.
- Preserve tests proving eval-ready and promotion-ready gates reject subjects
  whose checked-in status is already runtime-enabled.
- Update CLI, release, and roadmap docs so business is described as
  runtime-enabled after the activation PR.
- Re-run subject gates for business, accounting, finance, and economics.

Out of scope:

- Do not change business signal weights or regex patterns unless the activation
  gate exposes a measured failure.
- Do not add political economy, geoeconomics, or economics-accounting subject
  packs.
- Do not change provider configuration, literature retrieval, Zotero behavior,
  full-text search, local-agent execution, or release automation.
- Do not add network-dependent tests.
- Do not remove the promotion-ready gate; it remains useful for future
  eval-ready subjects.

## Runtime Safety Rules

Business can become the primary subject only when evidence includes at least
two independent scholarly business dimensions, such as:

- business method plus business data or outcome,
- business venue plus theory or construct contribution,
- organization, firm, team, or platform data plus management mechanism,
- strategy or marketing research framing plus journal positioning.

Business must not become the primary subject when evidence is limited to:

- practitioner product launch, sales enablement, customer journey, or channel
  execution work,
- consulting market analysis, competitor bullets, or client workshop requests,
- startup or small-business planning,
- teaching case assignments or case interview preparation,
- project management workflow requests,
- generic strategy, customer, market, firm, capability, or competitive
  advantage wording without scholarly research context,
- finance, economics, or accounting tasks where business language is secondary.

Method-only business evidence may still borrow business method lenses such as
`qualitative-transparency`, `business-positioning`, or `construct-level-fit`
without making business primary.

## Fixture Strategy

After activation, `runtime-enabled` becomes the normal gate for business. The
fixture expectations should reflect that:

- clear business positives use the same expected outcome under
  `runtime-enabled` as they currently use under `promotion-ready`,
- method-only business cases continue to borrow method lenses without
  suggesting business,
- finance-dominant mixed cases keep finance as the primary subject and may
  allow business only as a neighbor,
- locked economics cases keep the locked primary subject and may allow business
  only as a neighbor,
- confirmed business cases remain business without relying on eval-only
  subject measurement,
- practitioner near-misses continue to forbid business.

No new business fixtures are required for the activation PR unless the
runtime-enabled gate exposes a false positive or false negative after the
manifest status changes.

## Expected Code Changes

### Business manifest

Change:

```yaml
activation_status: eval_ready
```

to:

```yaml
activation_status: runtime_enabled
```

Do not change `overlay`, `subject_skill`, signal groups, method lenses, or
activation gate thresholds in the activation commit unless a failing
runtime-enabled gate proves a narrower fix is required.

### Business fixtures

Update `gate_expected["runtime-enabled"]` for clear positive fixtures so
business can be the primary subject after activation. Use the existing
promotion-ready expected blocks as the source of truth.

Update locked and mixed cases only when the actual runtime-enabled output
contains business as an adjacent suggestion. The primary subject must remain
locked economics or finance respectively.

### Subject contract tests

Update default repository contract expectations:

- `subject_activation_status("business", contracts)` becomes
  `runtime_enabled`.
- The business manifest test should be renamed from eval-ready wording to
  runtime-enabled wording.
- Business must still declare signal groups, method lenses, evaluation pack,
  subject skill, near-miss policy, and gate metrics.

### Subject router gate tests

Update real business gate tests:

- `business --gate runtime-enabled` must pass.
- `business --gate eval-ready` should no longer be the real-fixture success
  gate after activation.
- `business --gate promotion-ready` should reject the real manifest because
  promotion-ready requires checked-in `eval_ready`.
- Patched tests should remain to prove eval-ready and promotion-ready semantics
  for future subject candidates.

### Subject refinement tests

Update runtime behavior expectations:

- Clear business evidence in auto mode can suggest business.
- Method-only business evidence in auto mode borrows a business lens without
  suggesting business.
- Confirmed business projects can load business subject resources now that
  business is runtime-enabled.
- Locked finance/economics projects still keep the lock and may only borrow or
  neighbor business where fixtures explicitly allow it.

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
uv run python tooling/scripts/evaluate_subject_router.py --json

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

Required docs and hygiene checks:

```bash
rg -n "business|promotion-ready|runtime-enabled|eval-ready|eligible_for_runtime" \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md \
  docs/reference/cli.md \
  docs/advanced/publish-pypi.md

git diff --check
```

## Acceptance Criteria

- Business manifest has `activation_status: runtime_enabled`.
- Business runtime-enabled gate exits 0.
- Business fixture metrics meet all manifest thresholds.
- Business near-miss false positives remain 0.
- Business method-only cases do not suggest business as a primary subject.
- Accounting, finance, and economics runtime-enabled gates still exit 0.
- Default subject router evaluation exits 0.
- Docs no longer describe business as only eval-ready after the activation PR.

## Rollback

If activation creates noisy business over-activation:

- revert business `activation_status` to `eval_ready`,
- keep the promotion-ready gate and business promotion-ready fixtures,
- keep additional near-miss fixtures if they improve coverage,
- open a follow-up hardening slice for Stage 5 explainability or narrower
  business signal boundaries.

No user project state migration is required for rollback because activation is
controlled by the runtime subject manifest.
