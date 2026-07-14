# Qiongli 2.0 R3G Current-target Native Artifact Execution Plan

Date: 2026-07-14

Status: active

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

- [ ] Add typed current-target identity and artifact-path helpers.
- [ ] Add explicit target approval with canonical leaf and parent checks.
- [ ] Add bounded source-binary validation and deterministic manifest creation.
- [ ] Add private staging, target locking, create-new writes, no-replace commit,
  persistence, cleanup, and committed verification.
- [ ] Export only path-redacted fixed-reason errors and verified public values.

Gate: focused platform unit tests and strict Clippy.

## Batch 3 — Verify Determinism And Tamper Rejection

- [ ] Prove equivalent inputs create byte-identical canonical manifests.
- [ ] Reject invalid target identity, generic/wrong leaf, existing target,
  source link, source hard link, source mode drift, and oversized source.
- [ ] Reject manifest, binary, mode, hard-link, extra-file, extra-directory,
  target-name, entry, pack, digest, and content-root drift.
- [ ] Check Windows reparse/DACL behavior through the existing Windows security
  helper and MSVC cross-target gate.

Gate: focused deterministic, failure-path, Unix, and Windows checks.

## Batch 4 — Prove The Copied Artifact Runtime

- [ ] Compose from the canonical Cargo-built executable into an isolated
  private artifact parent.
- [ ] Launch the committed artifact binary from outside the checkout with an
  empty `PATH` and isolated HOME/config roots.
- [ ] Verify `--version` and `content list` against manifest identity.
- [ ] Verify MCP initialize, exact Lite `tools/list`, and one bounded read-only
  call over stdio.
- [ ] Retain the existing all-12-tool copied-binary MCP compatibility test.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [ ] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, and Windows MSVC check/Clippy.
- [ ] Commit and push cohesive checkpoints on the same rolling branch.
- [ ] Monitor Native CI and Cloudflare on the exact implementation head.
- [ ] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3G closes only after exact-head CI. Compression, signing, public release,
installer journeys, packaged-window startup, and clean-machine receipts remain
later alpha.1 gates and may not be inferred from a verified staging tree.
