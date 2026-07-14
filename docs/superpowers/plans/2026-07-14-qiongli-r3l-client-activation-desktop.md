# Qiongli R3L Client Activation And Desktop Intent Implementation Plan

Status: accepted

Date: July 14, 2026

**Goal:** Unify the accepted Codex and Claude Code registration lifecycles and
wire exact activation preview/confirmation into the dependency-free desktop
boundary.

**Architecture:** `qiongli-platform` owns target dispatch and lifecycle mapping;
`qiongli-ui` owns typed presentation only; the application service holds the
short-lived verified plan and operation token; the canonical binary exposes a
non-mutating packaged startup check.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-platform/src/activation.rs` | Unified target discovery, preview, and lifecycle coordinator |
| `packages/qiongli-native/crates/qiongli-ui/src/model.rs` | Typed approval and digest-bound preview model |
| `packages/qiongli-native/crates/qiongli-ui/src/app.rs` | Accessible approval rendering and snapshot refresh |
| `packages/qiongli-native/apps/qiongli/src/desktop.rs` | Optional trusted activation sessions and startup preflight |
| `packages/qiongli-native/apps/qiongli/src/command.rs` | Closed `ui --startup-check` grammar and JSON output |
| native Rust tests and `packages/qiongli-native/README.md` | Coordinator, UI, copied-binary, and boundary evidence |

## Task 1 — Freeze R3L

- [x] Audit R3C, R3D, R3E, R3F, R3K, target adapters, desktop service, and
  current artifact/runtime tests.
- [x] Keep plugin-source composition and release-candidate assembly in R3M.
- [x] Freeze one-target coordination, exact approvals, desktop token binding,
  startup-check output, and nonclaims.
- [x] Commit the design checkpoint on the rolling branch.

## Task 2 — Add The Coordinator

- [x] Add closed Codex/Claude discovery and a path-redacted capability handle.
- [x] Re-verify target, signed grant, generation, scope, mode, and plan before
  exposing a preview.
- [x] Map apply/replay, verify, repair, remove, and rollback without weakening
  either accepted adapter.
- [x] Reject target mismatch and roll back an unexpected post-apply verification
  failure.
- [x] Add two-target lifecycle, error, redaction, and replay tests.

## Task 3 — Wire Desktop Intent

- [x] Extend operation previews with exact plan digest and typed approvals.
- [x] Render approvals accessibly without exposing paths or private values.
- [x] Store the verified plan behind one operation token and require exact
  confirmation before coordinator apply.
- [x] Refresh the validated snapshot after success and preserve non-mutating
  cancel/wrong-token behavior.
- [x] Keep ordinary source-build sessions blocked and apply unavailable.

## Task 4 — Prove Packaged Startup

- [x] Add exact `qiongli ui --startup-check` parsing and versioned JSON.
- [x] Construct the same service, snapshot, and app state without opening a
  window or starting a process.
- [x] Run a copied current-target artifact binary outside the checkout with an
  empty `PATH`.
- [x] Keep actual clean-machine window/display/accessibility acceptance in R3M.

## Task 5 — Accept R3L

- [x] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [x] Commit and push implementation to the single rolling Draft PR #63.
- [x] Require exact-head Native CI and Cloudflare Pages to pass.
- [x] Update this receipt, native README, accelerated roadmap, and PR body with
  observed evidence only.

## Required Commands

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked \
  --test mcp_protocol --test mcp_server --test provider_http \
  --test providers --test search_orchestration \
  --test literature_planning_mcp --test searchplan --test zotero_export \
  --test zotero_companion --test orchestrator_preview
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
```

Full legacy Python and Node suites remain outside this accelerated native
batch. The focused Lite selection is taken from the accepted Native CI gate.

## Completion Definition

R3L is complete when both supported local adapters use one target-safe
coordinator; a trusted desktop session can preview the exact plan and apply it
only after explicit confirmation; source builds remain unable to activate; a
copied native artifact passes the desktop startup preflight without a language
runtime; and all required exact-head gates pass without claiming Alpha.1
publication or clean-machine window acceptance.

## Acceptance Record

R3L is accepted at design checkpoint `789af6ba` and implementation head
`a4aa9172`.

The platform now dispatches the accepted Codex and Claude Code adapters through
one handle-bound coordinator. Preview re-verifies the target-specific signed
PluginBundle grant and complete plan, requires the exact three activation
approvals, and retains the accepted adapter behavior for apply/replay, verify,
repair, remove, rollback, recovery, and immediate post-mutation verification.
An independently discovered handle cannot apply another handle's preview.

The desktop boundary accepts only prepared trusted sessions, displays the exact
lowercase plan digest and all three approvals, and retains the verified plan
behind one OS-random 128-bit operation token. Ordinary source builds still have
`apply: false` and can only produce a blocked preview. `qiongli ui
--startup-check` constructs and validates the embedded service, snapshot, app
state, and linked window entrypoint without opening a window or starting a
subprocess.

Local acceptance passed:

- native and Lite Rust formatting checks;
- locked all-target, all-feature native workspace check and strict Clippy;
- 236 passing native Rust tests, with the two real external-client tests still
  explicitly ignored;
- the three-test target-binding and two-client coordinator suite in under one
  second after replacing a redundant full-pack fixture with a minimal canonical
  pack;
- all 69 focused Lite compatibility tests;
- Windows MSVC all-target, all-feature check and strict Clippy;
- the committed native 2.x frozen-boundary check; and
- the copied current-target artifact `ui --startup-check` proof with an empty
  runtime `PATH` and no window.

Exact implementation-head Native CI run `29373107891` passed `a4aa9172`:
frozen boundary in 6s, focused Lite in 36s, Linux in 8m17s, Windows in 8m39s,
and macOS in 11m13s. Cloudflare Pages passed on the same head.

R3L does not assemble release inputs, create or discover managed roots, select
production keys, invoke client CLIs, mutate client-owned cache or enablement
state, support desktop/cloud Marketplace bypass, display a clean-machine
window, publish an artifact, provide an updater, produce signing/SBOM/provenance
outputs, or publish `v2.0.0-alpha.1`. Those release-candidate and clean-machine
claims remain R3M gates.
