# Qiongli 2.0 R3H Deterministic Portable Archive Execution Plan

Date: 2026-07-14

Status: active

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

- [ ] Reuse one R3G payload verifier for staging-tree and archive inputs.
- [ ] Add typed archive target approval and canonical filename helpers.
- [ ] Add deterministic ZIP writer with fixed local and central records.
- [ ] Add bounded read-only parser that accepts only the canonical ZIP profile.
- [ ] Add private create-new staging, target locking, no-replace promotion,
  persistence, cleanup, and committed archive verification.

Gate: focused platform unit tests and strict Clippy.

## Batch 3 — Implement Safe Extraction And Failure Coverage

- [ ] Commit only fully verified fixed payloads through the R3G staging path.
- [ ] Preserve an existing destination and remove partial staging on failure.
- [ ] Reject wrong identity, target, pack, entry order/name/type/mode, CRC,
  size, offset, central directory, truncation, trailing bytes, and source drift.
- [ ] Cover Unix ownership/mode/link behavior and Windows owner-only/reparse
  behavior through existing target-native helpers and CI.

Gate: deterministic, tamper, no-replace, Unix, and Windows checks.

## Batch 4 — Prove The Extracted Runtime

- [ ] Compose R3G staging, archive it, and extract into a second isolated root.
- [ ] Verify source and extracted R3G manifests and content roots agree.
- [ ] Launch only the extracted executable from outside the checkout with an
  empty `PATH` and isolated HOME/config roots.
- [ ] Verify `--version`, content identity, MCP initialize, exact Lite
  `tools/list`, and one bounded read-only tool call.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [ ] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [ ] Commit and push cohesive checkpoints on the same rolling branch.
- [ ] Monitor Native CI and Cloudflare on the exact implementation head.
- [ ] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3H closes only after exact-head target-native CI. A deterministic ZIP is not a
signed or published release artifact; later supply-chain and installed-product
gates may not be inferred from this acceptance.

