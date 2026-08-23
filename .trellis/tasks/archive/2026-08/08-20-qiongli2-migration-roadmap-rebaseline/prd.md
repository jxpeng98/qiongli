# Rebaseline Qiongli 2 migration and master roadmap

## Goal

Rebase the Qiongli 2 master roadmap and verification flow around a reliable
replacement path for Qiongli 1.19, so the team can stop adding features to
1.19, concentrate development feedback on 2.x business behavior, and retire
1.19 only after observable replacement gates have passed.

This task plans and applies the roadmap/governance changes. It does not itself
implement every missing product capability discovered by the rebaseline.

## Background and Confirmed Constraints

- The repository is currently on the clean `2.x` branch.
- Qiongli 1.19 and 2.x are being developed or supported in parallel, which is
  diluting progress toward a stable 2.x release.
- The reported 2.x product flow is not yet dependable: App behavior is
  unstable, expected Research Graph behavior is incomplete, and capabilities
  inherited from 1.x across CLI, Plugin, Skills, and MCP are not all perceived
  as usable end to end.
- The current product-control contract defines the Alpha 3 spine as
  `App -> native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero`, prioritizes a
  Graph v1 semantic-continuity repair, and defers Graph v2 unless release scope
  is deliberately changed.
- Canonical Workflow/Skill content lives under `content/`; Plugin trees,
  installed Skills, embedded packs, and release payloads are generated outputs.
- The 2.x App must consume native product contracts rather than own a parallel
  implementation of product logic.
- `v1.19.0-beta.1`, `release/1.x-python`, and the merge base of that release
  with `2.x` all resolve to `8d2e99866ce4`; the two product lines therefore
  have an exact comparison baseline rather than an inferred historical one.
- `tooling/migration/qiongli-1x-product-parity.json` marks classification as
  complete, but it inventories only 16 outcomes. Seven have no acceptance
  evidence and six are explicitly deferred to `r4`; classification is not a
  replacement-readiness claim.
- The current master roadmap contains 233 task IDs across eight milestones and
  describes Academic Graph v1 and the internal first-usable spine as
  implemented. The live ledger currently records 25 accepted, 14 blocked, 19
  proposed, and 175 deferred tasks.
- The Alpha 3 ledger's Graph and Plugin/Skills evidence is primarily
  deterministic fixture, focused test, isolated-client, and historical exact-
  candidate evidence. Manual target claims, complete live-Host qualification,
  migration/rollback, and publication remain open.
- The August 18 Graph v1 continuity repair corrected readiness so structural
  `project`/`artifact` nodes and `contains` edges no longer count as scholarly
  continuity, and added a safe Skill-guided normalization path. Its accepted
  scope deliberately excluded Graph v2, a Typed Research Kernel, arbitrary
  prose inference, editable graph edges, and a new graph store.
- The user selected a complete, real-project-verified Graph v1 semantic and UI
  workflow as a 2.x replacement gate. Graph v2 and the Typed Research Kernel do
  not block 1.19 retirement and remain later roadmap work.
- `docs/maintainer/release-branch-policy.md` already freezes 1.19 to critical
  security and release-breakage fixes and ends its planned support window 90
  days after Qiongli 2 Stable. The master roadmap does not surface that accepted
  policy or connect it to an observable replacement gate.
- The roadmap's time-based 90-day development projection assumes previously
  closed technical slices are sufficient foundations, which conflicts with the
  reported end-user replacement experience.
- The current verification workflow is perceived as too long and too expensive
  in agent tokens. Development should spend its feedback budget on the changed
  business behavior, then widen verification only at explicit version and
  final-acceptance boundaries.

## Requirements

- Reuse the accepted branch policy that makes 2.x the only feature-development
  line, limits 1.19 to critical maintenance, and ends planned 1.x support 90
  days after Qiongli 2 Stable. Do not create a second branch-policy source.
- Produce an evidence-backed replacement matrix comparing user-visible 1.19
  and 2.x workflows across CLI, Plugin, Skills, Lite/Full MCP, Zotero, App, and
  Research Graph. Keep the current 16-outcome parity ledger as its bounded
  classification contract; the wider matrix supplements it because code
  presence and classification alone are not evidence of usability.
- Reorder the master roadmap around replacement-critical outcomes before broad
  App expansion or unrelated new functionality.
- Give Research Graph work an explicit versioned scope and observable
  acceptance boundary; distinguish Graph v1 semantic continuity from Graph v2
  expansion and from chart/UI polish.
- Make a complete Graph v1 journey a 2.x replacement gate: a representative
  migrated project must produce source-bound scholarly semantics, usable query
  and visualization behavior, deterministic rebuild, and truthful empty/sparse
  diagnostics. Keep Graph v2 and the Typed Research Kernel off this cutover
  path.
- Keep Workflow/Skill and Plugin content single-source. Define when a compatible
  content change is materialized for both release lines and when a 2.x-only
  runtime dependency requires a truthful 1.19 fallback.
- Define promotion, migration, rollback, and 1.19 end-of-life gates using exact
  build/package evidence rather than branch state or merged changes alone.
- Update the authoritative master roadmap and any directly derived governance
  view needed to keep task state and release claims consistent.
- Preserve accepted historical receipts while reopening or narrowing any
  current replacement claim contradicted by a reproduced user journey. Do not
  erase evidence merely because its scope was narrower than users expected.
- Turn newly exposed implementation gaps into bounded follow-up work instead of
  implementing them opportunistically inside this roadmap task.
- Replace the current implicit "run everything at the end of every task" habit
  with exactly three verification tiers:
  - during implementation, run the smallest focused check that can falsify the
    changed behavior;
  - after one complete business slice or small-version checkpoint, run affected
    package and cross-contract checks plus the required three-platform native
    source matrix;
  - run workspace-wide, target/package, live-Host, migration/rollback, and
    release evidence only for an explicit 2.x cutover or release candidate.
- Keep mandatory security, data-loss, schema-compatibility, and trust-boundary
  checks at the earliest tier that can catch their risk; test reduction must not
  weaken these protections.
- Make test output concise and evidence-oriented so agents consume summaries
  and failure details rather than full successful logs.

## Acceptance Criteria

- [ ] The roadmap names one authoritative 2.x recovery sequence from capability
      inventory through 1.19 retirement, with milestone entry and exit gates.
- [ ] The roadmap explicitly classifies 1.19 as feature-frozen maintenance and
      references the existing exceptional-change and 90-day post-Stable support
      policy.
- [ ] Every replacement-critical surface has an owner, current evidence state,
      target outcome, and verification path; unknown or unverified capability is
      not recorded as complete.
- [ ] Research Graph v1 semantics, Graph UI completion, and any deferred Graph
      v2 work have separate scope and acceptance claims.
- [ ] Graph v1 replacement acceptance includes one representative migrated
      project and does not depend only on synthetic fixtures or code presence.
- [ ] CLI, Plugin/Skills, Lite/Full MCP, Zotero, and App work is ordered by the
      shared product flow and does not rely on App-only success as proof of the
      underlying capability.
- [ ] The compatibility policy avoids two manually maintained copies of
      canonical Skill/Plugin content.
- [ ] The retirement gate requires a packaged 2.x candidate, representative
      migration evidence, rollback evidence, and successful critical workflows
      before 1.19 support ends.
- [ ] Existing roadmap identifiers and accepted evidence remain traceable, and
      historical completion is not silently rewritten.
- [ ] A roadmap reader can distinguish implementation evidence, packaged
      evidence, live-host evidence, migrated-user evidence, and release
      authorization without treating one as a substitute for another.
- [ ] The current 1.x parity ledger remains a bounded 16-outcome classification
      contract, while the wider roadmap replacement matrix makes clear that its
      `classification_status: complete` is not a 2.x parity claim.
- [ ] The roadmap and Trellis workflow define focused, business-slice, and final
      acceptance test tiers with unambiguous promotion triggers.
- [ ] Routine implementation does not run unrelated full-workspace, packaging,
      live-Host, or cross-target suites unless a changed trust boundary or an
      explicit promotion gate requires them.
- [ ] A failed higher-tier run reports the owning business slice and the minimum
      focused reproduction; successful output remains compact.
- [ ] The four required `2.x` branch-protection check identities remain intact;
      portable frontend checks run once per Slice, while three-platform Rust
      source checks remain required.
- [ ] Non-publishing desktop package assembly, packaged-product acceptance, Lite
      candidate acceptance, and Community Alpha promotion do not run during
      ordinary implementation; the existing explicit candidate flow owns them.
- [ ] Roadmap/governance validation commands pass after the documentation change.

## Out of Scope

- Implementing the missing CLI, Plugin, Skill, MCP, App, or Graph capabilities.
- Ending 1.19 support before the newly defined retirement gate is satisfied.
- Moving Graph v2, the Typed Research Kernel, more providers, more agents, or
  remote collaboration onto the 1.19-retirement critical path.

## Key Decisions

- Graph v1 replacement readiness blocks 1.19 retirement; Graph v2 and the Typed
  Research Kernel do not.
- Accepted historical evidence and task IDs remain traceable. Unaccepted tasks
  may be reordered, narrowed, deferred, or superseded to restore a truthful 2.0
  critical path.
- Add only `GOV-320`, `PLT-320`, `PLT-321`, and `PLT-322` for responsibilities
  not already owned by the roadmap. Reuse existing `REL-*` cutover tasks; the
  authoritative inventory therefore grows from 233 to 237 IDs.
- Alpha/beta business slices use Slice verification. Full Acceptance runs only
  for an explicit 2.x cutover or release candidate, with risk-triggered security
  and data-safety checks allowed earlier.
