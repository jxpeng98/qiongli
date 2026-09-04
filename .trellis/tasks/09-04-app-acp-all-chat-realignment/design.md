# ACP and All Chat State design

## Architecture and boundaries

The first vertical reuses the existing native orchestration owners instead of
adding a second scheduler:

```text
Orchestrator page
      |
All Chat App contract
      |
AllChatStateV1 reducer ---- existing task / handoff / evidence / CAS owners
      |
ACP session client (protocol v1)
      |                         |
pinned Codex adapter     pinned Claude adapter
```

- ACP owns process transport, capability negotiation, private Agent sessions,
  prompts, updates, permission requests, and cancellation.
- `AllChatStateV1` owns only Qiongli-visible collaboration state. It never
  claims to contain the Agent's hidden context or provider credential.
- The existing `OrchestrationPlanV1`, role/worker packets, handoff bindings,
  evidence references, and project compare-and-swap checks remain authoritative.
- Full MCP and ToolHost remain the only Agent-accessible Qiongli project tools.

## Roadmap realignment

Do not add another family of long-lived task IDs. Reuse the existing platform
slice whose real IPC and recovery work is still open:

- `PLT-401`--`PLT-403` remain accepted and move from stale `NOW` prose to a
  recently closed line.
- `PLT-404` becomes the ACP v1 process/session and first App round-trip slice.
- `PLT-405` becomes the unified All Chat State, stale-state, and resume slice.
- `PLT-406` becomes bounded coordinator/worker execution plus cancellation,
  crash, and restart recovery.
- `PLT-407`--`PLT-408` remain next because budgets and concurrency policy should
  be measured against the new ACP path, not the retired App shape.
- `PILOT-702` is corrected from "without sharing conversation state" to sharing
  bounded Qiongli collaboration state while retaining private provider sessions.
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

## App integration

Use a separate versioned All Chat Tauri contract rather than mutating the frozen
App IPC schema 19 in place. Rust owns the new contract and emits one Draft
2020-12 schema plus golden fixtures; the TypeScript/Zod adapter consumes them.

The first App interaction provides:

- supported-Agent and capability status;
- coordinator plus optional worker/reviewer selection;
- one prompt composer;
- a unified ordered timeline labelled by Agent and role;
- explicit pending-permission, failed, cancelled, and recoverable states;
- cancellation and reload.

The existing Orchestrator route is extended rather than replaced.

## Data flow

1. The user selects a project, coordinator, and optional collaborators.
2. Qiongli creates the bounded plan and commits the run-start event.
3. ACP creates one private session for each selected participant.
4. The coordinator receives the user request and produces bounded delegations.
5. Qiongli validates and routes at most two assignments.
6. Worker/reviewer results return through structured task-result events.
7. The coordinator receives a bounded projection of those results and produces
   the user-visible synthesis.
8. Any project mutation still uses Full MCP/ToolHost preview, approval, revision,
   digest, evidence, and compare-and-swap checks.

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
