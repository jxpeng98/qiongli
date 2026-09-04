# Current-state and ACP evidence

## Repository evidence

- `docs/architecture/decisions/0211-host-driven-model-execution.md` assigns
  authentication, conversation, transport, approvals, and Agent behavior to an
  external host and prohibits Qiongli from launching or parsing Codex and Claude
  CLIs. App-owned ACP sessions therefore require a superseding decision.
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
