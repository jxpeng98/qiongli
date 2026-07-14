# Qiongli R2A Lite Contract And Framing Design

Status: approved for execution

Date: July 14, 2026

Scope: first dependency-contiguous R2 slice over the accepted Rust Lite MCP

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Create `qiongli-runtime` inside the native workspace and move the two Lite MCP
boundaries that are already independent of providers and domain behavior into
it:

1. the verified Contract v2 Lite tool registry; and
2. bounded newline/Content-Length stdio framing.

The existing `qiongli-lite-mcp` package becomes a compatibility consumer of
those shared implementations. Its provider, evidence, Zotero, orchestrator
preview, and JSON-RPC server behavior remain in place for this slice. The
canonical `qiongli` executable proves that it can load the Lite registry from
its already verified embedded resource pack, but it does not expose an MCP
process mode yet.

This is extraction, not parity reimplementation. There must be one parser for
the Lite tool contract and one framing implementation after the batch.

## Crate Boundary

`qiongli-runtime` depends on `qiongli-content`, `serde`, and `serde_json`. It
owns reusable runtime contracts and protocol transport primitives. It does not
own UI, host installation, filesystem materialization, config persistence,
provider HTTP clients, credential storage, process launch, or application
argument parsing.

The dependency direction is:

```text
qiongli-content -> qiongli-runtime -> apps/qiongli
                           ^
                           |
                 qiongli-lite-mcp compatibility entry
```

There is no dependency from the native runtime back to the legacy Lite
package.

## Lite Tool Registry Contract

The authoritative resource remains
`mcp-contracts/lite-tools.json` inside the verified embedded pack. The parser
accepts at most 1 MiB and requires:

- top-level `schema_version` exactly `1.0`;
- exactly 12 public definitions in the frozen order;
- the accepted public names with no missing, duplicate, reordered, or extra
  entries;
- non-empty bounded descriptions; and
- an object-valued `inputSchema` for every entry.

Unknown top-level or tool-definition fields fail closed. JSON Schema content
inside `inputSchema` remains opaque and round-trippable so the runtime does not
invent a second schema dialect.

The registry exposes 11 canonical typed tool identities. The public
`qiongli_open_config_wizard` name resolves to the same
`ConfigureProvider` identity as `qiongli_configure_provider`. All other public
names map one-to-one. The old Lite server uses this typed resolver for handler
selection instead of maintaining a second name array.

The canonical app loads this contract only through `EmbeddedContent`, after
the pack digest, manifest, entries, and profile projection have already been
verified. It must not fall back to a loose checkout file at runtime. The old
compatibility package may continue compiling the same canonical JSON resource
into its binary while it is reduced, but parsing and name resolution are
shared.

## Error And Privacy Contract

`qiongli-runtime` returns typed errors with stable allowlisted reason codes.
Public `Display` output contains only the reason code and never includes:

- JSON parser text or source bytes;
- tool, method, header, or field values supplied by a peer;
- concrete paths or profile lookup internals;
- I/O error text; or
- credentials, provider values, environment values, or request payloads.

The compatibility adapter maps runtime errors back to the existing
`std::io::Error` API while preserving useful error kinds. It does not restore
the hidden source message. Unknown JSON-RPC methods and tool names in the old
server return static messages rather than echoing attacker-controlled names.

## Framing Contract

The shared protocol accepts the two behaviors already supported by Lite:

- one UTF-8 JSON payload per line; and
- case-insensitive `Content-Length` headers followed by a UTF-8 byte payload.

Blank lines before a message are ignored. Responses use the request framing
and are flushed after one complete message.

Hard limits are frozen at:

| Boundary | Limit |
|---|---:|
| message payload or line | 8 MiB |
| complete header section | 64 KiB |

Oversized input is rejected before unbounded allocation. Invalid lengths,
invalid UTF-8, incomplete headers/payloads, serialization failures, input I/O,
and output I/O have distinct typed reason codes. No process, shell, network,
or async runtime is introduced by framing.

## Compatibility And CI

Existing Lite protocol and server tests continue to call the old public
module paths. Focused compatibility tests must prove:

- newline and Content-Length requests still round-trip;
- the tools/list definitions all resolve to handlers;
- the alias resolves to the canonical configure-provider identity; and
- unknown method/tool canaries are not echoed.

The native workspace runs format, locked check, strict Clippy, and all-target
tests on Linux, macOS, and Windows. A parallel Linux compatibility job runs the
focused old Lite protocol/server tests so modifying the wrapper is covered
without restoring Python/Node suites or serializing the native platform
matrix behind a legacy full-suite gate.

## Nonclaims

R2A does not implement or claim:

- `qiongli mcp serve`, initialize, tools/list, or tools/call in the canonical
  executable;
- provider configuration, secrets, search transport, timeouts, or
  cancellation in `qiongli-runtime`;
- evidence export, Zotero, route preview, task planning, or Full MCP dispatch
  in the native workspace;
- provider/domain behavior parity beyond continued focused compatibility tests;
- Codex or Claude registration, desktop UI, packaging, or release readiness;
  or
- completion of R2.

## Acceptance Criteria

The slice is complete only when:

1. the native workspace contains a lint-clean `qiongli-runtime` crate;
2. the strict registry accepts the canonical 12-name contract and resolves the
   compatibility alias to one of 11 typed identities;
3. the canonical app loads that registry from verified embedded content;
4. malformed, oversized, missing, extra, reordered, and unknown-field
   contracts fail with stable redacted codes;
5. shared framing covers both accepted transports and every declared bound;
6. the old Lite definitions, framing, and handler-name policy reuse the shared
   runtime implementation;
7. focused compatibility tests prove behavior and no-echo errors;
8. the local native gate and Windows cross-target check pass without Python or
   Node; and
9. exact-head GitHub native and Lite compatibility jobs pass before the
   roadmap or PR records completion.

## Approval Record

The user instructed continuation under the accepted accelerated roadmap on
July 14, 2026. That authorizes this first R2 extraction slice on the existing
rolling branch and Draft PR, without expanding its public capability claims.
