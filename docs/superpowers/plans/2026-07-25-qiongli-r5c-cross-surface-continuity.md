# Qiongli R5C Cross-Surface Continuity Plan

Status: in progress — C0 and C1 complete; C2.1 is the next implementation batch

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

At plan creation, the remaining continuity gaps were:

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

## Execution status

### C0 accepted on July 26, 2026

C0 is complete at product source commit
`5bd8606c9e6cca31c5321e1b636742b488497acf`.

- `pnpm desktop:macos:acceptance --diagnostics` built the complete embedded
  Svelte App and passed the non-publishing packaged-product lifecycle.
- The receipt at
  `dist/macos-acceptance/current/qiongli-packaged-product-acceptance.receipt.json`
  is bound to the exact source commit and records every required C0 check as
  `true`, including the isolated 1.x migration fixture.
- The packaged App, copied CLI, embedded content, Codex Plugin, and Claude Code
  Plugin report `2.0.0-alpha.2`.
- Manual acceptance used only
  `dist/macos-acceptance/current/manual-home`: the UI previewed and confirmed a
  receipt-owned batch install, reported `packaged-product-batch-applied`, and
  rediscovered the managed 2.x installation after App restart.
- The installed Codex and Claude Code sources use only `qiongli-next`; no
  `1.19.0` metadata or legacy Plugin/standalone-Skill root was present.
- Host activation and live MCP attachment remain correctly reported as
  client-owned or not directly observable. C0 does not claim a live Codex or
  Claude Code session, publication, notarization, or another platform.

The C0 fixes were committed independently:

1. `86e43b21` upgrades recognized legacy Claude marketplaces in place;
2. `cb8ee882` aligns canonical embedded workflow metadata to
   `2.0.0-alpha.2`;
3. `6c1df166` covers parallel recognized legacy marketplaces;
4. `4d55ef1c` adds opt-in bounded acceptance diagnostics; and
5. `5bd8606c` permits verified 2.x additions to shared legacy containers while
   continuing to reject legacy item drift.

### C1.1 accepted on July 26, 2026

C1.1 is complete at
`cf557e851ed8fee349524431c35955d555991941`.

- `qiongli-project::capture_delivery` defines strict, versioned, bounded
  envelope, delivery-record, transition, and acknowledgement contracts.
- Envelope and acknowledgement identities are derived from canonical JSON;
  the embedded capture digest, source, delivery class, optional destination,
  revision, and timestamps are revalidated after parsing.
- Delivery state is one bounded causal chain across `queued`, `delivering`,
  `delivered`, `acknowledged`, `retry-required`, `conflicted`, and
  `cancelled`. Each legal transition has a closed reason vocabulary.
- Unknown fields, duplicate keys, oversized documents, malformed identities,
  impossible histories, invalid acknowledgement binding, and path-shaped
  reason data fail closed. Debug output redacts the embedded capture.
- Focused validation passed 95 `qiongli-project` tests, Clippy with
  warnings-as-errors, Rust formatting, and `git diff --check`.

### C1.2 accepted on July 26, 2026

C1.2 is complete at
`a7c37eaa309081d2299abb2016de4c96be06ca67`.

- The private `capture-delivery/v1` ledger stores immutable envelopes,
  generation-and-digest CAS records, and immutable acknowledgements beneath
  the Qiongli state root.
- Authoritative documents are bounded, owner-private, link-safe, atomically
  replaced, and rebuilt into a deterministic sorted projection after reopen.
- Interrupted enqueue and acknowledgement write boundaries recover from the
  immutable document that was already made durable. Stale writers cannot
  replace a newer causal record.
- Focused fixtures cover exact replay, acknowledgement replay, stale CAS,
  corrupt and oversized records, unknown or orphaned entries, links,
  broadened permissions, and bounded ledger size without adding a transport
  runner or external runtime.

### C1.3 accepted on July 26, 2026

C1.3 is complete at
`1f1145e3aea63fe50b755d9a9aa21fe225a00353`.

- `ProjectStateService` owns typed enqueue, inspect, list, begin, record,
  acknowledge, retry, and cancel operations over the durable ledger.
- Acknowledgement is bound to the exact envelope, destination project,
  accepted capture, expected project revision, and resulting project
  revision. Wrong-project and wrong-revision acknowledgements persist a
  causal conflict and never create a second academic capture.
- Exact transition and acknowledgement replay is idempotent across service
  reopen. Conflict, cancellation, or failed delivery preserves the immutable
  envelope.
- Delivery recovery remains a state operation only; it does not invoke a
  provider, model, Git, Python, Node, or host CLI.

### C1.4 and C1 accepted on July 26, 2026

C1.4 and the complete C1 gate are accepted at
`27bcf5c9052c0eae32087480780335578984035e`.

- `qiongli project capture delivery list|inspect|retry|cancel` exposes strict,
  path-redacted JSON containing causal state, retry count, destination
  identity, record digests, and acknowledgement summary.
- The public destination projection excludes the registered root path and the
  envelope capture body. Mutations require the exact current generation and
  record digest.
- Reopen fixtures cover every committed delivery state. Additional fixtures
  cover exact replay, duplicate acknowledgement, wrong project and revision,
  cancellation, lock contention without mutation, and corrupt records.
- A copied native binary outside the checkout inspected, retried, listed,
  cancelled, and exactly replayed one delivery across isolated process
  restarts with an empty `PATH`.
- Closing validation passed all 105 `qiongli-project` tests, all 109
  `qiongli` library tests, the focused copied-binary restart acceptance,
  Clippy with warnings-as-errors, Rust formatting, and `git diff --check`.
  No broad cybersecurity scan was run.

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

#### C1 implementation batches

C1 will be delivered as four independently reviewable commits.

**C1.1 — Envelope and transition contracts**

- add a `capture_delivery` module to `qiongli-project`;
- define a strict versioned delivery envelope that binds the existing
  `ResearchCaptureV1` digest, optional project binding, source surface,
  delivery class, creation time, and destination identity;
- derive one content-addressed envelope ID from canonical JSON rather than
  random or process-local state;
- define the durable states and legal transition table for `queued`,
  `delivering`, `delivered`, `acknowledged`, `retry-required`, `conflicted`,
  and `cancelled`; and
- reject unknown fields, invalid timestamps, oversized payloads, invalid
  digests, impossible transitions, and identities that do not match the
  embedded capture.

**C1.2 — Private atomic ledger storage**

- add a private versioned delivery ledger under the existing Qiongli state
  root rather than inside a project or UI cache;
- store immutable envelopes separately from mutable delivery records and
  acknowledgement records;
- use the existing bounded-read, private-directory, atomic-replace, lock, and
  compare-and-swap patterns from `qiongli-project::storage`;
- make interrupted writes reconstruct either the previous state or the exact
  next state, never a synthetic intermediate state; and
- rebuild ledger indexes from authoritative records after restart.

#### C1.2 executable plan

C1.2 remains one independently reviewable storage commit and does not add a
CLI, Desktop mutation, transport runner, provider call, model call, Git,
Python, or Node execution.

1. **Private versioned layout**
   - add one crate-private `CaptureDeliveryStore` rooted at
     `<qiongli-state-root>/capture-delivery/v1`;
   - keep owner-private `envelopes/`, `records/`, and `acknowledgements/`
     directories plus one owner-private ledger lock;
   - derive every filename only from a validated `env_…` or `dack_…`
     identifier; and
   - reuse the existing state-root preparation, private-directory,
     no-link/single-link file validation, bounded-read, atomic-write, and
     directory-sync helpers.
2. **Immutable documents and record CAS**
   - write an envelope once, accepting an existing file only when its
     canonical bytes and digest are identical;
   - create the exact matching queued record after the envelope is durable,
     with its initial timestamp fixed to the envelope creation time so an
     interrupted enqueue has one deterministic reconstruction;
   - replace a mutable record only while holding the ledger lock and only when
     envelope identity, envelope digest, expected generation, expected state,
     and expected record digest still match; and
   - write an acknowledgement once, accepting replay only when its canonical
     bytes are identical.
3. **Deterministic recovery and indexing**
   - treat immutable envelopes and acknowledgements plus the latest valid
     record as authoritative; no index or in-memory state may override them;
   - reconstruct an interrupted initial enqueue from the exact envelope and
     deterministic queued record, while an interrupted acknowledgement remains
     recoverable from the exact immutable acknowledgement and delivered
     record;
   - enumerate only bounded, valid identifier filenames, reject stray or
     malformed entries, and sort all rebuilt entries by envelope identity; and
   - expose a derived ledger snapshot that is rebuilt after store reopen and
     never stores paths or research payload text.
4. **Focused qualification**
   - cover clean insert, exact replay, conflicting replay, record CAS success,
     stale generation/digest rejection, acknowledgement replay, and restart
     rebuild;
   - inject interruption after each durable write boundary and prove recovery
     yields either the previous valid state or the exact intended next state;
   - reject corrupt, oversized, symlinked, hard-linked, permission-broadened,
     unknown, and over-count ledger artifacts at the changed persistence
     boundary; and
   - run `qiongli-project` tests, Rust formatting, Clippy with
     warnings-as-errors, and `git diff --check`. No broad cybersecurity scan
     is part of C1.2.

C1.2 is accepted only when closing and reopening the store rebuilds the same
bounded causal snapshot, exact replay is idempotent, stale writers cannot
replace a record, and deleting any derived index cannot remove an envelope,
record, acknowledgement, or canonical academic state.

**C1.3 — Service acknowledgement, retry, and cancellation**

- expose typed `ProjectStateService` operations to enqueue, inspect, begin
  delivery, record delivery, acknowledge, retry, and cancel;
- bind acknowledgement to envelope ID, destination project ID, accepted
  capture ID, expected pre-apply revision, and resulting project revision;
- reuse the existing capture intake identity so replay returns the original
  capture outcome instead of creating a second capture;
- preserve the envelope on conflict, cancellation, or failed delivery; and
- keep transport execution outside the ledger service so recovery cannot
  invoke a provider, model, Git, Python, Node, or host CLI.

**C1.4 — CLI boundary and restart qualification**

- add strict CLI inspect, retry, and cancel commands before any Desktop
  mutation control;
- return path-redacted typed JSON with envelope, causal state, retry count,
  destination binding, and acknowledgement summary;
- add deterministic process-restart fixtures for every committed transition;
- add replay, duplicate acknowledgement, wrong-project, wrong-revision,
  cancellation, lock contention, and corrupted-record tests; and
- run focused Rust format, warnings, crate tests, CLI parser tests, and one
  isolated restart acceptance. No broad cybersecurity scan is part of C1.

C1 is complete only when the same envelope can be queued offline, recovered
after process loss, delivered or retried idempotently, acknowledged exactly
once, and inspected after restart without creating a second academic
authority.

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

#### C2 invariants

C2 extends C1 without weakening its immutable identities.

- The source envelope and source capture are never edited, rebound in place,
  or deleted by assignment.
- Assigning an unbound or stale input creates an explicitly derived capture
  and child delivery envelope. A content-addressed receipt links the source
  envelope, source capture, derived capture, child envelope, selected project,
  and observed revisions.
- The original delivery is cancelled only after the child and receipt are
  recoverably durable. Rejection produces a receipt without a child. An
  unresolved input remains inspectable and retryable.
- Rebinding a capture to the selected project's current revision makes it
  eligible for that project's Inbox; it does not assert that its academic
  content is current or approved.
- Current-project counterparts are claimed only from exact typed identity or
  exact canonical artifact evidence. C2 does not use fuzzy labels, model
  output, session history, or Git authorship to invent a match.
- `accept-current`, `accept-capture`, `retain-both`, and `reject-capture` are
  permitted per item only when the renderer can preserve their stated
  meaning. Unsupported dispositions fail closed.
- Project artifacts change only through a digest-bound academic resolution
  preview/apply transaction. Assignment alone changes private delivery
  control state, not canonical academic artifacts.

#### C2 implementation batches

C2 will be delivered as five independently reviewable commits. C2.1 is the
next implementation batch.

**C2.1 — Resolution identities and bounded contracts**

- add a `capture_resolution` module to `qiongli-project`;
- define strict versioned identifiers and documents for assignment intent,
  per-item comparison, selected disposition, assignment receipt, and academic
  resolution receipt;
- derive each item ID from source envelope identity, item kind, source index,
  and canonical item digest so duplicate text cannot collapse two inputs;
- bind every plan and receipt to source envelope and record digests, source
  capture digest, target project and Library revisions, target manifest and
  observed artifact digests, and the complete ordered disposition set;
- freeze the disposition capability table by item kind and exact-counterpart
  state; and
- reject unknown fields, duplicate keys, missing decisions, unsupported
  dispositions, oversized documents, path-shaped metadata, and identities
  that do not match their canonical semantics.

C2.1 adds contracts and deterministic fixtures only. It does not write a
ledger, assign a project, mutate an academic artifact, add a CLI, or change
the C1 delivery-state vocabulary.

**C2.2 — Private assignment ledger and interruption recovery**

- add a private versioned `capture-resolution/v1` ledger beneath the Qiongli
  state root;
- store immutable assignment intents and receipts separately from a bounded
  recovery journal and derived index;
- reuse C1 private-directory, bounded-read, link validation, atomic-write,
  lock, and generation-and-digest CAS patterns;
- make source cancellation, derived child enqueue, and receipt publication
  one recoverable sequence whose restart result is either the unchanged
  source state or the exact completed assignment; and
- rebuild source-to-child and child-to-resolution lineage from authoritative
  documents after every reopen.

No C2.2 recovery path invokes a provider, model, Git, Python, Node, or host
CLI. Injected interruption tests cover every durable write boundary.

**C2.3 — Assignment preview/apply service**

- expose typed preview/apply operations that select one active registered
  target project for an unresolved source envelope;
- reject missing, archived, identity-conflicting, Library-revision-drifted, or
  project-revision-drifted targets before any write;
- classify exact direct assignment, revision/stage rebind, duplicate target
  capture, resolution-required, and explicit rejection outcomes;
- for an accepted assignment, derive a new content-addressed capture with the
  target's current binding, enqueue one child envelope, persist one assignment
  receipt, and only then close the original delivery through the recoverable
  C2.2 sequence; and
- make exact replay return the original receipt and child identity while a
  changed target, source record, or decision document fails with a revision or
  plan conflict.

The derived capture retains the bounded source payload and source surface.
Its new identity and complete source lineage are explicit; it is never passed
off as the original capture.

**C2.4 — Item-scoped academic reconciliation**

- compare the assigned capture against the selected project's exact current
  manifest, registered artifacts, accepted captures, consolidation receipts,
  and typed evidence identities;
- return bounded differences for semantic changes, decisions, evidence,
  contradictions, and next actions, including the allowed dispositions for
  each item and why a destructive alternative is unavailable;
- require one current plan digest, complete item decisions,
  `academic-review`, and `filesystem-write` approval before apply;
- reuse the recoverable project-file transaction to write only previewed
  artifact deltas, the next manifest revision, and one immutable resolution
  receipt;
- retain the source and derived captures plus assignment receipt as lineage;
  and
- project the resolution into capture/decision lineage without making the
  private assignment ledger an authority for academic meaning.

Exact replay returns the original academic outcome. A changed Library,
manifest, artifact, capture, receipt, or disposition document aborts before
mutation. Failure or cancellation preserves both the unresolved input and the
previous project state.

**C2.5 — CLI, repository adapter, and restart qualification**

- add strict path-redacted CLI inspect/preview/apply commands for assignment
  and academic resolution;
- route repository-delivered captures and registered-artifact drift through
  the same service without invoking Git or inferring a human or client author;
- expose source, child, assignment, acknowledgement, and academic-resolution
  lineage by opaque identity rather than filesystem path;
- add copied-binary process-restart fixtures for direct assignment, stale
  rebind, duplicate, reject, every supported item disposition, exact replay,
  stale target, archived target, lock contention, interruption recovery, and
  corrupt record rejection; and
- run focused Rust crate and CLI tests, formatting, Clippy with
  warnings-as-errors, and one isolated restart acceptance.

No broad cybersecurity scan is part of C2. Security-specific checks remain
limited to the changed identity, path, approval, atomicity, and ownership
boundaries.

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
