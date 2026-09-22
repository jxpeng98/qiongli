# Full MCP Profile-Sensitive Routing

## 1. Scope / Trigger

Apply this contract whenever `FullMcpServer` exposes a public tool whose Full
result differs from Marketplace Lite. It prevents an active Full server from
telling Codex or Claude to install the Full runtime it is already using.

## 2. Signatures

- Owner: `FullMcpServer::handle_tool_call(&self, request: Value) -> Option<Value>`
- Validation reuse: `LiteMcpServer::handle(&self, request: Value) -> Option<Value>`
- Current profile-sensitive tool: `qiongli_orchestrator_route`

## 3. Contracts

- Live native route input is `request: string` plus optional `platform`.
- Marketplace Lite returns its bounded preview and may include
  `preview_only`, `runtime_profile`, `recommended_runtime`, and `upgrade`.
- Full returns the Contract v2 Full fields: `route`, `recommended_tool`,
  `requires_full_runtime`, `platform`, `platform_note`, `why`, `sequence`,
  `missing`, and `safety`.
- A valid Full route names the existing host-driven chain beginning with
  `qiongli_project_list` and `qiongli_orchestration_doctor`; it does not launch
  a model process or mutate a project.
- Full output must not include Lite-only `preview_only`, `runtime_profile`,
  `recommended_runtime`, or `upgrade` fields.

### Host candidate delegation observations

`qiongli_orchestration_submit` advertises optional `candidate.delegationResults`
through live `tools/list`. The shared `HostCandidateEnvelopeV1` owner validates
up to eight native-subagent or configured external-agent observations: actual
dispatch tool/execution ID, bounded scope, originating handoff SHA-256, completed
status and exact returned UTF-8 text/SHA-256. Combined candidate and result text
must fit the handoff's candidate byte limit. Duplicate adapter/execution IDs,
stale bindings, non-completed states and tampered output fail before checkpoint
CAS. Native observations also require the declared NativeSubagents capability.

This is an additive candidate v2 field, omitted when empty to preserve existing
canonical bytes/digests. Clients inspect the live schema before sending it;
older servers use the existing collaboration trace. Only the accepted candidate
digest is checkpointed, including these observations; raw delegated content is
not persisted by submission. Debug output excludes result text and scope.

Observations are coordinator-reported, not authenticated execution, independent
review certification or source evidence. Full still requires its own process's
authenticated reads and current run/revision/generation. Host tools own dispatch,
wait/read/cancel and reconciliation of late/partial results. This adds no model
launcher, external runtime adapter, cross-Host claim store or artifact approval.

### Host-executed Codex adapter

The native CLI now provides `agent codex prepare` and `agent codex collect`.
They adapt the existing configured `codex exec --json --ephemeral` transport to
the shared delegation result without changing Full MCP's execution boundary.
The calling Host launches fixed argv, supplies stdin, supervises deadlines and
cancellation, retains JSONL and reports its observed outcome/exit code. Qiongli
does not launch or resume a process, alter model/auth configuration or create a
parallel task store. Read-only sandbox does not imply isolated MCP/hooks.

Prepare validates the supplied handoff and bounded `{scope, sourceText}` packet;
its prompt requests exact handoff/packet digest acknowledgements. Collect accepts
one coherent thread/turn, successful process exit and a completed final reply.
It rejects failure, cancellation, timeout, malformed/truncated/oversized streams,
duplicate terminal events, conflicting thread IDs and changed packet/handoff
bindings. The result retains the exact final message text/hash and actual reported
Codex thread ID; intermediate reasoning and error text are not returned. Only
explicit regular-file paths are read. No output file or canonical state is written.

An interrupted ephemeral run has no automatic reconnection or replay: the Host
settles the original process, checks current sources and explicitly prepares a
fresh bounded assignment. A portable adapter result remains a proposal; the
coordinator rechecks current authenticated project evidence before submission.
Protocol reference: [Codex non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode).

## 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Missing, empty, oversized, non-string, or unsupported route input | Preserve the existing JSON-RPC `-32602` validation error |
| Valid route call on Marketplace Lite | Return the Lite preview unchanged |
| Valid route call on Full | Return the Full host-orchestration route |
| Unknown tool on Full | Preserve normal Lite/Full `-32601` delegation |
| Project service unavailable during route selection | Still return the read-only sequence; the later doctor call owns readiness failure |

## 5. Good / Base / Bad Cases

- Good: Full returns `orchestrator_mcp`, starts with `qiongli_project_list`, and
  contains no upgrade object.
- Base: Marketplace Lite remains preview-only and truthfully recommends Full.
- Bad: Full delegates the valid call directly to Lite and reports
  `runtime_profile: marketplace_lite` or `upgrade.required_for_execution: true`.

## 6. Tests Required

- `mcp_stdio`: call the copied binary with `--profile full`; assert the exact
  host-driven tool sequence and absence of Lite-only fields.
- `mcp_stdio`: retain the existing Marketplace Lite preview assertions.
- Codex and Claude Plugin bundle tests: materialize the embedded Skill and
  verify the Full host tools remain visible under an empty runtime `PATH`.

## 7. Wrong vs Correct

Wrong:

```rust
if project_tool.is_none() && orchestration_tool.is_none() {
    return self.lite.handle(request);
}
```

Correct:

```rust
if requested_name == Some("qiongli_orchestrator_route") {
    return self.handle_full_orchestrator_route(request);
}
```

Intercept the profile-sensitive name before generic Lite delegation, reuse Lite
only for input validation, and build the Full result at the Full owner.
