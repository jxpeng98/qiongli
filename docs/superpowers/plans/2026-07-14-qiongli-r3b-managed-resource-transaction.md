# Qiongli R3B Managed Resource Transaction Implementation Plan

Status: local implementation complete; exact-head CI pending

Date: July 14, 2026

**Goal:** Close the first receipt-backed current-user filesystem transaction
vertical behind R3A without adding host adapters or false installability claims.

## Task 1 — Freeze The Executor Slice

- [x] Audit ADR 0206, ADR 0204/0205/0207, R3A plan types, the content
  materializer, config persistence, and Windows owner-only filesystem adapter.
- [x] Freeze the one-operation Lite materialization subset, explicit approval
  and root trust boundaries, persistent receipt/state/journal schemas,
  lifecycle semantics, failure policy, status wording, and nonclaims.
- [x] Commit the design checkpoint on the rolling branch.

## Task 2 — Add Read-Only State Primitives

- [x] Expose read-only materialized-tree verification from `qiongli-content`.
- [x] Add the canonical missing/managed observed-state digest helper used by
  planners and the executor.
- [x] Add strict bounded canonical receipt, lifecycle, state, and journal
  schemas with static redacted errors.

## Task 3 — Add Trust And Persistence Boundaries

- [x] Add a private plan-approval token bound to exact digest, expiry, and
  approval set.
- [x] Add explicit owner-only `QiongliManagedData` root approval with redacted
  debug/error output.
- [x] Add owner-only canonical state/journal persistence, atomic replacement,
  directory sync, identity checks, and recovery-required detection on Unix
  and Windows.

## Task 4 — Implement Apply And Repair

- [x] Revalidate the exact single materialization action and embedded pack
  before every mutation.
- [x] Persist the root-scoped journal before delegating the atomic Lite
  resource write.
- [x] Verify the resulting tree and commit the active platform receipt last.
- [x] Add idempotent identical replay and receipt-backed missing-target repair.
- [x] Reject drift, foreign ownership, managed replacement, unsupported action
  kinds, multiple operations, nested/reserved destinations, and extra scope
  before durable mutation.

## Task 5 — Implement Verify, Remove, And Rollback

- [x] Add read-only exact active-state verification.
- [x] Add identity-pinned quarantine and state-commit removal/rollback.
- [x] Restore quarantine after pre-commit failure only when safe.
- [x] Add distinct idempotent removed and rolled-back lifecycle receipts.
- [x] Add fault injection at journal, mutation, state commit, rollback, and
  cleanup boundaries.

## Task 6 — Compose Truthful Status And Verify

- [x] Add the receipt contract and gated-engine state to
  `qiongli install status` while keeping grant/preview/apply unavailable.
- [x] Update the native README, accelerated roadmap, and execution receipt with
  observed evidence only.
- [ ] Update the rolling Draft PR with observed exact-head evidence only.
- [x] Run boundary, format, locked check, strict Clippy, and all native tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [ ] Push to Draft PR #63 and require exact-head boundary, Lite, Linux,
  macOS, Windows, and Cloudflare checks to pass.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
```

Python and Node suites remain outside this accelerated native batch.

## Completion Definition

R3B is complete when a verified and exactly approved single Lite resource plan
can safely materialize, verify, repair an absent target, remove, and roll back
inside one explicitly approved private root with canonical receipt evidence;
all drift and ambiguous recovery fail closed; and required local plus
exact-head Rust gates pass without making client-adapter, package, activation,
upgrade, or release claims.

## Local Execution Receipt

- Design and implementation plan checkpoint: `714315cd`.
- Managed transaction implementation checkpoint:
  `b3a6ea6b811ec50f891a5e32ee53820f084f857d`.
- `qiongli-platform` now accepts exactly one verified and exactly approved
  Lite materialization below an explicitly approved owner-only managed root.
  It provides fresh apply, exact replay, read-only verify, missing-target
  repair, remove, and rollback with canonical active/lifecycle receipts.
- One root-scoped immutable journal serializes all install IDs. Supported Unix
  targets validate current UID and mode, pin root/target/quarantine identity,
  and use no-replace rename for non-overwrite transitions; Windows uses the
  isolated owner-only security and write-through move adapter.
- Fault evidence covers journal, materializer ambiguity, post-materialization,
  pre-state-commit, post-state-rename ambiguity, rollback, and post-commit
  cleanup. Uncertain ownership or commit state preserves data and the journal
  and returns `install-recovery-required`.
- Local evidence passed the 2.x native boundary, format, locked workspace
  check, strict Clippy, all 177 native Rust tests, and Windows MSVC
  cross-target workspace check/strict Clippy. The first restricted-sandbox
  test run denied three loopback listener binds; the same locked test command
  passed 177/177 when the existing loopback-only harness was permitted.
- The source-built canonical binary remains read-only for installation:
  `launch_grant`, `preview`, and `apply` are still `unavailable`.
- No production signing key, packaged grant, client discovery/configuration,
  Codex/Claude registration, Marketplace/Desktop/cloud activation, package,
  updater, release, or alpha readiness was added or exercised.
- Exact-head Native CI and Cloudflare evidence remain pending until this local
  receipt is pushed to Draft PR #63.
