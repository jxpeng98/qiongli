# Qiongli REL-902 Persisted-State Migration Acceptance

Status: accepted at Slice tier

Date: August 25, 2026

Target branch: `2.x`

Pull request: `#145`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `dbe1d57dac8b25e4980021e03bbb861d35b8442c` |
| Evaluation Truth | run `32904225153`: success |
| Native CI | run `32904225002`: success |
| N-1 | `v1.19.0-beta.1`; `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f` |
| N-2 | `v1.18.0-beta.3`; `12aea420bff9a3fbfa5e421c482ae8da2588c9ed` |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, promotion, and publication jobs were skipped
by the ordinary Slice boundary.

## Accepted migration and rollback contract

One checked-in fixture manifest binds exactly the two published predecessors
above. For each row, one native test materializes an isolated 1.x project and
legacy `providers.json`, then uses `ProjectStateService` and the existing
legacy-provider stage, verify, and rollback owners.

Before rollback, the migrated project is registered and readable, and provider
plaintext secrets exist only in the memory-only secret store while current
settings hold a `SecretRef`. After rollback, the migration-owned destination
and registration are absent, prior current settings and secrets are restored,
and the complete legacy project shape, project bytes, and provider bytes are
unchanged.

## Verification

- Focused REL-902 native test: 1 passed for both predecessor rows.
- App library Slice: 185 passed.
- Rust 1.97 affected-crate Clippy passed with warnings denied.
- Public-schema policy validator and all 12 mutation tests passed.
- Program Ledger v1 retained all 237 ordered task identities.
- Exact-head Linux, macOS, Windows, Lite compatibility, change-boundary, and
  Evaluation Truth jobs passed.

## Nonclaims

This Slice does not prove future-version file rejection (`REL-903`), disaster
recovery (`REL-904`), legacy runtime execution, candidate packaging, live Host
migration, release qualification, publication, or 1.x retirement.
