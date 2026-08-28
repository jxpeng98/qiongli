# REL-903 forward-version immutability

## Goal

Prove at Slice tier that the Qiongli 2 native runtime refuses persisted project
and global-state documents whose schema version is newer than the running
binary, without changing their bytes or silently repairing/downgrading them.

## Background

- Program Ledger v1 defines `REL-903` as “Prove forward-version files fail
  closed and remain unmodified” and records `REL-901` as its accepted dependency
  (`docs/superpowers/roadmaps/qiongli-program-ledger-v1.json:919`).
- ADR 0204 requires unknown/future mutable documents to fail closed without a
  write (`docs/architecture/decisions/0204-versioned-state-and-secret-storage.md:52`).
- The frozen policy names one persisted-state rule:
  `future_persisted_state = fail-closed-unmodified`
  (`tooling/architecture/public-schema-policy.json:19`).
- `GlobalSettingsStore::load` already returns `ConfigError::UnsupportedSchema`
  before mutation, while `ProjectStateService` already validates both the
  private research-library index and portable project manifest
  (`packages/qiongli-native/crates/qiongli-config/src/document.rs:454`,
  `packages/qiongli-native/crates/qiongli-project/src/model.rs:136`, and
  `packages/qiongli-native/crates/qiongli-project/src/model.rs:354`).
- The config crate already has an isolated future-settings immutability test;
  the missing release proof is one named native scenario that exercises the
  global and project owners together.

## Requirements

1. Add one focused `rel_903` native test that creates valid current state only
   through `GlobalSettingsStore` and `ProjectStateService`.
2. Exercise the three entry documents that own this compatibility boundary:
   global `settings.json`, private `research-library/library.json`, and portable
   `context/project_manifest.json`.
3. For each document, change only `schema_version` from the supported value to
   the next value, invoke the normal read/inspection owner, and assert the
   stable fail-closed result.
4. Compare the exact future-version bytes before and after every rejected read.
   No repair, downgrade, migration, or replacement may occur.
5. Reuse existing types, stores, temporary-home fixtures, and error/health
   values. Do not add a second schema registry, migration abstraction, or
   production compatibility branch.
6. Add the executable REL-903 contract to the existing public-schema Trellis
   spec, then bind acceptance evidence to the exact product commit and Native
   CI run before changing the Program Ledger row to `accepted`.

## Acceptance Criteria

- [ ] Future global settings return
      `ConfigError::UnsupportedSchema { observed: Some(2) }` and preserve exact
      `settings.json` bytes.
- [ ] A future research-library index makes `ProjectStateService::snapshot`
      return `ProjectError::InvalidLibraryDocument` and preserves exact
      `library.json` bytes.
- [ ] A future portable manifest is reported as
      `ProjectHealth::InspectionBlocked`; a mutating preview such as
      `preview_refresh` returns `ProjectError::InvalidProjectDocument`; the
      manifest bytes remain exact.
- [ ] The test restores the supported library bytes between the library and
      manifest cases so each failure is independently attributable.
- [ ] No production behavior changes unless the focused test exposes a concrete
      defect in a shared owner; any such fix must stay at that owner.
- [ ] Focused test, affected native Slice checks, public-schema policy tests,
      roadmap validation, and exact-head required CI all pass.
- [ ] Acceptance evidence records the exact product commit and CI run before
      `REL-903` becomes `accepted` in Program Ledger v1.

## Out of Scope

- N-2 migration/rollback, already accepted by `REL-902`.
- Interrupted migration, missing index, corrupt derived state, lost
  registration, and partial-update recovery, owned by `REL-904`.
- Exhaustive mutation of every derived receipt/cache/artifact schema; REL-903
  proves the frozen project/global entry-document boundary.
- New schema versions, downgrade support, automatic repair, candidate
  packaging, promotion, publication, or release authorization.

## Technical Notes

- Prefer the existing test module in
  `packages/qiongli-native/apps/qiongli/src/legacy_migration_cli.rs`, beside the
  REL-902 persisted-state proof, so one isolated native scenario can call both
  owners without adding a fixture file.
- Expected production diff: none. Expected executable diff: one test plus the
  existing seven-section Trellis scenario and closeout evidence.
