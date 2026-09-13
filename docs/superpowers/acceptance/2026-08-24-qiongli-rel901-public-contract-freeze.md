# Qiongli REL-901 Public Contract Freeze Acceptance

Status: accepted at Slice tier

Date: August 24, 2026

Target branch: `2.x`

Pull request: `#144`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `9e750e05510494747b4e2f5fa2f8fce8fac03256` |
| Evaluation Truth | run `32744003701`: success |
| Native CI | run `32744003560`: success |
| App IPC | schema `19`; exact bundled product |
| MCP | registry/schema v2; contract `2.0.0-preview.5`; Qiongli `2.x` |
| CLI JSON | schema `1`; Qiongli `2.x` |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, and promotion jobs were skipped by the
ordinary Slice boundary.

## Accepted compatibility contract

The canonical public-schema policy now freezes exactly the App IPC, MCP, and
public CLI JSON families. An unchanged public ID has immutable meaning, and
removing a frozen ID requires a separately accepted release gate.

Persisted project and global state support the current version plus two
predecessors. Migration is forward-only with rollback, while documents newer
than the running binary must fail closed and remain unmodified. `REL-902` and
`REL-903` retain ownership of executing those migration and future-version
acceptance proofs.

The validator binds App schema `19` to its Rust and TypeScript owners and binds
the MCP contract version and schema identity to the checked-in v2 registry.
No runtime schema, tool name, serialized wire shape, project file, or global
state changed.

## Verification

- The public-schema validator passed all three frozen families and the N-2
  compatibility window.
- The control-plane unit Slice passed 47 tests, including mutation coverage for
  every global compatibility and family freeze field.
- Capability Contract v2 validation passed without MCP registry drift.
- The canonical evaluation suite passed all 12 academic-quality cases.
- The generated Program Ledger index was current before acceptance closeout.

## Nonclaims

This Slice does not execute N-2 migration or rollback (`REL-902`), test
future-version file immutability (`REL-903`), package a candidate, publish an
artifact, promote Stable, retire a legacy path, or authorize a release.
