# Qiongli R2B Provider Runtime Implementation Plan

Status: complete

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
- [x] Commit the design and plan before production changes.

## Task 2 — Add Shared Provider Access

- [x] Add canonical provider identities, aliases, order, and redacted status.
- [x] Add non-serializable zeroizing in-memory provider access.
- [x] Add an optional `qiongli-config` adapter that resolves references only
  through `SecretStore` and distinguishes unavailable storage.
- [x] Test default arXiv readiness, disabled providers, missing secret,
  unavailable secret storage, and secret/reference redaction.

## Task 3 — Extract Provider Clients

- [x] Move the five accepted provider request/response implementations into
  `qiongli-runtime` without changing production endpoints.
- [x] Share a blocking HTTP client with 3-second connect timeout, 15-second
  request timeout, disabled redirects, and a 4 MiB body bound.
- [x] Add sanitized typed HTTP, timeout, transport, decode, endpoint, and
  cancellation failures.
- [x] Add cooperative cancellation checks around request boundaries and the
  PubMed two-step flow.
- [x] Preserve hidden loopback endpoint injection for deterministic tests only.

## Task 4 — Add Bounded Search

- [x] Add canonical validated search mode, provider selection, query, and
  per-provider/total limits.
- [x] Move concurrent fan-out, canonical ordering, diagnostics, deduplication,
  and final limiting into the shared runtime.
- [x] Retain the old `SearchInput` compatibility facade over the same kernel.
- [x] Move deterministic search-plan generation into the shared runtime.
- [x] Test invalid bounds before networking, cancellation, fixtures,
  deduplication, partial failure, all-failure, order, and limits.

## Task 5 — Reduce Old Lite

- [x] Add one resolved-config-to-runtime-access adapter at the compatibility
  edge.
- [x] Replace old provider HTTP/parser/search modules with shared re-exports or
  thin constructors.
- [x] Replace old search planning with a shared re-export.
- [x] Keep legacy persistence/wizard/env behavior outside the native path.
- [x] Confirm there is no duplicate Rust provider/search kernel left.

## Task 6 — Gate And Record

- [x] Expand Linux compatibility CI to provider HTTP, parser, orchestration,
  literature planning, and search-plan tests.
- [x] Run native boundary, format, locked check, strict Clippy, and all native
  Rust tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [x] Run all focused old Lite Rust compatibility tests without live services.
- [x] Audit secrets, endpoint injection, error rendering, and cancellation
  claims.
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
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked --test provider_http --test providers --test search_orchestration --test literature_planning_mcp --test searchplan --test mcp_protocol --test mcp_server
```

Python and Node suites remain outside this accelerated migration batch.

## Completion Definition

R2B is complete when both Rust entrypoints consume one provider/search kernel,
native settings reach it only through typed settings plus `SecretStore`, all
bounds/redaction/network/cancellation claims have deterministic proof, and
exact-head CI is green while native MCP availability remains explicitly
unclaimed.

## Execution Receipt

- Design and implementation plan: `a7a505eb`.
- Shared provider/search implementation: `2eaadfb1`.
- Local native evidence: change boundary, format, locked workspace check,
  strict Clippy, and all 116 native Rust tests passed.
- Cross-target evidence: Windows MSVC workspace check and strict Clippy passed;
  the Windows dependency graph uses SChannel and does not cross-compile Ring.
- Compatibility evidence: all 57 focused old Lite tests and strict all-target
  Clippy passed. Loopback coverage includes each provider's request/auth shape,
  redirect refusal, timeout, 4 MiB response rejection, PubMed's two-step and
  over-return bounds, orchestration, deduplication, planning, and MCP behavior.
- GitHub evidence: Native CI run `29326303112` passed exact implementation head
  `2eaadfb137bb26a71ab578f0c2bcdf6dc140a869`: boundary in 5s, focused Lite in
  35s, Linux in 1m02s, macOS in 1m28s, and real Windows in 1m53s.
- Scope evidence: no live scholarly service, Python suite, or Node suite ran;
  canonical CLI grammar did not change; native MCP mode, production secure
  storage, evidence, Zotero, orchestration, UI, integration installation, and
  packaging remain outside R2B.
