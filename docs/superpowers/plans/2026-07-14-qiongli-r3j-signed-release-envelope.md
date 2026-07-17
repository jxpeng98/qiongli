# Qiongli 2.0 R3J Signed Native Release Envelope Execution Plan

Date: 2026-07-14

Status: complete — accepted on exact implementation head `cc33360b`

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Design:
`docs/superpowers/specs/2026-07-14-qiongli-r3j-signed-release-envelope-design.md`

## Batch 1 — Freeze The Release Trust Contract

- [x] Keep release-envelope and launch-grant keys as separate trust roles.
- [x] Fix the canonical envelope fields, signing domain, bounds, and validity
  relationships.
- [x] Define generation-window key rotation with no unsigned fallback.
- [x] Require a verified release token for R3I preview and executable mutation.
- [x] Keep automatic install UX, updater, OS signing, publication, and
  clean-machine acceptance outside R3J.

Gate: reviewed R3J design and execution-plan checkpoint.

## Batch 2 — Implement Canonical Release Verification

- [x] Add strict bounded canonical release-envelope and signature schemas.
- [x] Add deterministic envelope construction and domain-separated signing
  bytes without adding private-key handling.
- [x] Add bounded trusted release keys with generation windows and distinct
  release-key verification.
- [x] Verify time, generation, channel, artifact, archive, R3G payload, pack,
  and independent launch-grant authority into one private verified token.

Gate: release-envelope unit tests, format, locked check, and strict Clippy.

## Batch 3 — Bind R3I To The Verified Release

- [x] Add the release-envelope digest to the native-payload plan action and
  receipt.
- [x] Require the verified release token for preview, apply, and repair.
- [x] Re-verify the token-retained archive immediately before extraction.
- [x] Preserve offline receipt-backed verify, remove, rollback, recovery, and
  caller-data safety.

Gate: R3I lifecycle and fault-injection tests.

## Batch 4 — Prove The Signed Installed Runtime

- [x] Build and sign one current-target release envelope with distinct test-only
  release and launch-grant keys.
- [x] Parse and verify the canonical envelope before producing the install plan.
- [x] Install only through the verified release token and run only the managed
  installed executable with an empty runtime `PATH`.
- [x] Reject envelope, key, generation, archive, payload, and grant tampering
  before managed mutation.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [x] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [x] Commit and push cohesive checkpoints on the same rolling branch.
- [x] Monitor Native CI and Cloudflare on the exact implementation head.
- [x] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3J closes only after exact-head target-native CI. The accepted result is a
signed release-verification boundary, not yet an end-user installer or a
published Alpha.1 artifact.

## Acceptance Record

R3J is accepted at design checkpoint `3554ba69` and implementation head
`cc33360b`.

Local acceptance passed:

- native format, locked all-target/all-feature workspace check, and strict
  Clippy;
- all 227 normal native Rust tests, with the two external real-client tests
  remaining explicitly ignored;
- all 57 `qiongli-platform` tests and the signed current-target installed-
  runtime integration test;
- all 69 focused Lite compatibility tests on the complete rerun;
- Windows MSVC all-target/all-feature check and strict Clippy; and
- the committed native 2.x frozen-boundary check.

One existing loopback-session Lite test failed transiently during the first
focused invocation. It passed immediately in isolation and the complete
69-test rerun passed without a legacy Lite code change.

Exact-head Native CI run `29365515446` passed `cc33360b`: frozen boundary in
6s, focused Lite in 37s, Windows in 6m51s, Linux in 9m16s, and macOS in 9m40s.
Cloudflare Pages passed on the same head.

The accepted result remains test-signed. No production release public key or
private-key provisioning, user-facing install command, desktop mutation,
Marketplace publication, updater, OS signing/notarization, checksum sidecar,
SBOM, provenance, cross-target package, or clean-machine Alpha.1 claim exists.
