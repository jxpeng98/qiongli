# Qiongli R5C C4 Desktop Continuity Execution Plan

Status: planned — C4.1 is the next implementation batch

Date: July 26, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: C3 completion commit `863bfbc9`

Parent plan:
`docs/superpowers/plans/2026-07-25-qiongli-r5c-cross-surface-continuity.md`

## Outcome

Expose the accepted C1 delivery, C2 assignment/resolution, and C3 portfolio
authority through the strict App API and the macOS Desktop App. The Desktop
must let a user understand incomplete work, inspect exact lineage, preview
recovery or academic mutations, confirm the current native plan, cancel
bounded work, and recover after restart.

Rust remains the authority. TypeScript validates and transports strict
documents. Svelte presents those documents and collects choices; it does not
reimplement delivery transitions, assignment classification, academic
resolution, catalog reconciliation, lineage joins, or timeline construction.

## Current baseline

The current product already has:

- App API schema version 4, with strict Rust serialization and Zod parsing;
- one `qiongli_snapshot` command and one typed `qiongli_execute` intent/event
  boundary;
- operation-token preview, confirm, and cancel behavior for existing Desktop
  mutations;
- native Research Library, Capture Inbox, Coverage, artifact-change, graph,
  and full portfolio events;
- Svelte Research Library, Captures, and Academic Graph routes;
- shared confirmation, feedback, status, localization, and application-state
  primitives; and
- focused Rust, App API, Svelte component, and production-build gates.

The remaining C4 gaps are explicit:

1. C1 delivery records and acknowledgement state are visible only through the
   CLI;
2. C2 assignment and item-scoped resolution previews are not exposed to the
   Desktop;
3. the Desktop still uses the pre-C3 full graph portfolio snapshot rather than
   the rebuildable catalog status and bounded query boundary;
4. semantic timeline, revision, and merge-resolution history have no Desktop
   contract or view;
5. catalog reconcile, full rebuild, doctor, and derived-state deletion are not
   available as native Desktop operations; and
6. cancellation, stale-preview rejection, restart recovery, localization, and
   accessibility have not been qualified as one continuity experience.

## Frozen product rules

- Qiongli remains a shell and native control plane, not a model client.
- No C4 intent invokes Codex, Claude Code, a provider API, Git, Python, Node,
  or an inferred host session.
- The App API exposes no absolute project root, raw client configuration,
  credential, prompt, transcript, provider response, session identifier, or
  inferred author.
- Project, capture, delivery, acknowledgement, assignment, resolution,
  catalog, query, event, cursor, and operation identities remain opaque.
- `unknown`, `missing`, `stale`, `conflicted`, and `recovery-required` remain
  distinct states. Presentation must not convert any of them to connected,
  synchronized, current, or successful.
- Every mutating action requires a current native preview. Confirmation binds
  to the exact preview digest or operation token and current authoritative
  revisions.
- A successful confirm returns refreshed native state. A stale, missing, or
  recovered preview cannot be reconstructed in Svelte and silently retried.
- Cancellation reaches the Rust service between bounded work batches.
  Dismissing a dialog or hiding progress does not claim native cancellation.
- Portfolio and timeline pagination use content-bound cursors. The UI never
  derives an offset cursor or merges results from different catalog IDs.
- C4 does not change C1, C2, or C3 canonical storage contracts unless an App
  API requirement reveals a correctness defect. Any such correction is a
  separate source commit with focused regression tests.

## App API v5 boundary

C4.1 performs one deliberate schema transition from App API version 4 to 5.
Rust fixtures and Zod schemas change in the same commit. Unknown fields,
unknown variants, invalid opaque IDs, oversized lists, stale revisions, and
foreign cursors remain rejected.

The Desktop boundary adds bounded native views for:

- delivery list and inspection, including causal state, retry count,
  destination binding, acknowledgement summary, current generation, and
  record digest;
- assignment list, inspection, and digest-bound preview;
- academic resolution inspection and complete item-scoped selection preview;
- portfolio catalog status, maintenance preview/result, doctor outcome,
  bounded query result, and content-bound cursor;
- project and portfolio timeline results, revision history, and
  merge-resolution history; and
- operation progress and cancellation outcome for bounded portfolio and
  timeline work.

The boundary uses presentation-specific wrappers only where necessary to
remove private fields, add closed bounds, or express capabilities. It does not
create a second state machine. Native project-domain types may cross the
boundary only when their serialized form is already strict, bounded,
path-redacted, and stable enough for App API v5.

## Navigation and information architecture

C4 preserves the existing top-level product shape and adds continuity without
turning the Desktop into a graph-only interface:

- **Captures** gains Inbox, Outbox, and Conflicts modes for one selected
  project;
- **Coverage** remains inside Captures and reports evidence separately from
  delivery state;
- **Timeline** becomes a bounded project/portfolio activity route;
- **Portfolio** becomes the catalog-backed cross-project search and lineage
  route; and
- **Academic Graph** remains the detailed project graph and may deep-link from
  a portfolio result using opaque identities.

The default screen emphasizes pending, failed, stale, conflicted, and
recovery-required work. Successful history remains available without
overwhelming the actions that need attention.

## Batches

### C4.1 — Strict continuity App API contracts

Extend `desktop_api.rs`, `@qiongli/app-api`, and the canonical fixture together.

Add intents and events for:

- delivery list/inspect and acknowledgement inspection;
- assignment list/inspect/preview;
- resolution inspect/preview with a complete typed item selection;
- portfolio status, query, timeline, and doctor;
- portfolio reconcile, full rebuild, and derived-state deletion preview;
- delivery retry/cancel and portfolio-operation cancellation; and
- content-bound next-page requests.

Mutation previews use the existing top-level operation token only when the
token can bind the complete native domain preview. The event must also expose
an item-scoped, path-redacted explanation so the confirmation dialog can say
what will change and why.

Acceptance:

- the Rust fixture contains every new intent-adjacent event variant;
- Zod accepts the canonical fixture and rejects added fields, invalid IDs,
  excessive arrays, invalid selection sets, and mismatched cursors;
- App API v4 input cannot be mistaken for v5;
- TypeScript exports remain generated from or directly tied to the Zod
  schemas; and
- App API and App-library focused tests, formatting, Clippy, workspace check,
  and `git diff --check` pass.

### C4.2 — Native Desktop continuity service

Extend `desktop.rs` and the thin Tauri adapter over the accepted project
services. Add a native read model that loads one coherent continuity snapshot
for the selected project and one coherent catalog status for portfolio views.

Service behavior:

- delivery, assignment, resolution, coverage, and project revision data are
  loaded from the same validated project/Library observation;
- state drift during a request returns stale or validation failure rather than
  mixed data;
- list and query limits are set by Rust and cannot be raised by Svelte;
- retry, cancel, acknowledgement, assignment, resolution, reconcile, rebuild,
  and deletion confirmations call their existing native services;
- previews are discarded after completion, cancellation, snapshot revision
  change, or process restart;
- recoverable native transactions surface explicit recovery actions after
  restart; and
- cancellation handles are scoped to one operation token and checked at the
  existing project/node/edge/event boundaries.

Acceptance covers exact replay, wrong token, wrong revision, expired preview,
cross-project identity, cancellation before publication, cancellation during
bounded work, process restart, corrupt private state, and path redaction.

### C4.3 — Inbox, Outbox, Conflict, and Coverage experience

Evolve the current Captures route and feature module.

Inbox:

- show capture source, capture state, target binding, revision, and reviewed
  outcome from native documents;
- preserve unknown or unattributed source evidence;
- make assignment the explicit next action for unbound or divergent work; and
- deep-link to the exact source/child lineage after assignment.

Outbox:

- show queued, delivering, delivered, acknowledged, retryable, conflicted, and
  cancelled delivery records;
- explain why an acknowledgement is missing or conflicted;
- offer retry or cancel only when the native record exposes that capability;
  and
- bind confirmation to the displayed generation and record digest.

Conflict and academic review:

- display the native assignment outcome and every resolution item;
- require one allowed disposition for every item before requesting preview;
- show the native effects, revision transition, and approval requirements;
- never default a destructive or meaning-changing disposition; and
- restore a clear inspection/re-preview path after stale confirmation.

Coverage remains evidence-oriented. `unknown` source delivery uses neutral
language and cannot receive the same visual or textual treatment as observed,
connected, delivered, or current.

### C4.4 — Portfolio and Timeline experience

Replace the Desktop's pre-C3 full-rebuild portfolio load path with the
incremental catalog boundary.

Portfolio:

- show `current`, `missing`, `stale`, or `recovery-required` catalog status;
- provide bounded filters for the C3 query contract;
- render deterministic project, node, edge, and lineage results without
  client-side entity merging;
- preserve the returned ordering and append only a matching content-bound
  cursor page;
- offer reconcile or full rebuild through native previews; and
- offer derived-state deletion only from a dedicated explanation and
  confirmation flow that states canonical academic data is retained.

Timeline:

- support project, portfolio, revision-history, and merge-resolution views;
- label the exact timestamp source and avoid inferred human attribution;
- paginate only with the returned catalog-bound cursor;
- link events to available opaque project, delivery, assignment, resolution,
  node, or edge identities; and
- show a stale-catalog recovery action instead of presenting partial history
  as current.

The UI may use tables, lists, and small relationship summaries. A dense graph
is not required for C4 acceptance; causal clarity and bounded navigation take
priority.

### C4.5 — Localization, accessibility, and source qualification

Complete Chinese and English copy for every new route, state, explanation,
confirmation, failure, cancellation, and recovery action.

Focused fixtures cover:

- keyboard-only navigation and activation;
- visible focus and focus restoration after dialogs;
- dialog labelling, initial focus, Escape behavior, and non-destructive
  defaults;
- screen-reader status announcements without repeated progress noise;
- reduced-motion operation feedback;
- narrow-window layouts without clipped actions or horizontal page overflow;
- long Chinese and English labels, empty state, bounded truncation, and next
  page;
- truthful unknown, stale, conflict, recovery, cancelled, and failed states;
  and
- App restart with pending recovery or invalidated preview state.

Source qualification runs:

- focused `qiongli-project` and App-library Rust tests;
- App API parser and client tests;
- Desktop feature and component tests;
- App API TypeScript check;
- Svelte check and production build;
- Rust formatting and warnings-as-errors Clippy;
- complete Rust workspace all-target check; and
- `git diff --check`.

C4 deliberately does not claim packaged macOS installation or live client
interaction. Those manual, copied-App, isolated-home checks belong to C5.
No broad cybersecurity scan is run.

## Expected file ownership

Primary native files:

- `packages/qiongli-native/apps/qiongli/src/desktop_api.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop/tauri_adapter.rs`; and
- focused App-library/Desktop tests and the canonical contract fixture.

Primary App API files:

- `packages/qiongli-app-api/src/schema.ts`;
- `packages/qiongli-app-api/src/client.ts`;
- `packages/qiongli-app-api/src/index.ts`; and
- `packages/qiongli-app-api/tests/client.test.ts`.

Primary presentation files:

- `packages/qiongli-desktop/src/lib/app-state.svelte.ts`;
- `packages/qiongli-desktop/src/lib/features/captures/`;
- new focused portfolio and timeline feature modules;
- the Captures route plus new Portfolio and Timeline routes;
- shared confirmation, feedback, and status components only where the new
  semantics require an extension;
- `packages/qiongli-desktop/src/lib/i18n.svelte.ts`; and
- focused feature, component, route, and localization fixtures.

The C1–C3 project-domain modules remain the business authority. C4 must call
them rather than copy their transition or validation logic into the App crate.

## Commit sequence

1. `feat(app-api): expose continuity control contracts`
2. `feat(desktop): project native continuity state`
3. `feat(desktop): confirm capture recovery actions`
4. `feat(ui): add portfolio and timeline continuity views`
5. `test(desktop): qualify localized continuity experience`
6. `docs(roadmap): accept desktop continuity experience`

Each commit is independently testable. If the App API v5 contract needs to be
split for review, Rust serialization, the Zod schema, the canonical fixture,
and parser rejection tests for each new variant must still land together.

## C4 completion gate

C4 is complete only when:

1. Desktop and CLI report the same delivery, assignment, resolution,
   portfolio, and timeline causal states from the same native services;
2. acknowledge, retry, cancel, assign, resolve, reconcile, rebuild, and
   derived-state deletion cannot execute without a current native preview or
   exact current-record binding;
3. cancellation reaches the Rust boundary and never publishes partial catalog
   or mixed-revision results;
4. restart recovery is explicit and stale preview tokens cannot be reused;
5. `unknown` is never displayed as observed, connected, delivered, or current;
6. App API v5 remains strict, bounded, path-redacted, and free of credentials,
   prompts, transcripts, provider responses, and host sessions;
7. Chinese, English, keyboard, focus, reduced-motion, and narrow-window
   fixtures pass; and
8. all focused C4 source gates pass without a broad cybersecurity scan.
