# ACP and All Chat State implementation plan

Closed as a source stage September 6, 2026 under ADR 0218. This is a historical
execution record; the master
roadmap owns priority and the program ledger owns acceptance. The PRD and design
retain requirements and detailed contracts. No Trellis skills, task transitions,
JSONL maintenance, mandatory Agent handoffs or additional phase approvals are
needed to execute authorized work.

The completed source is retained for integration into `2.x`. Unfinished App/ACP
stages below are deferred, not accepted or a current execution queue. Continue
from [the CLI closeout and extraction plan](../../../docs/superpowers/plans/2026-09-06-cli-first-closeout-and-extraction.md).

## Historical evidence and delivery order

- [x] Stage 0: ADR 0217, roadmap rebaseline, ledger/index and product priority.
- [x] Stage 1: bounded in-memory All Chat reducer and transition tests.
- [x] Stage 2 partial: pinned ACP v1 SDK, deterministic offline Agent, fixed
      development presets and atomic first-coordinator turn projection.
- [x] Stage 2a: participant-scoped session identity and connection-level unknown-session rejection; focused regressions passed.
- [x] Stage 2b.1: scoped session ownership, sequential prompts and isolated turn outcomes; offline lifecycle regressions passed.
- [x] Stage 2b.2 / 2b.3: wakeable bounded lifecycle, owned Unix teardown, exact-bound permissions, capabilities and transient activity; source implementation complete.
- [x] Stage 3 partial: Rust committed-snapshot contract, schema and golden fixtures.
- [x] Stage 3b: versioned Tauri session/control interface, strict consumer and thin Orchestrator UI; offline source integration complete.

- [x] Stage 3a: private bounded history, intent-before-send and restart recovery; offline source integration complete.

These source increments establish offline App interaction and restart-safe local
history; live adapter readiness and packaged acceptance remain unverified.

| Increment | Start condition | Concrete result |
|---|---|---|
| 2a: reviewed boundary fixes | Completed source increment, September 5 | Correct session identity and fail-closed notification routing |
| 2b: retained session | 2b.1–2b.3 source complete; consumed by 3b | Repeated prompts, permission decisions and prompt-independent cancel |
| 3b: App interface and thin UI | Offline source integration complete September 5 | Prompt/activity/permission/recovery states through Tauri |
| 3a: persistence and recovery | Offline source integration complete September 5 | Restart-safe owned history without replay |
| 3c: research context/tools | Stable session/control interface | Project-scoped tools and source-linked candidates under existing authority |
| 3d: single-Agent integration | 2b and 3a/3b/3c converge | One complete research journey including restart |
| 4: bounded collaboration | Deterministic 3d acceptance | The same journey with at most two collaborators |

Stage numbers retain existing references; they are not separate approval gates.
Keep one integration outcome. After interface agreement, 3a, the thin 3b UI and
3c can advance independently with clear file ownership. Integrate small working
increments instead of waiting for every subsystem to be complete. Live and
packaged observations run in separate lanes; missing external access blocks
only the associated readiness claim. It never waives an offline integration,
security or data-loss check.

## Stage 2a — Completed source increment

Outcome: independent Agents may return the same opaque session string, while a
notification for an unowned session can never produce a successful turn or
change All Chat state. This is a bounded `PLT-404` correction, not acceptance of
its full lifecycle. Implement and check in the main session; this patch is too
small to benefit from an Agent handoff.

### Files and ownership

| File (relative to repository root) | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-execution/src/all_chat.rs` | Correct `apply_event` session-ready validation; extend its existing tests |
| `packages/qiongli-native/crates/qiongli-execution/src/acp.rs` | Guard inbound session notifications at the connection boundary; extend `fixture_agent` and production-path tests |
| `.trellis/spec/native/runtime/all-chat-state-v1.md` | Record participant-scoped identity and unchanged-state rejection |
| `.trellis/spec/native/runtime/acp-v1-client.md` | Record connection-level unknown-session rejection and its error mapping |
| This plan, the roadmap checkpoint and Product Control priority | Record focused results and advance the source checkpoint without changing ledger acceptance |

Preserve the existing uncommitted App snapshot/schema work and workflow removal.
No dependency upgrade, new public schema, UI, storage or provider launch belongs
in this patch. `(run_id, role, provider_session_id)` describes ownership; the
existing state already owns run/role, so do not add a composite-ID type or registry.

### Ordered work

1. **Reproduce through existing owners.** Add the identity cases below to the
   All Chat tests. Extend the in-process Agent fixture to emit an unknown-session
   notification, then valid text and `EndTurn`, through `AcpV1Client::run_turn`.
   Record that the two reported regressions fail on the unchanged implementation;
   tests calling only `text_delta` cannot reproduce SDK routing loss.
2. **Fix identity at `apply_event`.** Remove cross-participant string uniqueness.
   Keep session format/bounds, role membership and one-time participant setup.
   Duplicate initialization with either the same or a different valid string
   returns `InvalidTransition`; invalid input remains `InvalidEvent`. Assert that
   rejected appends preserve participants, events, sequence and generation.
3. **Fix the connection ingress.** Use the pinned SDK's existing dispatch-handler
   extension to validate session ownership before unhandled notifications can be
   ignored. Preserve valid traffic for the SDK session router; an observer must
   not consume it accidentally. Map an unknown session to `ResponseInvalid`,
   discard accumulated output, and prevent a later successful stop from masking it.
   Keep `text_delta` validation as a second check, not the only check.
4. **Check ordering and error propagation.** The SDK 2.1.0 sources expose
   `on_receive_dispatch` / `Handled` and document user handlers before dynamic
   session handlers; `ActiveSession::read_update` only consumes the routed queue.
   Establish session ownership coherently with session creation, including a valid
   update immediately after the new-session response. Prove callback errors reach
   `run_turn`; logging or setting a flag without rejecting the outcome is insufficient.
   Do not patch SDK sources or block its dispatch loop waiting for later traffic.
5. **Close the patch.** Run the checks below, review the combined diff, then
   update the two native contracts and this checklist from actual results.
   Keep `PLT-404` active; leave program acceptance evidence unchanged.

### Regression cases and completion gate

| Case | Required result |
|---|---|
| Primary, Reviewer and Verifier use the same valid session string | All initialize; role/task ownership stays distinct |
| One role is initialized again, with the same or another valid string | `InvalidTransition`; complete state unchanged |
| Empty/control-containing/oversized session ID or absent role | `InvalidEvent`; complete state unchanged |
| Unknown-session notification -> owned text -> `EndTurn` | `ResponseInvalid`; no returned outcome or partial projection |
| Owned update directly after session creation; normal owned prompt response | Valid routing still works; no initialization race or consumed update |
| Existing permission refusal, cancellation and first-turn projection cases | Existing semantics and atomic rollback remain intact |

Run the two new regressions before and after the fix, then these shared checks
once after the patch is stable:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --all-targets --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat_api::tests::generated_schema_and_representative_fixtures_are_stable --locked
git diff --check
```

Exit: the two reported regressions demonstrate fail-before/pass-after, the
remaining matrix and existing golden test pass, and only the planned files
changed for this patch. Roll back its narrow edits if needed, preserving other
working changes. Retained sessions begin with Stage 2b below; no additional
planning document or workflow-stage approval is required within authorized scope.

## Stage 2b — Retained, controllable session (`PLT-408`, `PLT-404`)

Deliver this as three small behavior increments after 2a. Each starts once the
previous focused check passes; each records results in this plan.

| Increment | Implementation focus | Exit evidence |
|---|---|---|
| 2b.1: session owner and repeated turns — source complete September 5 | Narrow ownership/lock contract in the existing design; scoped ACP session, local turn IDs and compatible single-turn wrapper | Offline fixture: one initialization/session, two isolated outcomes, failed/abandoned-turn retirement and idle-update rejection; no run completion |
| 2b.2: cancellation and teardown — source complete | Reuse existing native background-operation patterns; wake a silent turn, bound waits and clean up only owned processes | Silent-Agent cancel, timeout and transport loss end within declared test bounds; no App-wide lock spans a wait |
| 2b.3: capabilities and permissions — source complete | Bind advertised capabilities/auth state and request/turn/participant permission decisions; define 3b control/stream fixtures alongside this behavior | Deny, stale/duplicate choice and permission-pending cancel pass; unsupported capabilities stay unavailable |

The session/turn owner remains in `qiongli-execution`; the App reuses `desktop.rs`
and its Tauri adapter. Inspect `start_mcp_self_test` / `poll_mcp_self_test` /
`cancel_mcp_self_test` as the existing bounded-operation precedent. This is reuse
of ownership and lifecycle patterns, not an all-command concurrency rewrite.
Once control/stream contracts stabilize, 3a persistence, the thin 3b UI and 3c
research-tool binding can proceed independently toward the same 3d journey.

- [x] Define scoped connection/session/turn ownership and lock boundaries in the
      existing design; retain one session for repeated prompts with local turn IDs.
- [x] Complete timeout, owned-process cleanup, permission-response and cancellation
      contracts using the SDK runner, wakeable signals and absolute phase deadlines.
      Unix process-group teardown is verified; Windows development launch is closed
      until a Windows Job owns wrapper descendants. No generic job engine was added.
- [x] Record pinned adapter version, auth state and advertised load/resume and
      mode/model state without persisting secrets or exposing SDK types to the App.
      Advertised support remains distinct from enabled controls.
- [x] Bind interactive permission choices to connection/run/participant/session/
      turn/request; handle one-time allow/deny, stale/duplicate choice, pending
      cancel, timeout and adapter exit. Remembered choices remain disabled.
- [x] Wake cancellation without Agent output; bound initialization, prompt and
      permission waits, plus cancel acknowledgement and owned-process teardown.
      No App-wide lock spans waits; rejected startup closes the control stream.
- [x] Handle bounded plan/tool/status updates; exclude hidden reasoning and raw
      tool payloads. Keep unsupported capabilities explicitly unavailable.
- [x] Prove repeated prompts, silent-Agent and permission cancel, timeout and
      transport loss with deterministic fixtures. Local fixture bounds are 2 s for
      cancellation/phase exit and 3 s for owned Unix process exit; real UI timing
      remains a later `PLT-407` observation, not an inferred benchmark result.

Exit: the complete offline session lifecycle passes. An optional pinned-adapter
smoke uses an isolated non-research workspace with project writes unavailable
until 3c. Each live adapter still needs its own observed capabilities and evidence.

## Stage 3b — App interface and thin interaction

- [x] Define Rust-owned controls and transient stream alongside the committed
      snapshot; update schema, fixtures, compatibility and TypeScript/Zod together.
- [x] Bind prompt/cancel/permission to current run/participant/turn/request; reject
      stale/duplicate controls and late/out-of-order updates at the native owner.
- [x] Extend the existing Orchestrator: one Agent selector, composer, labelled
      timeline, source/context selection, plan/tool activity and permission state.
      Use deterministic dev fixtures while native binding is being completed.
- [x] Bind the actual Tauri transport and cover idle, active, permission-pending,
      interrupted/recoverable and terminal states; distinguish turn end/run end.
      Preserve schema 19, External Host behavior, keyboard access and accessibility.

Exit: strict Rust-to-TypeScript decoding and actual Tauri control/stream behavior
pass. Browser fixtures alone cannot close this increment.

### Delivered source boundary and focused evidence

- `qiongli_all_chat` owns read/start/prompt/control/close. One native worker retains
  an ACP session; short locks never span an await. Revision-bound prompt admission
  and exact-bound permissions reject duplicate or stale actions. Polling snapshots
  retain bounded history, and closing/cancellation retires the owned SDK runner.
- Rust generates a separate v1 control schema and golden snapshots from actual
  Tauri MockRuntime commands. The strict TypeScript client decodes these plus the
  unchanged committed snapshot goldens. No schema 19 or ledger acceptance change.
- Orchestrator includes the offline Agent selector, consecutive turns, source-label
  selection, explicit text context, plan/tool activity, permission decisions and
  an interrupted/new-session path. External Host controls remain intact. No source
  bytes or project mutations are sent through this increment.
- The enabled Agent is a debug-only in-process ACP demonstration using the actual
  SDK connection/control path. App launch of Codex/Claude and release-build chat
  remain unavailable until 3c binds project tools and permissions. The browser
  replay is explicitly development-only, bounded to two turns and excluded from
  the production bundle. Neither fixture is provider or packaged acceptance.
- Passed: 112 execution tests, 2 native App `all_chat` checks (including actual Tauri IPC), 2 strict
  TypeScript contract checks, 11 focused desktop/locale/deferred-client tests,
  TypeScript and Svelte checks, 13 schema/development policy checks and schema
  validator. Browser checks at 1200 and 500 px showed no horizontal overflow;
  a fresh visual review passed layout and identified the permission announcement,
  now fixed with a polite status and covered by the interaction test. The reviewer
  subsequently scored this specific accessibility fix resolved.
- Production build passes frozen-prototype import and fixture exclusion. Measured
  client output: 1972.1 KiB, 87 files; JS 1629.0 KiB; CSS 246.9 KiB; shared shell
  370.8 KiB. Reused utilities and route-local copy keep shell and language budgets
  unchanged. CSS receives an explicit 245 -> 250 KiB allowance for this new area;
  no other size limit was raised.
- Clippy passes for affected App/execution targets with the newer Rust 1.98
  `chunks_exact_to_as_chunks` lint disabled on the command line. The unqualified
  command finds that new lint in four pre-existing files outside this increment;
  their implementation and lint policy were preserved. No release qualification,
  provider authentication, real project read/write, Windows descendant ownership
  or restart persistence was observed here.

Stage 3b handed off to **Stage 3a persistence/recovery** below (now source complete). Reuse registered private
project paths, locks and atomic/CAS owners; persist intent before send, then replay
validated committed events and expose interrupted work without resending a turn.
Update privacy/retention and lifecycle contracts with that increment. Stage 3c
connects actual research context/tools after this stable control interface; 3d
still owns the complete source-linked, approval-bound, restart-safe journey.

## Stage 3a — Persistence and recovery

- [x] Reuse existing private project paths, locks, atomic writes and recovery for
      a validated versioned event store with explicit event/text bounds.
- [x] Persist user intent before send. Bind events to run/sequence,
      participant/session, turn, causal source, timestamp, adapter/capabilities.
- [x] Rebuild committed history, deduplicate provider replay and show interrupted
      work. Use only advertised load/resume; unsupported resume remains unavailable
      and does not count as successful continuation.
- [x] Update both lifecycle guides and closest tests for private chat retention,
      backup, deliberate deletion, portable-export exclusion and diagnostic
      redaction before enabling real conversation storage.
- [x] Cover corrupt/truncated events, future schemas, exhausted bounds, crash
      between authoritative commit and projection, and no duplicate apply.

Exit: restart preserves committed history and exposes interrupted work; failures
preserve original bytes and chat stays out of portable export/diagnostics.

Source result (September 5): the existing project private storage owner now
provides an OS session lease and bounded CAS replacement for 32 retained runs.
`all_chat_history.rs` serializes one canonical v1 observation log (64 turns,
2,048 updates, 2,304 records, 8 MiB), commits prompt intent before channel send,
and derives each App view from committed records. Recovery obtains the lease,
validates all retained logs, marks unfinished work interrupted, and never starts
an Agent, resends a prompt, resolves a saved permission or applies a mutation.
New sessions retain old files; corrupt/future files block writes without eviction.
The latest session is exposed; archive browsing remains deferred until needed.

Task/attempt/checkpoint/evidence receipts are moved to **3c**, where the actual
research-tool owner is bound. Stage 3a repairs the App projection from the log;
it does not fabricate research receipts or introduce a second authoritative
ledger. This preserves the small integration boundary while keeping 3d's full
restart journey requirement intact.

Fresh focused validation (September 5, macOS source workspace):

- Native `qiongli --lib all_chat`: **5 passed**, including real Tauri IPC after
  recreating the App owner, original goldens, canonical history, crash-before-view,
  duplicate/conflicting updates, corruption/future versions and capacity recovery.
  Data appends reserve 4 KiB and three records for closing/interruption; a real
  near-8-MiB log rejects an update that would starve its recovery receipt.
- `qiongli-project runtime_state`: **4 passed**, including existing checkpoint
  regressions, exclusive OS lease, 32-run/byte limits, stale CAS, revision drift,
  private modes, symlink rejection and preservation of external bytes.
- Product diagnostics canary: **1 passed**; history test also executes actual
  portable export and verifies private data exclusion and exact stopped deletion.
- App API All Chat: **2 passed**, TypeScript check passed. All Chat UI: **3 passed**,
  including restored pending work without active permissions. Svelte: zero errors
  or warnings. Lifecycle/public-schema/project-policy: **15 passed**.
- Frontend production build and bundle contract passed: **1972.9 KiB / 87 files**,
  JS **1629.9 KiB**, CSS **246.9 KiB**, shared shell **370.8 KiB**; browser fixture
  remains excluded. No dependency or bundle-budget change in this increment.
- `cargo fmt --all -- --check` and `git diff --check` passed. Clippy for `qiongli`
  and `qiongli-project`, all targets, locked/offline, passed with the same local
  Rust 1.98 baseline qualification: `-D warnings -A clippy::chunks_exact_to_as_chunks`.
  The four pre-existing newer-toolchain lint sites were not edited.
- Public schema validator passed; roadmap index remains current at **237 tasks**.
  Baseline SHA checks preserve the ledger/index, control v1 schema/golden and
  desktop API. No pre-existing working file was removed by this increment.

Live adapters, Windows session lease behavior and packaged restart are unverified.
Program acceptance is unchanged.
Next increment: **3c**, first wire one project-scoped read-only context/tool path
through existing source owners and Full MCP, prove SEC-401/402/403 negatives,
then attach receipt projections and preview/approval/CAS writes. Keep Codex and
Claude unavailable until that invocation boundary is proved.

## Stage 3c — Existing research context and tools

The September 5 [EMB-Q01 review](research/embedded-agent-v3-q01-review.md)
maps the independent embedded-Agent v3 proposal onto this existing plan.
`EMB-Q01`—`EMB-Q08` are proposal aliases, not new program-ledger rows.
ADR 0217 already owns the App transport decision; direct Codex App Server is
an alternative requiring a superseding decision, not an implicit extension of
ADR 0211. Preserve the current ACP source and External Host behavior.

Q02 bounded input (source implementation recorded below): extend the existing native context/candidate
boundary and ACP fixture for two synthetic readable sources. Reuse
`ProjectArtifactViewV1` and its digest/range/truncation data, but prove how each
selected literature object resolves to approved content. Current source refs
are labels; registered-artifact reads are closed to a fixed inventory and do
not establish arbitrary PDF or attachment support. Do not rebuild session,
control, history, schemas or the Orchestrator page from scratch.

The first saved note maps to an existing `ResearchCaptureDraftV1`, followed by
capture preview/apply and reviewed consolidation into research state (and the
decision log only when a real decision exists). Pending Capture history is not
yet a consolidated artifact. A separate Markdown note writer is a distinct
business increment if needed. Bind candidate identity to the native manifest,
revalidate source digests at preview and commit, and regenerate the preview
after an edit. Existing Capture locator validation alone does not prove the
source bytes are unchanged.

Complete the Fake candidate/save journey (**Q03/Q05**) and local observation,
stale-source and receipt recovery (**Q06**) without waiting for a model account.
Codex protocol qualification (**Q04**) may proceed after the context/permission
contract; its actual tool, isolation and provider-resume evidence joins the
same 3d journey. No real research input is enabled before the security boundary
and explicit data authorization. Preserve the named live/packaged acceptance
requirements; a Codex-only increment does not close Claude qualification.

Q01 verification: 23 roadmap/architecture/schema-policy tests, roadmap freshness,
schema validation and document/diff checks passed. The fresh default All Chat
run had 4 passes and one fixture project-creation `PersistenceFailed(AlreadyExists)`
at `all_chat_history.rs:581`; that test passed alone and all 5 passed serially.
The cause is unresolved: retain the failure and investigate before relying on
the default baseline for Q02. No runtime source, ledger acceptance, live adapter
or package qualification changed in this documentation increment.

### Q02/Q03/Q05 offline execution checkpoint — September 5, 2026

Repository: `research-skills`; branch `codex/app-acp-all-chat-realignment`;
HEAD remains `4012ee1329620ae34bcfb59cadd0d40db6768214` (working-tree changes,
no commit/push). The existing dirty tree and 237 ledger rows/46 accepted records
were preserved. These are source checks, not program acceptance.

| Alias | Actual source outcome | Remaining qualification |
|---|---|---|
| Q02 | Native exact excerpts/context, canonical method digest, typed candidate checks, scripted ACP responses, Rust schemas/goldens and strict TypeScript consumer | Arbitrary attachments, local method variants and real Host content loading are not included |
| Q03 | Existing native session supports two excerpt selections, two turns, actual source preview, candidate text editing/citations/rejection and existing error/cancel controls | Only an explicitly labelled development fixture; no model analysis |
| Q05 | Edited candidate enters existing Capture preview/confirmation; stable capture identity; exact source digests checked at preview and under the existing commit lock; consolidation and graph service reads verified | Real-adapter production of the candidate and packaged/native UI journey remain unverified |
| Q06 | Native state survives page remount; restart displays saved observations and rejects old candidate authority; stale sources/projects and duplicate writes fail closed | Provider resume remains unsupported; unsaved candidates require a newly authorized comparison after App restart |
| Q04/Q07/Q08 | Existing ACP negative cases and local integration checks retained | Real tools/isolation/authentication/Skills loading, named native packages and observed App-vs-Plugin user comparison are unfinished |

Implementation owners: `apps/qiongli/src/all_chat_research.rs` (new context,
mapping and actual IPC fixtures); existing `all_chat_control.rs` (retained
session and latest validated candidate); `desktop.rs`/`desktop/tauri_adapter.rs`
(existing Capture preview owner); `crates/qiongli-project/src/capture.rs`
(optional exact source preconditions in verified plans); existing `acp.rs`
(debug-only scripted responses); App API `research.ts`, All Chat components,
AppState and Orchestrator integration. No dependency or second history/approval
store was added. Public additions and generated evidence are registered in the
existing public-schema policy; schema 19 and persisted history v1 are unchanged.

The Q01 baseline failure was reproduced as test fixture directory collisions:
wall-clock readings are not unique. Test directory names now append an atomic
sequence; normal parallel All Chat checks pass. This changes only fixture setup.
The new excerpt parser also rejects a phantom line after a trailing newline and
reports source-end truncation from actual readable lines. A UI assertion that
expected that phantom omission was corrected to assert the real source end.

Focused commands/results (Cargo used `--locked --offline`):

- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline`: 6 passed. The research test covers two actual Tauri/ACP turns, native review edits and stable identity, dismissal/restart rejection, no implicit writes, source drift, Capture commit/consolidation and graph projection. Subsequent focused research rerun covers the final line-boundary and Full MCP service-read assertions.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-project capture --locked --offline`: 59 passed, including old Capture/assignment/delivery/consolidation paths.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution acp --locked --offline`: 14 passed, including unknown sessions/permissions, cancellation, partial failure, bounded timeout and owned process cleanup.
- `pnpm --dir packages/qiongli-app-api test`: 38 passed, with freshly generated schema-19 fixture plus new Rust research/control goldens. API type check passed.
- Desktop All Chat, research and AppState focused tests: 28 passed; Svelte check: zero errors/warnings. `pnpm --dir packages/qiongli-desktop build`: passed production/deferred-bundle checks; development replay fixture excluded.
- Public-schema validator passed; lifecycle/schema/frozen-architecture tests: 18 passed. Rust format and all-targets Clippy passed with the existing Rust 1.98 `chunks_exact_to_as_chunks` lint exception; no new lint exemption was added to source.

Test invocation mistakes (a nonexistent Python module name and a camelCase
argument to the existing snake_case MCP service) were corrected and rerun;
neither is recorded as a product success. No model, account, private research
data, live Host, installer or publication was invoked. Local build and tests do
not close Q04, Q07, Q08, PLT-404 or any acceptance row.

### September 5 — Q04 stable ACP read-view checkpoint

Repository and branch remain `research-skills` /
`codex/app-acp-all-chat-realignment`, HEAD
`4012ee1329620ae34bcfb59cadd0d40db6768214`. This is an uncommitted increment
on the existing working diff, not a new acceptance row or completion of Q04.

The next runtime boundary now has an opt-in in-memory ACP `fs/read_text_file`
implementation in `crates/qiongli-execution/src/acp.rs`. It takes at most three
approved snapshots under exact virtual paths and advertises only stable ACP
read capability. It never opens a source path on disk. Strict raw request
decoding prevents the SDK's malformed `line`/`limit` fallback to a full read.
EOF truncation follows ACP's maximum-line-count semantics. An attached existing
`AcpV1Control` supplies session/turn/cancel/stop/deadline admission and per-turn
16-request/256-KiB budgets; invalid requests latch failure even if the Agent
continues. All other client requests, including permission, write and terminal
requests, are rejected in this opt-in mode. Invalid view construction closes
the attached unclaimed control. No new dependency, SDK extension or task owner.

This does **not** enable tools on the App's research manifest v1 (whose allowed
tools remain empty). The current App synthetic journey and candidate ownership
remain as previously implemented. Binding the read view into that journey is
the next contract change, with explicit capability display and native-derived
resource paths. Do not claim this client reader isolates an external process.
SDK 2.1.0 `acp_agent.rs` starts its configured command with inherited environment
and startup cwd; `session/new.cwd` and Unix process groups are not an OS sandbox.
Existing full-project read tools exceed a two-excerpt authorization and must not
be exposed wholesale. Stable MCP stdio remains a later separately scoped bridge;
the SDK's MCP-over-ACP helper requires an unstable feature, not enabled here.

Focused evidence for this increment:

- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --locked --offline --quiet`: **115 passed**, including three new read-view tests with real in-process ACP messages and a two-turn request-budget reset. Cases include malformed ranges, EOF, exact-path aliases, unknown sessions/methods, denied writes/terminal/permissions, startup/idle/cancel requests, deadline, scope drop and failed configuration stream closure.
- The full rerun initially had **114 passed / 1 environment failure**: the existing Unix cleanup test could not execute read-only `ps` under the sandbox. The authorized local offline rerun with that restriction lifted passed all 115; no process-test assertion was weakened.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline --quiet`: **6 passed** on the final runtime changes, including real Tauri IPC, retained turns, research candidates and private history recovery (39.71 s).
- `cargo clippy --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --all-targets --locked --offline -- -D warnings -A clippy::chunks_exact_to_as_chunks`: passed with the already documented Rust 1.98 lint exception. Rust format check passed.
- `python3 tooling/scripts/validate_public_schema_policy.py`: passed. `python3 tooling/scripts/update_program_roadmap.py --check`: 237 tasks, index current. A mistyped nonexistent schema script path was corrected; it is not counted as a passed check.
- Independent bounded code review found the EOF compatibility issue; fixed and rerun. Constructor-error stream closure was also fixed and covered. No provider, account, private research source, network installation, package qualification or publication was invoked.

### September 5 — Q04 research panel read-view integration

Same repository/branch and HEAD `4012ee1329620ae34bcfb59cadd0d40db6768214`;
all changes remain in the existing uncommitted diff. The runtime-only checkpoint
above is now connected to the development App research journey.

- Research manifest v2 declares exactly `fs/read_text_file` and three exact
  native-derived paths for the two selected excerpts and canonical method.
  Starting requires `contextAccess: "selected_excerpts"`; the panel obtains
  scope-bound consent and validates the response's selections. Missing or
  expanded access is rejected before a native session starts.
- `for_development_read_responses` reuses the existing in-process ACP fixture
  and read-view constructor. Every turn reads all three resources through ACP,
  checks exact returned bytes and emits distinct bounded activity before its
  deterministic response. Native IPC tests observe all three completed reads
  in each of two turns. No real model, provider command or network was used.
- Source and method revalidation occurs before turn admission and candidate
  acceptance. A changed method byte, source/project drift, old manifest candidate,
  injected source instruction or malformed response cannot widen scope or create
  an approved artifact. Saving still uses existing Capture preview/approval/CAS
  and reviewed consolidation. Restart restores observations, not read authority.
- Changes are owned by native `all_chat_research.rs` / `all_chat_control.rs`,
  execution `acp.rs`, app-api `research.ts` and its tests, the existing research
  panel/copy/test, generated v2 schemas/IPC goldens and the schema policy/specs.
  Historical v1 schemas and fixtures are retained; no ledger acceptance changed.

**Q04 research v2 integration/old-consumer gate — open.** The v1-to-v2 wire
change is `unsupported-breaking` under ADR 0216: prior data-only consent cannot
be automatically converted into tool authority. This authorized development
increment rejects old start payloads and old candidate digests. Before any
accepted v1 support is removed or v2 published, a separate product/release review
must verify the consumer/support boundary and approve the transition (or retain
a separate data-only compatibility path). Local tests do not close this gate.
The native research route stays development-only, and real providers stay
unavailable through it. Private history v1 and schema 19 are unchanged.

Validation on this increment:

- Final native `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline --quiet`: **6 passed** (41.21 s), including generated v2 contracts, actual Tauri IPC, source/method/old-candidate negatives, two turns of three reads, Capture flow and recovery.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --locked --offline --quiet`: **115 passed** (1.40 s). All-targets Clippy for `qiongli` and `qiongli-execution` passed with `-D warnings` and the existing `chunks_exact_to_as_chunks` exception. Final Rust formatting and diff checks passed.
- `pnpm --dir packages/qiongli-app-api test`: **38 passed**; API TypeScript check passed. Desktop AllChat/ResearchChat focused tests: **4 passed**; Svelte check: zero errors/warnings. The research UI check was rerun after final edits.
- `pnpm --dir packages/qiongli-desktop build`: passed, 1991.4 KiB; production/deferred-bundle contract passed and development fixture excluded. This is frontend packaging evidence, not native runtime/package qualification.
- Public-schema/lifecycle/frozen-architecture policy tests: **18 passed**. Schema policy validator passed; roadmap `--check`: **237 tasks, index current**. Bounded independent review found no remaining required fixes in this increment.

Next bounded work remains **Q04 / Stage 3c real tool boundary**: bind existing
project-scoped read tools and task content to the approved ACP runtime; prove
filesystem/Shell/hooks/MCP/network isolation and actual content loading before
enabling real source input. Then run a specifically authorized synthetic two-turn
Codex journey. Runtime/OS teardown and advertised load/resume capabilities must
be qualified as supported, unsupported or unrun. Q07 package evidence and Q08
observed user comparison follow; neither can be manufactured by the offline demo.

- [ ] Project task/role/attempt/checkpoint/evidence receipts from their existing
      owner after binding the real tool path; repair missed projections without
      replaying a project mutation (moved from 3a to avoid speculative receipts).

- [ ] Complete the `SEC-401` -> `SEC-402` -> `SEC-403` boundary on the actual
      invocation path before enabling research tools. Source text cannot widen
      instructions, tool permissions, project approval or evidence status.
- [ ] Resolve project/revision/sources through existing owners. Reuse canonical
      Workflow/Skill and role/handoff context; attach fixed project-scoped Full
      MCP during create/load rather than assuming Host installation is inherited.
- [ ] Enforce Agent-native file/terminal restrictions so canonical project writes
      cannot bypass preview, approval, evidence, digest and CAS. Keep unsupported
      write capabilities unavailable; permission alone never grants project apply.
- [ ] Return source-linked candidates through existing artifacts/transitions;
      reject stale writes and refresh affected project, Graph and Artifact views.
- [ ] Bind actual permitted tools and source scope at native invocation, not by
      trusting a manifest or installing the full MCP inventory. Qualify native
      file/shell/hooks/network paths too; a staging cwd is not a sandbox.
- [ ] After an approved save advances project revision, reconcile the receipt
      and explicitly rebind context before a follow-up. Keep observation reload,
      a new session with bounded context, and exact backend resume distinct.

Exit: declared research tools work and their negative authority cases fail closed.

## Stage 3d — Single-Agent integration (`PLT-405`)

- [ ] Prove this journey with a deterministic native Agent and actual Tauri IPC:
      select project/sources -> compare literature evidence -> inspect a
      source-linked draft candidate -> preview/approve its project change ->
      refresh affected views -> restart and continue the conversation.
- [ ] Prove denial, stale revision, cancellation, adapter loss and restart recovery
      on the integrated path. Keep generated claims distinct from verified evidence.
- [ ] Record pinned Codex and Claude live observations independently when available;
      keep adapter readiness and packaged-product claims open until qualified.

Exit for Stage 4 development: the deterministic journey and failure/recovery cases
pass. `PLT-406` program acceptance still depends on accepted `PLT-405`; advancing
offline code does not close either ledger row. Live evidence cannot be borrowed
between adapters or from External Host receipts; all declared adapters still
require it for product acceptance.

## Stage 4 — Bounded collaboration (`PLT-406`)

- [ ] One coordinator creates at most two validated assignments; independent
      worker/reviewer sessions use existing task/handoff/evidence owners.
- [ ] Project committed transitions into the timeline and return bounded,
      evidence-linked results to the coordinator. Do not add a second scheduler.
- [ ] Prove the same research journey with child-session cancellation, adapter
      loss, restart, duplicate/out-of-order update rejection and stale-write rejection.

Then measure under `PLT-407`, finish `SEC-404`/`SEC-405` import/quarantine work,
qualify self-contained sidecars per claimed target, and enter the existing M2/M3
replacement/cutover lanes. Kernel, Evidence v2, reproducibility and integrated
`PILOT-702` remain M4+.

## Checks and recording

Run the affected command once per changed behavior; broaden only at integration
or after a new failure. Record results here rather than generating more manifests.

| Changed scope | Focused check |
|---|---|
| Stage 2 native execution | `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --locked`; Rust format check |
| Stage 2a / All Chat snapshot | `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat_api::tests::generated_schema_and_representative_fixtures_are_stable --locked` |
| Roadmap/ledger | `python3 tooling/scripts/update_program_roadmap.py --check`; `python3 -m unittest tests.test_program_roadmap` |
| Architecture/public schema | Relevant existing ADR/schema validator and negative tests only when those contracts change |
| 3a/3b/3c | Closest persistence/recovery, App API/Desktop, MCP/permission or lifecycle/export checks; actual-IPC integration at 3d |

Required exact-head PR CI, named target builds and packaged acceptance retain
separate evidence. Planning changes do not close runtime checkboxes or change
accepted ledger rows. Roll back the narrow faulty increment; preserve existing
External Host checkpoints and original persisted bytes. This plan does not grant
credentials, external research-data transfer, adapter installation or publication.

September 4 process/plan verification: 8 focused policy/roadmap tests passed;
roadmap freshness, authorization-policy validation and diff checks passed.
All 237 program IDs and 46 accepted records were preserved. No runtime increment
was completed by this workflow change; Stage 2a remains the next code patch.

## Stage 2a execution record — September 5, 2026

- [x] Reproduced both defects before implementation: the new All Chat identity
      test failed with `InvalidEvent`; the connection regression returned a
      successful outcome after an unknown-session update.
- [x] Removed cross-participant session-string uniqueness while retaining role,
      format/bounds and one-time initialization checks. Rejection preserves the
      full state, including generation, sequence, participants and history.
- [x] Added the existing SDK dispatch callback: bind session identity during the
      new-session response, pass valid traffic through, and reject invalid updates
      using a fatal background task plus a latched error. A mere notification
      callback error would only be logged by SDK 2.1.0.
- [x] Verified immediate owned/unowned updates, unknown updates after valid text,
      and rejection without a terminal response using a bounded test watchdog.
- [x] Rust format, execution Clippy (`--all-targets -- -D warnings`), all 100
      execution tests and the existing App schema/golden test passed. Cargo
      validation used `--locked --offline`; no live adapter was launched.
- [x] Updated native contracts and the current source checkpoint. Existing App
      snapshot work and the program ledger remain unchanged; this working-tree
      result is not live-adapter, package or program acceptance evidence.

## Stage 2b.1 execution record — September 5, 2026

- [x] Defined the narrow ownership/lock contract in the existing design. Reused
      the pinned SDK runner via an asynchronous scoped callback; no dependency,
      process registry or App-wide job engine was added.
- [x] Added `AcpV1Session` and sequential `run_turn`, session-local bounded turn
      IDs and independent result buffers. Preserved the existing single-turn
      wrapper; rejected later-turn use of the first-coordinator projection.
- [x] Verified one initialization/session for two prompts, preflight rejection
      without ID consumption, counter exhaustion, failed/abandoned-turn retirement
      and idle-update rejection. ACP v1 cannot identify causal origin for late
      traffic arriving after a subsequent prompt has begun; the contract states
      this limitation explicitly.
- [x] Rust format, execution Clippy (`--all-targets -- -D warnings`) and all 103
      execution tests passed with locked/offline Cargo validation.
- [x] Existing App schema/golden regression passed (1 test). App snapshot/schema
      files, dependency manifests/lockfile and program acceptance records remain
      unchanged by this increment; no provider process or network was used.

## Stage 2b.2 / 2b.3 execution record — September 5, 2026

- Implemented wakeable multi-observer cancellation and configurable bounded
  initialize/session/prompt/permission/cancel-grace budgets. Reused already-locked
  async timers, events, futures and OS randomness; no SDK upgrade or job framework.
- Reused SDK 2.1.0 process ownership; verified local Unix launcher and descendant
  cleanup after timeout. Windows `npx` launch now fails before spawning because
  the SDK has no descendant Job owner. This is an explicit platform limitation,
  not Windows live-adapter readiness.
- Added participant-scoped native controls with a fresh connection nonce, local
  turn/request identities, bounded transient updates and exact-once permission
  decisions. Startup rejection and scope closure terminate observers; discarded
  turns invalidate pending approvals. Existing single-turn/default-deny behavior
  and first-coordinator projection remain compatible.
- Recorded capabilities/authentication readiness, normalized plan/tool activity,
  excluded hidden/raw payloads and generated the 3b control/stream fixture through
  its Rust owner. Supported/advertised state does not silently enable load/resume,
  mode/model selection or remembered permission grants.
- Validation: execution Clippy with all targets and warnings denied; all 112
  execution tests; the existing App schema/golden regression; public-schema policy
  validation and 12 policy tests; roadmap freshness and diff checks. Cargo used
  locked/offline dependencies. The macOS sandbox denied the test-only `ps` probe;
  the approved local test run outside that sandbox verified owned process exit.
- Final review also covers startup rejection closing observers and suppressing
  delayed permission events after a turn terminates. Existing App snapshot/schema,
  All Chat reducer, Trellis removal and all 237 ledger rows/46 acceptance records
  were preserved. No live adapter, private research input, package, commit, push
  or ledger acceptance is part of this source increment.

Next implementation increment: **Stage 3b**, the versioned App control/stream
schema, strict TypeScript decoder, Tauri binding and thin Orchestrator interaction.
Persistence (3a) and research context/tools (3c) can proceed independently against
these native bindings; they converge in the single-Agent 3d journey. Program
acceptance, Windows process ownership and live/package qualification remain open.
