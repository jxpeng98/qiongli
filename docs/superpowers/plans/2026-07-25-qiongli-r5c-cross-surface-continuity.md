# Qiongli R5C Cross-Surface Continuity Plan

Status: planned

Date: July 25, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: R5A completion commit `12388b7a`

Roadmap:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

## Naming

The acceleration design already reserves `R5B` for legacy Python and Node
source retirement after `v2.0.0-beta.1` acceptance. This next pre-Beta stage is
therefore named `R5C`, for cross-surface continuity. R5C must prove that the
native product owns the supported behavior before R5B is allowed to delete
superseded source.

## Goal

Make academic work captured from Codex, Claude Code, the CLI, repository
delivery, and portable files survive offline use, restart, replay, and
cross-project navigation without turning Qiongli into a model client.

Qiongli remains the native shell and control plane:

- it installs and verifies the Qiongli CLI, Skills, Plugins, and orchestrator;
- it owns project identity, capture delivery state, reconciliation, and
  derived portfolio indexes;
- Codex, Claude Code, and supported CLIs remain the execution surfaces; and
- no R5C path stores provider credentials or directly invokes a model API.

## Starting point

R5A provides native project identity, migration, recovery, reconciliation,
rollback, Research Library registration, and deterministic graph rebuilds.
The current native project crate also provides capture normalization,
content-derived capture identities, project intake, consolidation receipts,
capture and repository inbox projections, coverage states, artifact-change
inspection, graph comparison, and an exact-identity portfolio snapshot.

The remaining continuity gaps are:

1. delivery does not yet have one durable Outbox/acknowledgement ledger for
   offline queueing, resend, and restart recovery;
2. duplicate, stale, divergent, and unbound captures can be reported, but the
   full explicit assignment and conflict-resolution workflow is incomplete;
3. portfolio construction rebuilds the ready project set as one snapshot and
   lacks a persistent incremental query boundary, saved views, and bounded
   cancellation;
4. timeline, source overlay, capture/decision lineage, and merge-resolution
   history are not yet one coherent cross-project view; and
5. the clean-commit packaged macOS client-install journey has not yet been
   rerun after the R5A and embedded `2.0.0-alpha.2` fixes.

## Product sequence

```text
host or portable surface creates a bounded capture
  -> persist a content-addressed delivery envelope
  -> queue while the destination is unavailable
  -> deliver or export without changing the envelope identity
  -> acknowledge the exact envelope and destination revision
  -> reconcile pending, duplicate, stale, divergent, or unbound state
  -> apply only an explicitly approved academic mutation
  -> update the affected derived project and portfolio indexes
  -> rebuild the same projections after restart
```

Delivery metadata records transport truth. Canonical academic state remains in
the registered project. A queue, acknowledgement, index, saved view, or UI
cache must never become a second authority for research meaning.

## Batches

### C0 — Clean-head macOS package and installation baseline

Purpose: establish that the committed R5A head produces the same complete App
that was tested from source and installs the current embedded integration.

- build the release-profile macOS App through the documented single-command
  path;
- run the product-controlled, non-publishing acceptance from a clean commit;
- verify the App, copied CLI, embedded content pack, and Plugin manifests all
  report `2.0.0-alpha.2`;
- use an isolated home to install and discover the current Codex or Claude Code
  Plugin and its Skills, restart the client, and verify the installed source is
  the receipt-owned 2.x location;
- confirm that a detected 1.x location is offered only as a migration source,
  never as the current installation; and
- record a path-redacted local receipt without claiming signing,
  notarization, publication, or another operating system.

Gate:

- a clean committed checkout produces a complete App rather than an empty
  shell;
- the App never reports or installs the previous alpha;
- install, restart, discovery, and removal affect only the isolated home; and
- failure leaves the previous usable isolated-client state recoverable.

### C1 — Durable capture delivery ledger

Purpose: add the minimum native persistence needed for offline and restart-safe
delivery.

- define a bounded content-addressed envelope around the existing normalized
  capture, project binding, source surface, delivery class, and payload digest;
- persist Outbox entries and acknowledgements through private, atomic,
  versioned Rust storage;
- model `queued`, `delivering`, `delivered`, `acknowledged`,
  `retry-required`, `conflicted`, and `cancelled` without using a transient
  process state as authority;
- bind acknowledgement to envelope identity, destination project identity,
  accepted capture identity, and resulting project revision;
- make retry and replay idempotent across process restart; and
- expose typed service and CLI inspect/retry/cancel operations before adding UI
  mutation controls.

Gate:

- process loss at every write boundary reconstructs one causal state;
- replay never creates a second capture or a second acknowledgement;
- wrong-project or wrong-revision acknowledgements are rejected; and
- queue recovery does not invoke a provider, model, Git, Python, or Node.

### C2 — Assignment and conflict reconciliation

Purpose: turn the existing reported states into explicit, bounded user
decisions.

- add digest-bound preview/apply for assigning an unbound capture to one
  selected registered project;
- compare the capture base revision with the current project revision and
  produce item-scoped decisions, evidence, contradiction, and next-action
  differences;
- add explicit accept-current, accept-capture, retain-both, and reject
  dispositions only where the artifact type can support them without silent
  information loss;
- persist resolution receipts and expose the resulting capture/decision
  lineage;
- reconcile repository-delivered artifact changes without requiring Git or
  inferring a human author; and
- retain unresolved inputs until the user explicitly rejects or exports them.

Gate:

- unbound assignment cannot target an unregistered, archived, or changed
  project;
- stale or divergent capture state is never silently overwritten;
- every accepted academic mutation has an exact input, approval digest,
  resulting revision, and resolution receipt; and
- cancellation or failure retains the original envelope and project state.

### C3 — Incremental portfolio continuity

Purpose: make cross-project graph and search useful without weakening exact
identity semantics.

- persist a rebuildable derived catalog keyed by Library revision, project
  semantic revision, and graph projection identity;
- update only affected project contributions after registration, archive,
  restore, migration rollback, capture consolidation, or explicit refresh;
- remove only the derived contribution owned by an archived, unregistered, or
  rolled-back project;
- add bounded portfolio queries for project stage, evidence gap,
  contradiction, manuscript section, shared source/concept/method, transport,
  capture state, and lineage;
- add a semantic activity timeline and revision/merge-resolution history;
- support cancellation and deterministic full rebuild from registered
  canonical project artifacts; and
- preserve exact canonical identities and reviewed lineage: similar labels are
  never merged automatically.

Gate:

- an incremental update equals a clean full rebuild for the same Library
  revision;
- deleting every derived index changes no canonical project artifact;
- archive, restore, unregister, migration rollback, and conflict resolution
  leave no stale project contribution;
- a changed Library or project revision aborts a query/update instead of
  publishing a mixed snapshot; and
- bounded large-library fixtures meet explicit memory, result, and cancellation
  limits.

### C4 — Desktop and App API continuity experience

Purpose: expose the native authority without moving business logic into
Svelte.

- extend the strict App API for delivery status, retry, acknowledgement,
  assignment, conflict preview/apply, timeline, and portfolio query;
- add Inbox, Outbox, Conflict, Coverage, Timeline, and Portfolio states with
  truthful `unknown`, `stale`, and `conflicted` presentation;
- keep absolute project paths and raw client configuration behind opaque
  native selections;
- provide item-scoped explanations before confirmation and recovery actions
  after restart;
- add accessible keyboard, focus, reduced-motion, narrow-window, Chinese, and
  English component fixtures; and
- ensure cancellation reaches the Rust operation boundary rather than only
  hiding UI progress.

Gate:

- CLI and Desktop present the same causal and approval states;
- the UI cannot acknowledge, assign, resolve, or delete without a current
  native preview;
- `unknown` is never displayed as observed or connected; and
- private paths, credentials, sessions, prompts, and transcripts do not cross
  the App API.

### C5 — Isolated packaged acceptance

Purpose: qualify the complete R5C path on macOS before expanding to the Tier 1
distribution matrix.

- use at least three disposable registered projects and exact shared source,
  concept, method, and reviewed-lineage fixtures;
- queue captures offline, restart, replay, acknowledge, duplicate, diverge,
  assign, resolve, archive, restore, and rebuild;
- remove all derived indexes and prove deterministic reconstruction;
- restart the packaged App and supported local client from an isolated home;
- verify Plugin/Skills discovery and host-driven execution without a Qiongli
  model backend; and
- record source commit, package/content identities, bounded fixture counts,
  state transitions, and path-redacted outcomes.

Gate:

- all C0 through C4 source and interaction gates pass from one committed head;
- the packaged App and copied CLI agree after restart;
- no fixture writes outside the isolated home and disposable project roots;
- the current 2.x Plugin and Skills remain discoverable after client restart;
  and
- the receipt makes no Windows, Linux, cloud relay, public distribution,
  signing, notarization, or Beta claim.

## Implementation order

1. add delivery envelope, storage, ledger validation, and restart tests to
   `qiongli-project`;
2. add project-service and CLI inspect/mutation boundaries;
3. add assignment, conflict, and resolution receipt types;
4. add incremental portfolio catalog and query services;
5. extend Desktop service, Tauri adapter, and strict App API;
6. implement localized Svelte views and component fixtures; and
7. run the focused source gates, clean-head package gate, then manual macOS
   acceptance.

Each implementation batch should be committed separately. A schema or App API
version change must include parser rejection tests and deterministic fixtures
in the same commit.

## Validation policy

R5C uses focused correctness and boundary checks:

- affected Rust formatting, tests, compile, and warnings-as-errors checks;
- App API parser, client, and type checks;
- Svelte component, accessibility fixture, and production-build checks;
- deterministic restart, replay, reconciliation, incremental/full rebuild,
  cancellation, and rollback fixtures; and
- one isolated, non-publishing packaged macOS interaction receipt.

This is not a request for a broader cybersecurity audit. Add security-specific
tests only for changed trust boundaries such as path containment, envelope
identity, acknowledgement binding, atomic persistence, and destructive
ownership.

## Non-goals

R5C does not:

- add a Qiongli model chat surface or direct model backend;
- store provider credentials or replace Codex/Claude Code as execution hosts;
- add an unauthenticated or nominally authenticated cloud relay;
- automatically scan the user's home directory for projects;
- run or restore a Qiongli 1.x runtime;
- delete legacy Python or Node source reserved for post-Beta R5B;
- publish Homebrew, Scoop, WinGet, Marketplace, or release assets;
- claim Windows/Linux packaged interaction acceptance;
- claim Developer ID, notarization, Authenticode, Beta, or Stable readiness; or
- merge projects, concepts, methods, papers, or sources from fuzzy label
  similarity.

## R5C completion gate

R5C is complete when:

1. offline queueing, process restart, retry, replay, acknowledgement, and
   cancellation preserve one delivery identity;
2. duplicate, stale, divergent, and unbound captures have explicit,
   receipt-backed outcomes;
3. incremental portfolio state equals a deterministic full rebuild and can be
   deleted without losing academic state;
4. portfolio query, timeline, lineage, coverage, and conflict views remain
   usable on bounded multi-project fixtures;
5. the clean committed macOS package installs and discovers only the current
   2.x Plugin and Skills in an isolated home;
6. CLI, packaged App, and supported host surfaces report the same causal
   state after restart;
7. no R5C product path invokes a model provider, Python, Node, or a 1.x runtime;
   and
8. all nonclaims remain explicit, leaving Tier 1 distribution and Beta
   qualification for the following R5 stage.
