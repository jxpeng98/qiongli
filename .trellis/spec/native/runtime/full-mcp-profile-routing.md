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
