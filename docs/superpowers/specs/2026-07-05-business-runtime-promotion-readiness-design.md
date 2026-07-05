# Business Runtime Promotion Readiness Design

## Goal

Add a promotion-readiness review path for business that proves default runtime
precision under a simulated `runtime_enabled` status while keeping the checked
in business manifest at `activation_status: eval_ready`.

This is not the activation change. The output of this slice is a stronger
harness and fixture pack that can say whether business is safe to promote in a
later, smaller PR.

## Current Baseline

As of July 5, 2026 on `dev`:

- Accounting, economics, and finance are `runtime_enabled`.
- Business is `eval_ready`.
- Business eval-ready gate passes with a business-owned fixture pack.
- Business runtime-enabled gate fails closed because the manifest is still
  `activation_status: eval_ready`.
- Business default runtime does not suggest business from auto-mode task text.

The current eval-ready gate proves that business signals can be measured when
`evaluation_subjects={"business"}` is provided. It does not prove that default
runtime routing would remain precise after business becomes runtime-enabled.

## Decision

Introduce a distinct `promotion-ready` subject gate.

The new gate evaluates the requested subject's fixture pack as if that subject
were runtime-enabled, but it does not edit the runtime subject manifest. It is a
pre-promotion harness:

- `eval-ready`: current status must be `eval_ready`; eval-only measurement is
  allowed through `evaluation_subjects`.
- `promotion-ready`: current status must still be `eval_ready`; evaluation runs
  default routing with a test-only activation override for the target subject.
- `runtime-enabled`: current status must be `runtime_enabled`; no activation
  override is used.

This separation keeps review semantics explicit:

- A subject can be eval-ready without being promotion-ready.
- A subject can be promotion-ready without being activated.
- Runtime activation remains a separate reviewed manifest-status change.

## Scope

In scope:

- Add a `promotion-ready` gate to `tooling/scripts/evaluate_subject_router.py`.
- Add a test-only activation override path to subject refinement so the gate can
  evaluate default routing as if business were runtime-enabled.
- Keep `content/subjects/business/runtime-subject.yaml` at
  `activation_status: eval_ready`.
- Fill business runtime resource metadata needed for a later promotion review,
  using existing checked-in business resources.
- Add promotion-ready fixture expectations for business positive, mixed,
  method-only, locked, confirmed, and near-miss cases.
- Expand business near-miss coverage for practitioner marketing, sales,
  consulting, strategy, customer, and launch vocabulary.
- Add a second non-qualitative business positive fixture so promotion readiness
  is not only case-study or marketing-journal driven.
- Update CLI and roadmap docs to explain the new gate.
- Verify accounting, economics, and finance runtime-enabled gates still pass.

Out of scope:

- Do not change business to `runtime_enabled`.
- Do not add political economy, geoeconomics, or economics-accounting subject
  packs.
- Do not change provider configuration, Zotero, literature retrieval, full-text
  search, local-agent execution, or release automation.
- Do not add network-dependent verification.
- Do not change normal install behavior or project subject lifecycle commands.

## Business Promotion Boundary

Promotion-ready business may suggest business in default routing only when a
request includes subject-level scholarly business evidence from at least two
dimensions. Acceptable promotion-ready positives include:

- management theory plus empirical business design,
- business venue plus construct or theory contribution,
- organizational, firm, team, or platform research data plus mechanism,
- strategic management or marketing research framing plus manuscript venue.

Promotion-ready business must not suggest business from:

- practitioner product launch, sales enablement, customer journey, or channel
  execution work,
- consulting market analysis, competitor bullets, or client workshop requests,
- startup or small-business planning,
- teaching case assignments or case interview preparation,
- project management workflow requests,
- generic competitive advantage, customer, market, firm, or capability wording
  that lacks scholarly research context,
- finance, economics, or accounting tasks where business language is secondary.

Method-only business evidence may still borrow a business method lens without
making business primary.

## Architecture

### Activation Override

Add an optional activation override mapping to the subject refinement call path:

```python
activation_status_overrides: Mapping[str, str] | None = None
```

The override is used only by evaluation tooling. It should:

- affect `_subject_can_be_suggested()` for the requested subject,
- affect manifest-backed runtime subject matches so their
  `activation_status` reflects the override,
- affect candidate filtering in the same way normal runtime status does,
- leave checked-in runtime subject manifests unchanged,
- remain absent from normal runtime callers.

The override must not use `evaluation_subjects`; promotion-ready is explicitly
testing default runtime behavior after activation, not eval-only behavior.

### Subject Gate Semantics

Extend gate choices:

```python
GATE_CHOICES = ("runtime-enabled", "eval-ready", "promotion-ready")
```

For `promotion-ready`:

- current contract status must be `eval_ready`,
- subject cases are evaluated with
  `activation_status_overrides={subject: "runtime_enabled"}`,
- `evaluation_subjects` is `None`,
- runtime resources required by `runtime-enabled` are checked,
- threshold failures block `eligible_for_runtime_promotion`.

The report should expose:

```json
{
  "eligible_for_eval_ready": false,
  "eligible_for_runtime_promotion": true,
  "eligible_for_runtime_enabled": false
}
```

when the promotion-ready gate passes.

### Business Runtime Resource Metadata

Keep business status `eval_ready`, but set its subject skill to an existing
business runtime resource so promotion-ready resource checks can pass:

```yaml
subject_skill: content/subjects/business/skills/business-journal-positioning-auditor.md
```

Keep `overlay: ""` for this slice. The runtime resource check treats a blank
overlay as optional, matching accounting.

### Fixture Strategy

Add promotion-ready expectations only where default eval-ready expectations are
not enough:

- clear business positives should expect primary business under
  `promotion-ready`,
- method-only business should continue to borrow a lens without suggesting
  business,
- finance-dominant mixed cases should keep finance primary while allowing
  business only as a neighbor when appropriate,
- confirmed business should stay business and should not rely on eval-only
  subject measurement,
- near-misses should forbid business under promotion-ready.

Add at least four new business fixtures:

- a clear non-qualitative management/organization positive,
- a clear strategic management positive,
- a practitioner customer/marketing/sales near-miss,
- a practitioner strategy/competitive-advantage/capability near-miss.

## Testing Requirements

Minimum verification:

```bash
uv run python -m pytest tests/test_subject_refinement.py tests/test_subject_router_eval.py -q
uv run python -m pytest tests/test_subject_contracts.py tests/test_subject_router_eval.py tests/test_subject_refinement.py -q
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate eval-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate promotion-ready --json
uv run python tooling/scripts/evaluate_subject_router.py --subject business --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject finance --gate runtime-enabled --json
uv run python tooling/scripts/evaluate_subject_router.py --subject economics --gate runtime-enabled --json
git diff --check
```

Expected results:

- default eval exits 0,
- business eval-ready exits 0,
- business promotion-ready exits 0 if the new readiness pack passes,
- business runtime-enabled exits 1 while the manifest remains `eval_ready`,
- accounting, finance, and economics runtime-enabled gates exit 0,
- whitespace check exits 0.

## Risks

- Promotion-ready may create false confidence if the fixture pack is too narrow.
  Mitigation: add practitioner near-misses with business-like wording before
  allowing the gate to pass.
- Activation override code could leak into normal runtime behavior. Mitigation:
  keep the parameter optional, default to no override, and cover default business
  suppression tests.
- Adding a new gate can confuse CLI consumers. Mitigation: document the exact
  difference between eval-ready, promotion-ready, and runtime-enabled.
- Business vocabulary remains broad by nature. Mitigation: keep runtime
  activation as a separate PR even after promotion-ready passes.

## Rollback

If the promotion-ready gate is noisy or misleading:

- remove `promotion-ready` from gate choices,
- remove activation override plumbing,
- keep business as `eval_ready`,
- keep added near-miss fixtures if they improve default eval coverage.

No user project state migration is required because this slice does not write
project manifests or change runtime activation status.
