# Current roadmap and verification audit

Audit date: 2026-08-21

## Executive finding

The repository already contains the right low-cost development principle, but
the execution surfaces do not apply it consistently. Product control says to
run one focused local check, one exact-head CI run after the slice is frozen,
and packaged acceptance only when package inputs or release claims require it.
`trellis-check` instead says to run the project's lint, type-check, and tests
without selecting a tier, and `Native CI` currently builds three platform
packages plus macOS packaged acceptance on every `2.x` pull request and push.

The roadmap has the same mismatch. It records technically accepted Graph v1
and App/CLI/Plugin/Skills/MCP slices, but it does not make a representative
migrated-user journey the authority for replacing 1.19. The existing 1.x
parity ledger is a 16-outcome classification contract, not a complete product
replacement ledger.

## Confirmed branch and replacement policy

- `docs/maintainer/release-branch-policy.md` already freezes
  `release/1.x-python` at `v1.19.0-beta.1` for critical security or release-
  breakage fixes only. No second branch policy is needed.
- The same policy ends the planned 1.x support window 90 days after Qiongli 2
  Stable unless a later explicit decision supersedes it.
- The master roadmap must surface and depend on that accepted policy; it must
  not restate a conflicting maintenance window.
- `v1.19.0-beta.1`, `release/1.x-python`, and the merge base with `2.x` identify
  the same source commit, `8d2e99866ce4`, so comparison has an exact oracle.

## Confirmed replacement-evidence gap

- `tooling/migration/qiongli-1x-product-parity.json` classifies 16 accepted 1.x
  outcomes. Seven have no acceptance evidence; six are deferred to `r4`.
- Its schema covers installation, setup, discovery, doctor, update, remove, and
  orchestration. It does not claim a complete App, research workflow, Graph,
  Zotero, or migrated-user replacement journey.
- `classification_status: complete` is therefore true for its bounded inventory
  and must remain distinct from product replacement readiness.
- The master roadmap contains 233 task IDs and currently describes Academic
  Graph v1 and the internal first-usable spine as implemented. The program
  ledger records only 25 accepted tasks; 14 are blocked, 19 proposed, and 175
  deferred.
- Historical accepted receipts must remain accepted for their exact technical
  scope. A narrower accepted receipt is not proof of the wider user outcome.

## Confirmed Graph boundary

- The August 18 Graph v1 repair makes readiness ignore structural
  `project`/`artifact` nodes and `contains` edges and gives Skills a safe
  canonical normalization sequence.
- That task intentionally excludes Graph v2, the Typed Research Kernel,
  arbitrary prose inference, editable graph edges, and new graph storage.
- The user selected a complete Graph v1 journey over moving Graph v2/Kernel
  onto the 1.19-retirement path.
- Replacement evidence must use a representative migrated project and prove
  source-bound semantic records, non-containment relations, useful query and
  visualization, deterministic rebuild, and truthful empty/sparse states.

## Current verification surfaces

### Local Trellis flow

- `.trellis/spec/product/control/index.md` already owns the evidence ladder.
- `.agents/skills/trellis-check/SKILL.md` does not select focused, slice, or
  final-acceptance scope and can be read as a command to run everything.
- `.trellis/workflow.md` requires the last Phase 2.2 pass to be “full-scope” but
  does not distinguish full task scope from full repository/release scope.
- Successful verbose logs are often returned to the agent even though only the
  summary and failure reproduction affect the next decision.

### GitHub flow

- Required `2.x` checks are the change boundary plus Linux, macOS, and Windows
  Rust foundation jobs. The repository branch policy and live ruleset depend on
  those check identities.
- Each Rust foundation job currently repeats App API, Desktop, and npm tests
  through `setup-qiongli-desktop`; only one platform needs to own those portable
  frontend checks while all three retain their target Rust checks and builds.
- `Native CI` also runs three non-publishing package assemblies, macOS packaged
  product acceptance, Lite candidate acceptance, and an automatic Community
  Alpha promotion dispatch after every successful `2.x` push.
- `native-community-alpha-promotion.yml` already provides the explicit exact-
  candidate, fresh three-target promotion path. Ordinary development does not
  need to duplicate that release evidence.
- `evaluation-truth.yml` is a bounded five-minute governance/evaluation lane and
  should remain automatic; it protects evidence and roadmap truth cheaply.

## Minimum change that fits the request

1. Reuse the product-control evidence ladder as three named tiers:
   - **Focused** while changing behavior;
   - **Slice** once a complete user-visible vertical is frozen for integration;
   - **Acceptance** only for an explicit 2.x cutover/release candidate.
2. Make Trellis Phase 2.2 and `trellis-check` select and report a tier. The last
   task pass covers the whole task and affected packages, not unrelated release
   acceptance.
3. Keep required three-platform Rust source checks at Slice tier, run portable
   frontend checks once, and gate package/promotion jobs behind the explicit
   Acceptance path.
4. Report successful commands as compact summaries. On failure, show the owning
   tier, command, and minimum focused reproduction rather than streaming every
   successful log.
5. Rebase the master roadmap around a reliable 1.19 replacement sequence. Keep
   accepted evidence traceable, reorder unaccepted work, and move Graph v2 and
   Kernel-dependent expansion behind the 2.0 replacement/cutover path.

No new test runner, compatibility framework, graph renderer, or duplicate
branch-policy document is required.
