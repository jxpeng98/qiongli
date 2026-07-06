# Subject Expansion Evaluation Gates Design

## Goal

Define the contract that every new runtime subject must satisfy before it can
participate in adaptive subject activation.

Qiongli now installs as adaptive core and can infer economics or finance
refinements at runtime. The next risk is uncontrolled subject expansion:
accounting, business, political economy, geoeconomics, and future disciplines
already have or may gain content assets, but runtime activation must not become
"keyword spotted, subject switched." New subjects should enter the adaptive
runtime only after they provide auditable resources, curated evaluation
fixtures, near-miss guards, and regression thresholds that preserve existing
economics and finance precision.

This spec is the fourth adaptive subject runtime slice. It establishes a
subject onboarding contract and evaluation gate. It does not fully activate a
new subject.

## Current Context

The current `dev` baseline includes:

- Runtime subject refinement in
  `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`.
- Resource activation planning in
  `packages/python-qiongli/src/qiongli/bridges/subject_resources.py`.
- Project-local lifecycle controls in
  `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`.
- Managed subject guidance materialization in
  `packages/python-qiongli/src/qiongli/bridges/subject_guidance.py`.
- Preview and opt-in local-agent subject runtime smoke in
  `tooling/scripts/run_subject_runtime_smoke.py`.
- Router evaluation fixtures under `tests/fixtures/subject_router_eval/`.
- Candidate content resources under `content/subjects/` and domain profiles
  under `content/skills/domain-profiles/`.

The current router behavior is deliberately narrow. Economics and finance have
hard-coded signal patterns, default overlays, default subject skills, and
method-pack paths. Other subjects may exist in the content catalog, but they
must not become runtime-activatable simply because content files exist.

## Product Model

Installation remains full adaptive core by default:

```bash
qiongli install --profile full --target all --surface plugin
```

Users do not choose a subject during normal installation. During project use:

1. Runtime subject refinement starts from core guidance.
2. Task text, project manifest state, local guidance, and trace memory produce
   subject evidence.
3. The router may keep core, borrow a method lens, suggest a subject, or respect
   a confirmed or locked subject.
4. A subject can be runtime-activatable only if it has passed the onboarding
   gate.
5. Subjects that have content but no passing gate remain install/package
   resources only; they may appear in focused packages or documentation, but
   adaptive runtime should treat them as inactive candidates.

## Non-Goals

- Do not fully activate accounting, business, political economy, geoeconomics,
  or any other new subject in this slice.
- Do not make install-time subject selection part of the default flow.
- Do not rewrite the existing economics or finance router behavior unless a
  gate requires a compatible metadata representation.
- Do not replace lifecycle controls, subject guidance materialization, or real
  local-agent smoke.
- Do not generate large project-local subject content from subject packages.
- Do not let marketplace or Desktop ZIP installs bypass the runtime gate.
- Do not require local model CLIs or provider network access for evaluation
  gates.

## Subject Onboarding Contract

Every new runtime subject must provide a versioned onboarding manifest before
it can be considered for adaptive activation.

Proposed path:

```text
content/subjects/<subject>/runtime-subject.yaml
```

The manifest is subject-owned and small. It references existing resources
rather than copying their contents.

Required fields:

```yaml
schema_version: 1.0
subject: accounting
display_name: Accounting
activation_status: candidate
extends: core
domain_profile: content/skills/domain-profiles/accounting.yaml
overlay: overlays/accounting.yaml
subject_skill: skills/accounting/SKILL.md
signal_groups:
  method: []
  data_or_outcome: []
  venue: []
  theory_or_construct: []
method_lenses: {}
evaluation_pack: tests/fixtures/subject_router_eval/accounting/
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

Allowed `activation_status` values:

- `candidate`: content may be packaged, but runtime activation is disabled.
- `eval_ready`: fixtures and resources are complete enough to run the gate.
- `runtime_enabled`: subject passed the gate and can participate in adaptive
  activation.
- `disabled`: subject exists but must not be activated or packaged by default.

The first implementation may store the manifest in a different path if it fits
existing packaging better, but it must expose the same contract fields through
a stable loader.

## Resource Contract

The onboarding gate must validate resources before evaluating behavior.

Required resources for `eval_ready`:

- Domain profile reference.
- Subject overlay reference if the subject can be suggested or confirmed.
- Subject skill reference if the subject can be suggested or confirmed.
- Method-pack references for method-only borrowing.
- Signal groups with ids, dimensions, weights, and examples.
- Evaluation fixture pack.
- Near-miss fixture pack.
- Documentation entry explaining activation status and install-surface behavior.

Resource rules:

- A subject can define method packs without enabling full subject activation.
  That permits method-only borrowing when evidence is method-specific.
- `subject_overlay` and `subject_skill` must not be loaded for a subject whose
  `activation_status` is below `runtime_enabled`.
- A missing optional method pack should disable that lens, not fail the whole
  router.
- A missing required overlay or subject skill must keep the subject below
  `runtime_enabled`.
- Resource paths must be relative to the packaged Qiongli payload and must not
  point outside the payload root.

## Signal Contract

Signals should move from hard-coded subject-specific blocks toward a data-backed
registry. The current economics and finance patterns may remain as compatibility
fallbacks during migration, but new subjects should enter through the registry.

Each signal group must define:

- `id`: stable machine-readable id, such as
  `accounting.method.accrual-quality`.
- `subject`: owning subject.
- `dimension`: one of `method`, `data_or_outcome`, `venue`,
  `theory_or_construct`, or a future explicitly documented dimension.
- `value`: short value used in `method_lenses` or candidate records.
- `weight`: numeric contribution to confidence.
- `activation`: `subject`, `method_only`, or `context_only`.
- `patterns`: conservative text patterns or exact aliases.
- `examples`: positive text snippets for review and fixture generation.
- `near_misses`: snippets that must not trigger the signal.

Activation semantics:

- `subject` signals can contribute to `suggest_subject`.
- `method_only` signals can contribute to `borrow_lens` but cannot by
  themselves switch primary subject.
- `context_only` signals can explain context but cannot trigger subject or
  method-pack activation alone.

The router must explain which signal dimension caused each candidate. A subject
candidate without at least two independent subject-level dimensions should not
be suggested unless the subject's contract explicitly allows a single
high-confidence dimension and the evaluation gate proves it is precise.

## Evaluation Pack Contract

Each subject needs a dedicated fixture directory:

```text
tests/fixtures/subject_router_eval/<subject>/
```

Required fixture categories:

- `clear_positive`: the subject should be suggested.
- `method_only_borrow`: only a method lens should be borrowed.
- `mixed_subject`: the subject appears with an adjacent discipline; expected
  primary subject and allowed neighboring subjects must be explicit.
- `near_miss`: wording is adjacent but should remain core or another subject.
- `locked_subject`: an existing locked subject should not be replaced.
- `confirmed_subject`: confirmed state should load the subject as applied.
- `dismissed_subject`: prior dismissal should suppress repeated promotion until
  new evidence appears.
- `legacy_regression`: economics and finance behavior must remain unchanged.

Fixture schema should extend the current router-eval shape without breaking
existing fixtures:

```json
{
  "id": "accounting_clear_accrual_quality",
  "subject_under_test": "accounting",
  "description": "Accounting request with measurement and accrual-quality signals.",
  "task_packet": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "accrual quality and earnings management",
    "context": "Estimate discretionary accruals and compare audit committee effects."
  },
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "accounting",
    "suggest_subjects": ["accounting"],
    "allowed_neighbor_subjects": [],
    "forbidden_subjects": ["finance", "economics"],
    "method_lenses": ["accrual-quality"]
  },
  "tags": ["accounting", "clear-positive"]
}
```

Existing top-level fixtures may stay in place. The runner should accept both
legacy flat fixtures and subject-scoped fixture packs.

## Evaluation Gate

The existing evaluation runner should gain subject-scoped gate mode:

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

Gate output should include:

```json
{
  "subject": "accounting",
  "activation_status": "eval_ready",
  "eligible_for_runtime_enabled": false,
  "metrics": {
    "primary_subject_accuracy": 0.0,
    "suggest_subject_precision": 0.0,
    "near_miss_false_positives": 0,
    "legacy_regression_failures": 0
  },
  "blocking_failures": [
    "missing clear_positive fixtures",
    "subject activation_status is candidate"
  ]
}
```

Minimum gate checks:

- Subject manifest validates against schema.
- Required resources exist and stay inside the payload root.
- Required fixture categories are present.
- Subject-specific cases pass thresholds.
- All legacy economics and finance eval fixtures still pass.
- Core-only and near-miss false positives remain zero.
- Method-only cases produce `borrow_lens`, not `suggest_subject`.
- Locked-subject cases preserve locked state.
- Report exits non-zero when the subject is not eligible.

The gate should fail closed. If a subject manifest is malformed or missing, the
runtime should treat that subject as `candidate` or unavailable.

## Activation Policy

Runtime activation is controlled by both evidence and gate state.

Policy table:

| Evidence | Gate state | Runtime result |
| --- | --- | --- |
| Weak or single context-only signal | any | `no_subject` / core only |
| Method-only signal | method pack available | `borrow_lens` |
| Method-only signal | no method pack | core only with diagnostic note |
| Strong subject signal | below `runtime_enabled` | core or borrow only; include candidate diagnostics |
| Strong subject signal | `runtime_enabled` | `suggest_subject` |
| Confirmed subject | resources available | `confirm_subject` |
| Locked subject | resources available | `lock_subject` |
| Locked subject plus neighboring method signal | method pack available | locked subject plus borrowed lens |

Important constraints:

- Content presence is not activation permission.
- Subject suggestions require `runtime_enabled`.
- Borrowed method lenses require only the method-pack part of the resource gate.
- Confirmed or locked subjects should report missing-resource warnings instead
  of silently loading unrelated resources.
- Near-miss evidence must bias toward core or borrowed-lens behavior.

## Cross-Surface Behavior

### Full CLI Runtime

Full Python runtime can run the gate, inspect subject status, and update
project-local guidance. It should be the authoritative validation path for
maintainers who want to promote packaged subject metadata from `eval_ready` to
`runtime_enabled`.

### MCP And Local Client Runtime

MCP tools should expose activation diagnostics in preview packets:

- candidate subjects blocked by gate,
- missing resources,
- near-miss protection,
- whether a method lens was borrowed instead of suggesting a subject.

MCP clients should not need to understand packaging paths.

### Marketplace Plugins

Marketplace installs may include complete subject content, but client-native
plugins should still rely on the packaged activation metadata. A plugin that
ships a subject skill does not imply that adaptive runtime can suggest that
subject.

### npm-lite And Desktop ZIPs

npm-lite and Desktop ZIPs may package focused subject content for compatibility
or upload-size reasons. They do not by themselves grant full runtime activation.
Docs should describe focused packages as content packages, while runtime
subject behavior belongs to the full runtime gate.

### Read-Only Clients

Read-only clients can display recommendations and blocked-candidate
diagnostics. They should export proposed actions rather than writing
`.qiongli/` files directly.

## Migration Strategy

The migration should be incremental:

1. Add schema and loader for subject runtime manifests.
2. Represent existing economics and finance as `runtime_enabled` through the
   contract while preserving current router behavior.
3. Mark accounting, business, political economy, geoeconomics, and
   economics-accounting as `candidate` or `eval_ready`, depending on available
   fixtures.
4. Extend evaluation runner to report gate eligibility.
5. Only after the gate is reliable, enable one additional subject in a later
   implementation slice.

This avoids a broad behavior change while still making future expansion
mechanical and auditable.

## Testing Strategy

Unit tests:

- Manifest schema validation accepts valid subject contracts.
- Manifest schema validation rejects missing resource references, invalid
  activation status, path escapes, and unknown dimensions.
- Resource loader treats missing candidate subject resources as inactive.
- Gate output fails closed for malformed manifests.

Router tests:

- Existing economics and finance fixtures remain green.
- Candidate subjects with strong text evidence do not trigger
  `suggest_subject` before `runtime_enabled`.
- Method-only candidate signals can borrow lenses only when method packs are
  present and allowed.
- Near-miss fixtures stay core-only or preserve the expected primary subject.

Integration tests:

- `evaluate_subject_router.py --json` keeps existing output compatible.
- `evaluate_subject_router.py --subject <subject> --gate runtime-enabled --json`
  reports eligibility and blocking failures.
- Subject runtime smoke still passes for economics and finance behavior.
- Release-readiness checks can include subject gate summaries without launching
  local agents.

## Documentation Updates

Documentation should explain:

- Normal install remains adaptive core.
- Subject packages may contain content before they are runtime-enabled.
- Runtime-enabled subjects require evaluation gates.
- Near-miss fixtures are mandatory, not optional.
- Focused Desktop ZIPs and marketplace subject entries are package surfaces,
  not automatic adaptive runtime activation.

Recommended docs:

- `docs/reference/cli.md`: gate commands and subject activation status.
- `docs/guide/install.md`: distinction between content package and runtime
  activation.
- `docs/advanced/publish-pypi.md`: release checklist for subject gates.
- `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`:
  update Stage 4 status after implementation.

## Success Criteria

The first implementation is complete when:

- A subject runtime manifest contract exists and is validated.
- Existing economics and finance can be represented without behavioral
  regression.
- Candidate subjects cannot be suggested even if they have content resources.
- Evaluation runner reports subject gate eligibility and blocking failures.
- Near-miss false positives remain zero in current economics/finance eval.
- Docs clearly separate install/package surfaces from runtime activation.
- All existing subject runtime smoke and router evaluation tests remain green.

## Rollback Plan

The feature should be easy to disable:

- Treat all new subject manifests as `candidate`.
- Keep existing economics and finance fallback rules active.
- Leave content packaging untouched.
- Skip subject-scoped gate checks in release automation until fixed.

No user project files should require migration or rollback because this spec
does not change `.qiongli/guidance_manifest.yaml` semantics.

## Open Implementation Notes

- The first implementation plan should decide whether subject manifests live
  under `content/subjects/<subject>/runtime-subject.yaml` or a central
  registry file.
- The gate should prefer structured YAML/JSON validation over ad hoc string
  inspection.
- New subject signal patterns should be conservative enough that a single
  ambiguous phrase does not activate a subject.
- The code path should avoid making `subject_refinement.py` larger by moving
  registry loading and signal detection into focused modules.
