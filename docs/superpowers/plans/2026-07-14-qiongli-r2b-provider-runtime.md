# Qiongli R2B Provider Runtime Implementation Plan

Status: in progress

Date: July 14, 2026

**Goal:** Put provider access/status, bounded literature search, and search
planning behind the shared Rust runtime while reducing old Lite code to
compatibility adapters.

**Architecture:** Preserve native `qiongli-config` as the settings and secret
authority, add an optional native-config adapter in `qiongli-runtime`, extract
the proven Lite provider kernel into that runtime, and keep all public native
MCP claims closed until binary-level MCP dispatch is implemented later.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/Cargo.toml` | Add shared provider dependencies |
| `packages/qiongli-native/crates/qiongli-runtime/` | Add provider access, status, HTTP clients, search, cancellation, and planning |
| `packages/qiongli-native/apps/qiongli/` | Enable the native config adapter and add composition tests |
| `packages/qiongli-lite-mcp/src/config/provider_config.rs` | Convert resolved compatibility config into runtime access |
| `packages/qiongli-lite-mcp/src/providers/` | Replace provider kernel with shared re-exports/adapters |
| `packages/qiongli-lite-mcp/src/searchplan.rs` | Re-export shared deterministic planning |
| `.github/workflows/native-ci.yml` | Expand focused Rust Lite provider coverage |
| accelerated roadmap and Draft PR #63 | Record exact-head evidence only after gates pass |

## Task 1 — Freeze R2B

- [x] Inspect the accepted roadmap, native config/secret boundary, old Lite
  provider clients, search orchestration, MCP validation, and focused tests.
- [x] Freeze provider identity, access/status, request bounds, network policy,
  cancellation limitations, compatibility extraction, CI, and nonclaims.
- [ ] Commit the design and plan before production changes.

## Task 2 — Add Shared Provider Access

- [ ] Add canonical provider identities, aliases, order, and redacted status.
- [ ] Add non-serializable zeroizing in-memory provider access.
- [ ] Add an optional `qiongli-config` adapter that resolves references only
  through `SecretStore` and distinguishes unavailable storage.
- [ ] Test default arXiv readiness, disabled providers, missing secret,
  unavailable secret storage, and secret/reference redaction.

## Task 3 — Extract Provider Clients

- [ ] Move the five accepted provider request/response implementations into
  `qiongli-runtime` without changing production endpoints.
- [ ] Share a blocking HTTP client with 3-second connect timeout, 15-second
  request timeout, disabled redirects, and a 4 MiB body bound.
- [ ] Add sanitized typed HTTP, timeout, transport, decode, endpoint, and
  cancellation failures.
- [ ] Add cooperative cancellation checks around request boundaries and the
  PubMed two-step flow.
- [ ] Preserve hidden loopback endpoint injection for deterministic tests only.

## Task 4 — Add Bounded Search

- [ ] Add canonical validated search mode, provider selection, query, and
  per-provider/total limits.
- [ ] Move concurrent fan-out, canonical ordering, diagnostics, deduplication,
  and final limiting into the shared runtime.
- [ ] Retain the old `SearchInput` compatibility facade over the same kernel.
- [ ] Move deterministic search-plan generation into the shared runtime.
- [ ] Test invalid bounds before networking, cancellation, fixtures,
  deduplication, partial failure, all-failure, order, and limits.

## Task 5 — Reduce Old Lite

- [ ] Add one resolved-config-to-runtime-access adapter at the compatibility
  edge.
- [ ] Replace old provider HTTP/parser/search modules with shared re-exports or
  thin constructors.
- [ ] Replace old search planning with a shared re-export.
- [ ] Keep legacy persistence/wizard/env behavior outside the native path.
- [ ] Confirm there is no duplicate Rust provider/search kernel left.

## Task 6 — Gate And Record

- [ ] Expand Linux compatibility CI to provider HTTP, parser, orchestration,
  literature planning, and search-plan tests.
- [ ] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [ ] Run Windows MSVC cross-target check and strict Clippy.
- [ ] Run all focused old Lite Rust compatibility tests without live services.
- [ ] Audit secrets, endpoint injection, error rendering, and cancellation
  claims.
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
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test provider_http --test providers --test search_orchestration --test literature_planning_mcp --test searchplan --test mcp_protocol --test mcp_server
```

Python and Node suites remain outside this accelerated migration batch.

## Completion Definition

R2B is complete when both Rust entrypoints consume one provider/search kernel,
native settings reach it only through typed settings plus `SecretStore`, all
bounds/redaction/network/cancellation claims have deterministic proof, and
exact-head CI is green while native MCP availability remains explicitly
unclaimed.
