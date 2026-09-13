# Qiongli R3A Install Plan And Lite Launch Grant Implementation Plan

Status: complete

Date: July 14, 2026

**Goal:** Establish the strict, deterministic, signature-gated installation
contract that every later CLI, UI, adapter, transaction, and package uses.

**Architecture:** Add `qiongli-platform` for artifact identity, signed Lite
launch-grant verification, and declarative install plans. Keep the source-built
canonical app unpackaged and expose only truthful read-only install status.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/Cargo.toml` | Add the `qiongli-platform` workspace member and shared cryptographic dependencies |
| `packages/qiongli-native/crates/qiongli-platform/` | New strict artifact, grant, target, operation, and install-plan contract crate |
| `packages/qiongli-native/apps/qiongli/` | Compose read-only `install status` without a fabricated grant or adapter |
| `packages/qiongli-native/apps/qiongli/tests/cli.rs` | Empty-runtime copied-binary status and nonclaim checks |
| `packages/qiongli-native/README.md` | Document the R3A boundary and exact noncapabilities |
| accelerated roadmap and Draft PR #63 | Record only observed exact-head evidence |

## Task 1 — Freeze R3A

- [x] Audit ADR 0206, ADR 0207, R3, PLT-201/202, current artifact registry,
  canonical app composition, and native CI.
- [x] Freeze the artifact tuple, Lite grant schema, signature/trust model,
  typed plan model, semantic digest, bounds, CLI status, and nonclaims.
- [x] Reject embedded development private keys, caller-selected trust roots,
  host-cache writes, and premature adapter/activation claims.
- [x] Commit the design checkpoint on the rolling branch.

## Task 2 — Add Artifact And Grant Contracts

- [x] Add closed product/channel/profile/OS/architecture/installer enums and
  strict SemVer/channel validation.
- [x] Add bounded strict JSON for `LaunchGrantV1` and its Ed25519 signature
  envelope.
- [x] Canonicalize domain-separated grant bytes and verify against injected
  trusted public keys.
- [x] Bind time window, minimum generation, artifact tuple, binary/resource
  pack digests, requested mode, and requested local integration scope.
- [x] Return an unforgeable `VerifiedLaunchGrant` and static reason-coded
  failures.
- [x] Test signature success plus tampering, expiry, replay, identity, digest,
  mode, scope, schema, size, and redaction failures.

## Task 3 — Add Declarative Install Plan

- [x] Add exact target, scope, surface, allowed-root, state, approval, and host
  action vocabularies.
- [x] Add typed materialize, plugin-source, Lite-MCP, and managed-remove
  actions with explicit inverse and postcondition data.
- [x] Reject unknown roots, traversal, unsorted/duplicate IDs, excess counts,
  mismatched targets, non-Lite profiles, and non-invertible operations.
- [x] Compute the canonical semantic digest excluding only plan identity and
  display timestamps.
- [x] Parse and revalidate bounded plan JSON, expiry, signed grant, and digest.
- [x] Test deterministic equivalent previews and semantic mutation families.

## Task 4 — Expose Truthful Source-Build Status

- [x] Add closed `qiongli install status` grammar and help.
- [x] Report current compiled target and contract-only local target families.
- [x] Report launch grant, preview, and apply as unavailable without performing
  client discovery, config/home access, network, process, or filesystem I/O.
- [x] Add copied-binary empty-`PATH` and no-home proof plus unknown/extra
  argument rejection.

## Task 5 — Verify And Record

- [x] Run native boundary, format, locked check, strict Clippy, and all native
  tests.
- [x] Run Windows MSVC cross-target check and strict Clippy.
- [x] Review public errors/output for secrets, paths, attacker input, and false
  installation claims.
- [x] Commit and push the implementation checkpoint to rolling Draft PR #63.
- [x] Require exact-head boundary, Linux, macOS, and Windows jobs to pass.
- [x] Update this receipt, roadmap, native README, and PR body with observed
  evidence only.

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

R3A is complete when only a trusted, valid, exact Lite artifact grant can
produce a verified token; every install plan is strict, bounded, typed,
invertible, target-matched, and deterministically digestible; the ordinary
source binary reports that it is not packaged or installable; and required
local plus exact-head CI gates pass without making adapter, mutation, UI, or
release claims.

## Execution Receipt

- Design and implementation plan checkpoint: `60c50526`.
- Platform contract implementation checkpoint:
  `60c2ddc5dc21bfefc5c4767b9a3275614c8fae26`.
- `qiongli-platform` now owns closed artifact identities, strict bounded
  launch-grant envelopes, injected Ed25519 trust roots, private verified
  capability tokens, typed invertible install plans, and deterministic
  semantic digests.
- Plan verification rejects stale or not-yet-valid plans, foreign ownership,
  missing approvals, duplicate destinations, traversal and reserved names,
  non-Lite grants, target mismatches, and mode/scope token reuse.
- `qiongli install status` is intentionally read-only. It reports the compiled
  target and contract-only local families while launch grant, preview, and
  apply remain unavailable; copied-binary proof ran with empty `PATH` and no
  home directory.
- Local evidence passed the 2.x native boundary, format, locked workspace
  check, strict Clippy, all 161 native Rust tests, and Windows MSVC
  cross-target check/Clippy.
- Tests use runtime-generated ephemeral signing keys. No production private
  key, publisher trust root, signed artifact, real install plan, filesystem
  executor, host discovery, client activation, packaging, or release was
  added or exercised.
- Native CI run `29332864357` passed exact implementation-and-local-receipt
  head `a971ae1ecba9f19b85e7dbe3b782b40a69502cf8`: boundary in 8s, focused
  Lite compatibility in 37s, Linux in 1m08s, macOS in 1m05s, and real Windows
  in 2m19s. Cloudflare Pages also passed on Draft PR #63.
