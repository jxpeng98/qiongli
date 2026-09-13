# Qiongli REL-905 Data Lifecycle Policy Acceptance

Status: accepted at Slice tier

Date: August 29, 2026

Target branch: `2.x`

Pull request: `#148`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `f046376890efd7d44acd250e1eff5f769c0e0243` |
| Evaluation Truth | run `33270228317`: success |
| Native CI | run `33270228293`: success |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, promotion, and publication jobs were skipped by the ordinary Slice
boundary. Cloudflare Pages also passed for the same pull-request head.

## Accepted policy contract

The English and Simplified Chinese Guide pages now define the user-owned project
and global v2 roots, secure credential boundary, Host- and provider-owned data,
and a stopped-writer backup checkpoint. Portable project export is explicitly a
privacy-filtered exchange format rather than a complete backup.

Removal of receipt- or Host-owned integrations is separate from deliberate data
deletion. The policy preserves project directories, global state, credentials,
Host records, provider records, and required 1.x recovery sources unless the
user intentionally removes the exact retained data.

The accepted 1.x rule remains source-bound to the release branch policy:
`v1.19.0-beta.1` is the final feature-bearing 1.x release and the planned
support window ends 90 days after actual Qiongli 2 Stable publication. Alpha,
Beta, REL-905, and ordinary merges do not start that clock.

## Verification

- Focused REL-905 policy test: 1 passed.
- Evaluation Truth architecture and policy suite: 48 passed.
- VitePress documentation build passed.
- Program Ledger v1 retained all 237 ordered task identities; all 7 roadmap
  tests passed.
- Trellis task validation and `git diff --check` passed.
- Exact-head Linux, macOS, Windows, Lite compatibility, change-boundary,
  Evaluation Truth, and Cloudflare Pages checks passed.

## Nonclaims

This Slice adds no backup, restore, purge, credential-export, migration,
packaging, promotion, publication, remote-retention, or 1.x retirement feature.
It does not establish a calendar 1.x end date before Qiongli 2 Stable is
published. REL-906 remains separate work.
