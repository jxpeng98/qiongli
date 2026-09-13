# Repository evidence -- PLT-401--PLT-403

## Selected scope and authority

- The master roadmap names `PLT-401`--`PLT-403` as the current bounded package
  and leaves budget definition to `PLT-407`
  (`docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md:53-60,487-496`).
- Product control repeats the same order and requires Focused checks to remain
  separate from PR, Build, and Acceptance lanes
  (`.trellis/spec/product/control/index.md:39-47,116-169`).

## Existing limits and fail-closed owners

- Research Library: `MAX_LIBRARY_PROJECTS = 512`; document validation rejects
  larger vectors and service registration returns `LibraryFull`
  (`packages/qiongli-native/crates/qiongli-project/src/model.rs:11,357`,
  `packages/qiongli-native/crates/qiongli-project/src/service.rs:1604-1617`).
- Capture: history listing stops after 1,024 directory entries with
  `DocumentTooLarge`
  (`packages/qiongli-native/crates/qiongli-project/src/storage.rs:456-487`).
- Graph: snapshots cap nodes and edges at 4,096 and reject larger documents as
  `InvalidGraphDocument`
  (`packages/qiongli-native/crates/qiongli-project/src/academic_graph.rs:32-44,468-482`).
- Portfolio: the pure builder caps nodes at 16,384, edges at 32,768,
  occurrences at 65,536, and serialized output at 16 MiB
  (`packages/qiongli-native/crates/qiongli-project/src/academic_graph_portfolio.rs:15-18,250-263,400-432`).
- Portable import/export independently caps the package inventory at 1,024
  files (`packages/qiongli-native/crates/qiongli-project/src/portable.rs:32`).

## Existing coverage gap

- Graph's current deterministic large fixture has 200 nodes and 397 edges,
  below its product limit
  (`packages/qiongli-native/crates/qiongli-project/src/academic_graph_index.rs:911-994,1084-1110`).
- Portfolio's current large fixture has 64 projects, 65 nodes, and 64 edges
  (`packages/qiongli-native/crates/qiongli-project/src/academic_graph_portfolio.rs:709-752`).
- The service suite proves three-project lifecycle behavior but has no
  512-project capacity fixture
  (`packages/qiongli-native/crates/qiongli-project/src/service.rs:2140-2190`).

## Feasible minimal implementation

- A crate-root `#[cfg(test)]` module in `qiongli-project` can call the existing
  crate-private document validators and Graph/Portfolio pure builders. This is
  necessary for exact-limit fixtures: semantic Graph input itself is capped at
  2,048 records before the 4,096 snapshot ceilings
  (`packages/qiongli-native/crates/qiongli-project/src/academic_graph.rs:31-35,1387-1421`).
- A separate crate-root `#[cfg(test)]` module in `qiongli` can call the existing
  `pub(crate)` App snapshot and Desktop startup functions without exposing a
  public product API
  (`packages/qiongli-native/apps/qiongli/src/desktop.rs:2152-2180,2311-2344`).
- The workspace has no Criterion or process-memory dependency. Existing crates
  already provide `serde`, canonical JSON, SHA-256, and Unix `rustix`; timing,
  percentile selection, process invocation, and file output can remain test-
  only standard-library code
  (`packages/qiongli-native/Cargo.toml:26-48`,
  `packages/qiongli-native/crates/qiongli-project/Cargo.toml:12-28`).
- Two distinct receipt parts avoid an aggregator: one filtered workspace
  `cargo test` command can run both ignored tests, and each writes a unique
  filename under the same requested output directory.

## Three-target execution path

- `native-ci.yml` already owns a Linux/macOS/Windows Rust foundation matrix and
  runs it for explicit `workflow_dispatch`
  (`.github/workflows/native-ci.yml:58-91`).
- Test and upload steps guarded by `github.event_name == 'workflow_dispatch'`
  add no pull-request measurement cost.
- On a feature-branch dispatch, existing package, packaged-product, candidate,
  and promotion jobs remain skipped because each requires
  `github.ref == 'refs/heads/2.x'`
  (`.github/workflows/native-ci.yml:200-203,457-460,630-633,696-710`).
- Therefore the smallest persistent change is two test-only modules plus two
  conditional steps in the existing workflow. No new workflow, dependency,
  public command, or recurring gate is required.

## Risks retained explicitly

- Hosted-runner latency and RSS values are noisy; receipts must remain labelled
  observations and cannot define an SLO in this task.
- Exact Portfolio node, edge, and occurrence maxima are independent stress
  cases. Combining every ceiling into one corpus is unnecessary and can make a
  valid limit unreachable through an earlier byte/count guard.
- The product-limit run may be slow; keep it ignored outside manual dispatch and
  fail rather than silently reducing samples or fixture sizes.
