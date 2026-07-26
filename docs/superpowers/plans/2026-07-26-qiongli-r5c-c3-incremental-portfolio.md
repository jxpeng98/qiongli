# Qiongli R5C C3 Incremental Portfolio Execution Plan

Status: complete — C3.1 through C3.5 are accepted; C4.1 is next

Date: July 26, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: C2 completion commit `8b156970`

Parent plan:
`docs/superpowers/plans/2026-07-25-qiongli-r5c-cross-surface-continuity.md`

## Accepted implementation

C3 was delivered as the five independently testable source commits frozen by
this plan:

1. `8e9913cf` persists strict derived project contributions;
2. `411bd855` reconciles incremental catalog state and proves clean/full
   equivalence;
3. `7caa9d28` adds bounded, content-bound portfolio and lineage queries;
4. `89f77523` projects deterministic semantic activity and revision history;
   and
5. `863bfbc9` exposes path-redacted CLI maintenance, query, timeline, doctor,
   cancellation, restart, corruption, and reconstruction qualification.

The accepted implementation keeps canonical project artifacts, the Research
Library, delivery records, acknowledgements, assignment receipts,
consolidation receipts, and resolution receipts authoritative. All catalog and
contribution documents can be removed and deterministically reconstructed.
The private store may retain empty owner-private lock scaffolding so concurrent
maintenance cannot race with root deletion; this scaffolding contains no
catalog or academic data.

Final focused gates passed with 154 `qiongli-project` tests, 115 App-library
tests, warnings-as-errors Clippy, the complete Rust workspace all-target
check, Rust formatting, and `git diff --check`. A copied binary also passed the
empty-`PATH`, outside-checkout restart journey covering empty and multi-project
state, reconcile, query, timeline, doctor, derived-state deletion, full
rebuild, corruption failure, and path redaction. No broad cybersecurity scan
was run.

## Outcome

Replace the current full, process-local portfolio rebuild boundary with a
private, rebuildable, incrementally reconciled catalog and bounded query
service. The catalog must make cross-project graph, capture, and resolution
lineage useful after restart without becoming a second authority for academic
meaning.

C3 remains part of Qiongli's native control plane. It does not add a model
backend, provider credentials, fuzzy entity merging, Git authorship inference,
or a portable index.

## Current baseline

The Rust implementation already provides the source primitives:

- `AcademicGraphService` deterministically rebuilds one project projection
  from registered canonical artifacts;
- `AcademicGraphPortfolioService::rebuild` snapshots the Library, rebuilds
  every ready project, confirms that the Library did not change, and combines
  only exact global paper, concept, and method identities;
- portfolio nodes and edges preserve project, projection, graph-node,
  artifact, and source-anchor lineage;
- capture delivery, assignment, consolidation, and resolution expose durable
  opaque identities and exact receipts; and
- archive, restore, unregister, refresh, migration rollback, consolidation,
  and resolution change authoritative Library or project revisions.

The remaining gaps are explicit:

1. every portfolio request rebuilds all ready projects;
2. no restart-persistent contribution catalog exists;
3. no bounded query contract spans portfolio, capture, and resolution lineage;
4. no coherent semantic activity timeline exists; and
5. rebuild has no cancellation or incremental/full equivalence qualification.

## Frozen authority rules

- Registered project artifacts, the Research Library, delivery records,
  acknowledgements, assignment receipts, consolidation receipts, and
  resolution receipts remain authoritative.
- The portfolio catalog is private derived state. It is excluded from portable
  packages and can be deleted without losing academic data.
- A contribution is keyed by exact
  `(projectId, semanticRevision, projectionId)`. A different tuple replaces
  only that project's derived contribution.
- A catalog snapshot is keyed by the exact Library revision plus the sorted
  contribution identities used to build it.
- Global nodes merge only when node type, global identity scope, and canonical
  ID are exactly equal. Similar labels never merge automatically.
- Queries bind to one catalog identity and fail on Library or contribution
  drift. They never publish a mixed-revision result.
- Catalog files contain no absolute project root, credential, prompt,
  transcript, host session, provider response, or inferred author.
- Interruption leaves the previous valid catalog or recoverable staging
  evidence, never a partially authoritative next catalog.

## Private derived layout

C3.1 freezes a versioned layout below the existing Qiongli state root:

```text
portfolio-catalog/
  v1/
    contributions/
      <prj_id>.json
    catalog.json
    transactions/
      <transaction_id>.json
    .catalog.lock
```

Filenames derive only from validated opaque identities. Documents are strict,
bounded canonical JSON with unknown fields rejected. Storage reuses the
owner-private, link-safe, bounded-read, atomic-write, directory-sync, and lock
patterns already used by `qiongli-project`.

The catalog is an acceleration boundary, not a project artifact. Nothing in
this layout enters portable export inventory.

## Batches

### C3.1 — Catalog contracts and private storage

Add strict versioned contracts for:

- `PortfolioContributionV1`, binding project identity, lifecycle, health,
  semantic revision and digest, projection ID, contribution digest, bounded
  counts, and the exact-identity contribution body;
- `PortfolioCatalogManifestV1`, binding Library revision, sorted contribution
  identities, catalog identity, generation, and creation time;
- `PortfolioCatalogSnapshotV1`, exposing only path-redacted derived status;
  and
- a recoverable transaction that publishes changed contributions before
  atomically replacing the manifest.

Storage behavior:

- contribution insert or replacement accepts exact replay only;
- one transaction replaces a bounded project set, removes a bounded stale set,
  and publishes one next manifest;
- restart recovery completes an exact staged transaction or preserves the
  prior valid manifest;
- missing, changed, duplicate, or extra contributions make the manifest
  invalid and rebuildable; and
- corruption, oversized documents, links, broadened permissions, unknown
  files, stale generations, and lock contention are covered.

C3.1 does not add queries, lifecycle hooks, Desktop work, or a CLI mutation.

Acceptance requires exact insert, replacement, removal, replay, reopen, and
interruption fixtures; deletion leaving every project and Library byte
unchanged; project tests; formatting; warnings-as-errors Clippy; workspace
check; and `git diff --check`. No broad cybersecurity scan is run.

### C3.2 — Incremental reconciliation and full-rebuild equivalence

Add `IncrementalPortfolioService` over `ProjectStateService`,
`AcademicGraphService`, and the C3.1 store.

One reconciliation:

1. snapshots and validates the Library;
2. compares each project with its stored contribution key;
3. rebuilds only a missing or changed ready contribution;
4. removes only contributions no longer owned by an included active project;
5. combines the exact contribution set into the next portfolio;
6. confirms the Library and every included project revision still match; and
7. publishes one recoverable catalog transaction.

A clean full rebuild uses the same contribution builder. Incremental and full
output must be byte-equivalent for the same authoritative state.

Fixtures cover registration, archive, restore, unregister, refresh, migration
import and rollback, consolidation, and resolution. A successful canonical
mutation may leave derived state stale until reconciliation, but stale state
is never returned as current.

### C3.3 — Bounded portfolio and lineage queries

Freeze a strict query document bound to one catalog ID. Filters cover:

- project ID and stage;
- evidence gap or contradiction presence;
- manuscript section;
- exact shared paper/source, concept, or method identity;
- capture source and delivery class;
- current delivery state and assignment outcome;
- consolidation or resolution lineage identity; and
- bounded text matching over derived labels without entity merging.

Results use deterministic ordering, explicit truncation, and closed limits for
nodes, edges, projects, events, and bytes. Any cursor is content-addressed to
the exact catalog and filter document rather than a process-local offset.

The query service joins only exact durable identities. It does not inspect
Git, raw sessions, client databases, or model output.

### C3.4 — Semantic activity timeline and revision history

Build deterministic derived events from:

- registration and lifecycle revision evidence;
- accepted captures and consolidation receipts;
- delivery acknowledgement, retry, conflict, and cancellation;
- assignment receipts and source-to-child lineage; and
- item-scoped resolution receipts and resulting revisions.

Each event has a content-addressed ID, exact timestamp source, project IDs,
opaque lineage IDs, event kind, and revision transition. Ordering is timestamp
then event ID. No event guesses a user, model, session, or author.

Expose bounded project and portfolio timelines plus revision and
merge-resolution history. Deleting this projection changes no receipt.

### C3.5 — CLI, cancellation, restart, and scale qualification

Add path-redacted commands for:

- catalog status, reconcile, full rebuild, and delete-derived-state preview;
- bounded portfolio query;
- project and portfolio timeline; and
- deterministic doctor comparison between incremental and clean rebuild.

Catalog mutations use digest-bound preview/apply and explicit derived-state
approval. Delete removes only validated C3 catalog and contribution documents;
empty private lock scaffolding may remain to preserve safe concurrent locking.

A cancellation token is checked between project rebuilds and bounded
node/edge/event batches. Cancellation publishes no partial catalog.

Copied-binary restart fixtures cover empty and multi-project catalogs,
one-project incremental refresh, lifecycle cleanup, consolidation and
resolution replacement, stale revisions, exact replay, lock contention,
interruption, corruption, deletion and reconstruction, bounded large-library
queries, and cancellation.

## File ownership

Expected primary files:

- new `qiongli-project` catalog contract, storage, reconciliation, query, and
  timeline modules;
- `qiongli-project/src/lib.rs` for the typed boundary;
- `apps/qiongli/src/project_cli.rs` and focused portfolio CLI modules;
- `apps/qiongli/tests/cli.rs` for copied-binary restart acceptance; and
- this plan, the parent plan, and the roadmap for acceptance status.

`academic_graph_portfolio.rs` remains the exact combination authority. C3 may
extract a reusable contribution builder, but must not add a second
global-identity merge implementation.

`portable.rs` changes only to assert that private catalog state stays excluded.
Canonical project mutation transactions do not gain catalog writes.

## Commit sequence

1. `feat(portfolio): persist derived project contributions`
2. `feat(portfolio): reconcile incremental catalog state`
3. `feat(portfolio): add bounded lineage queries`
4. `feat(portfolio): project semantic activity history`
5. `feat(cli): qualify incremental portfolio recovery`
6. `docs(roadmap): accept incremental portfolio continuity`

Each commit is independently testable and preserves the prior authoritative
project behavior.

## C3 completion gate

C3 is complete because:

1. [x] incremental reconcile is byte-equivalent to a clean full rebuild for the
   same Library revision;
2. [x] every lifecycle and accepted academic mutation leaves no stale included
   contribution after reconciliation;
3. [x] deleting all C3 catalog documents changes no canonical project or
   receipt;
4. [x] changed Library or project evidence aborts instead of publishing a mixed
   catalog;
5. [x] portfolio, lineage, and timeline queries are bounded, deterministic, and
   path-redacted;
6. [x] cancellation leaves the previous valid catalog usable;
7. [x] copied-binary restart and corruption fixtures pass outside the checkout
   with an empty `PATH`; and
8. [x] all C3 source gates pass without a broad cybersecurity scan.
