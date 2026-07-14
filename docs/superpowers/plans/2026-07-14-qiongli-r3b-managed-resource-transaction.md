# Qiongli R3B Managed Resource Transaction Implementation Plan

Status: in progress

Date: July 14, 2026

**Goal:** Close the first receipt-backed current-user filesystem transaction
vertical behind R3A without adding host adapters or false installability claims.

## Task 1 — Freeze The Executor Slice

- [x] Audit ADR 0206, ADR 0204/0205/0207, R3A plan types, the content
  materializer, config persistence, and Windows owner-only filesystem adapter.
- [x] Freeze the one-operation Lite materialization subset, explicit approval
  and root trust boundaries, persistent receipt/state/journal schemas,
  lifecycle semantics, failure policy, status wording, and nonclaims.
- [ ] Commit the design checkpoint on the rolling branch.

## Task 2 — Add Read-Only State Primitives

- [ ] Expose read-only materialized-tree verification from `qiongli-content`.
- [ ] Add the canonical missing/managed observed-state digest helper used by
  planners and the executor.
- [ ] Add strict bounded canonical receipt, lifecycle, state, and journal
  schemas with static redacted errors.

## Task 3 — Add Trust And Persistence Boundaries

- [ ] Add a private plan-approval token bound to exact digest, expiry, and
  approval set.
- [ ] Add explicit owner-only `QiongliManagedData` root approval with redacted
  debug/error output.
- [ ] Add owner-only canonical state/journal persistence, atomic replacement,
  directory sync, identity checks, and recovery-required detection on Unix
  and Windows.

## Task 4 — Implement Apply And Repair

- [ ] Revalidate the exact single materialization action and embedded pack
  before every mutation.
- [ ] Persist the journal before delegating the atomic Lite resource write.
- [ ] Verify the resulting tree and commit the active platform receipt last.
- [ ] Add idempotent identical replay and receipt-backed missing-target repair.
- [ ] Reject drift, foreign ownership, managed replacement, unsupported action
  kinds, multiple operations, nested/reserved destinations, and extra scope
  before durable mutation.

## Task 5 — Implement Verify, Remove, And Rollback

- [ ] Add read-only exact active-state verification.
- [ ] Add identity-pinned quarantine and state-commit removal/rollback.
- [ ] Restore quarantine after pre-commit failure only when safe.
- [ ] Add distinct idempotent removed and rolled-back lifecycle receipts.
- [ ] Add fault injection at journal, mutation, state commit, rollback, and
  cleanup boundaries.

## Task 6 — Compose Truthful Status And Verify

- [ ] Add the receipt contract and gated-engine state to
  `qiongli install status` while keeping grant/preview/apply unavailable.
- [ ] Update the native README, accelerated roadmap, execution receipt, and
  rolling Draft PR with observed evidence only.
- [ ] Run boundary, format, locked check, strict Clippy, and all native tests.
- [ ] Run Windows MSVC cross-target check and strict Clippy.
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
