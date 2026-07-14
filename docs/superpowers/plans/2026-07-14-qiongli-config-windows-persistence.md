# Qiongli Config Windows Persistence Implementation Plan

Status: active execution

Date: July 14, 2026

Task: `CFG-201B`

**Goal:** Add protected, revision-safe Windows global-settings persistence to
the accepted CFG-201A service without weakening its schema, redaction, or
fail-closed behavior.

**Architecture:** Isolate all Win32 FFI in a new safe-boundary
`qiongli-windows-security` crate. Generalize the existing tested transaction
orchestration across Unix and Windows, while keeping target-specific creation,
permission, identity, replacement, and prior-absence rollback primitives
small and explicit.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/Cargo.toml` | Add the isolated workspace crate |
| `packages/qiongli-native/Cargo.lock` | Record the local crate dependency |
| `packages/qiongli-native/crates/qiongli-windows-security/` | Add safe Win32 security/replace boundary and Windows tests |
| `packages/qiongli-native/crates/qiongli-config/Cargo.toml` | Add target-specific Windows dependency |
| `packages/qiongli-native/crates/qiongli-config/src/document.rs` | Compile the private encoder for Windows writes |
| `packages/qiongli-native/crates/qiongli-config/src/secret.rs` | Compile private secret-ref serialization for Windows writes |
| `packages/qiongli-native/crates/qiongli-config/src/store.rs` | Share transaction logic and add Windows primitives |
| `packages/qiongli-native/crates/qiongli-config/tests/store_contract.rs` | Add real Windows persistence/security contracts |
| `.github/workflows/native-ci.yml` | Change only if a dedicated acceptance command is proven necessary |
| accelerated roadmap and Draft PR #63 | Record only exact observed evidence |

## Task 1 — Freeze The Windows Boundary

- [x] Review CFG-201A follow-on requirements and ADR 0204.
- [x] Reuse the accepted Rust Lite protected-DACL semantics.
- [x] Verify `CreateFileW`, `MoveFileExW`, `GetSecurityInfo`, file identity, and
  Rust locking against primary documentation.
- [x] Record the unsafe-code isolation and explicit nonclaims in the design.
- [ ] Commit the design and this implementation plan.

## Task 2 — Build The Isolated Win32 Security Adapter

- [ ] Add `qiongli-windows-security` without weakening workspace lints.
- [ ] Expose safe functions for owner-only directory/file creation, DACL
  verification, handle facts, and write-through replacement.
- [ ] Keep all path and private-value data out of adapter errors.
- [ ] Add Windows tests for protected single-user DACLs, broad-DACL rejection,
  handle identity/link facts, and replacement preservation.
- [ ] Pass format/check/Clippy on the local host and compile in Windows CI.
- [ ] Commit the isolated adapter as one rollback checkpoint.

## Task 3 — Integrate Windows With The Shared Transaction

- [ ] Compile the strict encoder and private SecretRef adapter for Windows.
- [ ] Generalize lock timeout, fault points, transaction bytes, and high-level
  replace/rollback flow across Unix and Windows.
- [ ] Keep Unix behavior and its existing fault suite unchanged.
- [ ] Add Windows private-directory, no-follow open, DACL, identity, hard-link,
  and write-through move primitives.
- [ ] Change Windows status from `write-unsupported` to shared ready/missing
  semantics only after the write path exists.
- [ ] Commit the integrated store as one rollback checkpoint.

## Task 4 — Prove The Windows Failure Matrix

- [ ] Replace the old zero-side-effect unsupported-write test with first-write
  and replacement success tests.
- [ ] Verify protected DACLs on state root, lock, settings, and retained
  transaction artifacts.
- [ ] Run stale revision and concurrent-writer tests on Windows.
- [ ] Reject state-root and settings reparse points without touching their
  targets.
- [ ] Reject hard-linked settings and insecure managed objects.
- [ ] Run shared pre-activation, post-activation, prior-absence, cleanup,
  rollback-failure, and lock-timeout tests on Windows.
- [ ] Confirm every error/status surface remains path- and secret-redacted.
- [ ] Commit the Windows acceptance tests.

## Task 5 — Run The Native Gate And Close The Receipt

- [ ] Run the native change-boundary check.
- [ ] Run workspace format, locked check, Clippy with warnings denied, and all
  Rust tests locally.
- [ ] Audit production code for Python/Node/process launch, plaintext fallback,
  and private canaries.
- [ ] Update the roadmap with implementation commits, local count, exact
  nonclaims, and the next native-command batch.
- [ ] Push the same rolling branch; do not create another PR.
- [ ] Require exact-head boundary, Linux, macOS, and Windows jobs to pass.
- [ ] Fix owned Windows findings on the same branch and rerun exact-head CI.
- [ ] Update Draft PR #63 only after current-head evidence exists.
- [ ] Leave the branch pushed, Draft, clean, and synchronized with origin.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
```

Legacy Python and Node suites remain diagnostic-only and are not part of this
CFG-201B gate.

## Completion Definition

CFG-201B is complete when Windows no longer reports write-unsupported, every
accepted Windows mutation uses a verified current-user-only DACL and
write-through transaction, the shared fault matrix has no false-success or
prior-state-loss path, all required exact-head jobs are green, and the Draft PR
contains no CLI/UI/MCP/keychain/migration claim.
