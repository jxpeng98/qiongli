# Qiongli 2.0 R3I Verified Native Payload Installation Execution Plan

Date: 2026-07-14

Status: accepted at `25335d430d2d13977caffc92adbb813f5c00af48`

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Design:
`docs/superpowers/specs/2026-07-14-qiongli-r3i-native-payload-install-design.md`

## Batch 1 — Freeze The Installed-Payload Contract

- [x] Retain R3A signed grant, verified plan, and trusted approval boundaries.
- [x] Retain R3G payload identity and R3H strict archive verification.
- [x] Fix the canonical managed leaf, private state, journal, and quarantine
  layout.
- [x] Define fresh apply, replay, verify, absent-target repair, remove, rollback,
  recovery-required, and no-upgrade semantics.
- [x] Keep client discovery/config, UI callbacks, signing, publication, updater,
  and cross-target work outside R3I.

Gate: reviewed R3I design and execution-plan checkpoint.

## Batch 2 — Bind Plans To Verified Native Payloads

- [x] Add the bounded `InstallNativePayload` plan action without changing
  existing serialized plan semantics.
- [x] Expose the verified R3H payload manifest needed to bind archive, manifest,
  pack, content-root, and binary digests.
- [x] Add deterministic preview generation from a verified launch grant and
  verified archive.
- [x] Reject target, grant, archive, resource-pack, root, path, approval, and
  inverse mismatches before execution.

Gate: plan unit tests, existing plan fixtures, format, and strict Clippy.

## Batch 3 — Implement The Transactional Service

- [x] Add dedicated canonical native-payload receipt and state schemas.
- [x] Add owner-private state and journal writes with bounded reads, persistence,
  identity rechecks, and path-redacted failures.
- [x] Implement apply, identical replay, read-only verify, and absent-target
  repair through R3H extraction and R3G verification.
- [x] Implement verified quarantine, remove, rollback, idempotent terminal
  replay, restoration on pre-commit failure, and fail-closed recovery.
- [x] Preserve foreign, drifted, linked, or existing caller data without
  overwrite or adoption.

Gate: platform lifecycle, tamper, fault-injection, Unix, and Windows checks.

## Batch 4 — Prove The Installed Runtime

- [x] Compose and verify one real current-target R3H archive.
- [x] Create and verify an explicit test-signed launch grant and native-payload
  install plan.
- [x] Apply into an isolated approved managed root and verify the persisted
  receipt contains no private path.
- [x] Launch only the installed executable from outside the checkout and archive
  extraction tree with empty `PATH` and isolated HOME/config roots.
- [x] Verify `--version`, embedded content, MCP initialize, exact Lite
  `tools/list`, and one bounded read-only tool call.
- [x] Exercise replay, repair, remove, rollback, and tamper refusal without
  duplicating the full legacy product suites.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [x] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [x] Commit and push cohesive checkpoints on the same rolling branch.
- [x] Monitor Native CI and Cloudflare on the exact implementation head.
- [x] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3I closes only after exact-head target-native CI. A test-signed transactional
service is not a production installer or release: signing, managed-root
discovery/creation, client activation, updater, and clean-machine gates remain
explicit later work.

## Local Acceptance Receipt

Accepted implementation head:
`25335d430d2d13977caffc92adbb813f5c00af48`.

- `cargo fmt --all -- --check` passed.
- Locked all-target, all-feature workspace check and strict host Clippy passed.
- Windows MSVC all-target, all-feature workspace check and strict Clippy
  passed after correcting the Windows owner-private directory return type.
- All-target, all-feature native tests passed: 224 normal tests; the two
  external-client tests remain explicitly ignored by the normal gate.
- All 54 `qiongli-platform` tests passed.
- The current-target installed-runtime integration test passed.
- All 69 focused Lite compatibility tests passed, including their existing
  ephemeral localhost fixtures.
- The native 2.x frozen-boundary check passed on the committed implementation
  head.

The accepted tests cover signed plan and approval binding, canonical private
state, apply/replay/verify/repair/remove/rollback, commit fault restoration,
ambiguous recovery evidence, linked-state and payload-drift refusal, caller-data
preservation, and execution of only the installed binary with an empty runtime
`PATH`.

## Remote Acceptance Receipt

- Design and plan checkpoint: `33e331c9`.
- Implementation checkpoint:
  `25335d430d2d13977caffc92adbb813f5c00af48`.
- Exact-head Native CI run `29361710636` passed: boundary in 4s, focused Lite
  in 35s, Linux in 7m33s, Windows in 7m59s, and macOS in 8m12s.
- Cloudflare Pages passed on the same implementation head.

This receipt does not claim a production signing key, archive signature,
notarization, checksum sidecar, SBOM, provenance, automatic managed-root
discovery, client activation, updater, public release, packaged-window startup,
cross-target build, or clean-machine release acceptance. Python and Node
product suites remain outside the frozen 2.x native migration gate and did not
run.
