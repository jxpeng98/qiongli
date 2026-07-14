# Qiongli 2.0 R3H Deterministic Portable Archive Execution Plan

Date: 2026-07-14

Status: accepted at `f1e5007471d1572376ce1bfd9a0f967e4a05d596`

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Design:
`docs/superpowers/specs/2026-07-14-qiongli-r3h-portable-archive-design.md`

## Batch 1 — Freeze The Container Contract

- [x] Keep ADR 0207 and the R3G manifest as identity authorities.
- [x] Select one strict store-only ZIP profile and canonical `.zip` filename.
- [x] Fix entry order, timestamp, modes, bounds, and rejected ZIP features.
- [x] Preserve signing, installation, publication, updater, cross-target, and
  clean-machine gates as explicit later work.

Gate: reviewed R3H design and execution-plan checkpoint.

## Batch 2 — Implement Canonical Composition And Verification

- [x] Reuse one R3G payload verifier for staging-tree and archive inputs.
- [x] Add typed archive target approval and canonical filename helpers.
- [x] Add deterministic ZIP writer with fixed local and central records.
- [x] Add bounded read-only parser that accepts only the canonical ZIP profile.
- [x] Add private create-new staging, target locking, no-replace promotion,
  persistence, cleanup, and committed archive verification.

Gate: focused platform unit tests and strict Clippy.

## Batch 3 — Implement Safe Extraction And Failure Coverage

- [x] Commit only fully verified fixed payloads through the R3G staging path.
- [x] Preserve an existing destination and remove partial staging on failure.
- [x] Reject wrong identity, target, pack, entry order/name/type/mode, CRC,
  size, offset, central directory, truncation, trailing bytes, and source drift.
- [x] Cover Unix ownership/mode/link behavior and Windows owner-only/reparse
  behavior through existing target-native helpers and CI.

Gate: deterministic, tamper, no-replace, Unix, and Windows checks.

## Batch 4 — Prove The Extracted Runtime

- [x] Compose R3G staging, archive it, and extract into a second isolated root.
- [x] Verify source and extracted R3G manifests and content roots agree.
- [x] Launch only the extracted executable from outside the checkout with an
  empty `PATH` and isolated HOME/config roots.
- [x] Verify `--version`, content identity, MCP initialize, exact Lite
  `tools/list`, and one bounded read-only tool call.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [x] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [x] Commit and push cohesive checkpoints on the same rolling branch.
- [x] Monitor Native CI and Cloudflare on the exact implementation head.
- [x] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3H closes only after exact-head target-native CI. A deterministic ZIP is not a
signed or published release artifact; later supply-chain and installed-product
gates may not be inferred from this acceptance.

## Local Acceptance Receipt

Accepted implementation head:
`f1e5007471d1572376ce1bfd9a0f967e4a05d596`.

- `cargo fmt --all -- --check` passed.
- Locked all-target, all-feature workspace check and strict host Clippy passed.
- Windows MSVC all-target, all-feature workspace check and strict Clippy
  passed.
- All-target, all-feature native tests passed: 220 normal tests; the two
  external-client tests remain explicitly ignored by the normal gate.
- All 50 `qiongli-platform` tests passed.
- The focused real-binary R3H archive/extraction/runtime test passed.
- All 69 focused Lite compatibility tests passed, including their existing
  ephemeral localhost fixtures.
- The native 2.x frozen-boundary check passed.

The accepted tests cover byte-for-byte deterministic archive composition,
the exact four-entry ZIP profile, bounded structural and content tampering,
private no-replace composition, safe fixed-path extraction, destination and
lock conflicts, hard links, source drift, and execution of only the extracted
binary with an empty runtime `PATH`.

## Remote Acceptance Receipt

- Design and plan checkpoint: `dad77e66`.
- Implementation checkpoint:
  `f1e5007471d1572376ce1bfd9a0f967e4a05d596`.
- Exact-head Native CI run `29357292961` passed: boundary in 7s, focused Lite
  in 37s, Windows in 6m33s, Linux in 7m12s, and macOS in 9m36s.
- Cloudflare Pages passed on the same implementation head.

This receipt does not claim a signature, notarization, checksum sidecar, SBOM,
provenance, launch grant, installer, updater, public release, packaged-window
startup, cross-target build, or clean-machine release acceptance. Python and
Node product suites are outside the frozen 2.x native migration gate and did
not run.
