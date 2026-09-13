# ACP and All Chat State design

## Architecture and boundaries

The first vertical reuses the existing native orchestration owners instead of
adding a second scheduler:

```text
Orchestrator page / existing project and source views
      |
All Chat snapshot + control + transient stream contracts
      |
bounded native ACP session owner ---- private chat event store / reducer
      |                                      |
      +---- existing task / handoff / evidence / CAS owners
      |
ACP v1 private sessions ---- pinned Codex or Claude adapter
                                      |
                             existing Full MCP / ToolHost
```

- ACP owns process transport, capability negotiation, private Agent sessions,
  prompts, updates, permission requests, and cancellation.
- `AllChatStateV1` owns only Qiongli-visible collaboration state. It never
  claims to contain the Agent's hidden context or provider credential.
- The existing `OrchestrationPlanV1`, role/worker packets, handoff bindings,
  evidence references, and project compare-and-swap checks remain authoritative.
- Full MCP and ToolHost remain the only Agent-accessible Qiongli project tools.
- Implement the session owner for a single Agent before enabling collaborators.
  Its background work must not hold a Desktop service or project mutex while
  waiting on Agent updates, permissions, cancellation, or shutdown.

## Roadmap realignment

Do not add another family of long-lived task IDs. Reuse the existing platform
slice whose real IPC and recovery work is still open:

- `PLT-401`--`PLT-403` remain accepted and move from stale `NOW` prose to a
  recently closed line.
- `PLT-408` moves ahead of the remaining `PLT-404` work: define only the ACP
  process/session/job, lock, timeout, permission-response, and cancellation
  contract needed by this vertical. General M6 jobs remain deferred.
- `PLT-404` completes session identity/routing, negotiated capabilities and
  authentication, repeated turns, permissions, wakeable cancellation and teardown.
- `SEC-401` -> `SEC-402` -> `SEC-403` must pass before `PLT-405` research tools
  are enabled. Contract and offline fixture work may proceed before that gate.
- `PLT-405` closes persistence, the complete App boundary, research context/tools,
  approved candidate application and single-Agent restart/resume.
- `PLT-406` extends the accepted single-Agent journey to bounded collaboration.
- `PLT-407` measures the resulting path; `SEC-404`--`SEC-405` complete wider
  import hardening. Neither replaces immediate focused boundary checks.
- `PILOT-702` remains the later integrated Kernel/Evidence/Gate pilot over the
  earlier coordination foundation, retaining private provider sessions.
- The full post-2.0 scientific governance in `ORC-601`--`ORC-609` remains later;
  the MVP reuses only already-implemented role, evidence, cancellation, and
  stale-write protections.

ADR 0217 supersedes only ADR 0211's claim that Qiongli never launches an Agent
for ordinary work. External Host mode and ADR 0211's project/evidence safety
rules remain valid. Direct provider APIs do not return as the default.

## All Chat State v1

The first code slice adds a small pure Rust contract in `qiongli-execution`.
It is independent of Tauri and of any specific ACP SDK type.

### State

- one existing `RunId` and project revision binding;
- one coordinator participant and zero to two worker/reviewer participants;
- monotonically increasing generation and event sequence;
- ordered committed events;
- derived run and participant lifecycle state.

The implemented v1 is an in-memory reducer, not a recovery store. The current
Rust App v1 is a committed-snapshot projection, not a complete interaction API.
The additions below are planned contracts; update their Rust-owned schemas,
compatibility records and strict consumers together when each slice implements
them. Do not treat this design as evidence that those additions already exist.

### Initial committed events

- run started;
- user message;
- Agent session ready;
- coordinator delegation;
- worker/reviewer result;
- coordinator message;
- run completed, failed, or cancelled.

Tool-call deltas, streaming fragments, permission choices, and provider-specific
metadata are added only with the ACP/App slice that consumes them. Partial stream
content is presentation state, not committed project state.

### Invariants

- exactly one coordinator;
- at most two non-coordinator participants;
- only the coordinator may delegate;
- workers may return only tasks assigned to them;
- event sequence and generation are exact compare-and-swap values;
- terminal runs reject later events;
- provider session identifiers are opaque, bounded, and never used as project
  authority;
- serialized state excludes credentials, approval tokens, reasoning traces, and
  absolute paths.

### Session identity and authoritative task state

The current bounded participant identity is `(run_id, role)`. Bind its opaque
provider session ID inside that namespace; equal strings from independent
participants are valid. A second session-ready event for the same participant
remains invalid. Connection dispatch validates every session update against its
owned sessions before SDK routing can silently discard an unknown session.

Chat-event generation orders chat history; it is not an orchestration checkpoint
generation. Delegation and accepted results must reference the existing task,
role, attempt, checkpoint generation/document digest, and evidence receipt.
Existing orchestration validation performs the transition first; the timeline
projects its committed result idempotently. A crash between the authoritative
commit and projection is repaired from the receipt/checkpoint, not by rerunning
the project mutation or independently advancing a second scheduler.

### Private persistence and recovery

- Put the versioned chat namespace under the existing project-private v2 root
  using current path, locking, atomic-write and recovery primitives. Do not
  persist the current full-state serialization without a validated decoder.
- A committed chat event is identified by `(run_id, sequence)`. Bind turns,
  causal parent/source receipt, participant/session, timestamps, adapter version
  and negotiated capability profile explicitly when adding the durable schema.
- Persist the user-turn intent before sending it. Keep streamed answer fragments
  transient; only a completed, validated message becomes committed history.
- Rebuild from validated ordered events and authoritative task receipts. Replayed
  provider history cannot append duplicate completed messages or apply actions.
  Mark unfinished turns interrupted; use only advertised load/resume support.
  Unsupported continuation is shown as unavailable and leaves that adapter's
  required recovery acceptance open.
- Enforce the existing event/text bounds with an explicit full/terminal outcome;
  do not silently truncate history or discard evidence to make room. Reject
  unsupported future schemas without rewriting their bytes.
- Before enabling this store, update both data-lifecycle guides to distinguish
  Qiongli-visible chat from Host-private history. Specify bounded retention,
  stopped-writer backup, deliberate deletion, portable-export exclusion and
  diagnostic redaction. Reuse existing lifecycle/export owners and test them;
  this task does not introduce a whole-product backup or purge service.

## ACP runtime slice

- Pin `agent-client-protocol` and negotiate `ProtocolVersion::V1` even when the
  SDK package has a newer release number.
- Use fixed argument arrays and no shell. Development may launch the exact
  registry versions `@agentclientprotocol/codex-acp@1.9.0` and
  `@agentclientprotocol/claude-agent-acp@0.74.0` only when explicitly enabled.
- A packaged-product claim requires reviewed bundled sidecars; the development
  `npx` path is never presented as self-contained production support.
- Map ACP notifications into Qiongli events through one adapter layer. Do not
  leak ACP schema types into the project state or Svelte UI.
- Permission requests remain user-owned. The MVP never auto-approves a write or
  selects a broader permission option on the user's behalf.
- Retain the connection/session across prompts. Record advertised authentication,
  model/mode and load/resume capabilities; unsupported features remain explicit.
- Give each active turn an independent wakeable cancellation signal and bounded
  timeout/teardown. A silent Agent and a pending permission must be cancellable.
  Cancel pending permission responses, reconcile the terminal result and stop
  owned processes when graceful shutdown does not finish within the bound.
- Reuse the existing bounded background-operation pattern in `desktop.rs`
  (`start_mcp_self_test`/poll/cancel) where it fits; ACP needs its own retained
  session and control lifetime, not a general-purpose job framework.

## Narrow ACP session ownership contract (Stage 2b.1)

- `AcpV1Client::with_session` owns one connection runner, one initialization and
  one new provider session for the lifetime of an asynchronous callback. The
  callback borrows a Qiongli-owned `AcpV1Session`; SDK handles remain private.
  Returning/dropping this scope ends connection ownership. The session cannot
  escape the borrow or be cloned into concurrent prompt callers.
- `AcpV1Session::run_turn(&mut self, ...)` admits one prompt at a time. Each sent
  prompt receives a monotonically increasing session-local `turn_id` starting
  at 1, bounded to the JSON-safe integer range. Returned outcomes own separate
  event buffers. Prompt input rejection or pre-cancellation consumes no turn ID.
- Mark the session unavailable before awaiting a sent turn; only a validated
  terminal outcome makes it reusable. Failure or dropping an unfinished prompt
  future leaves it unavailable, so queued responses cannot enter another turn.
- Record the end of a prompt in SDK response dispatch. Reject session updates
  observed while idle between turns. Initial owned updates after session creation
  remain supported. ACP v1 updates do not carry a turn ID: local turn IDs identify
  serialized prompt windows, not proof of causal origin for traffic sent after
  the next prompt begins.
- No project/App lock is acquired by the session owner or held over an ACP await.
  Dispatch callbacks only validate/bind messages, update atomic signals and pass
  traffic onward; they never wait for future messages, a UI choice or a project
  writer. Future App wiring briefly takes its owner lock to reserve/read a job,
  releases it before session work, then reacquires it to publish bounded results.
- A prompt terminal event never completes the All Chat run. The existing first-
  coordinator projection accepts only turn 1; later-turn projection and durable
  App state remain separate increments.
- The default client still fails permissions closed. Stage 2b.3 adds an explicit
  participant control with exact connection/run/role/session/turn/request binding.
  Its short mutex sections never await SDK traffic or UI. SDK background tasks
  own permission responders; one-time choices, cancellation, expiration and
  connection loss retire them. Remembered choices remain disabled.
- Stage 2b.2 now races wakeable cancellation and absolute phase deadlines against
  silent protocol waits. Defaults are 30 s initialize, 30 s session creation,
  300 s prompt, 120 s permission and 2 s cancel grace. Timeout is not successful
  cancellation; only the Agent's matching acknowledgement confirms cancellation.
  SDK scope drop kills its Unix process group. The Windows `npx` preset is
  unavailable until a Windows Job owns wrapper descendants; in-process fixtures
  remain portable. No claim of live or packaged readiness follows these checks.
- Stage 2b.3 records advertised authentication, load/resume, modes/models and
  actual session establishment separately from Qiongli-enabled operations. It
  emits bounded transient text, plan/tool status, permission and lifecycle data;
  hidden reasoning and raw tool payloads are excluded. Connection nonces prevent
  replay when a recreated provider session reuses its IDs. The native contract
  and generated control/stream fixture live with `qiongli-execution`; the existing
  [ACP runtime contract](../../spec/native/runtime/acp-v1-client.md) owns bounds,
  error mappings and source validation requirements.
- App IPC, strict TypeScript decoding, restart persistence and the research/tool
  security boundary remain Stage 3 work. No ACP permission replaces a Qiongli
  project mutation's preview/approval/CAS decision.

## App integration

Use a separate versioned All Chat Tauri contract rather than mutating the frozen
App IPC schema 19 in place. Rust owns the new contract and emits one Draft
2020-12 schema plus golden fixtures; the TypeScript/Zod adapter consumes them.

The complete boundary has three separately identified payload classes:

| Class | Minimum behavior |
| --- | --- |
| Committed snapshot | Versioned run, participants, ordered committed messages and references; validated reload/reconstruction |
| Control request/result | Start/load, prompt, cancel and permission response, bound to project, run, participant, turn/request identity and expected generation |
| Transient update | Capability/readiness, streaming text, plan/tool activity, pending permission and turn status; never a project commit |

These are logical operations, not new schema 19 commands. Define the exact Rust
wire types and unknown/stale/duplicate behavior before adding their TypeScript
consumer. A permission choice must match its still-pending request, turn and
participant; reject stale or repeated responses. A turn ending leaves the run
open until a separate validated run transition closes it.

The first App interaction provides:

- supported-Agent and capability status;
- coordinator plus optional worker/reviewer selection;
- one prompt composer;
- a unified ordered timeline labelled by Agent and role;
- explicit pending-permission, failed, cancelled, and recoverable states;
- cancellation and reload.

Initially enable one coordinator; collaborator execution follows Stage 4. The
existing Orchestrator route is extended rather than replaced. Sources and
candidate artifacts reuse the existing Library, Capture, Graph v1 and Artifacts
views, with affected views refreshed after an approved project transition.

## Research context and tool bridge

Resolve project identity/revision through `ProjectStateService`; reuse the
canonical Workflow/Skill and existing role/worker/handoff builders for the
selected task and relevant evidence. Supply a bounded context projection and
fixed, project-scoped Full MCP configuration when creating/loading the session.
Do not assume the adapter inherits the user's existing Host plugins or tools.

Classify source documents as untrusted data and keep them separate from control
instructions. Qualify each adapter's native file/terminal policy before allowing
it to operate on canonical research artifacts: ACP permission alone does not
authorize a Qiongli project write. If this boundary cannot be enforced, keep
the affected write capability unavailable. Project changes continue through the
existing preview, approval, digest, evidence and CAS owners.

The first acceptance workflow is a source-linked literature comparison and
draft candidate using an existing project. Source links remain inspectable and
unsupported claims remain explicit. Any saved candidate or canonical change
uses the existing project workflow; chat text is never promoted automatically.

## Data flow

1. The user selects a project, relevant sources and one coordinator.
2. Qiongli binds the current revision, records the run/turn intent, and configures
   the private ACP session with bounded research context and Full MCP tools.
3. The Agent streams visible progress; permission and cancellation controls remain
   independently responsive. A further prompt reuses the same session.
4. A valid completed turn commits a source-linked response/candidate reference.
   Any approved project change passes existing owners and refreshes affected views.
5. On restart, reconstruct committed history and task projections, then restore
   only advertised provider session capabilities; interrupted work stays explicit.
6. After this journey passes, the coordinator may produce at most two assignments.
   Existing task/handoff owners validate them and receive bounded worker results;
   the coordinator synthesizes their structured projection.

## Compatibility and migration

- Existing host-driven runs and checkpoints remain readable and continue through
  External Host mode.
- No existing provider credential or host conversation is migrated into ACP.
- All Chat State starts at schema v1 in a distinct namespace. An unsupported
  future version fails closed and remains unmodified.
- The initial contract is additive to the product; it does not redefine existing
  App schema 19 messages.

## Security and privacy

- Agent launch commands are fixed, version-pinned, shell-free, and bounded.
- ACP stdout is protocol data; stderr is bounded diagnostic data and is never
  interpreted as authority.
- Unknown events, participants, sessions, task IDs, permissions, and stale
  generations fail closed.
- All Chat persistence contains only Qiongli-visible content. Secrets, approval
  tokens, hidden reasoning, and environment values are excluded and redacted.
- Workers cannot recursively spawn peers through Qiongli's coordinator model.

## Rollback

- Disable the ACP App entrypoint and retain External Host mode.
- Stop child processes and leave committed project state untouched.
- Preserve All Chat files as read-only evidence or remove only uncommitted
  prototype state; never rewrite existing orchestration checkpoints.
- Revert the new contract and adapter modules without changing project schemas.

## Trade-offs

- One coordinator is less flexible than a peer swarm but gives deterministic
  ownership, bounded cost, and a clear cancellation path.
- A development-only pinned `npx` adapter proves protocol viability quickly but
  does not satisfy packaged-product requirements.
- A separate All Chat App contract adds one narrow boundary while avoiding a
  risky rewrite of the broad frozen App schema.
