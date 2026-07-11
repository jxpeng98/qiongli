# ADR 0203: Agent Backend And Native Tool Host Boundary

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201C`
- Owners: Qiongli maintainers
- Decision scope: Model execution, agent events, tool policy, and orchestration

## Context

Qiongli's orchestrator must remain functional after Python and Node.js are
removed from the production path. Requiring an installed Codex, Claude, or
other external CLI would merely replace one runtime dependency with another.
Direct provider APIs are therefore first-class, while host-native and external
CLI integrations remain optional adapters.

Models can request tools that read research data, write project artifacts, call
MCP services, use the network, or execute commands. If each model backend owns
that tool loop, provider adapters silently acquire filesystem and shell power,
approval behavior diverges, and audit or cancellation cannot be enforced
consistently. Model transport and local tool execution need separate native
trust boundaries.

## Decision drivers

- provider-independent orchestration and deterministic test backends;
- direct API operation with no required external CLI;
- explicit backend capability negotiation before a run starts;
- one policy engine for project, shell, MCP, service, and network tools;
- bounded approvals, cancellation, limits, redaction, and audit;
- compatibility adapters that cannot weaken the native execution policy.

## Decision

### Typed AgentBackend protocol

`qiongli-agent-runtime` defines a versioned asynchronous `AgentBackend`
protocol. A backend declares its stable ID, protocol version, authentication
state, model and context limits, streaming support, structured-output support,
tool-call support, multimodal support, cancellation semantics, retry classes,
and any host-owned constraints.

The normalized request contains conversation input, bounded attachments,
response constraints, and tool schemas selected by policy. It does not contain
a filesystem handle, shell executor, raw credential, or direct ToolHost
capability. The normalized event stream distinguishes content deltas, reasoning
status where supported, tool requests, usage, retryable errors, terminal
errors, cancellation, and completion. Provider-specific fields remain inside
the adapter unless a versioned extension is declared.

The orchestrator checks required capabilities before starting a run. Missing
auth, unsupported tools, incompatible context, or unavailable cancellation
returns a structured preflight result rather than failing midway through a
research workflow.

Direct OpenAI and Anthropic HTTP/API implementations are first-class backends.
A deterministic fake backend is mandatory for tests. Host-native backends may
be added only through a documented host contract. Codex, Claude, Antigravity,
or other external CLIs are optional compatibility adapters and are never
installed, discovered, or invoked as an automatic production fallback.

### Orchestrator-owned tool loop

The orchestrator owns task state and the model/tool loop. A backend may emit a
normalized tool request, but it cannot execute the tool. The orchestrator sends
the request, run identity, declared purpose, and least-privilege context to
`AgentExecutionPolicy`. Only an allowed decision is forwarded to the native
`ToolHost`; the redacted result then returns to the backend as model input.

Policy evaluates the execution profile, project root, tool class, arguments,
read/write/network scope, user or administrator rules, required approval,
remaining limits, and run history. Decisions are `allow`, `deny`, or
`approval-required` with stable reason codes. A model cannot approve its own
request, expand its project root, change policy, or supply its own executable
search path.

### Native ToolHost boundary

`qiongli-tool-host` implements tool dispatch, sandbox enforcement, bounded
process execution, cancellation, output limits, redaction, and audit. Dangerous
or native OS tools execute in a reserved child mode of the canonical binary
defined by ADR 0201, using short-lived authenticated IPC created by the parent.
Pure read-only service calls may execute in-process only when their registered
tool class and policy explicitly allow it; they still use the same request,
limit, redaction, and audit contract.

The ToolHost uses an allowlisted registry of typed tools. It canonicalizes
project roots and arguments, rejects traversal and symlink/reparse escapes,
sets a minimal environment, never searches an untrusted current directory, and
applies platform sandboxing where available. Shell, process, network, secret,
and out-of-project writes require explicit policy classes and approval rules.
Lite profile never exposes arbitrary shell or agent-launch tools.

Every run has wall-clock, model-turn, tool-call, process, byte, network, and
artifact limits plus a cancellation token. Cancellation propagates from UI or
CLI through orchestrator, backend request, ToolHost, and owned child processes.
Audit records stable identities, decisions, hashes, timings, outcomes, and
redacted error classes; they never contain secrets or unrestricted model text.

## Alternatives considered

### Require Codex or Claude CLI for all agents

This reuses existing login and tool behavior but violates the dependency-free
product promise, makes orchestration unavailable without that CLI, and gives an
external process control of semantics and updates. Rejected as the core path.

### Let every backend implement its own tool loop

This follows provider SDK examples, but duplicates approvals, sandboxing,
limits, cancellation, and audit and lets transport code gain local authority.
Rejected.

### Give the model an unrestricted shell tool

This is flexible but cannot preserve research data, secrets, project
boundaries, or predictable unattended behavior. Rejected for every profile.

### Run every tool in the orchestrator process

This reduces IPC but increases the blast radius of a parser, command, or native
library failure. Rejected for dangerous tools; bounded pure service calls may
use the explicitly allowed in-process path.

### Put all tools in a permanent privileged daemon

A daemon adds credential, multi-user, service-update, and remote-control risk
before it is needed. Rejected for the native alpha topology.

## Consequences

Positive consequences:

- the orchestrator runs through direct APIs on a clean machine without an
  external agent CLI;
- provider adapters remain small transport/auth implementations and can be
  tested against one normalized event contract;
- tool access, approvals, cancellation, limits, and audit are consistent across
  direct, host-native, fake, and optional CLI backends;
- the Lite/Full distinction is enforceable as policy rather than convention.

Costs and limitations:

- streaming and tool-call semantics must be normalized across providers;
- the internal IPC and sandbox need target-specific security acceptance;
- direct API backends require user-supplied credentials and provider network
  access even though no language runtime is required;
- optional CLI adapters may expose fewer guarantees and cannot be advertised
  until their capability and cancellation behavior is verified.

## Security and privacy

- Backends receive only the content and tool schemas authorized for the run;
  credential resolution occurs inside the backend boundary through opaque
  secret references.
- Tool results and model-visible errors pass through size limits and redaction
  before entering prompts, logs, UI status, or audit records.
- ToolHost IPC is local, short-lived, authenticated, parent-bound, and closed
  after the run; it is not a general local RPC listener.
- Project-root, no-follow path, environment, command, network, and process-tree
  policies are revalidated immediately before execution.
- Approval records bind the normalized request digest and expire when arguments,
  policy, target, or run identity changes.
- Prompt instructions cannot change policy, mark approval complete, read the
  secret store, or disable audit and cancellation.
- Telemetry is absent by default and is never required for enforcement or
  recovery.

## Rollback

A run can be cancelled without changing backend or ToolHost policy state.
Interrupted project mutations use the domain or installer transaction rollback
owned by the invoked service. Audit preserves the last accepted result and
redacted failure without treating partial model text as a committed artifact.

If a direct backend regresses, disable only that backend capability and retain
the protocol, fake backend, orchestrator state, and ToolHost policy. An optional
verified adapter may be selected explicitly, but the product must not silently
fall back to an external CLI. If child-process isolation fails a target gate,
dangerous tools remain unavailable on that target until a superseding sandbox
decision passes; they do not move in-process.

## Acceptance tests

1. Protocol tests cover capability negotiation, auth readiness, streaming,
   structured output, tool requests, retry classes, usage, completion, and
   cancellation with deterministic fake backends.
2. Preflight rejects missing capabilities before provider or ToolHost side
   effects and returns stable redacted reason codes.
3. Direct-backend opt-in tests complete one end-to-end workflow without Codex,
   Claude, Python, Node.js, or another external CLI process.
4. Backend conformance tests prove no adapter can execute a local tool or obtain
   a ToolHost IPC capability directly.
5. Policy tests cover read, project write, out-of-project write, shell, process,
   network, MCP, secret, and service classes in Lite and Full profiles.
6. Approval tests bind request digests, expire on any material change, and prove
   that model output cannot satisfy or bypass approval.
7. Sandbox tests reject traversal, symlink/reparse escape, untrusted executable
   lookup, environment leakage, undeclared network access, and orphan children.
8. Limit and cancellation tests interrupt backend streams and complete process
   trees at every boundary while retaining a valid redacted audit record.
9. Canary tests prove credentials, authorization headers, private paths, and
   restricted research data do not enter logs, errors, fixtures, or audit.
10. Optional CLI adapter tests cover absent executable, incompatible version,
    auth failure, timeout, cancellation, malformed output, and explicit
    selection without automatic fallback.

## Follow-up tasks

- `AGT-201`: implement the protocol, fake backend, capability preflight, and
  normalized event/error model.
- `AGT-202`: implement direct OpenAI and Anthropic backends with redacted
  opt-in acceptance.
- `AGT-203`: implement policy, authenticated ToolHost IPC, sandbox, approval,
  limits, cancellation, redaction, and audit.
- `AGT-204`: add optional CLI adapters only after core direct operation works.
- `AGT-205`: publish the tested backend capability/advertising matrix.
- `ORC-201` through `ORC-203`: port orchestration through these boundaries.
- `MCP-204`: expose agent/orchestrator tools only through Full policy.
