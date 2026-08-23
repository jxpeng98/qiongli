# Design: App setup, recovery, and Full MCP self-test

## Boundary

Extend the existing integration control plane in place:

```text
Svelte App
  -> versioned App intent/event
  -> NativeDesktopService
  -> existing receipt-owned setup / refresh / repair owners
  -> shared FullMcpServer factory
  -> initialize
  -> exact Lite + Full project + Full orchestration registry
  -> Full-only qiongli_orchestrator_route dispatch
  -> typed bounded result in the App
```

No Host cache writer, alternate MCP implementation, new route, or new test
runner is introduced.

## Existing owners and changes

| Concern | Existing owner | Planned change |
|---|---|---|
| Setup and repair | `desktop.rs` plus packaged product/client activation | Reuse unchanged unless the end-to-end regression exposes a shared defect. |
| Fresh status | native integration inventory and Host probes | Reuse; assert stale/error states survive App projection. |
| Full MCP runtime | `apps/qiongli/src/mcp.rs::FullMcpServer` | Extract/reuse one internal constructor so stdio and App self-test cannot drift. |
| Self-test state | `qiongli-ui` model and `desktop.rs` worker | Generalize the existing Lite-only test to Full while preserving cancel, timeout, and no-secret-read behavior. |
| App bridge | `desktop_api.rs` and `@qiongli/app-api` | Add typed Full self-test intents/event/view; bump the contract version only if the App API spec requires it. |
| App UI | Client Integrations route and shared App state | Add one compact Full MCP health panel beside existing integration controls. |
| Evidence | Program Ledger v1 and acceptance note | Bind PLT-321 to the exact product commit and Slice run. |

## Full MCP proof

The self-test uses the same embedded contracts and runtime constructor as
`qiongli mcp full`. It checks:

1. MCP initialization returns the supported protocol and tools capability.
2. `tools/list` names the exact ordered union of:
   - `LITE_PUBLIC_TOOL_NAMES`;
   - `FULL_PROJECT_PUBLIC_TOOL_NAMES`;
   - `FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES`.
3. `qiongli_orchestrator_route` returns the Full orchestration route with
   `requires_full_runtime=true` and no Lite `upgrade` result.

The public tool count is derived from those three authoritative arrays rather
than copied as another magic number. Existing domain suites continue to own
per-tool behavior.

## Status and recovery truth

The Full self-test reports embedded/runtime health only. Integration Ready stays
owned by fresh receipt, Host activation/cache, and MCP attachment observations.
The App displays both facts and never promotes a stale, drifted, failed,
unavailable, or recovery-required integration because the embedded self-test
passes.

Setup and recovery continue through preview, digest-bound confirmation, fixed
official Host plans, post-apply verification, and fresh probes. The new App
event cannot mutate integrations.

## App contract

Expose the existing bounded worker lifecycle through three App intents:

- run Full MCP self-test;
- poll Full MCP self-test;
- cancel Full MCP self-test.

Return one typed event containing state, six existing check rows, combined tool
count, and bounded provider/client counts. The frontend stores only the latest
view, polls while running, and renders explicit failed/cancelled/timed-out
states. It does not infer success from text.

## Compatibility and safety

- Existing Lite and Full public MCP contracts and tool schemas do not change.
- No credentials, network request, project mutation, or Host command is needed
  for the Full self-test.
- Cancellation and fixed timeout remain native-owned.
- App API additions remain strict and versioned; unknown states still fail
  validation.
- Normal Codex/Claude profiles and user project/Zotero data are not test inputs.

## Verification tiers

- **Focused:** Full server registry/route, native self-test lifecycle and
  no-secret-read checks, App API fixture/schema, frontend state and panel tests,
  existing setup/status/recovery regressions.
- **Slice:** affected Rust and pnpm checks plus exact-head Evaluation Truth and
  three-platform Native CI.
- **Acceptance:** packages, live authenticated Hosts, real user data, signing,
  promotion, and release remain deferred.

## Rollback

Revert the Full self-test bridge/UI and restore PLT-321 to its prior ledger
state. Setup, repair, MCP stdio, Host profiles, and user data remain unchanged.
