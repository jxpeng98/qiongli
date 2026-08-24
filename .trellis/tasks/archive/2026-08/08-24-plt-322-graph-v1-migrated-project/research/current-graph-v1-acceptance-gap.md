# Current Graph v1 acceptance gap

Date: 2026-08-24

## Existing owners to reuse

- Project migration: `packages/qiongli-native/crates/qiongli-project/src/migration.rs`
- Native/Desktop migration flow:
  `packages/qiongli-native/apps/qiongli/src/desktop.rs`
- Graph projection and canonical extractors:
  `packages/qiongli-native/crates/qiongli-project/src/academic_graph.rs` and
  `academic_graph_extract.rs`
- Readiness and bounded query:
  `academic_graph_readiness.rs` and `academic_graph_index.rs`
- App contract: `packages/qiongli-app-api/src/schema.ts`
- Desktop presentation:
  `packages/qiongli-desktop/src/lib/features/academic-graph/`

## Confirmed implemented behavior

- Migration is preview/apply, copy-based, digest-bound, source-retaining, and
  excludes recognized private entries.
- Native migration confirmation already runs two Graph rebuild passes and
  exposes `AppProjectMigrationQualification::deterministic_rebuild`.
- Graph v1 extracts structured scholarly records from canonical artifacts and
  preserves artifact paths/source anchors.
- Query, explanatory path, readiness, layout, Cytoscape rendering, search,
  focus history, and artifact inspection all have focused tests.
- Structural-only and relationless readiness cases are already distinguished.

## Evidence that does not close PLT-322

- `copied_binary_round_trips_portable_and_legacy_projects_without_runtime`
  migrates a one-line legacy research-state file and does not prove semantic
  Graph continuity.
- `project_desktop_state_migrates_recovers_and_qualifies_graph_rebuilds`
  proves deterministic migration mechanics but only asserts that readiness is
  not stale.
- Packaged-product acceptance creates deterministic semantic fixtures and
  proves App/CLI/MCP parity, but its graph facts are synthetic and partly
  authored in `graph/semantic_links.jsonl`.
- The August 18 continuity task fixed classification and workflow guidance; it
  explicitly did not provide representative migrated-project evidence.

## Minimum implementation direction

Add one acceptance coordinator around the existing migration, Graph, App API,
and Desktop contracts. It should accept a user-selected source path, operate on
an isolated copy/destination, emit only redacted evidence, and fail closed if
any required semantic, deterministic, UI, or negative-control assertion is not
executed. Do not add another projector, renderer, store, or migration path.

The repository contains no approved real 1.19 project. The acceptance gate
therefore cannot be finalized or moved to `accepted` until the user names a
source path and authorizes its local inspection.
