# Qiongli Config Windows Persistence Implementation Plan

Status: complete

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
- [x] Commit the design and this implementation plan (`974e539b`).

## Task 2 — Build The Isolated Win32 Security Adapter

- [x] Add `qiongli-windows-security` without weakening workspace lints.
- [x] Expose safe functions for owner-only directory/file creation, DACL
  verification, handle facts, and write-through replacement.
- [x] Keep all path and private-value data out of adapter errors.
- [x] Add Windows tests for protected single-user DACLs, broad-DACL rejection,
  handle identity/link facts, and replacement preservation.
- [x] Pass format/check/Clippy on the local host and compile in Windows CI.
- [x] Commit the isolated adapter as one rollback checkpoint (`1c3df663`).

## Task 3 — Integrate Windows With The Shared Transaction

- [x] Compile the strict encoder and private SecretRef adapter for Windows.
- [x] Generalize lock timeout, fault points, transaction bytes, and high-level
  replace/rollback flow across Unix and Windows.
- [x] Keep Unix behavior and its existing fault suite unchanged.
- [x] Add Windows private-directory, no-follow open, DACL, identity, hard-link,
  and write-through move primitives.
- [x] Change Windows status from `write-unsupported` to shared ready/missing
  semantics only after the write path exists.
- [x] Commit the integrated store as one rollback checkpoint (`90190612`).

## Task 4 — Prove The Windows Failure Matrix

- [x] Replace the old zero-side-effect unsupported-write test with first-write
  and replacement success tests.
- [x] Verify protected DACLs on state root, lock, settings, and retained
  transaction artifacts.
- [x] Run stale revision and concurrent-writer tests on Windows.
- [x] Reject state-root and settings reparse points without touching their
  targets.
- [x] Reject hard-linked settings and insecure managed objects.
- [x] Run shared pre-activation, post-activation, prior-absence, cleanup,
  rollback-failure, and lock-timeout tests on Windows.
- [x] Confirm every error/status surface remains path- and secret-redacted.
- [x] Commit the Windows acceptance tests with the integrated checkpoint to
  keep the accelerated migration history compact (`90190612`).

## Task 5 — Run The Native Gate And Close The Receipt

- [x] Run the native change-boundary check.
- [x] Run workspace format, locked check, Clippy with warnings denied, and all
  Rust tests locally.
- [x] Audit production code for Python/Node/process launch, plaintext fallback,
  and private canaries.
- [x] Update the roadmap with implementation commits, local count, exact
  nonclaims, and the next native-command batch.
- [x] Push the same rolling branch; do not create another PR.
- [x] Require exact-head boundary, Linux, macOS, and Windows jobs to pass.
- [x] Fix owned Windows findings on the same branch and rerun exact-head CI;
  the first real Windows behavior run passed, so no fix cycle was required.
- [x] Update Draft PR #63 only after current-head evidence exists.
- [x] Leave the branch pushed, Draft, clean, and synchronized with origin.

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

## Completion Receipt

- Design checkpoint: `974e539b`.
- Isolated Win32 adapter checkpoint: `1c3df663`.
- Shared transaction and Windows acceptance checkpoint: `90190612`.
- Local gate: native boundary, format, locked workspace check, Clippy with
  warnings denied, and all 84 Rust tests passed.
- Local Windows preflight: full workspace check and Clippy passed for
  `x86_64-pc-windows-msvc`, including all Windows-only test targets.
- Exact implementation CI: run `29321393463` passed boundary, Linux, macOS,
  and Windows at `90190612d2ba66b93ae98c536d688ba86ab78b34` in 6s, 31s,
  49s, and 1m29s.
- Legacy Python/Node suites were not run and were not required.
- The next batch is the R1 native content/config/status command composition;
  CFG-201B makes no CLI, UI, MCP, credential-store, project-state, migration,
  installer, or release claim.

## Completion Definition

CFG-201B is complete when Windows no longer reports write-unsupported, every
accepted Windows mutation uses a verified current-user-only DACL and
write-through transaction, the shared fault matrix has no false-success or
prior-state-loss path, all required exact-head jobs are green, and the Draft PR
contains no CLI/UI/MCP/keychain/migration claim.
