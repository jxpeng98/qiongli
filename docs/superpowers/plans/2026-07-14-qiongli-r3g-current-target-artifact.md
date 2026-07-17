# Qiongli 2.0 R3G Current-target Native Artifact Execution Plan

Date: 2026-07-14

Status: accepted at `7fddca1535028a18ac5864372dc8f349c8472270`

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Design:
`docs/superpowers/specs/2026-07-14-qiongli-r3g-current-target-artifact-design.md`

## Batch 1 — Freeze Identity And Staging Contract

- [x] Keep ADR 0207 as the canonical release-identity authority.
- [x] Select current-target `lite` plus `portable-archive` as the R3G tuple.
- [x] Define the canonical artifact ID, staging layout, manifest, bounds, and
  `assembled-unpublished` status.
- [x] Preserve signing, compression, publication, installer, updater, and
  clean-machine work as explicit later gates.

Gate: reviewed design and execution-plan checkpoint.

## Batch 2 — Implement The Platform Artifact Boundary

- [x] Add typed current-target identity and artifact-path helpers.
- [x] Add explicit target approval with canonical leaf and parent checks.
- [x] Add bounded source-binary validation and deterministic manifest creation.
- [x] Add private staging, target locking, create-new writes, no-replace commit,
  persistence, cleanup, and committed verification.
- [x] Export only path-redacted fixed-reason errors and verified public values.

Gate: focused platform unit tests and strict Clippy.

## Batch 3 — Verify Determinism And Tamper Rejection

- [x] Prove equivalent inputs create byte-identical canonical manifests.
- [x] Reject invalid target identity, generic/wrong leaf, existing target,
  source link, source hard link, source mode drift, and oversized source.
- [x] Reject manifest, binary, mode, hard-link, extra-file, extra-directory,
  target-name, entry, pack, digest, and content-root drift.
- [x] Check Windows reparse/DACL behavior through the existing Windows security
  helper and MSVC cross-target gate.

Gate: focused deterministic, failure-path, Unix, and Windows checks.

## Batch 4 — Prove The Copied Artifact Runtime

- [x] Compose from the canonical Cargo-built executable into an isolated
  private artifact parent.
- [x] Launch the committed artifact binary from outside the checkout with an
  empty `PATH` and isolated HOME/config roots.
- [x] Verify `--version` and `content list` against manifest identity.
- [x] Verify MCP initialize, exact Lite `tools/list`, and one bounded read-only
  call over stdio.
- [x] Retain the existing all-12-tool copied-binary MCP compatibility test.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [x] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, and Windows MSVC check/Clippy.
- [x] Commit and push cohesive checkpoints on the same rolling branch.
- [x] Monitor Native CI and Cloudflare on the exact implementation head.
- [x] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3G closes only after exact-head CI. Compression, signing, public release,
installer journeys, packaged-window startup, and clean-machine receipts remain
later alpha.1 gates and may not be inferred from a verified staging tree.

## Local Acceptance Receipt

Implementation head `df493e13e2830bc6c9ab40b59ec36247d020353a`
passes:

- native formatting, locked all-target/all-feature workspace check, and strict
  host Clippy with warnings denied;
- Windows MSVC all-target/all-feature workspace check and strict Clippy;
- all native workspace tests: 215 passed and two explicit external-client
  tests remained ignored by the normal gate;
- both focused R3G application tests, including deterministic manifest and
  tamper/conflict/unsafe-source rejection coverage;
- all 69 focused Lite compatibility tests; and
- the native 2.x frozen-boundary check.

The copied artifact acceptance runs only `bin/qiongli[.exe]` from the committed
artifact tree, outside the checkout and with an empty runtime `PATH`. It proves
`--version`, verified content listing, MCP initialization, the exact 12-tool
Lite surface, and one bounded read-only MCP call. The verifier also requires a
caller-supplied verified resource pack as an external content-identity anchor.

Windows-only test-fixture correction head
`7fddca1535028a18ac5864372dc8f349c8472270` passes formatting plus the Windows
MSVC workspace check and strict Clippy locally. It changes only the test parent
directory constructor to use the existing owner-only Windows helper; production
target validation remains fail-closed.

## Remote Acceptance Receipt

- Design and execution-plan checkpoint: `0b895a88`.
- Implementation checkpoint: `df493e13`.
- Accepted implementation-and-CI head: `7fddca15`.
- Native CI run `29353736596` on `df493e13` passed the boundary, Lite, Linux,
  and macOS jobs. Its Windows R3G fixtures failed because their default DACL
  did not satisfy the production owner-only target rule.
- Native CI run `29354332680` on exact head `7fddca15` passed the native 2.x
  boundary in 7s, focused Lite in 32s, Linux in 5m22s, macOS in 6m34s, and
  Windows in 6m59s. Cloudflare Pages also passed.

This receipt accepts an `assembled-unpublished` staging tree only. It does not
claim a compressed archive, signature, notarization, checksum sidecar, SBOM,
provenance statement, launch grant, installer, updater, public artifact,
packaged-window startup, cross-target build, or clean-machine acceptance.
