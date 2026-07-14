# Qiongli 2.0 R3I Verified Native Payload Installation Execution Plan

Date: 2026-07-14

Status: active

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

- [ ] Add the bounded `InstallNativePayload` plan action without changing
  existing serialized plan semantics.
- [ ] Expose the verified R3H payload manifest needed to bind archive, manifest,
  pack, content-root, and binary digests.
- [ ] Add deterministic preview generation from a verified launch grant and
  verified archive.
- [ ] Reject target, grant, archive, resource-pack, root, path, approval, and
  inverse mismatches before execution.

Gate: plan unit tests, existing plan fixtures, format, and strict Clippy.

## Batch 3 — Implement The Transactional Service

- [ ] Add dedicated canonical native-payload receipt and state schemas.
- [ ] Add owner-private state and journal writes with bounded reads, persistence,
  identity rechecks, and path-redacted failures.
- [ ] Implement apply, identical replay, read-only verify, and absent-target
  repair through R3H extraction and R3G verification.
- [ ] Implement verified quarantine, remove, rollback, idempotent terminal
  replay, restoration on pre-commit failure, and fail-closed recovery.
- [ ] Preserve foreign, drifted, linked, or existing caller data without
  overwrite or adoption.

Gate: platform lifecycle, tamper, fault-injection, Unix, and Windows checks.

## Batch 4 — Prove The Installed Runtime

- [ ] Compose and verify one real current-target R3H archive.
- [ ] Create and verify an explicit test-signed launch grant and native-payload
  install plan.
- [ ] Apply into an isolated approved managed root and verify the persisted
  receipt contains no private path.
- [ ] Launch only the installed executable from outside the checkout and archive
  extraction tree with empty `PATH` and isolated HOME/config roots.
- [ ] Verify `--version`, embedded content, MCP initialize, exact Lite
  `tools/list`, and one bounded read-only tool call.
- [ ] Exercise replay, repair, remove, rollback, and tamper refusal without
  duplicating the full legacy product suites.

Gate: current-target application integration test plus focused Lite suite.

## Batch 5 — Acceptance And Rolling PR

- [ ] Run format, locked workspace check, strict Clippy, all native tests,
  focused Lite compatibility, Windows MSVC check/Clippy, and frozen boundary.
- [ ] Commit and push cohesive checkpoints on the same rolling branch.
- [ ] Monitor Native CI and Cloudflare on the exact implementation head.
- [ ] Update the accelerated roadmap, native README, this acceptance record,
  and Draft PR #63 with factual evidence and non-claims.

R3I closes only after exact-head target-native CI. A test-signed transactional
service is not a production installer or release: signing, managed-root
discovery/creation, client activation, updater, and clean-machine gates remain
explicit later work.
