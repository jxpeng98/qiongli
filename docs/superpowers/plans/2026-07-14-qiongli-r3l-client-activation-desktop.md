# Qiongli R3L Client Activation And Desktop Intent Implementation Plan

Status: in progress

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
- [ ] Commit the design checkpoint on the rolling branch.

## Task 2 — Add The Coordinator

- [ ] Add closed Codex/Claude discovery and a path-redacted capability handle.
- [ ] Re-verify target, signed grant, generation, scope, mode, and plan before
  exposing a preview.
- [ ] Map apply/replay, verify, repair, remove, and rollback without weakening
  either accepted adapter.
- [ ] Reject target mismatch and roll back an unexpected post-apply verification
  failure.
- [ ] Add two-target lifecycle, error, redaction, and replay tests.

## Task 3 — Wire Desktop Intent

- [ ] Extend operation previews with exact plan digest and typed approvals.
- [ ] Render approvals accessibly without exposing paths or private values.
- [ ] Store the verified plan behind one operation token and require exact
  confirmation before coordinator apply.
- [ ] Refresh the validated snapshot after success and preserve non-mutating
  cancel/wrong-token behavior.
- [ ] Keep ordinary source-build sessions blocked and apply unavailable.

## Task 4 — Prove Packaged Startup

- [ ] Add exact `qiongli ui --startup-check` parsing and versioned JSON.
- [ ] Construct the same service, snapshot, and app state without opening a
  window or starting a process.
- [ ] Run a copied current-target artifact binary outside the checkout with an
  empty `PATH`.
- [ ] Keep actual clean-machine window/display/accessibility acceptance in R3M.

## Task 5 — Accept R3L

- [ ] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [ ] Commit and push implementation to the single rolling Draft PR #63.
- [ ] Require exact-head Native CI and Cloudflare Pages to pass.
- [ ] Update this receipt, native README, accelerated roadmap, and PR body with
  observed evidence only.

## Required Commands

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
python -m pytest <focused Lite compatibility selection>
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
