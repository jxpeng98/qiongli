# Qiongli REL-903 Forward-Version Immutability Acceptance

Status: accepted at Slice tier

Date: August 29, 2026

Target branch: `2.x`

Pull request: `#146`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `fc6a4eed9bbfac48970d51d9f46bf8543a7bed50` |
| Evaluation Truth | run `33218045101`: success |
| Native CI | run `33218045142`: success |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, promotion, and publication jobs were skipped
by the ordinary Slice boundary.

## Accepted forward-version contract

One named native test creates current global settings, the private project
library, and a portable project manifest through `GlobalSettingsStore` and
`ProjectStateService`. It advances only each root `schema_version` from `1` to
`2`, then exercises the normal read and inspection owners.

Future global settings return `UnsupportedSchema`; a future private project
index returns `InvalidLibraryDocument`; and a future portable manifest reports
`InspectionBlocked` and cannot produce a refresh mutation plan. The exact
future-version bytes remain unchanged after every rejected owner call. No
production persistence, migration, repair, or downgrade behavior changed.

## Verification

- Focused REL-903 native test: 1 passed.
- App library Slice: 186 passed.
- Rust 1.97 affected-app Clippy passed with warnings denied.
- Public-schema policy validator and all 12 mutation tests passed.
- Capability Contract v2 validation passed.
- Program Ledger v1 retained all 237 ordered task identities; all 7 roadmap
  tests passed.
- Exact-head Linux, macOS, Windows, Lite compatibility, change-boundary, and
  Evaluation Truth jobs passed.

## Nonclaims

This Slice does not prove disaster recovery (`REL-904`), exhaustive rejection
of every derived receipt/cache schema, candidate packaging, live Host
migration, release qualification, publication, or 1.x retirement.
