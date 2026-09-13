# Current-state and ACP evidence

## Repository evidence

- `docs/architecture/decisions/0211-host-driven-model-execution.md` assigns
  authentication, conversation, transport, approvals, and Agent behavior to an
  external host and prohibits Qiongli from launching or parsing Codex and Claude
  CLIs. ADR 0217 now supersedes that original external-Host-only restriction.
- `packages/qiongli-desktop/src/routes/orchestrator/+page.svelte` currently shows
  run and checkpoint state and hands work back to the host; it does not own an
  Agent transcript or prompt lifecycle.
- `packages/qiongli-native/crates/qiongli-execution/src/orchestration*.rs` and
  `worker_orchestration*.rs` already model deterministic run, task, role,
  attempt, and generation state.
- `packages/qiongli-native/crates/qiongli-execution/src/host_handoff.rs` and the
  Full MCP surface already bind handoffs and submissions to project revision,
  document digest, evidence, task, role, attempt, and generation.
- The current `solo`, `duo`, and `triad` profiles already provide the smallest
  useful user model for one coordinator plus up to two collaborators. They do
  not yet launch Agent processes.
- Roadmap items ORC-601--ORC-609 contain useful later controls, but the MVP needs
  only role separation, cancellation, recoverable state, and stale-write
  protection. Scientific disagreement policy and majority governance stay later.

## External protocol evidence

- ACP's [architecture](https://agentclientprotocol.com/get-started/architecture)
  defines JSON-RPC communication, normally over stdio, with streaming updates,
  bidirectional permission requests, and concurrent sessions.
- The stable [ACP v1 overview](https://agentclientprotocol.com/protocol/v1/overview)
  covers initialization, authentication, session creation/loading, prompts,
  updates, cancellation, permissions, files, terminals, plans, and tool calls.
- ACP exposes negotiated [session configuration options](https://agentclientprotocol.com/protocol/v1/session-config-options)
  such as model, mode, and reasoning choices.
- The official [Rust SDK](https://agentclientprotocol.com/libraries/rust) supports
  both sides of ACP. The Rust and TypeScript SDKs reached 1.0 in the
  [June 2026 release](https://agentclientprotocol.com/announcements/sdk-1-0-releases).
- The initial 2026-09-04 feasibility check pinned `agent-client-protocol` `2.1.0`
  as a test-only dependency. The later `a1a093d9` increment made it a normal
  dependency for the narrow development client. The in-process connection proved
  `ProtocolVersion::V1`, initialize, session creation, prompt, and streamed text
  without a process, network, credential, Node.js, Codex, or Claude dependency.
- [ACP v2](https://agentclientprotocol.com/announcements/acp-v2-draft) remains a
  draft and should not be the first production contract.
- The ACP-maintained [Codex adapter](https://github.com/agentclientprotocol/codex-acp)
  maps ACP to Codex App Server and advertises models, reasoning, approvals,
  sandboxing, tools, permissions, MCP, terminals, plan, review, and optional
  sub-Agent sessions.
- The ACP-maintained [Claude adapter](https://github.com/agentclientprotocol/claude-agent-acp)
  uses the Claude Agent SDK and advertises context, images, permissions, edit
  review, tasks, terminals, commands, and client MCP servers.
- The current ACP registry distributes both adapters through `npx`; packaged
  Qiongli therefore needs pinned bundled sidecars or an explicitly approved
  packaging alternative rather than a hidden Node.js prerequisite.

## Terminology result

No official ACP document defines an "All Chat State" primitive. Exact-phrase
searches find unrelated implementation descriptions rather than an interoperable
protocol contract. Qiongli should own and version this term locally.

## Minimal architectural conclusion

```text
Qiongli App / unified timeline
              |
All Chat State + existing task/evidence/CAS owners
              |
        ACP session manager
          /             \
  Codex ACP session   Claude ACP session
              |
      existing Full MCP / ToolHost
```

ACP should remain the Agent-session transport. All Chat State should be the
smallest Qiongli-owned event and orchestration layer that joins multiple private
sessions without cloning their entire hidden context or replacing existing
project transaction controls.

## September 4 implementation review

Review baseline: source through `4012ee13`, plus the working-tree Rust All Chat
App snapshot/schema/fixture changes. This is local review evidence, not program
acceptance or a live/packaged capability claim.

| Area | Observed evidence | Remaining work |
| --- | --- | --- |
| Stage 1 | Pure in-memory reducer, bounded roles/events and deterministic transitions | Validated persistence and task/checkpoint projection binding |
| Stage 2 | Offline v1 single-turn client, fixed development presets and atomic first-coordinator result projection | Retained sessions, auth/capabilities, interactive permission, wakeable cancel and live readiness |
| Stage 3 | Rust committed-snapshot schema and two golden fixtures | Control/stream contracts, TypeScript, Tauri, research tools and the user journey |

### P2: participant session namespaces are incorrectly global

`packages/qiongli-native/crates/qiongli-execution/src/all_chat.rs:278` rejects
any provider session string already held by another participant. Two independent
Agents may both return `session-1`; the second legitimate participant is rejected.
Static review confirmed the branch; existing tests use distinct strings.

Scope identity to `(run_id, role, provider_session_id)` in the bounded model.
Remove cross-participant string deduplication while retaining the same-role
reinitialization guard. Merely adding backend ID is insufficient when independent
processes use the same backend. Add both positive and same-participant negative
cases to the existing state test.

### P2: unknown-session updates do not reach the rejecting guard

`packages/qiongli-native/crates/qiongli-execution/src/acp.rs:287` reads through
SDK session routing before the local session check at `acp.rs:342`. A temporary
in-process Agent sent unknown-session text, valid-session text and EndTurn. The
expected rejection assertion failed (exit 101); the turn returned success because
SDK routing discarded the unknown-session notification before the guard saw it.

Validate owned session IDs at connection dispatch. Port this scenario into the
repository's existing deterministic ACP fixture; a successful later EndTurn must
not erase a protocol-boundary failure or return accumulated output.

### Integration and ordering gaps

- `acp.rs:261` creates a session with an empty MCP-server list, and the prompt
  path does not use the existing role/Workflow/handoff context builders in
  `apps/qiongli/src/orchestration_control.rs:275`.
- `acp.rs:199` consumes one client per turn; `acp.rs:285` explicitly defers
  wakeable cancellation. These are declared prototype limits, not completed
  session/recovery capabilities.
- `apps/qiongli/src/all_chat_api.rs:18` exposes only a committed snapshot. Its
  existence does not supply native control, streaming or permission interactions.
- All Chat event generations do not establish the existing handoff's attempt,
  checkpoint generation/document digest or evidence authority. Connect these
  owners before executing worker tasks; no production double-write defect was
  observed because that wiring does not yet exist.
- `docs/guide/data-lifecycle.md:15` assigns chats to Hosts. The new Qiongli-owned
  private event store needs explicit lifecycle/export policy before use.
- Current ACP permissions are rejected. No current permission bypass was found;
  source isolation and native-tool/project authority must be qualified before
  enabling research actions (`SEC-401`--`SEC-403`).

### Checks observed during the review

- Execution tests: 98 passed; existing All Chat App golden: 1 passed.
- Roadmap/ADR/public-schema tests: 38 passed; their validators passed.
- Execution Rust fmt and clippy with `-D warnings`: passed.
- Additional unknown-session regression: 1 failed, as described above.
- No live adapter, credential, network execution or packaged acceptance was run.

The updated PRD, design and implementation plan own the response to these
findings. The master roadmap owns the reordered horizon; this evidence file is
not an additional backlog or acceptance ledger.
