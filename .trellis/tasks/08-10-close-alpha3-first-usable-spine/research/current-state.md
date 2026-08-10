# Current-State Audit

Date: 2026-08-10

## Candidate State

- `HEAD` and `origin/2.x`: `19b549424cc417dd70140dbc5b3ce080848544af`.
- Native CI `31380976763` passed for that head.
- Community Alpha promotion `31382705299` rebuilt/aggregated the three targets
  and remains at protected authorization; it contains none of the current
  uncommitted App fixes.
- Public release: `v2.0.0-alpha.1`; Alpha 3 is not published or authorized.

## Current Working Slice

Branch `fix/alpha3-app-usability` modifies App API, Desktop, the native Desktop
adapter/service, and `qiongli-project` for:

- all provider-declared configuration fields;
- one intentional Research Library preview scroll owner; and
- embedded Plugin/Skill preview plus digest-checked project-local guidance.

The App API wire version moves from 16 to 17. These changes need focused checks
and a new exact candidate before they can support any release claim.

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

## Execution Decision

M0 remains active, M1 is queued, and M2+ stays deferred. The shortest reliable
path is App usability -> native Zotero contract -> truthful package -> one
exact-head CI and packaged vertical. Manual/public gates resume afterwards.
