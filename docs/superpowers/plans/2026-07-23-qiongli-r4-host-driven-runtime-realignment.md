# Qiongli R4 Host-Driven Runtime Realignment Execution Plan

Status: in progress; H0 containment, the H1 task-role checkpoint, the H2
generic Full MCP host service, the H3 Codex Plugin/native host mapping, the H4
Claude Code Plugin/native host mapping, and the H5 Desktop App information
architecture are implemented. H6 local Desktop packaging and remote-boundary
qualification has an implemented current-host package contract; real
Claude Desktop installation observation and release promotion remain external
acceptance. H7 now has a fixed fixture, candidate schema 2, a redacted receipt
contract, and an offline copied-binary preflight; real Codex and Claude Code
receipts remain manual acceptance.

Date: July 23, 2026

Target branch: `feat/r4b-ui-localization-polish`

Roadmap:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

Architecture dependency:
ADR 0211 is accepted and supersedes ADR 0203's direct-provider-as-default
decision. ADR 0203 remains historical authority for the isolated experimental
backend boundary.

## Goal

Restore the intended Qiongli product boundary:

- Qiongli is the native App, CLI, Plugin/Skills installer, Full MCP server,
  project service, and deterministic Orchestrator;
- Codex, Claude Code, or a separately qualified Desktop host owns model
  authentication, conversation state, model transport, and native agent
  execution;
- the host calls Qiongli through Plugin + Full MCP;
- Qiongli never treats free-form model text as an authoritative project
  mutation;
- no Qiongli-owned provider credential or direct provider request is required
  for the default R4 workflow.

The first closure target is local Codex plus local Claude Code on macOS. Claude
Desktop is a separately packaged local-host follow-up. Codex Cloud, Claude web,
and other remote workers remain out of scope until a repository-bundle or
remote-MCP contract is approved.

## Why This Rebaseline Is Required

The implemented R4D path currently:

- exposes an OpenAI Responses backend and Keychain credential lifecycle;
- lets the App and Full MCP ask the direct backend to execute a project query;
- lets the native Orchestrator drive model roles through that backend;
- gates R4D on a live provider receipt.

That is internally bounded, but it implements Qiongli as another model client.
The intended product instead installs Qiongli into an existing model host and
lets the user work in that host. Direct-provider execution also cannot by
itself guarantee semantic correctness: model prose remains probabilistic.
Correctness must come from structured handoff, project/revision binding,
evidence-backed tools, candidate validation, review gates, and explicit
mutation approval.

## Target Architecture

```text
User
  |
  v
Codex / Claude Code / qualified Desktop host
  |- owns login, model, conversation, approvals, and native agents
  |- loads Qiongli Plugin and Skills
  |
  v
Qiongli Full MCP over host-supported local transport
  |- Research Library / Capture / Academic Graph services
  |- versioned host handoff and candidate submission
  |- project-scoped ToolHost policy and redacted audit
  |
  v
Qiongli Orchestrator
  |- task DAG and role packets
  |- project + semantic-revision binding
  |- checkpoints, compare-and-swap, pause/recovery/cancel
  |- worker barriers, review plans, and quality gates
  |- approval-gated artifact preview/apply
```

The dependency direction is important: the host launches or connects to
Qiongli Full MCP. Qiongli does not launch a model CLI during an ordinary
workflow.

## Reuse And Reclassification

| Existing implementation | Disposition |
|---|---|
| Research Library, Capture, Academic Graph, Full project services | Keep as production authority |
| Project-scoped read ToolHost, policy, limits, redaction, audit | Keep as production boundary |
| Task DAG, role packets, checkpoints, worker plans, review gates | Keep and adapt to host handoff |
| App/CLI client inventory and Plugin install | Keep and extend to Full MCP activation |
| Deterministic fake `AgentBackend` | Keep for internal compatibility tests only |
| Direct OpenAI adapter and bounded runner | Disable and isolate as experimental |
| OpenAI Keychain save/test UI | Remove from default product; preserve explicit credential removal |
| `qiongli_agent_*` Full MCP tools | Remove from default tool inventory |
| Direct-backend orchestration test/continue tools | Replace with host start/next/submit tools |
| R4D live provider acceptance harness | Retire from R4; do not run |
| Model Backend page | Replace with host execution status owned by Client Integrations |
| Orchestrator page | Keep as workflow state/recovery/review UI, not a chat UI |

No first batch deletes stored configuration or Keychain material. An upgrade
must show only a redacted legacy-credential-present state and provide an
explicit removal action. Qiongli must not silently reuse that credential.

## Host Handoff Contract

Batch H1 freezes a provider-independent protocol before changing adapters.

### `HostRuntimeDescriptorV1`

Records display and compatibility evidence only:

- host family and version;
- Qiongli adapter version;
- Full MCP protocol version;
- declared single-agent, native-subagent, attachment, and structured-output
  capabilities;
- observed Plugin, registration, enablement, trust, and active states.

Host identity and declared capabilities do not grant filesystem or mutation
authority. All Qiongli policy checks still apply.

### `OrchestrationHandoffV1`

Contains only:

- run, project, semantic revision, task, role, attempt, and checkpoint
  identities;
- exact workflow/profile/task-packet digests;
- bounded canonical instructions and output schema;
- allowed Qiongli tool IDs and evidence requirements;
- candidate byte, tool-call, time, and retry limits;
- a handoff digest bound to the current checkpoint generation and document
  SHA-256.

It contains no provider credential, provider endpoint, host conversation ID,
absolute project path, raw prior conversation, shell authority, or approval
grant.

### `HostCandidateEnvelopeV1`

The host returns:

- the exact handoff digest;
- a bounded typed candidate;
- cited Qiongli tool audit IDs and project evidence digests;
- declared unresolved conflicts, evidence gaps, and limitations;
- optional reviewer/verifier result when requested by the current packet.

Candidate content is untrusted input. Submission rechecks the project revision,
checkpoint generation, document SHA-256, allowed tool evidence, size limits,
and required schema before advancing state. Raw model conversation is not
persisted. Artifact mutation remains a separate preview/approval/apply
transaction.

## Proposed Full MCP Surface

The exact schemas are frozen in Batch H1. The intended default tool family is:

- `qiongli_orchestration_doctor`: project, contract, host, and interrupted-run
  readiness without a model or network request;
- `qiongli_orchestration_runs`: redacted checkpoint discovery;
- `qiongli_orchestration_start`: create a revision-bound run and return the
  first host handoff without executing a model;
- `qiongli_orchestration_next`: return the current handoff for an exact
  unchanged checkpoint;
- `qiongli_orchestration_submit`: validate one host candidate/evidence envelope
  and advance the exact checkpoint;
- `qiongli_orchestration_action`: pause, recover, resume, or cancel through the
  existing compare-and-swap boundary;
- worker and artifact-review operations reuse the same handoff/submit pattern
  rather than creating another model transport.

Existing project, capture, graph, and ToolHost reads remain available. Default
Full MCP does not advertise `qiongli_agent_backend_status`,
`qiongli_agent_backend_test`, or `qiongli_agent_run`.

## Execution Rules

1. Preserve one rolling R4 branch and cohesive Conventional Commit checkpoints.
2. Change architecture authority before changing production behavior.
3. Add failing contract/tool-inventory tests before removing a default surface.
4. Do not automatically migrate from host execution to direct-provider
   execution.
5. Do not invoke Codex or Claude Code from Qiongli during an ordinary workflow.
6. Real client CLI invocation is allowed only in an explicitly selected
   install/activation acceptance harness or user launch action.
7. Do not delete a stored direct-backend credential automatically.
8. Preserve host-controlled trust, enablement, restart, and administrator
   actions as distinct states.
9. Assert structured sequence and evidence, not exact model prose.
10. Use normal contract, unit, integration, UI, and target acceptance tests.
    No formal cybersecurity scan is added to this rebaseline.

## Batch H0 — Architecture Authority And Default-Surface Containment

Purpose: stop the default product from advertising the wrong execution model
before introducing the replacement protocol.

Status: implemented on July 23, 2026. Direct model tools and controls are
absent from default Full MCP, App capabilities, navigation, and ordinary CLI
help/dispatch. Existing configuration remains decode-compatible and old
credential removal remains explicit.

Implementation:

- add ADR 0211 for host-driven execution and mark the direct-provider-as-core
  portions of ADR 0203 superseded for the default product;
- update the ADR index, CLI/MCP reference, native README, local desktop build
  guide, and capability matrix;
- introduce an explicit compile/runtime experimental boundary for direct
  provider code, disabled in ordinary App, CLI, Full MCP, and release builds;
- remove `qiongli_agent_*` from the default Full MCP tool list;
- remove backend set/test/run commands from ordinary help and capabilities;
- keep configuration decoding compatible with existing settings;
- keep one explicit redacted legacy credential status/removal path;
- turn the old R4D provider harness into a historical non-gating fixture or
  delete its package script after its deterministic privacy tests are moved to
  the retained boundary.

Primary files:

- `docs/architecture/decisions/0211-host-driven-model-execution.md`;
- `docs/architecture/decisions/README.md`;
- `packages/qiongli-native/apps/qiongli/src/mcp.rs`;
- `packages/qiongli-native/apps/qiongli/src/command.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop_api.rs`;
- `packages/qiongli-native/crates/qiongli-config/`;
- `scripts/r4d_live_acceptance.mjs`;
- `package.json`.

Focused acceptance:

- default Full MCP tool inventory contains no direct-backend tool;
- ordinary App/CLI snapshots contain no enable/save/test/run capability;
- existing settings still load and a stored credential can be explicitly
  removed without exposing its value;
- startup, status, project reads, and orchestrator inspection make no model
  provider request and launch no model CLI.

Checkpoint H0:

The shipped surface truthfully says that model execution belongs to an
installed host. Direct-provider code cannot be selected accidentally.

## Batch H1 — Host Handoff And Candidate Contracts

Purpose: freeze the structured correctness boundary independently from Codex
or Claude Code.

Status: task-role checkpoint complete. `host_handoff.rs` now provides canonical, closed
`HostRuntimeDescriptorV1`, `OrchestrationHandoffV1`,
`HostEvidenceReferenceV1`, and `HostCandidateEnvelopeV1` contracts with bounds,
digest binding, evidence allowlisting, debug redaction, and deterministic
round-trip/substitution tests. `EmbeddedWorkflowHostHandoffBuilder` emits
contract-grounded primary/reviewer/verifier packets without an `AgentBackend`,
and `FullOrchestrationService` performs host-bound start/reissue/submit
transitions using exact generation and document CAS. Worker/subagent packet
mapping remains deliberately coupled to the real host adapters in H3/H4.

Implementation:

- add `HostRuntimeDescriptorV1`, `OrchestrationHandoffV1`, and
  `HostCandidateEnvelopeV1` in `qiongli-execution`;
- adapt role, worker, synthesis, review, and verifier input builders to emit
  handoffs without constructing a backend request;
- add candidate schema, byte/depth/count limits, evidence references, and
  digest binding;
- add deterministic transitions for start, handoff, submit, reject, retry,
  review, pause, recovery, cancellation, and ready-for-apply;
- keep raw candidate text in memory only until validation or artifact-preview
  construction; persist the existing bounded hashes and state;
- reject host, project, revision, run, role, attempt, generation, document,
  packet, evidence, or candidate substitution.

Primary files:

- `packages/qiongli-native/crates/qiongli-execution/src/host_handoff.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/orchestration.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/orchestration_input.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/orchestration_runtime.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/worker_orchestration.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/worker_orchestration_input.rs`;
- `packages/qiongli-native/crates/qiongli-execution/src/worker_orchestration_runtime.rs`.

Focused acceptance:

- deterministic round trips for every solo/duo/triad role and B1/H3 worker
  packet;
- stale checkpoint and changed evidence rejection before state mutation;
- malformed, oversized, missing-evidence, and unexpected-role candidates fail
  closed;
- checkpoint documents contain no candidate body, prompt, conversation ID,
  credential, or absolute path;
- fake hosts cover single-agent and native-subagent capability shapes without
  launching a process or using a network.

Checkpoint H1:

One pure Rust service can issue a task packet and validate a returned candidate
without an `AgentBackend`.

## Batch H2 — Full MCP Host Execution Service

Purpose: make the supported model host the caller of the Orchestrator.

Status: complete for the generic main-task workflow. Full MCP now advertises
closed `doctor`, `start`, `next`, `read`, and `submit` operations in addition
to the existing local run/action controls. `read` executes only an offered
read-only project operation, records a bounded run/handoff evidence reference,
and returns the reference in MCP metadata. `submit` authenticates and consumes
those references before persisting only the accepted candidate digest.
Initialization client information is retained only as bounded display-only
session evidence.

Implementation:

- compose a `FullHostOrchestrationService` over the embedded workflow,
  registered project store, ToolHost, checkpoints, candidate validator, and
  review gates;
- implement start/next/submit/action Full MCP tools with closed schemas;
- capture MCP initialization client information as display-only session
  evidence where the transport supplies it;
- require exact project revision and checkpoint compare-and-swap on every state
  transition;
- make ToolHost audit IDs available as bounded evidence references without
  returning paths, arguments, result text, or secrets;
- preserve project-write preview/apply as a separate explicit approval flow;
- update copied-binary stdio tests for the new inventory and complete
  host-driven task round trip.

Primary files:

- `packages/qiongli-native/apps/qiongli/src/orchestration_control.rs`;
- `packages/qiongli-native/apps/qiongli/src/mcp.rs`;
- `packages/qiongli-native/apps/qiongli/tests/mcp_stdio.rs`;
- `packages/qiongli-native/crates/qiongli-runtime/`.

Focused acceptance:

- initialize, list tools, doctor, start, project read, submit, next, and cancel
  succeed over a copied native binary with an empty `PATH`;
- no tool accepts a provider key, endpoint, model name, network confirmation,
  shell command, or executable path;
- a candidate cannot invent a ToolHost audit or reuse evidence from another
  project/revision/run;
- MCP disconnection leaves a recoverable checkpoint rather than a fabricated
  completion.

Checkpoint H2:

A generic MCP client can drive the complete structured workflow without a
model transport inside Qiongli.

## Batch H3 — Codex Plugin And Native Host Mapping

Purpose: make Codex the first real model execution host.

Implementation:

- update the embedded Codex Plugin/Skill to start and advance the host handoff
  protocol;
- make the plugin's MCP declaration launch the receipt-owned native Full MCP
  binary through an exact path, not `PATH` lookup;
- instruct the Codex controller to use project-scoped Qiongli tools, submit
  evidence-backed candidates, and request explicit artifact apply approval;
- map worker packets to Codex-native subagents only when the active host exposes
  that capability; otherwise use the truthful single-agent flow;
- extend App/CLI inventory with Full MCP attachment and observed activation
  evidence distinct from Plugin registration;
- extend install/verify/repair/remove receipts for the Full Plugin projection.

Primary files:

- embedded Codex Plugin/Skill resources in `packages/qiongli-native/`;
- `packages/qiongli-native/crates/qiongli-platform/`;
- `packages/qiongli-native/apps/qiongli/tests/codex_plugin_bundle.rs`;
- App/CLI client integration service and receipt fixtures.

Focused acceptance:

- isolated install materializes the exact Plugin, Skills, and Full MCP launch
  declaration;
- real Codex lists and launches the Qiongli Full MCP from the installed source;
- one authenticated Codex session reads a fixture project through Qiongli,
  submits a candidate bound to observed evidence, and advances the checkpoint;
- Qiongli holds no OpenAI key, makes no direct model request, and launches no
  `codex` child during the workflow;
- verify, repair, and remove preserve unmanaged Codex content.

Checkpoint H3:

The user can begin and complete a Qiongli workflow inside Codex.

Implementation status on July 23, 2026:

- the receipt-owned Codex Plugin binary now launches `mcp serve --profile
  full --transport stdio` through its exact bundle-relative path;
- Codex bundle receipt schema 2 separately records the Marketplace Lite Skills
  projection and the Full MCP runtime projection;
- Codex installation consumes an explicit signed `full-mcp` grant mode; a
  legacy Lite-only grant cannot authorize the Full Plugin projection;
- the generated Codex Skill contains the native doctor/start/read/submit/next
  controller loop, evidence binding, truthful single-agent fallback, optional
  native-subagent mapping, and explicit artifact-apply approval rule;
- App/CLI client inventory schema 2 reports a receipt-verified Full MCP
  declaration separately from Plugin registration. Host activation remains
  `not-observable` until the client supplies runtime evidence, so the UI does
  not infer Connected from installation alone;
- an isolated real-Codex acceptance installed, listed, cached, launched, and
  removed the Plugin. The cached binary exposed all 30 Full/host-handoff tools
  with an empty `PATH`, and unmanaged client content was preserved;
- the authenticated prose-producing Codex conversation is intentionally not
  part of the routine offline suite. The protocol round trip is covered with a
  deterministic host candidate; a release acceptance operator may run the
  authenticated session without giving Qiongli a provider credential.

## Batch H4 — Claude Code Plugin And Native Host Mapping

Purpose: provide the same product outcome inside Claude Code.

Implementation:

- update the managed Claude Code marketplace/plugin package and Skills to use
  the same Full MCP host handoff;
- keep Claude Code commands and Skill discovery aligned without creating a
  duplicate independent workflow contract;
- map worker packets to supported Claude Code native-agent behavior only when
  available, with the same single-agent fallback;
- extend client inventory and receipts with Full MCP attachment and activation
  evidence distinct from marketplace/source presence;
- preserve legacy/unmanaged Claude installations and host-owned trust actions.

Primary files:

- embedded Claude Code Plugin/Skill resources in `packages/qiongli-native/`;
- `packages/qiongli-native/crates/qiongli-platform/`;
- `packages/qiongli-native/apps/qiongli/tests/claude_plugin_bundle.rs`;
- App/CLI client integration service and receipt fixtures.

Focused acceptance:

- isolated install, registration, enablement, restart-required, activation,
  repair, and removal states remain truthful;
- real Claude Code lists and launches the same Full MCP contract;
- one authenticated Claude Code session completes the same fixture workflow;
- Qiongli holds no Anthropic key, makes no direct model request, and launches
  no `claude` child during the workflow;
- Codex and Claude candidates produce identical Qiongli checkpoint semantics
  even though their final prose may differ.

Checkpoint H4:

The user can begin and complete the same Qiongli workflow inside Claude Code.

Implementation status on July 23, 2026:

- the receipt-owned Claude Code Plugin binary now launches `mcp serve
  --profile full --transport stdio` through `${CLAUDE_PLUGIN_ROOT}` and its
  exact bundle-relative path;
- Claude bundle receipt schema 2 separately records the Marketplace Lite
  Skills projection and the Full MCP runtime projection;
- Claude installation and marketplace registration consume an explicit signed
  `full-mcp` grant mode. Product control schema 3 declares both Codex and
  Claude Code as Full MCP targets;
- the generated Claude Code Skill contains the shared
  doctor/start/read/submit/next controller loop, exact evidence binding,
  truthful single-agent fallback, optional native-subagent mapping, and
  explicit artifact-apply approval rule;
- client inventory reports a receipt-verified Claude Full MCP declaration
  separately from Plugin registration. Runtime activation remains
  `not-observable` until Claude Code supplies session evidence, so installation
  is never presented as an active connection;
- an isolated real-Claude-Code 2.1.216 acceptance discovered the direct Skills
  form, validated the Plugin, registered the local marketplace, installed and
  listed the Plugin, verified the client cache, launched all 30 Full/host
  handoff tools with an empty `PATH`, then uninstalled and removed the
  marketplace without touching unmanaged content;
- Qiongli makes no Anthropic request and launches no `claude` child in the
  workflow. The routine suite uses the shared deterministic handoff candidate;
  a release acceptance operator may exercise an authenticated prose-producing
  session without giving Qiongli an Anthropic credential.

## Batch H5 — Desktop App Information Architecture

Purpose: make the App an installer and workflow control surface rather than a
second model client.

Implementation:

- remove `Model Backend` from primary navigation;
- move host execution readiness into `Client Integrations`;
- keep `/model-backend` only as a compatibility redirect or migration
  explanation with an explicit legacy-credential removal action;
- remove prompt entry and model-result rendering from the App;
- update the Orchestrator view to show selected project, host readiness,
  current task/role, evidence status, checkpoint state, review gates, pause,
  recovery, cancellation, and approval-gated artifact preview/apply;
- replace backend readiness fields in the App API with host integration and
  Full MCP readiness fields;
- keep source builds visibly read-only and use the existing separately labelled
  local-installable acceptance App for mutation testing.

Primary files:

- `packages/qiongli-app-api/src/schema.ts`;
- `packages/qiongli-desktop/src/routes/+layout.svelte`;
- `packages/qiongli-desktop/src/routes/model-backend/+page.svelte`;
- `packages/qiongli-desktop/src/routes/client-integrations/+page.svelte`;
- `packages/qiongli-desktop/src/routes/orchestrator/+page.svelte`;
- `packages/qiongli-desktop/src/lib/i18n.svelte.ts`;
- `packages/qiongli-native/apps/qiongli/src/desktop_api.rs`.

Focused acceptance:

- API schema rejects legacy prompt, API-key-save, backend-test, and
  network-confirmation intents from the default surface;
- App displays detected, installed, registered, enabled, trusted, active, Full
  MCP ready, and host-action-required as distinct states;
- keyboard, focus, screen-reader, reduced-motion, compact-width, light, and
  dark presentation tests cover the revised views;
- source App remains read-only; the local-installable acceptance App completes
  preview/apply/verify/repair/remove in an isolated home.

Checkpoint H5:

No ordinary App screen asks for a model API key or invites a Qiongli-hosted
chat. The next action always points to Codex, Claude Code, or a qualified
Desktop host.

Implementation status on July 23, 2026:

- App API schema 2 removes default backend configuration, credential-save,
  backend-test, prompt-run, model-result, and direct orchestration-execution
  messages. It retains only a redacted legacy-credential state and explicit
  cleanup intent;
- the primary navigation has no Model Backend item. The compatibility route
  explains the host-driven migration, links to Client Integrations and
  Orchestrator, and can only preview removal of a legacy credential;
- Client Integrations remains the authority for distinct Plugin source,
  registration, activation, Full MCP attachment, connection observation, and
  required host action;
- Orchestrator now reads real persisted checkpoints from the native service,
  displays the selected project, observed/installed hosts, active task and
  role, evidence handoff state, role gate, generation and document digest, and
  exposes only pause, recovery, resume, cancellation, and refresh controls.
  Model-backed continuation is explicitly delegated to Codex or Claude Code;
- the Tauri adapter now wires checkpoint list/control to
  `FullOrchestrationService` with exact project revision, run generation, and
  document SHA-256 compare-and-swap references. The App cannot start a model
  conversation or render model output;
- the revised view includes keyboard focus, live loading state, compact-width
  layout, and reduced-motion behavior in the existing light presentation.
  Dark mode is not currently a declared Desktop product mode and is not
  inferred from the operating system;
- the host orchestration tool inventory now has one exported ordered constant
  shared by copied-binary, Codex, Claude, and packaged-product acceptance,
  preventing another silent `tools/list` drift;
- App API checks and 15 contract tests, 62 Svelte tests, Svelte diagnostics,
  101 native library tests, focused Full MCP/Codex/Claude bundle tests, and
  locked all-target workspace Clippy pass. The isolated non-publishing macOS
  acceptance App passes product control, restart, project App/CLI/Full MCP
  parity, and Codex/Claude install, verify, repair, and remove checks without
  touching the real client homes.

## Batch H6 — Local Desktop Packaging And Remote Boundary

Purpose: support Desktop use without confusing local and cloud execution.

Implementation:

- evaluate the existing Claude Desktop Plugin/MCPB artifacts against the Full
  native MCP contract;
- if the current public Desktop contract can launch the native binary, create a
  separate Full Desktop package and exact install/activation receipt;
- keep the literature-only MCPB truthfully Lite until it actually carries the
  Full native service;
- expose “Open in host” or package-export actions only through documented host
  mechanisms;
- report Codex Cloud, Claude web, and other remote workers as `remote-only`
  unless an explicit repository bundle or remote MCP is implemented;
- do not let a successful local file copy produce a cloud-active state.

Checkpoint H6:

Every Desktop/cloud label corresponds to a tested transport and installation
boundary. This batch may follow Alpha.2 if no supported Full Desktop mechanism
is ready; it cannot weaken the Codex/Claude Code local claims.

Implementation status on July 23, 2026:

| Surface | Qualified boundary | Evidence and remaining boundary |
| --- | --- | --- |
| Codex App, CLI, and IDE | Full local | The existing receipt-owned Codex Plugin and Full MCP configuration are shared across local Codex surfaces. Installation and active chat attachment remain separate observed states. |
| Claude Code | Full local | The existing receipt-owned Claude Plugin launches the native Full MCP server. Installation, activation, and a connected session remain separate states. |
| Claude Desktop | Manual Full MCPB, current host only | A separate package bundles the native Rust binary and launches `mcp serve --profile full --transport stdio`. The host still owns manual extension installation, trust, enablement, restart, and live attachment. |
| Literature MCPB | Marketplace Lite | The existing literature package is unchanged and does not claim project or orchestration tools. |
| Codex Cloud and Claude Web | Remote-only | No local file copy, Plugin receipt, or MCPB build is interpreted as remote activation. A repository bundle or remote MCP would require a separate contract. |

- `pnpm mcpb:pack:full` builds
  `qiongli-full-runtime-<version>.mcpb` and a non-publishing adjacent build
  receipt for the current target only;
- the builder compiles the release binary, records exact target, architecture,
  binary and artifact hashes, and source commit when the worktree is clean,
  then probes the staged binary with an empty `PATH` and isolated home;
- the generated manifest is populated from the actual Full MCP `tools/list`
  response and must contain exactly 30 unique Lite, project, and host
  orchestration tools. Direct-provider tools are rejected;
- the App presents Codex local, Claude Code local, Claude Desktop manual MCPB,
  and remote-only surfaces as separate cards. It does not expose a package as
  installed, trusted, enabled, or active without host evidence;
- the separate literature MCPB remains Lite, and the new Full package is not
  registered as a release publication target. Its receipt explicitly records
  `publication_allowed: false`;
- deterministic package construction, extracted-binary Full MCP probing,
  source-manifest boundaries, Svelte diagnostics, and localized UI tests pass.
  Real Claude Desktop installation and live attachment are intentionally left
  for explicit host acceptance rather than inferred from the build.

## Batch H7 — Alpha.2 Host-Driven Acceptance

Purpose: replace the retired direct-provider gate with evidence from the real
product path.

Acceptance fixture:

- one small registered project with a known semantic revision and several
  source-anchored facts;
- a task that requires at least one project-scoped Qiongli read;
- a candidate schema that requires evidence audit IDs, known fact digests,
  unresolved-gap reporting, and a review result;
- no exact natural-language answer assertion.

Required receipts:

- local-installable macOS App installs and verifies the native CLI, Plugin,
  Skills, and Full MCP for Codex and Claude Code in an isolated home;
- actual Codex and Claude Code each activate the installed Full MCP and
  complete start/read/submit/review/checkpoint flow;
- receipts contain only product/build/host/plugin/protocol identities, hashes,
  counts, fixed tool IDs, checkpoint transitions, and boundary verdicts;
- receipts contain no provider credential, host auth token, prompt, candidate
  body, model response, conversation ID, project ID/path, or tool result;
- direct Qiongli model-request count is zero and Qiongli-owned model-CLI child
  count is zero;
- install verification, restart, repair, removal, and unmanaged-content
  preservation pass;
- focused Rust, TypeScript/Svelte, production frontend, copied-binary, packaged
  macOS, and exact-head native CI gates pass.

Alpha.2 may claim only the exact host versions and surfaces in these receipts.
Model availability and authentication remain host-owned prerequisites.

Implementation status on July 23, 2026:

- host candidate schema 2 requires authenticated evidence references,
  `knownFactDigests` bound to observed evidence result hashes, an explicit
  `evidenceGaps` report, and a role-compatible `reviewResult`;
- Codex and Claude Code Plugin guidance now tells the host how to populate
  those fields without inventing ToolHost evidence, mutation approval, or
  persisted artifact claims;
- `alpha2-host-driven-v1.json` fixes the semantic revision, two
  source-anchored fact digests, required project-read tool, candidate
  requirements, and checkpoint transition sequence. It explicitly disables an
  exact natural-language-answer assertion;
- the native `HostAcceptanceReceiptV1` accepts only exact
  product/build/host/plugin/protocol identities, hashes, counts, fixed tool
  IDs, review result, checkpoint transitions, and boundary verdicts. Unknown
  fields, non-canonical JSON, fixture drift, non-zero direct-model requests,
  model-CLI child launches, provider credentials, or persisted private
  payloads fail closed;
- `pnpm acceptance:host:preflight` validates the fixture and emits only
  `fixture-ready-manual-host-required` with
  `publication_allowed: false`. It cannot manufacture an accepted host
  receipt;
- the copied release binary completes initialize, tools/list, doctor, start,
  project read, schema-2 submission, checkpoint persistence, and cancellation
  with an empty `PATH`, isolated home, and no model transport. Candidate text
  remains absent from the durable checkpoint;
- deterministic Codex and Claude Code Plugin bundle tests confirm the revised
  host instructions are carried by the installed bundle.

Deferred manual evidence:

- exact Codex and Claude Code versions, active Plugin/Full MCP attachment, and
  their real start/read/submit/review/checkpoint receipts;
- the separately deferred Claude Desktop MCPB install/enable/restart
  observation;
- release promotion. No local preflight result is an accepted host receipt or
  publication authority.

## Validation Tiers

Every batch:

- `cargo fmt --all -- --check`;
- focused `cargo check`, warnings-denied Clippy, and affected Rust tests;
- affected App API, Svelte check, component, and production-build tests.

Cohesive checkpoints:

- locked all-target/all-feature native workspace check, Clippy, and tests;
- copied-binary Full MCP stdio acceptance with an empty `PATH`;
- deterministic Plugin package and receipt tests.

Explicit external acceptance only:

- real Codex and Claude Code install/activation/workflow sessions;
- macOS local-installable App lifecycle;
- separately approved Desktop-host acceptance.

No direct provider live test and no formal cybersecurity scan is part of this
plan.

## Commit Checkpoints

1. `docs(architecture): rebaseline r4 around host execution`
2. `refactor(execution): add host handoff contracts`
3. `feat(mcp): expose host-driven orchestration`
4. `feat(codex): run qiongli through full mcp`
5. `feat(claude): run qiongli through full mcp`
6. `refactor(desktop): replace model backend with host status`
7. `test(desktop): accept host-driven r4 runtime`

Each checkpoint must be independently buildable and must not silently select
the experimental direct backend.

## Rollback

- A failed host adapter disables only that host claim; it does not fall back to
  a direct provider.
- Existing project, capture, graph, checkpoint, and receipt state remains
  readable through the native CLI and App.
- Failed install/repair uses the existing receipt-owned transaction rollback
  and preserves unmanaged client content.
- Existing direct-backend configuration is preserved but disabled until the
  user explicitly removes it or a later standalone-runtime decision is
  approved.
- A failed candidate submission leaves the prior compare-and-swap checkpoint
  unchanged and returns a structured retry/recovery reason.

## Immediate Next Batch

Continue Alpha.2 local release readiness while manual host evidence remains
deferred:

1. run the complete locked native workspace check, Clippy, tests, copied-binary
   Full MCP, deterministic Plugin, App API, Svelte, and production frontend
   gates against the candidate schema 2 contract;
2. rebuild the non-publishing local-installable macOS acceptance App and verify
   App/CLI/Full MCP parity, install, restart, repair, removal, and unmanaged
   content preservation in its isolated home;
3. bind all local receipts to the exact clean source commit before any release
   candidate is considered;
4. leave Codex, Claude Code, and Claude Desktop real-host sessions as explicit
   manual acceptance items, and do not infer them from the offline preflight;
5. keep Alpha.2 publication blocked until both required host receipts validate
   against the fixed fixture and exact build;
6. do not run the retired R4D live-provider acceptance or add a formal
   cybersecurity scan.
