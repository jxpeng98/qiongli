# Qiongli R2E Native Lite MCP Vertical Implementation Plan

Status: in progress

Date: July 14, 2026

**Goal:** Make the canonical native `qiongli` binary serve the frozen
Marketplace Lite MCP profile over bounded stdio by composing the already
shared Rust services.

**Architecture:** Put JSON-RPC/MCP serving and remaining Lite request parsing
in `qiongli-runtime`. Keep `apps/qiongli` as a typed command and dependency
composition shell, and reduce the old Lite server toward a compatibility
adapter.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-runtime/src/mcp.rs` | Shared JSON-RPC/MCP server, tool envelopes, status projection, unavailable-safe config handlers, and typed domain dispatch |
| `packages/qiongli-native/crates/qiongli-runtime/src/searchplan.rs` | Shared bounded Contract v2 argument parser |
| `packages/qiongli-native/crates/qiongli-runtime/src/providers/search.rs` | Shared strict Lite search argument parser |
| `packages/qiongli-native/crates/qiongli-runtime/src/lib.rs` | Export the native Lite server surface |
| `packages/qiongli-native/apps/qiongli/src/command.rs` | Closed MCP grammar and typed streaming product action |
| `packages/qiongli-native/apps/qiongli/src/mcp.rs` | Compose embedded registry, native config, provider access, stdin, and stdout |
| `packages/qiongli-native/apps/qiongli/src/main.rs` | Select one-shot CLI output or stdio serving before stdout writes |
| `packages/qiongli-native/apps/qiongli/tests/` | CLI grammar and copied-binary vertical acceptance |
| `packages/qiongli-lite-mcp/src/mcp/server.rs` | Delegate shared search-plan/search parsing and retain only compatibility-only adapters |
| `.github/workflows/native-ci.yml` | Include the focused native MCP/canonical binary acceptance target if explicit target enumeration requires it |
| accelerated roadmap and Draft PR #63 | Record only exact-head observed evidence |

## Task 1 — Freeze R2E

- [x] Audit the canonical command path, ADR 0201, Contract v2 schemas and smoke
  calls, shared registry/framing/dispatch/services, old Lite behavior, native
  config boundary, and current CI.
- [x] Freeze exact CLI grammar, protocol behavior, tool ownership, secure-store
  boundary, copied-binary proof, nonclaims, and R3 handoff.
- [x] Write design and implementation plan before production changes.
- [ ] Commit and push the design checkpoint on the rolling branch.

## Task 2 — Complete Shared Lite Inputs

- [ ] Move bounded search-plan argument parsing and aliases into the shared
  runtime with typed static errors.
- [ ] Add strict literature-search object parsing, provider selection, modes,
  and limits to the shared runtime.
- [ ] Delegate both parsers from the old Lite server and remove duplicate
  helper kernels.
- [ ] Cover unknown, missing, type-invalid, duplicate, oversized, alias, year,
  mode, and redaction-canary inputs.

## Task 3 — Add Shared MCP Server

- [ ] Implement JSON-RPC request parsing, notification suppression, static
  errors, malformed-JSON recovery, and framing-preserving responses.
- [ ] Load tools from `LiteToolRegistry` and dispatch only resolved typed
  domain handlers.
- [ ] Compose redacted native config/literature status and production bounded
  provider search.
- [ ] Compose shared search plan, evidence, Zotero fallback/export, and
  orchestration previews.
- [ ] Validate save/configure inputs and return fixed unavailable-safe tool
  errors without writes, listeners, secret echo, or path disclosure.
- [ ] Add defense-in-depth credential-key redaction to structured and text
  tool results.
- [ ] Test every public Lite name and both framing modes directly.

## Task 4 — Expose Canonical Binary Mode

- [ ] Add the exact `mcp serve --profile lite|marketplace-lite --transport
  stdio` grammar, help, duplicate/unknown rejection, and Full-profile closure.
- [ ] Add a typed product action so stdio serving bypasses one-shot CLI output.
- [ ] Compose the verified embedded registry and native provider settings in
  the app crate with `UnavailableSecretStore`.
- [ ] Keep initialize/list available when config is missing or invalid while
  returning fixed dependent-tool errors for invalid config.
- [ ] Ensure stdout is MCP-only and EOF exits cleanly.

## Task 5 — Prove The Vertical Slice

- [ ] Add a copied-binary test with isolated config, empty `PATH`, initialize,
  notification, tools/list, safe tools/call, unavailable mutation, and EOF.
- [ ] Assert all 12 frozen names, structured output, alias year equality,
  preview safety flags, disabled Zotero fallback, and secret/path redaction.
- [ ] Test invalid CLI modes before stdin and test line plus Content-Length
  sessions.
- [ ] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [ ] Run focused old Lite compatibility tests and Windows MSVC cross-target
  check/Clippy without Python, Node, or live services.

## Task 6 — Checkpoint And Record

- [ ] Commit and push the cohesive R2E implementation to rolling Draft PR #63.
- [ ] Require exact-head boundary, Linux, macOS, Windows, and focused Lite jobs
  to pass.
- [ ] Record observed commit IDs, test counts, timings, and explicit nonclaims
  in this plan, the accelerated roadmap, native README, and PR body.
- [ ] Mark R2 complete only if the copied-binary and exact-head gates pass;
  otherwise leave R2E in progress with a factual blocker.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test mcp_protocol --test mcp_server --test provider_http --test providers --test search_orchestration --test literature_planning_mcp --test searchplan --test zotero_export --test zotero_companion --test orchestrator_preview
```

Python and Node suites remain outside this accelerated migration batch.

## Completion Definition

R2E is complete when the copied canonical binary, not a library-only harness,
serves initialize, all 12 Lite definitions, and bounded safe calls over stdio;
both Rust entrypoints share request/domain kernels; unavailable config writes
fail safely; required local and exact-head CI gates pass; and packaging,
launch-grant, Marketplace, Full MCP, agent, and release claims remain closed.
