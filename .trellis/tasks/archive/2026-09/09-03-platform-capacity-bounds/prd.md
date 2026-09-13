# Establish platform capacity and bounds baseline

## Goal

Establish one repeatable, opt-in platform baseline for `PLT-401`--`PLT-403`
without making daily development or ordinary pull-request validation heavier.

The baseline must show how the current native product behaves at small,
medium, and documented product-limit workloads before `PLT-407` defines any
performance budgets.

## Background

- The master roadmap selects `PLT-401`--`PLT-403` as the current bounded work
  package and explicitly leaves latency and memory budgets to `PLT-407`.
- Existing native owners enforce 512 Research Library projects, 1,024 Capture
  documents, 4,096 Graph nodes and edges, and Portfolio bounds of 16,384 nodes,
  32,768 edges, and 65,536 occurrences.
- Existing large fixtures stop below the documented product limits: Graph
  index coverage uses 200 nodes and 397 edges, while Portfolio coverage uses
  64 projects.
- `ProjectStateService`, `AcademicGraphService`,
  `AcademicGraphIndexService`, `AcademicGraphPortfolioService`, and
  `IncrementalPortfolioService` already own the measured project operations.
  Desktop startup and snapshot construction already have testable native
  entrypoints in `apps/qiongli/src/desktop.rs`.
- The workspace has no benchmark framework or cross-platform process-memory
  dependency. This task must use existing code and the standard library.

## Requirements

### R1 -- Deterministic capacity profiles

- Generate small, medium, and product-limit fixtures at runtime in isolated
  temporary roots; do not check in thousands of generated project or Capture
  files.
- Keep each subsystem workload independent so the product-limit profile does
  not create an unnecessary Cartesian product of every maximum.
- Derive fixture identity from deterministic content and assert the documented
  512-project, 1,024-Capture, Graph, and Portfolio limits against their current
  native owners.

### R2 -- Opt-in measurement receipt

- Provide one explicit release-mode command that records warm-up plus at least
  20 samples and reports nearest-rank P50/P95 values.
- Record native startup validation, App snapshot construction, project refresh,
  Capture load, Graph build and query, Portfolio rebuild, portable export and
  import, resident memory, and serialized IPC payload bytes.
- Emit one machine-readable, host-labelled receipt set with source commit,
  operating system, architecture, Rust version, profile sizes, sample count,
  units, and fixture identity. Do not include user paths, host names,
  credentials, or project content.
- Measurements are observations only. They must not pass or fail against an
  invented latency, memory, or payload budget.

### R3 -- Explicit fail-closed boundaries

- Add focused tests for each owning boundary at its exact limit and at
  `limit + 1`.
- Preserve the current stable native error class for an over-limit Library,
  Capture set, Graph, or Portfolio; do not silently truncate stored state.
- Keep the ordinary focused checks cheap. The full capacity measurement remains
  opt-in and is not added to daily or pull-request gates.

### R4 -- No new product surface

- Keep fixture generation and percentile calculation in test/development code.
- Add no runtime dependency, public CLI command, App screen, schema migration,
  background service, automatic workflow, or release gate.
- Reuse current service entrypoints and local build guidance.

### R5 -- One-time Tier 1 evidence

- Reuse the existing Native CI Linux, macOS, and Windows foundation matrix.
- Run the capacity command only for an explicit `workflow_dispatch`; ordinary
  pull requests must compile the harness but skip its heavy measurements.
- Upload one receipt set per target from the same exact source. A task closeout
  requires all three sets and records the source and run identity.
- On a feature-branch dispatch, retain the existing branch guards that skip
  package assembly, candidate acceptance, promotion, signing, and publication.

## Acceptance Criteria

- [ ] One documented command regenerates a valid machine-readable receipt set
      from isolated deterministic fixtures.
- [ ] Each target receipt set contains P50/P95 timing for every operation named in `R2`,
      P50/P95 resident-memory observations, and P50/P95 IPC payload bytes.
- [ ] Repeated fixture generation produces the same fixture identity while
      measurement values remain host-labelled observations.
- [ ] Exact-limit fixtures succeed for Library, Capture, Graph, and Portfolio
      owners; their `limit + 1` counterparts fail with the expected native
      error class.
- [ ] The heavy harness is opt-in; normal package tests run only the focused
      boundary checks.
- [ ] One exact-source manual Native CI run uploads matching Linux, macOS, and
      Windows receipt sets while package, candidate, signing, promotion, and
      publication jobs remain skipped on the feature branch.
- [ ] Rust formatting, the affected native tests, task validation, and the
      normal PR Slice pass without a release, package promotion, or publication
      action.

## Out of Scope

- Defining or enforcing performance budgets (`PLT-407`).
- Real IPC, stale-preview recovery, crash, restart, locking, or concurrency
  journeys (`PLT-404`--`PLT-406` and `PLT-408`).
- Performance optimization, pagination, delta snapshots, or new storage
  formats before the measurements identify a demonstrated blocker.
- Running the heavy capacity harness on every commit or release.
- Tagging, publishing, signing, or changing release authority.

## Key Decisions

- Collect Linux, macOS, and Windows observations once before closeout, using an
  explicit manual run rather than a recurring gate.
- Keep fixture families independent and generated on demand; do not persist a
  giant cross-product corpus.
- Use test-only Rust plus the standard library and already-installed crates;
  do not add Criterion, a process-monitoring dependency, a public command, or
  another CI workflow.
- Keep the three roadmap IDs in one task because measurements and rejection
  cases consume the same fixtures and one exact-source receipt run.
