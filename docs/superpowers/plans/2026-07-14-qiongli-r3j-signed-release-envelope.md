# Qiongli 2.0 R3J Signed Native Release Envelope Execution Plan

Date: 2026-07-14

Status: active

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

- [ ] Add strict bounded canonical release-envelope and signature schemas.
- [ ] Add deterministic envelope construction and domain-separated signing
  bytes without adding private-key handling.
- [ ] Add bounded trusted release keys with generation windows and distinct
  release-key verification.
- [ ] Verify time, generation, channel, artifact, archive, R3G payload, pack,
  and independent launch-grant authority into one private verified token.

Gate: release-envelope unit tests, format, locked check, and strict Clippy.

## Batch 3 — Bind R3I To The Verified Release

- [ ] Add the release-envelope digest to the native-payload plan action and
  receipt.
- [ ] Require the verified release token for preview, apply, and repair.
- [ ] Re-verify the token-retained archive immediately before extraction.
- [ ] Preserve offline receipt-backed verify, remove, rollback, recovery, and
  caller-data safety.

Gate: R3I lifecycle and fault-injection tests.

## Batch 4 — Prove The Signed Installed Runtime

- [ ] Build and sign one current-target release envelope with distinct test-only
  release and launch-grant keys.
- [ ] Parse and verify the canonical envelope before producing the install plan.
- [ ] Install only through the verified release token and run only the managed
  installed executable with an empty runtime `PATH`.
- [ ] Reject envelope, key, generation, archive, payload, and grant tampering
  before managed mutation.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [ ] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [ ] Commit and push cohesive checkpoints on the same rolling branch.
- [ ] Monitor Native CI and Cloudflare on the exact implementation head.
- [ ] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3J closes only after exact-head target-native CI. The accepted result is a
signed release-verification boundary, not yet an end-user installer or a
published Alpha.1 artifact.
