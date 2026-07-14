# Qiongli R2C Evidence And Zotero Runtime Implementation Plan

Status: complete

Date: July 14, 2026

**Goal:** Put bounded evidence snapshots and the accepted read-only/in-memory
Zotero operations behind `qiongli-runtime` while preserving the Lite MCP as a
thin compatibility consumer.

**Architecture:** Add shared evidence and Zotero domain modules, keep legacy
environment resolution outside the native path, and leave canonical MCP
availability closed until binary-level dispatch is implemented.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-runtime/src/evidence.rs` | Add bounded redacted evidence snapshots |
| `packages/qiongli-native/crates/qiongli-runtime/src/zotero/` | Add bounded exporter and loopback probe |
| `packages/qiongli-lite-mcp/src/mcp/server.rs` | Delegate evidence/export validation and construction |
| `packages/qiongli-lite-mcp/src/zotero/` | Reduce to re-exports and environment adapter |
| `.github/workflows/native-ci.yml` | Add focused Lite evidence/Zotero coverage |
| accelerated roadmap and Draft PR #63 | Record exact-head evidence after gates pass |

## Task 1 — Freeze R2C

- [x] Inspect Contract v2, the roadmap, current Lite evidence/Zotero behavior,
  existing tests, and shared runtime boundaries.
- [x] Freeze operations, bounds, redaction, loopback/network policy,
  compatibility ownership, verification, and nonclaims.
- [x] Commit the design and plan before production changes.

## Task 2 — Add Shared Evidence Snapshots

- [x] Add typed canonical/alias parsing and reject ambiguous pairs.
- [x] Bound query, result count, JSON depth/value count, and total input.
- [x] Recursively remove credential-bearing keys while retaining benign keys.
- [x] Return a deterministic snapshot with no path or wall-clock leakage.
- [x] Test direct runtime behavior independently of MCP framing.

## Task 3 — Add Shared Zotero Export

- [x] Add exact format identities, selection validation, and record bounds.
- [x] Emit deterministic CSL-JSON, RIS, BibTeX, and report contents.
- [x] Fold RIS control characters and escape BibTeX syntax-bearing values.
- [x] Enforce aggregate input/output bounds before returning contents.
- [x] Test selected formats, malformed records, injection strings, and limits.

## Task 4 — Add Shared Zotero Probe

- [x] Move loopback URL validation and status types into the runtime.
- [x] Add redirect denial, implicit-proxy denial, fixed paths, bounded timeout,
  and bounded response reads.
- [x] Return static sanitized status without URL/body/library disclosure.
- [x] Test loopback variants, non-loopback/credential rejection, redirects,
  malformed/oversized responses, disabled behavior, and timeout.

## Task 5 — Reduce Old Lite

- [x] Replace evidence construction with shared parsing/building.
- [x] Replace Zotero exporter code with shared re-exports.
- [x] Retain only the historical environment-to-client adapter beside shared
  companion re-exports.
- [x] Map shared errors to static existing JSON-RPC messages.
- [x] Confirm no duplicate Rust evidence/Zotero kernel remains.

## Task 6 — Gate And Record

- [x] Expand focused Linux Lite CI to evidence and Zotero tests.
- [x] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [x] Run focused old Lite Rust compatibility tests without live services.
- [x] Commit and push the cohesive implementation to rolling Draft PR #63.
- [x] Require exact-head boundary, Linux, macOS, Windows, and focused Lite jobs
  to pass.
- [x] Record observed evidence in this plan, the roadmap, and PR while leaving
  R2 open.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test mcp_protocol --test mcp_server --test provider_http --test providers --test search_orchestration --test literature_planning_mcp --test searchplan --test zotero_export --test zotero_companion
```

Python and Node suites remain outside this accelerated migration batch.

## Completion Definition

R2C is complete when both Rust entrypoints consume one bounded evidence/Zotero
implementation, the old environment behavior remains isolated at the
compatibility edge, direct runtime tests prove the declared security limits,
and exact-head CI is green while native MCP availability remains explicitly
unclaimed.

## Execution Receipt

- Design and implementation plan: `68558ed0`.
- Shared evidence/Zotero implementation: `2513c52f`.
- Local native evidence: change boundary, native and Lite format, locked
  workspace check, strict native and Lite Clippy, all 131 native Rust tests,
  and Windows MSVC cross-target check/Clippy passed.
- Compatibility evidence: all 67 focused old Lite tests passed without a live
  provider or Zotero installation. The set covers framing, dispatch, provider
  HTTP/parsers/search, evidence redaction, selected exports, loopback rules,
  redirects, malformed/oversized responses, and timeout behavior.
- GitHub evidence: Native CI run `29327768079` passed exact implementation head
  `2513c52f9a39cb97a6d41f282dcbdff920eac979`: boundary in 4s, focused Lite in
  32s, Linux in 1m08s, macOS in 1m39s, and real Windows in 1m53s.
- Scope evidence: no live Zotero or scholarly service, Python suite, or Node
  suite ran. The canonical CLI grammar did not change; native MCP mode, Zotero
  library mutation/search, file writes, companion installation, production
  secure storage, orchestration execution, UI, installation, and packaging
  remain outside R2C.
