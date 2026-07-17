# Qiongli R2E Native Lite MCP Vertical Design

Status: approved for execution

Date: July 14, 2026

Scope: canonical `qiongli` binary Marketplace Lite MCP stdio vertical slice

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Compose the already shared Lite registry, bounded MCP framing, typed tool
dispatch, provider/search services, evidence export, supported Zotero behavior,
and route/task previews into one native MCP server owned by
`qiongli-runtime`. Expose that server from the canonical `qiongli` executable
through one closed command:

```text
qiongli mcp serve --profile lite --transport stdio
```

`marketplace-lite` is accepted as the exact profile alias. The profile and
transport are both required, option order is irrelevant, and duplicate,
unknown, non-UTF-8, Full-profile, or non-stdio values fail during command
parsing before configuration, network, or MCP input is initialized.

The application crate owns only command selection and dependency composition.
It loads the verified embedded Lite contract and redacted native provider
settings, then gives stdin and stdout to the shared server. No domain handler
is implemented in the binary dispatcher.

This is a development vertical proof. R2E does not claim release plugin
activation, an artifact-bound launch grant, target packaging, Marketplace
distribution, or a supported installed product. Those remain R3 gates.

## Protocol Boundary

The shared server owns:

- bounded line and `Content-Length` stdio framing;
- JSON-RPC parse, request, notification, result, and error envelopes;
- `initialize`, `notifications/initialized`, `ping`, `tools/list`, and
  `tools/call`;
- the embedded 12-name Marketplace Lite registry;
- typed public-name resolution and domain dispatch; and
- static, redacted invalid-argument and operational errors.

Malformed JSON receives a static parse error and does not terminate the next
valid framed request. Requests with an invalid JSON-RPC version, invalid ID,
missing method, malformed call parameters, unknown tool, or unknown argument
fail without reflecting peer-controlled content. Notifications produce no
stdout response. Tool results contain both MCP text content and structured
content; defense-in-depth output redaction removes credential-bearing keys.

The server advertises the frozen 12 public names, including the historical
`qiongli_open_config_wizard` alias. Both configure names resolve to the same
typed handler identity before dispatch.

## Tool Composition

| Domain | Native R2E behavior |
|---|---|
| config status | Redacted provider readiness with an opaque managed-config identifier; no home or config path is returned |
| save provider config | Validate bounded arguments, then return fixed `capability_unavailable`; never persist or echo the value |
| configure provider / wizard alias | Reject invalid host, port, or provider before side effects; valid input returns fixed `capability_unavailable` and starts no listener |
| literature status | Shared provider readiness and truthful Lite capability projection |
| search plan | Shared bounded alias-compatible parser plus deterministic shared planner |
| literature search | Shared bounded request parser and production provider runtime; no active provider means a local warning result without network |
| evidence export | Shared side-effect-free redacted snapshot builder |
| Zotero status | Explicit disabled/fallback status without an environment lookup or loopback probe |
| Zotero import-file export | Shared bounded in-memory exporter; no project write |
| route/task plan | Shared preview-only orchestration dispatcher; no agent, process, shell, or write |

Provider settings are read through the native global-settings boundary.
Secret references are resolved only through a `SecretStore`; R2E uses the
explicit unavailable store, so it never reads credentials from environment
variables or plaintext compatibility files. Public-setting providers and
credential-free arXiv may be active. Invalid or insecure native configuration
does not prevent MCP initialize/list, but dependent status/search calls return
a fixed configuration-unavailable tool error.

## Shared Input Ownership

R2E moves the remaining Lite request parsing needed by both Rust entrypoints
behind `qiongli-runtime`:

- search-plan canonical and compatibility aliases, bounds, duplicate checks,
  years, modes, platform normalization, and optional context;
- literature-search object shape, provider selection, modes, and limits; and
- config mutation/wizard validation used by the native unavailable-safe
  handlers.

The old `qiongli-lite-mcp` server delegates search-plan and literature-search
input handling to these shared parsers. It may retain its 1.x config/wizard
adapter temporarily, but it cannot keep a second provider/search kernel.

## Streaming Entry Point

The existing one-shot `CliOutput` path remains unchanged for ordinary CLI
commands. A typed product action separates one-shot output from Lite MCP stdio
serving. `main` matches that action before writing anything to stdout, so no
help, status, path, diagnostic, or log line can corrupt the MCP stream.

Once serving begins, stdout contains framed MCP responses only. Startup and
transport failures use a static reason code on stderr and a nonzero exit. EOF
after complete requests exits cleanly.

## Security And Privacy

- MCP input is bounded to 8 MiB and headers to 64 KiB by the shared framing
  layer before JSON parsing.
- Closed CLI parsing prevents profile escalation and alternate transports.
- R2E exposes Lite only; Full MCP remains unavailable.
- Save/configure handlers perform no write, listener, process, browser, or
  plaintext-secret fallback.
- Provider endpoints are production constants; MCP arguments and environment
  variables cannot override them.
- Config results use `<managed-native-config>` rather than a filesystem path.
- Invalid input and operational errors never include tool names, unknown keys,
  enum values, paths, provider response bodies, or secret values.
- Zotero status performs no loopback probe in this batch.
- Route/task tools remain pure previews and cannot launch an agent.
- Headless startup initializes no renderer or desktop persistence.

## Verification

Direct shared-runtime tests cover protocol methods, notifications, both
framings, malformed JSON recovery, all 12 public calls, unavailable-safe
config behavior, disabled-provider search, redaction canaries, and strict
input boundaries.

Canonical app tests cover the exact CLI grammar and prove invalid MCP commands
fail before serving. A copied-binary acceptance test clears `PATH`, uses an
isolated native config root, sends initialize, tools/list, and multiple bounded
tools/call requests over stdio, verifies all 12 names, checks structured
results and unavailable errors, confirms no secret/path canary appears, and
exits cleanly on EOF.

Required gates remain native boundary, format, locked check, strict Clippy,
all native Rust tests, focused old Lite Rust compatibility tests, and Windows
MSVC cross-target check/Clippy. Python and Node suites, live providers, live
Zotero, UI, installer, and release tests are outside this batch.

## Nonclaims

R2E does not implement or claim:

- production secure-store availability or provider secret mutation;
- a native configuration UI, wizard listener, or browser launch;
- Full MCP, Full workflow execution, agent backends, ToolHost, or project
  writes;
- local Zotero library search/write or live Companion probing;
- signed launch grants, plugin installation, Marketplace/Desktop activation,
  packaging, signing, update, rollback, or release readiness; or
- completion of R3 or `v2.0.0-alpha.1`.

## Acceptance Criteria

R2E is complete only when:

1. one shared server owns JSON-RPC/MCP and typed dispatch for all Lite names;
2. the canonical binary serves Lite stdio through the exact closed command;
3. a copied binary passes initialize, tools/list, and bounded safe calls with
   an empty `PATH` and without Python or Node;
4. config mutation and wizard calls are validated, unavailable-safe, and do
   not echo their secret canary;
5. search-plan and literature-search parsing no longer diverge between the two
   Rust entrypoints;
6. invalid config, input, framing, cancellation, and provider failures remain
   bounded and redacted;
7. local native, focused Lite, and Windows gates pass; and
8. exact-head GitHub jobs pass before roadmap and PR claims are updated.

## Approval Record

The user instructed continuation under the accepted accelerated roadmap on
July 14, 2026. This authorizes R2E on the existing rolling branch and Draft PR
without creating another branch or PR.
