# Qiongli R2C Evidence And Zotero Runtime Implementation Plan

Status: in progress

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
- [ ] Commit the design and plan before production changes.

## Task 2 — Add Shared Evidence Snapshots

- [ ] Add typed canonical/alias parsing and reject ambiguous pairs.
- [ ] Bound query, result count, JSON depth/value count, and total input.
- [ ] Recursively remove credential-bearing keys while retaining benign keys.
- [ ] Return a deterministic snapshot with no path or wall-clock leakage.
- [ ] Test direct runtime behavior independently of MCP framing.

## Task 3 — Add Shared Zotero Export

- [ ] Add exact format identities, selection validation, and record bounds.
- [ ] Emit deterministic CSL-JSON, RIS, BibTeX, and report contents.
- [ ] Fold RIS control characters and escape BibTeX syntax-bearing values.
- [ ] Enforce aggregate input/output bounds before returning contents.
- [ ] Test selected formats, malformed records, injection strings, and limits.

## Task 4 — Add Shared Zotero Probe

- [ ] Move loopback URL validation and status types into the runtime.
- [ ] Add redirect denial, implicit-proxy denial, fixed paths, bounded timeout,
  and bounded response reads.
- [ ] Return static sanitized status without URL/body/library disclosure.
- [ ] Test loopback variants, non-loopback/credential rejection, redirects,
  malformed/oversized responses, disabled behavior, and timeout.

## Task 5 — Reduce Old Lite

- [ ] Replace evidence construction with shared parsing/building.
- [ ] Replace Zotero exporter code with shared re-exports.
- [ ] Retain only the historical environment-to-client adapter beside shared
  companion re-exports.
- [ ] Map shared errors to static existing JSON-RPC messages.
- [ ] Confirm no duplicate Rust evidence/Zotero kernel remains.

## Task 6 — Gate And Record

- [ ] Expand focused Linux Lite CI to evidence and Zotero tests.
- [ ] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [ ] Run Windows MSVC cross-target check and strict Clippy.
- [ ] Run focused old Lite Rust compatibility tests without live services.
- [ ] Commit and push the cohesive implementation to rolling Draft PR #63.
- [ ] Require exact-head boundary, Linux, macOS, Windows, and focused Lite jobs
  to pass.
- [ ] Record observed evidence in this plan, the roadmap, and PR while leaving
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
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test mcp_protocol --test mcp_server --test zotero_export --test zotero_companion
```

Python and Node suites remain outside this accelerated migration batch.

## Completion Definition

R2C is complete when both Rust entrypoints consume one bounded evidence/Zotero
implementation, the old environment behavior remains isolated at the
compatibility edge, direct runtime tests prove the declared security limits,
and exact-head CI is green while native MCP availability remains explicitly
unclaimed.
