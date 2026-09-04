# ADR 0217: App-Owned ACP Sessions And All Chat State

- Status: Accepted
- Date: 2026-09-04
- Task ID: `ARC-217`
- Owners: Qiongli maintainers
- Decision scope: App Agent connectivity, cross-Agent collaboration state,
  orchestration topology, External Host compatibility, and adapter packaging
- Supersedes: ADR 0211's external-Host-only default and prohibition on launching
  Agents during ordinary App workflows

## Context

ADR 0211 correctly moved Qiongli away from direct provider APIs and made Codex
and Claude Code responsible for authentication, conversation, and Agent behavior.
That decision prevented Qiongli from becoming a second unbounded model client,
but it also made the App primarily an installation, checkpoint, and status
surface. Users must leave the App to perform the actual Agent work.

ACP v1 now provides a typed process and session protocol for initialization,
authentication, prompts, streaming updates, permissions, files, terminals,
cancellation, and session loading. Qiongli also already owns the deterministic
task graph, roles, handoffs, evidence bindings, checkpoints, cancellation, and
project compare-and-swap rules needed to coordinate multiple Agents safely.

## Decision drivers

- make the Qiongli App a useful working surface instead of only a handoff view;
- preserve provider-owned authentication and private Agent context;
- connect Codex and Claude through one negotiated protocol instead of direct
  provider APIs or parsed terminal text;
- reuse existing orchestration, evidence, review, and project transaction owners;
- keep multi-Agent execution bounded, inspectable, cancellable, and recoverable;
- retain a lightweight daily-development and pull-request path;
- keep the packaged App independent of a user-installed Python or Node runtime.

## Decision

### The App is an ACP v1 client

The Qiongli App may launch fixed, reviewed ACP Agent adapters as child processes
and communicate over ACP JSON-RPC. It negotiates protocol and capabilities,
creates or loads private sessions, submits prompts, consumes typed updates,
presents permission requests to the user, and propagates cancellation.

The production path does not issue direct provider API requests, parse Codex or
Claude terminal prose, or silently fall back to another provider or transport.
Provider credentials, authentication flows, model transport, and hidden context
remain owned by each Agent and its adapter.

### Qiongli owns All Chat State

All Chat State is a versioned Qiongli contract, not an ACP extension. It records
the user-visible, provider-neutral collaboration history across one run and its
private ACP sessions: participant identity and role, messages, tasks, handoffs,
tool activity, permission decisions, evidence, status, cancellation, errors,
and synthesis.

Each committed event has stable identity, causal order, run and session binding,
and an exact generation. Qiongli can rebuild the visible state after restart.
Partial stream fragments are presentation state and never become committed
project truth merely because an Agent emitted them.

All Chat State does not copy or claim every provider-internal message, reasoning
trace, credential, approval token, or hidden context. Cross-Agent context is a
bounded projection of the current task, relevant evidence, explicit decisions,
and returned results.

### Multi-Agent topology is bounded

Every run has exactly one user-selected coordinator. The coordinator may assign
work to at most two worker or reviewer sessions. Only the coordinator can
delegate through Qiongli. Workers cannot recursively create peers or open an
unrestricted conversation with one another; they return bounded results to the
coordinator for synthesis.

Independent proposals may execute concurrently, but project mutation remains on
the existing preview, approval, revision, digest, evidence, and compare-and-swap
path. An Agent message, majority vote, or successful ACP turn never grants
project-write authority.

### Existing execution paths remain available

External Host mode remains supported for users who prefer native Codex or Claude
interfaces. Existing host-driven checkpoints and handoffs remain readable. The
App ACP path and External Host path share the same Qiongli project, task,
evidence, ToolHost, and Full MCP owners rather than creating separate project
formats or orchestration engines.

Development may explicitly use version-pinned `npx` adapters to prove protocol
behavior. A packaged macOS or Windows product claim requires reviewed bundled
sidecars or an equally self-contained runtime; development `npx` evidence cannot
satisfy that claim.

## Alternatives considered

### Keep External Host mode as the only execution path

Rejected as the product default because it leaves the App unable to provide the
integrated conversation and multi-Agent experience requested for the 2.x line.
It remains available as a compatibility and power-user path.

### Restore direct provider APIs

Rejected because it would duplicate provider authentication and transport,
weaken Agent-native capability negotiation, and revive the architecture that ADR
0211 intentionally removed. ACP is the only new default Agent transport.

### Give every Agent equal peer-to-peer authority

Rejected for the MVP because unrestricted delegation makes ownership,
cancellation, cost, conflict resolution, and loop prevention substantially less
predictable. A single coordinator supplies one auditable authority path.

### Copy the complete transcript into every session

Rejected because provider sessions have different hidden context and limits.
Full copying increases privacy exposure and context cost while still failing to
create a genuinely shared provider conversation.

## Consequences

Positive consequences:

- users can work with supported Agents without leaving the Qiongli App;
- Codex and Claude share one Qiongli-visible task and evidence context while
  retaining separate provider sessions;
- current orchestration and transaction safety remains useful;
- capability negotiation makes unsupported behavior explicit;
- one coordinator and two collaborators bound cost and failure propagation.

Costs and limitations:

- Qiongli now owns child-process lifecycle, session recovery, and conversation
  presentation;
- Codex and Claude adapters must be version-pinned and independently qualified;
- exact desktop or CLI feature parity cannot be guaranteed across Agents;
- packaged sidecars require separate macOS and Windows build evidence;
- All Chat persistence adds private user content that needs bounded retention and
  redaction rules.

## Security and privacy

- Agent commands use fixed executable and argument arrays without a shell.
- Unknown capabilities, protocol messages, participants, task assignments,
  permission choices, and stale generations fail closed.
- Permission requests remain user-owned; Qiongli never auto-selects a broader
  write or execution permission.
- Credentials, environment secrets, approval tokens, hidden reasoning, and
  absolute private paths are excluded from All Chat persistence and receipts.
- ACP Agent output remains untrusted candidate content. Project writes continue
  through existing ToolHost/Full MCP policy, preview, approval, evidence, and
  compare-and-swap validation.
- Dropping or cancelling a run terminates its owned child sessions without
  deleting committed project state or unrelated Host state.

## Rollback

If the App ACP path is unsafe or unreliable, disable its entrypoint and stop its
owned Agent processes. External Host mode, native CLI, Plugin/Skills, Full MCP,
project services, and existing checkpoints remain available.

Preserve versioned All Chat files as read-only evidence when possible. A rollback
must not rewrite project data, provider credentials, Host conversations, or
historical handoffs. A later ADR may replace the topology, but it cannot silently
reinterpret events already committed under All Chat State v1.

## Acceptance tests

1. One App-owned ACP v1 client completes initialize, create/load, prompt,
   update, permission, cancellation, failure, and resume tests against a
   deterministic credential-free Agent fixture.
2. One run accepts exactly one coordinator and at most two workers/reviewers;
   worker delegation, unassigned results, duplicate/out-of-order events, stale
   generation, and post-terminal updates fail closed.
3. Codex and Claude adapter journeys expose negotiated capability differences
   without claiming exact desktop or CLI parity.
4. Restart reconstruction yields the same committed All Chat State and marks an
   interrupted turn as recoverable or terminal without applying partial output.
5. Project mutations still reject missing approval, evidence, digest, revision,
   or current-generation bindings.
6. Packaged macOS and Windows acceptance uses self-contained adapter sidecars and
   succeeds without a separately installed Node.js runtime.

## Follow-up tasks

- `PLT-404`: implement the bounded ACP v1 client and first App round trip.
- `PLT-405`: implement All Chat State persistence, timeline, stale rejection,
  and resume.
- `PLT-406`: implement coordinator plus two collaborator sessions and recovery.
- `PLT-407`: measure the resulting latency, memory, payload, and UI-blocking
  behavior before defining budgets.
- `PLT-408`: freeze concurrency, lock order, job, and cancellation contracts from
  those measurements.
