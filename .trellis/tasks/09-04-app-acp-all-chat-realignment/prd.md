# Realign App around ACP and All Chat State

## Goal

Make the Qiongli App the primary place where a user can work with Codex,
Claude Code, and later ACP-compatible agents, while Qiongli coordinates those
independent sessions through one durable, inspectable **All Chat State**.

This becomes the next primary 2.x product work package. It supersedes the
assumption that meaningful model interaction must happen outside Qiongli,
without removing External Host mode or weakening Qiongli's existing project,
evidence, approval, and transaction boundaries.

## Definitions

- **ACP** is the transport and lifecycle boundary between the App and an agent:
  capability negotiation, session creation or loading, prompts, streaming
  updates, permission requests, cancellation, files, and terminals.
- **All Chat State** is Qiongli's provider-neutral collaboration state across a
  user-visible run and all participating ACP sessions. It records conversation,
  agent identity, task ownership, status, plans, tool activity, permissions,
  handoffs, evidence, decisions, cancellation, and final synthesis.
- **Agent session** is one provider-owned ACP conversation. Codex and Claude do
  not share a native conversation identifier or hidden model context.
- **Capability profile** is the tested subset of an agent's desktop or CLI
  experience that Qiongli exposes. Unsupported capabilities remain explicit;
  Qiongli does not claim exact provider parity.

## Background

- Accepted ADR 0211 currently makes the selected external host responsible for
  model execution, authentication, conversation, approvals, and Agent behavior.
  The current App therefore exposes orchestration state and a "continue in
  host" handoff instead of an integrated Agent conversation.
- Qiongli already owns the useful deterministic half of multi-Agent work: run
  and task state, role packets, evidence, checkpoints, compare-and-swap project
  transitions, review packets, and strict host handoffs. This work should be
  reused rather than replaced with a second orchestration framework.
- Stable ACP v1 provides a suitable client/Agent protocol and Rust SDK. ACP v2
  remains a draft and is not a production target for this task.
- "All Chat State" is not an ACP-defined protocol object. It is a Qiongli
  product concept whose meaning and authority must be defined here.
- The currently discoverable Codex and Claude ACP adapters are distributed via
  `npx`. That is acceptable for a development spike, but a packaged Qiongli App
  must not silently introduce a user-installed Node.js prerequisite.

## Requirements

### R1 -- Rebaseline the 2.x product direction

- Replace App-as-status-surface with App-as-ACP-client as the primary integrated
  experience for the next 2.x vertical slice.
- Add a new architecture decision that explicitly supersedes the incompatible
  parts of ADR 0211; preserve the useful ToolHost, MCP, project transaction,
  evidence, and approval boundaries.
- Reorder the master roadmap so ACP session integration plus All Chat State is
  `NOW`. Keep daily development, pull-request, cross-platform build, and release
  lanes independent; do not make the new feature an excuse for heavier gates.
- Keep External Host mode as a supported compatibility and power-user path.

### R2 -- Implement a bounded ACP v1 client

- Negotiate protocol and Agent capabilities instead of branching on provider
  names for behavior.
- Support the minimum complete session lifecycle: initialize, authenticate when
  advertised, create/load, prompt, stream updates, request permission, cancel,
  fail, recover, and resume.
- Support Codex and Claude through fixed, tested adapter versions before opening
  an arbitrary registry or plugin surface.
- Expose model, mode, reasoning, tool, and session choices only when the active
  Agent advertises them.
- Record adapter version and negotiated capabilities so restored runs remain
  explainable.

### R3 -- Give Qiongli one authoritative All Chat State

- Persist an append-only logical event history with stable event IDs, causal
  parentage, run and Agent-session identity, timestamps, and explicit lifecycle
  state.
- Represent user and Agent messages, streaming completion, plans, tool calls,
  tool results, permission and elicitation requests, task delegation, handoffs,
  evidence links, decisions, cancellation, errors, and final synthesis.
- Rebuild the user-visible timeline and active task state deterministically after
  App or adapter restart without treating partial streamed content as committed
  project state.
- Keep provider-owned opaque session data separate from Qiongli's shared project
  state and never fabricate a single provider-native conversation across Agents.
- Define bounded context projection between Agent sessions. Copying every raw
  transcript and tool result into every Agent is not the default.

### R4 -- Connect multiple Agents through existing orchestration controls

- Give exactly one user-selected coordinator Agent the delegation authority for
  each run. Let it use up to two worker or reviewer Agent sessions, including
  sessions backed by different ACP Agents.
- Route delegation and return values through Qiongli-owned task, handoff,
  evidence, revision, and digest records; do not add an untracked peer-to-peer
  message bus.
- Do not let workers recursively delegate or start unrestricted conversations
  with one another. They return bounded results to the coordinator.
- Allow independent proposals to run concurrently, but keep Qiongli project
  mutation on the existing review, approval, and compare-and-swap path.
- Propagate cancellation to active child sessions and surface partial, failed,
  blocked, and recoverable work without losing already committed evidence.
- Keep the user as final authority for permission prompts and material project
  changes.

### R5 -- Present one coherent App experience

- Provide an integrated prompt surface, streaming timeline, Agent identity and
  status, plan and tool-call rendering, permission controls, cancellation, and
  resume in the Qiongli App.
- Show coordinator and child-Agent activity in one ordered timeline while
  preserving inspectable per-Agent branches.
- Make unavailable Agent features explicit instead of silently degrading or
  claiming equivalence with the corresponding desktop or CLI.
- Reuse the current Orchestrator screen and state owners where practical; do not
  create a parallel App solely for ACP.

### R6 -- Preserve lightweight packaging and delivery

- Keep secrets and provider authentication in the Agent boundary; do not store
  provider API keys in All Chat State.
- Ship pinned, reviewable adapter sidecars or an equally self-contained runtime
  before calling the feature packaged-product complete on macOS or Windows.
- Use focused checks during implementation. Cross-platform packaged builds and
  end-to-end Agent journeys belong to their existing build and acceptance lanes.
- Do not add a hosted coordinator service, a new general-purpose event platform,
  or a second project database for this MVP.

## Acceptance Criteria

- [ ] A superseding ADR and the master roadmap make ACP plus All Chat State the
      next primary 2.x vertical slice and retain External Host mode explicitly.
- [ ] The App can negotiate capabilities and complete create/load, prompt,
      streaming, permission, cancellation, restart, and resume journeys with the
      pinned Codex and Claude ACP adapters.
- [ ] One user-visible run can delegate bounded work to two Agent sessions and
      receive both results through the existing Qiongli task and evidence model.
- [ ] The unified timeline identifies every Agent and causally orders messages,
      tasks, tool activity, permissions, handoffs, failures, and synthesis.
- [ ] Restarting the App or an adapter reconstructs the same committed All Chat
      State and exposes any interrupted turn as recoverable or terminal.
- [ ] Concurrent proposals cannot silently overwrite newer project state; the
      current revision, digest, evidence, review, and approval checks still own
      project mutation.
- [ ] Unsupported Agent capabilities are disabled or labelled, and the tested
      capability profiles make clear that exact desktop/CLI parity is not claimed.
- [ ] Packaged macOS and Windows acceptance runs do not depend on a separately
      installed Node.js runtime.
- [ ] Focused tests cover state reduction, duplicate and out-of-order updates,
      child-session cancellation, restart recovery, and stale project writes.

## MVP Boundary

- Stable ACP v1 only.
- Codex and Claude only, using pinned adapters.
- One coordinator plus at most two worker/reviewer sessions per run.
- One Qiongli-owned timeline with structured cross-Agent context and handoffs.
- Existing Full MCP and ToolHost surfaces remain the only Qiongli project tool
  authority.

## Out of Scope

- ACP v2 production support or an arbitrary ACP Agent marketplace.
- Unlimited swarms, self-replicating Agents, or unrestricted Agent-to-Agent
  chatter.
- Making every Agent ingest every raw transcript, reasoning trace, or tool
  result.
- Exact visual or behavioral equivalence with every Codex or Claude desktop and
  CLI feature.
- Majority-vote governance, autonomous conflict merging, or a new hidden model
  used only to synthesize worker output.
- A cloud-hosted Qiongli control plane, account synchronization, publication,
  signing, or release automation changes.

## Key Decisions

- ACP owns Agent connectivity; Qiongli owns cross-Agent collaboration state and
  project authority.
- Reuse the current orchestration, handoff, evidence, and transaction machinery
  rather than introducing another orchestration engine.
- Treat All Chat State as a Qiongli product contract, not an extension to ACP.
- Preserve private provider sessions and project bounded views; a unified UI does
  not imply a fabricated shared model context.
- Use one coordinator with up to two bounded worker/reviewer sessions. Workers
  cannot recursively delegate or form a symmetric shared room.
- Keep the first implementation deliberately small and capability-driven.
