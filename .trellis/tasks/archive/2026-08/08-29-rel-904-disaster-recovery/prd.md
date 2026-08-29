# REL-904 disaster recovery

## Goal

Prove that the native project owner recovers the five failures named by
Program Ledger `REL-904` without changing canonical project artifacts, and
close the one observed gap that prevents an explicitly approved reset of a
corrupted, rebuildable Portfolio catalog.

## Background

- Program Ledger v1 defines `REL-904` as disaster recovery for interrupted
  migration, missing index, corrupted derived state, lost registration, and
  partial update. Its accepted dependencies are `REL-902` and `REL-903`.
- `ProjectStateService::{preview_migration_recovery,
  apply_migration_recovery}` already resumes an exact committed migration.
- `ProjectStateService::{preview_register,apply}` already restores a portable
  project manifest to a new or empty private Research Library index.
- `IncrementalPortfolioService::reconcile` already rebuilds a missing private
  Portfolio catalog from registered canonical project artifacts.
- `PortfolioCatalogStore` already replays one interrupted catalog transaction
  at every durable boundary.
- The current `delete-derived-state` preview calls
  `PortfolioCatalogStore::rebuild`; therefore `InvalidPortfolioCatalog` blocks
  the explicit reset that should recover corrupted rebuildable state.

## Requirements

1. Keep migration recovery, registration recovery, missing-catalog rebuild,
   and interrupted catalog transaction replay on their existing native owners.
2. Allow only `delete-derived-state` preview/apply to treat
   `InvalidPortfolioCatalog` as resettable derived state. Other errors,
   including unsafe paths, lock contention, and ambiguous recovery journals,
   remain fail closed.
3. Keep deletion digest-bound to the current Research Library revision and
   explicit `derived-state-write` approval. If a valid catalog appears before
   apply, normal preview revalidation must reject the stale plan.
4. Remove only the private `portfolio-catalog/v1` contents. Do not change the
   Research Library document, portable project manifests, academic artifacts,
   receipts, or registered project roots.
5. After reset, rebuild the Portfolio catalog from canonical registered
   projects and prove the resulting portfolio is identical to a clean rebuild.
6. Reuse existing tests for the already implemented failure modes; rename them
   into the `rel_904` focused filter instead of adding a duplicate umbrella
   suite. Add only the missing lost-registration and corrupt-state checks.
7. Add the executable seven-section REL-904 contract to the existing public
   schema/control Trellis spec without changing public schema IDs.

## Acceptance Criteria

- [ ] `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p
      qiongli-project rel_904 --locked` selects exactly the five roadmap failure
      classes and passes.
- [ ] An exact committed migration resumes after restart through the existing
      preview, approval, and apply owner without copying again.
- [ ] A missing Portfolio catalog rebuilds from canonical registered projects
      while Research Library and project bytes remain unchanged.
- [ ] A corrupted Portfolio catalog cannot be read, but an explicitly approved
      derived-state deletion succeeds; restart rebuild produces the clean
      portfolio and preserves canonical bytes.
- [ ] A portable project whose private registration is absent can be explicitly
      re-registered from its existing manifest without changing project bytes.
- [ ] Every existing durable Portfolio transaction interruption resumes to the
      exact next manifest and removes its transaction journal.
- [ ] Rust format, focused tests, the `qiongli-project` package tests, Clippy,
      public-schema policy, Capability Contract v2, roadmap validation, and
      exact-head Slice CI pass.
- [ ] Acceptance evidence records the exact product commit and CI run before
      Program Ledger `REL-904` becomes `accepted`.

## Out of Scope

- A new backup format, general filesystem repair tool, quarantine UI, schema
  migration framework, or automatic discovery of arbitrary project roots.
- Repairing canonical project artifacts, receipts, unsafe paths, permissions,
  multiple/invalid transaction journals, or future-version documents.
- Candidate packaging, live Host migration, promotion, publication, release
  authorization, `REL-905` ownership/support policy, or 1.x retirement.

## Technical Notes

- Preserve the public CLI/App contract: `delete-derived-state` remains the only
  recovery mutation and still requires its existing preview/apply approval.
- A different corrupt byte sequence need not change the plan digest because
  the approved operation deletes all rebuildable Portfolio state at a fixed
  private path; a transition to valid state does change the preview identity.
- No blocking product, scope, UX, compatibility, or risk decision remains.
