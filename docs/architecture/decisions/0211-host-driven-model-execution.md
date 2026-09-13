# ADR 0211: Host-Driven Model Execution

- Status: Accepted
- Date: 2026-07-23
- Task ID: `ARC-211`
- Owners: Qiongli maintainers
- Decision scope: Default model ownership, Full MCP host handoff, orchestration,
  client integration, and Alpha.2 acceptance
- Supersedes: ADR 0203's requirement that direct provider APIs are the core
  production model path

## Context

ADR 0203 separated model transport from local tool authority and selected
direct provider APIs as Qiongli's primary execution path. R4D implemented that
boundary with a normalized `AgentBackend`, a direct OpenAI adapter, OS
credential storage, a bounded runner, ToolHost policy, App and Full MCP model
surfaces, and a live-provider acceptance harness.

The product owner subsequently clarified a different product boundary:
Qiongli is the installation, project, workflow, and orchestration shell used
inside Codex, Claude Code, and supported Desktop hosts. The user should work in
the selected host's application. That host already owns model authentication,
conversation state, model transport, approvals, and native agent behavior.
Qiongli should install its native CLI, Plugin, Skills, and Full MCP service,
then provide deterministic project and orchestration capabilities to the host.

Making Qiongli a second model client duplicates authentication and conversation
UX, shifts the user out of the selected host, and does not guarantee semantic
correctness. Direct provider transport can bound requests and tools, but model
text remains probabilistic. Correctness comes from structured project reads,
revision-bound task packets, evidence references, candidate validation, review
gates, and explicit project mutation.

## Decision drivers

- preserve Qiongli's role as one installable shell across supported hosts;
- keep model credentials and conversations in the user's selected host;
- use one structured Full MCP boundary instead of parsing CLI prose;
- retain deterministic project, ToolHost, checkpoint, recovery, and review
  authority in native Rust;
- support host-native agents without pretending MCP can create them itself;
- avoid an automatic fallback that silently changes model or credential paths;
- distinguish local Desktop installation from cloud/web delivery.

## Decision

### The model host owns model execution

Codex, Claude Code, or a separately qualified Desktop host owns model
authentication, model selection, conversation state, network transport, native
approvals, and native agent execution. Qiongli's Plugin and Skills instruct the
host to call the native Full MCP service over a documented host-supported
transport.

During an ordinary workflow Qiongli does not:

- ask the user for a model-provider API key;
- issue a direct model-provider request;
- launch `codex`, `claude`, or another model CLI;
- parse a host CLI's free-form terminal output;
- claim that a local install also provisioned a remote/cloud worker.

A client executable may be launched only by an explicit install/activation
acceptance harness or explicit user launch action. That action is not the
workflow execution protocol.

### Full MCP carries a versioned host handoff

The native execution crate defines a provider-independent host-handoff
protocol. A handoff binds:

- host adapter and Full MCP protocol versions;
- run, project, semantic revision, task, role, and attempt;
- checkpoint generation and exact document digest;
- workflow, profile, and task-packet digests;
- bounded canonical instructions and candidate kind;
- allowed Qiongli tools, evidence requirements, and execution limits.

The host returns a bounded candidate envelope bound to the exact handoff. It
contains the candidate, Qiongli ToolHost audit/evidence references, and
declared conflicts or evidence gaps. Candidate content is untrusted. Qiongli
revalidates the project revision, checkpoint compare-and-swap identity, schema,
limits, allowed evidence, and review requirements before advancing state.

Raw host conversations, conversation IDs, provider credentials, provider
endpoints, absolute project paths, and host approval tokens are not part of the
handoff or durable checkpoint.

### The Orchestrator is a deterministic control plane

Qiongli owns the workflow contract, task DAG, role and worker packets,
checkpoint persistence, compare-and-swap transitions, recovery, cancellation,
candidate validation, review gates, and approval-gated artifact preview/apply.
It returns the next executable packet to the host and accepts a candidate for
the current packet. It does not need an `AgentBackend` to advance deterministic
state.

Codex and Claude Code adapters may map worker or reviewer packets to
host-native agents only when that host exposes the capability. Otherwise the
Plugin uses the same packet in a truthful single-agent flow. Full MCP never
reports that it launched a host-native agent.

### Direct backends are isolated experiments

Existing direct `AgentBackend` implementations, provider configuration,
Keychain integration, runners, and provider harnesses are retained only as
non-advertised experimental implementation. They are disabled in the ordinary
App, CLI, Full MCP, and release path and are never an automatic fallback.

Existing stored direct-backend configuration is decoded for compatibility and
is never removed automatically. The product retains a redacted status and
explicit credential-removal path. Re-enabling standalone direct execution
requires a separate accepted product decision and acceptance plan.

### Client and cloud boundaries remain truthful

The Qiongli App owns preview, install, verify, repair, and removal of the native
CLI, Plugin, Skills, and Full MCP projection through documented local host
mechanisms. Host registration, enablement, trust, restart, and activation
remain distinct host-owned states.

A local App cannot install into Codex Cloud, Claude web, or another remote
worker. Those surfaces require a documented repository bundle, host upload,
remote MCP, or the separate `REM-201` service boundary.

## Alternatives considered

### Keep the direct provider as the default

Rejected. It implements Qiongli as another model client, duplicates credentials
and conversation UX, and contradicts the intended host-shell product.

### Let Qiongli launch installed model CLIs for every workflow

Rejected as the primary path. It reuses host login but makes Qiongli own child
process discovery, version drift, output parsing, cancellation, and terminal
semantics. It also moves execution outside the user's active host conversation.

### Let the host write project files directly

Rejected. Host model output is untrusted candidate input. Project mutations
retain Qiongli's revision checks, preview, approval, transaction, receipt, and
rollback boundaries.

### Treat MCP client metadata as authorization

Rejected. Client name, version, and declared capabilities are useful
compatibility evidence but do not grant filesystem, tool, or mutation
authority.

### Automatically fall back to the direct backend

Rejected. A silent fallback changes credential ownership, provider transport,
conversation location, and privacy behavior. An unavailable host must produce a
structured recovery state.

## Consequences

Positive consequences:

- users stay inside Codex, Claude Code, or the selected Desktop host;
- Qiongli stores no default model-provider credential;
- communication uses typed MCP schemas instead of terminal-text parsing;
- project, ToolHost, orchestration, review, and mutation boundaries remain
  provider-independent;
- host-native agent capabilities can be used without becoming a Qiongli runtime
  dependency;
- direct-provider work can remain isolated without blocking the product path.

Costs and limitations:

- each advertised host needs an exact Plugin/Skill mapping and real activation
  evidence;
- model prose differs across hosts and cannot be asserted byte-for-byte;
- MCP disconnect and host cancellation need explicit recoverable checkpoint
  behavior;
- local installation cannot make a remote/cloud session active;
- the App must migrate away from its current Model Backend and prompt/result
  presentation.

## Security and privacy

- Full MCP accepts only closed, bounded handoff and candidate schemas.
- Every handoff and submission binds project identity, semantic revision,
  checkpoint generation, document digest, task packet, role, and attempt.
- Tool evidence references are accepted only from the current run and allowed
  ToolHost inventory; model claims cannot manufacture completed evidence.
- Candidate text, prompts, tool results, host conversation IDs, and provider
  credentials are excluded from durable checkpoints and redacted receipts.
- Project writes remain a separate preview and explicit approval transaction.
- Host metadata never bypasses Qiongli policy or host-owned trust controls.
- No direct-provider live test or formal cybersecurity scan is required to
  accept the default host-driven product path.

## Rollback

If a host adapter regresses, disable only that host claim and retain the native
CLI, project services, Full MCP reads, checkpoints, and recovery UI. Do not
fall back to a direct backend.

Failed Plugin installation or repair uses the existing receipt-owned rollback
and preserves unmanaged client content. A failed candidate submission leaves
the prior compare-and-swap checkpoint unchanged. Existing experimental backend
configuration remains disabled and removable.

## Acceptance tests

1. Default App, CLI, Full MCP, and release artifacts advertise no direct model
   backend, provider connection test, prompt execution, or automatic CLI
   fallback.
2. Host runtime, handoff, evidence, and candidate contracts reject unknown
   fields, oversized values, non-canonical JSON, stale checkpoint identities,
   changed project revisions, role substitution, and unoffered tools.
3. Copied-binary Full MCP can start, hand off, submit, recover, and cancel a
   workflow with an empty `PATH` and no provider credential or model network.
4. Real Codex and Claude Code Plugins each activate the same Full MCP contract
   and complete an evidence-grounded fixture workflow.
5. Exact model prose is not asserted. Acceptance asserts task identity, project
   reads, evidence references, candidate schema, checkpoint transition, review
   result, and mutation approval boundary.
6. Qiongli direct model-request and Qiongli-owned model-CLI child counts remain
   zero during host-driven acceptance.
7. Existing direct-backend settings remain readable and can be explicitly
   removed without exposing the credential.
8. Local Desktop and cloud/web states remain distinct; local materialization
   cannot produce a remote-active receipt.

## Follow-up tasks

- `R4D-H0`: remove direct execution from default product surfaces.
- `R4D-H1`: freeze host runtime, handoff, evidence, and candidate contracts.
- `R4D-H2`: expose start/next/submit through Full MCP.
- `R4D-H3`: qualify Codex Plugin execution.
- `R4D-H4`: qualify Claude Code Plugin execution.
- `R4E-H5`: replace Model Backend UI with host and workflow status.
- `R4E-H6`: separately qualify supported Desktop packaging.
- `R4E-H7`: close Alpha.2 with real host-driven receipts.

## Primary references

- [ADR 0203](0203-agent-backend-and-tool-host.md)
- [ADR 0206](0206-declarative-install-plan-and-client-trust.md)
- [ADR 0210](0210-tauri-svelte-desktop-presentation.md)
- [R4 host-driven execution plan](../../superpowers/plans/2026-07-23-qiongli-r4-host-driven-runtime-realignment.md)
