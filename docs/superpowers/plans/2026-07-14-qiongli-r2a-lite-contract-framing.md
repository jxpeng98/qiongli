# Qiongli R2A Lite Contract And Framing Implementation Plan

Status: in progress

Date: July 14, 2026

**Goal:** Establish the shared native Lite runtime boundary by extracting the
accepted tool registry and bounded stdio framing without reimplementing
provider behavior or advertising a native MCP server.

**Architecture:** Add `qiongli-runtime` between verified content and the app.
Keep Contract v2 JSON as the source of public tool definitions, add typed
canonical tool identities and redacted errors, and make the existing Rust Lite
package a compatibility consumer of the shared parser and protocol.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/Cargo.toml` | Add the runtime workspace member |
| `packages/qiongli-native/crates/qiongli-runtime/` | Add errors, Lite registry, framing, and tests |
| `packages/qiongli-native/apps/qiongli/` | Prove registry loading through verified embedded content |
| `packages/qiongli-lite-mcp/Cargo.toml` | Depend on the native shared runtime |
| `packages/qiongli-lite-mcp/src/tools/definitions.rs` | Replace local parsing with a compatibility adapter |
| `packages/qiongli-lite-mcp/src/mcp/protocol.rs` | Replace local framing with compatibility adapters |
| `packages/qiongli-lite-mcp/src/mcp/server.rs` | Use typed shared name resolution and static unknown-name errors |
| `packages/qiongli-lite-mcp/tests/` | Add alias/no-echo compatibility proof |
| `.github/workflows/native-ci.yml` | Add parallel focused Lite compatibility coverage |
| accelerated roadmap and Draft PR #63 | Record only observed exact-head evidence |

## Task 1 — Freeze The Runtime Boundary

- [x] Review the accelerated R2 scope, accepted architecture, embedded-content
  API, Lite contract, framing, dispatch, tests, and native CI.
- [x] Freeze strict registry, typed identity, bounded framing, redacted error,
  compatibility, CI, and nonclaim contracts.
- [x] Commit the design and implementation plan before production changes.

## Task 2 — Build The Shared Registry

- [x] Add `qiongli-runtime` with workspace metadata and lint policy.
- [x] Add stable path/input-free runtime errors.
- [x] Parse at most 1 MiB of strict Contract v2 Lite JSON.
- [x] Freeze the exact 12 public names and 11 typed canonical identities.
- [x] Resolve the config-wizard alias without duplicating a handler identity.
- [x] Load the registry from the verified `marketplace-lite` embedded profile.
- [x] Test malformed, oversized, drifted, reordered, and canary-bearing input.

## Task 3 — Share Bounded Framing

- [x] Move newline and Content-Length framing into `qiongli-runtime`.
- [x] Preserve the 8 MiB message and 64 KiB header bounds.
- [x] Return typed input/output/serialization/UTF-8/incomplete-message errors.
- [x] Test byte-length UTF-8, blank lines, EOF, malformed headers, all bounds,
  short payloads, input failures, output failures, and error redaction.

## Task 4 — Reduce The Old Lite Boundary

- [x] Point `qiongli-lite-mcp` at `qiongli-runtime`.
- [x] Preserve old tool-definition and protocol module call sites through thin
  adapters.
- [x] Replace the duplicate handler name table with the shared typed resolver.
- [x] Make unknown JSON-RPC method and tool errors static and no-echo.
- [x] Run focused existing protocol/server tests plus the new alias/no-echo
  assertions.

## Task 5 — Prove Native Composition

- [x] Add the runtime dependency to the canonical app.
- [x] Load all 12 definitions from the real verified embedded pack in an app
  integration test.
- [x] Confirm production runtime/app code adds no loose-file fallback, process
  launch, shell, network, Python, or Node dependency.
- [x] Keep the public CLI grammar unchanged and make no MCP availability claim.

## Task 6 — Gate And Record The Slice

- [x] Add a parallel Linux focused Lite compatibility CI job.
- [x] Run the native change boundary, format, locked check, strict Clippy, and
  all native Rust tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [x] Run focused old Lite protocol/server tests with the locked manifest.
- [x] Audit the diff for duplicate behavior and attacker-controlled error
  rendering.
- [ ] Commit cohesive implementation, push the rolling branch, and keep Draft
  PR #63 as the only PR.
- [ ] Require exact-head boundary, Linux, macOS, Windows, and focused Lite jobs
  to pass.
- [ ] Update the roadmap and PR only from observed evidence, leaving R2 open.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test mcp_protocol --test mcp_server
```

Legacy Python and Node suites remain diagnostic-only and are not part of this
batch.

## Completion Definition

R2A is complete when the canonical app can load the frozen Lite tool registry
from verified embedded content, the old Lite entrypoint consumes the same
registry and framing code, all declared bounds and redaction contracts are
proved locally and in exact-head CI, and the roadmap still truthfully states
that native MCP mode and the remaining R2 domain behavior are not available.
