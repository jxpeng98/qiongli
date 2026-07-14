# Qiongli R2D Orchestration Preview And Dispatch Implementation Plan

Status: complete

Date: July 14, 2026

**Goal:** Put bounded Marketplace Lite route/task-plan previews and exhaustive
domain-typed handler selection behind `qiongli-runtime` while retaining the old
Lite package as a protocol and compatibility adapter.

**Architecture:** Extend the shared contract identity with a typed domain
projection, add one pure orchestration preview module, and delegate the old
JSON-RPC handlers to it. Keep canonical MCP serving and all execution behavior
closed.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-runtime/src/contract.rs` | Add exhaustive domain-typed Lite dispatch targets |
| `packages/qiongli-native/crates/qiongli-runtime/src/orchestration.rs` | Add bounded preview inputs, outputs, errors, and typed dispatch |
| `packages/qiongli-native/crates/qiongli-runtime/src/lib.rs` | Export the shared preview and dispatch API |
| `packages/qiongli-lite-mcp/src/orchestrator/preview.rs` | Reduce to a shared-runtime re-export |
| `packages/qiongli-lite-mcp/src/mcp/server.rs` | Dispatch through shared domain identities and preview handler |
| `packages/qiongli-lite-mcp/tests/` | Add compatibility and boundary coverage |
| `.github/workflows/native-ci.yml` | Include focused orchestration compatibility tests |
| accelerated roadmap and Draft PR #63 | Record exact-head evidence after gates pass |

## Task 1 — Freeze R2D

- [x] Inspect Contract v2, the accepted roadmap, old Lite preview behavior,
  typed resolver, tests, and native CI.
- [x] Freeze bounds, compatibility output, typed dispatch ownership, privacy,
  verification, and nonclaims.
- [x] Write the design and implementation plan before production changes.
- [x] Commit the design and plan before production changes.

## Task 2 — Add Typed Lite Dispatch

- [x] Add exhaustive config, literature, Zotero, and orchestration handler
  identities.
- [x] Map every canonical `LiteToolId` to exactly one domain target.
- [x] Preserve the public config-wizard alias mapping before dispatch.
- [x] Test all identities and compile-time-exhaustive projection behavior.

## Task 3 — Add Shared Orchestration Previews

- [x] Add typed route and task-plan argument parsing with static errors.
- [x] Enforce field bounds, platform enum, unknown-field rejection, and current
  normalization behavior in the runtime.
- [x] Emit deterministic Contract v2 safety flags and compatibility upgrade
  projections.
- [x] Dispatch only typed route/task-plan handlers and test output variants.
- [x] Test missing, malformed, blank, oversized, UTF-8 boundary, and canary
  inputs directly against the runtime.

## Task 4 — Reduce Old Lite

- [x] Replace the local preview kernel with a shared-runtime re-export.
- [x] Replace the flat server handler match with shared domain-typed dispatch.
- [x] Delegate route/task validation and output construction to the shared
  dispatcher.
- [x] Preserve the JSON-RPC envelope and static invalid-argument transport.
- [x] Confirm no second Rust orchestration-preview implementation remains.

## Task 5 — Gate And Record

- [x] Add `orchestrator_preview` to the focused Lite CI test list.
- [x] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [x] Run focused old Lite Rust compatibility tests without live services.
- [x] Commit and push the cohesive implementation to rolling Draft PR #63.
- [x] Require exact-head boundary, Linux, macOS, Windows, and focused Lite jobs
  to pass.
- [x] Record observed evidence in this plan, the roadmap, and PR while leaving
  canonical MCP availability unclaimed.

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

R2D is complete when one shared Rust implementation owns bounded route/task
previews, every Lite identity has a typed domain target, the old Lite entrypoint
uses both, local and exact-head CI gates are green, and the canonical binary
still makes no MCP or execution claim.

## Execution Receipt

- Design and implementation plan: `bb6d2b08`.
- Shared orchestration preview and typed dispatch implementation: `5509d2c1`.
- Local native evidence: change boundary, native and Lite format, locked native
  workspace check, strict native and Lite Clippy, all 138 native Rust tests,
  and Windows MSVC cross-target check/Clippy passed.
- Compatibility evidence: all 69 focused old Lite tests passed without a live
  provider, Zotero installation, Full runtime, or agent process. The set now
  includes the explicit `orchestrator_preview` target plus public route and
  task-plan calls.
- GitHub evidence: Native CI run `29328767532` passed exact implementation head
  `5509d2c146e65265e1f47cc9b4badb3f258325c9`: boundary in 4s, focused Lite in
  33s, Linux in 52s, macOS in 1m22s, and real Windows in 1m59s.
- Scope evidence: no Python or Node suite, live scholarly/Zotero service,
  filesystem write, process launch, shell, environment lookup, or agent
  backend was used by the new preview implementation. The canonical CLI
  grammar did not change and native MCP availability remains unclaimed.
