# Qiongli REL-904 Disaster Recovery Acceptance

Status: accepted at Slice tier

Date: August 29, 2026

Target branch: `2.x`

Pull request: `#147`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `bc6aa9febf07b7053fa150001d73c343193e57bb` |
| Evaluation Truth | run `33267605852`: success |
| Native CI | run `33267605850`: success |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, promotion, and publication jobs were skipped
by the ordinary Slice boundary.

## Accepted recovery contract

Five named native tests qualify the REL-904 recovery cases through their normal
owners: interrupted migration resumes without an in-memory plan; a missing
Portfolio index rebuilds only derived state; corrupt rebuildable Portfolio
state can be explicitly deleted and rebuilt; lost registration reuses the
portable manifest; and a partial catalog update recovers to the exact next
manifest.

Corrupt Portfolio cleanup remains fail-closed. It requires an approved
`delete-derived-state` plan, the exact plan digest and current Research Library
revision, and `derived-state-write`. A valid catalog appearing after preview
causes a revision conflict, and invalid transaction evidence remains a
`RecoveryRequired` error. Library and project bytes are preserved.

## Verification

- Focused REL-904 native tests: 5 passed.
- Full `qiongli-project` suite: 178 passed.
- Affected-crate Clippy passed with warnings denied; Rust formatting passed.
- Public-schema policy validator and all 12 mutation tests passed.
- Capability Contract v2 validation passed.
- Program Ledger v1 retained all 237 ordered task identities; all 7 roadmap
  tests passed.
- Exact-head Linux, macOS, Windows, Lite compatibility, change-boundary, and
  Evaluation Truth jobs passed.

## Nonclaims

This Slice does not prove general backup or restore, REL-905 policy, candidate
packaging, live Host migration, release qualification, promotion, publication,
or 1.x retirement.
