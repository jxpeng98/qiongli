# Current-State Audit

Date: 2026-08-11

## Candidate State

- Frozen first-usable product source:
  `cced60826ac4d7dad596669103a7e15b61868e81`.
- Native CI `31438158969` passed that exact source, including packaged product
  control and native Linux, macOS, and Windows jobs.
- Community Alpha promotion `31439930097` rebuilt and aggregated the same
  three-target candidate. Its protected publication approval was explicitly
  rejected, so the overall workflow ended in the intended failure state.
- Candidate-set SHA-256:
  `47ef8d95449472bb6f01ed91d90364f729270bec29dd8961a481d37f757cc182`;
  `publication_allowed=false`.
- Public release: `v2.0.0-alpha.1`; Alpha 3 is not published or authorized.

## Current Working Slice

PR #121 merged App API, Desktop, native Desktop adapter/service, and
`qiongli-project` changes for:

- all provider-declared configuration fields;
- one intentional Research Library preview scroll owner; and
- embedded Plugin/Skill preview plus digest-checked project-local guidance.

PR #122 then merged native Lite/Full MCP Zotero status, search, and
receipt-bound upsert parity with the canonical Skill, schemas, and endpoint-2
Companion. Both slices are present in the exact candidate above.

## Confirmed Contract Gaps

1. `content/skills/B_literature/reference-manager-bridge.md` calls
   `qiongli_zotero_upsert_references`.
2. `packages/qiongli-native/crates/qiongli-runtime/src/contract.rs` exposes only
   Zotero status and import-file export, and native status does not yet perform
   Companion search/upsert.
3. `packages/qiongli-zotero-companion/` already exposes endpoint-contract-2
   ping, search, collections, and receipt-bound dry-run/apply upsert endpoints.
4. `qiongli_project_capture_apply` is a public Full MCP write, while several
   Alpha 3 documents claimed no Full MCP mutation.
5. `tooling/release/v2.0.0-alpha.3.md` referenced the nonexistent
   `qiongli app host-actions`; the parser exposes `qiongli app snapshot`.

## Resolution Update — 2026-08-11

- Gaps 1-3 are closed in `cced6082`: native Lite/Full
  MCP now share bounded Companion status, search, and receipt-bound upsert
  dispatch with the canonical Skill and schemas.
- Gap 5 is closed in the Alpha 3 release-note source; it names
  `qiongli app snapshot`.
- Gap 4 remains a documentation boundary: capture apply is the sole declared
  approval-bound Full MCP project write, not an unrestricted mutation surface.

## Remaining Release Gaps

- The macOS App manifests 31,970,161 B and stays within its 32 MiB budget.
- The packaged native CLI is 29,428,960 B, which is 68,832 B over its frozen
  28 MiB release budget and is a release No-Go until reduced or reviewed.
- Manual Zotero/visual/workspace target claims, real system-profile Codex and
  Claude Code handoffs, update/rollback, supply-chain authorization, and public
  observation remain open under A6-A9.

## Execution Decision

The Trellis first-usable task is complete for bounded internal use. M0 remains
active because public release qualification is open; M1 stays queued and M2+
deferred. No manual/public gate or Alpha 4 work starts from this closeout.
