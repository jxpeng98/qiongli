# Qiongli PLT-322 Migrated Graph v1 Acceptance

Status: accepted at Slice tier

Date: August 24, 2026

Target branch: `2.x`

Pull request: `#143`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `e01ad446e3b64eb2b5a3bc773d41f1874f1d2fe9` |
| Evaluation Truth | run `32681304668`: success |
| Native CI | run `32681304659`: success |
| Acceptance receipt | `994b6913f0075d804a1f6f96adfa0f7682c6b1e37dd682bbace3ad83ffdc05d5` |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, and promotion jobs were skipped by the
ordinary Slice boundary.

## Accepted representative project

The repository-owned source is `RESEARCH/asset-pricing-capm-ff3`. Its pinned
analysis compares CAPM and FF3 across 25 U.S. size/book-to-market portfolios,
two weighting schemes, and 756 monthly observations from July 1963 through
June 2026. The deterministic grid contains 100 fitted models. In the primary
value-weighted comparison, FF3 reduces mean absolute monthly alpha by 55.98%;
this is a descriptive in-sample benchmark result, not a causal, universal
model-validity, or investment claim.

The exact-source runner archived the clean product commit, added disposable
private-state markers only inside an isolated copy, and used the supported
migration preview/apply path. Migration retained the source, copied 56 files,
and excluded both private markers.

## Graph and App evidence

| Gate | Result |
|---|---|
| Graph projection | 60 nodes, 52 semantic nodes, 110 edges, 0 diagnostics |
| Readiness | `visualizable`; `academic-graph-visualizable` |
| Canonical relations | 9 types, including `supports`, `extends`, `complements`, and `addresses-gap` |
| Determinism | repeated rebuild and fresh-process reopen preserved projection, node, and edge identities |
| Query | stable canonical ID and relation filters returned bounded, internally consistent results |
| App artifact read | one accepted node and edge resolved at the exact revision and projection |
| Desktop | readiness, layout, Cytoscape elements, search/focus, and source inspection passed |
| Negative controls | native and Desktop empty/sparse readiness controls passed |

The receipt contains only repository-relative identity, digests, bounded
counts, type/relation names, readiness state, and required check IDs. It
contains no research rows, prose labels, citations, absolute paths, private
runtime state, credentials, or Host conversations.

## Focused verification

- Research standard: 6,204 passed, 0 failed, 0 warnings.
- Native `qiongli-project`: 176 passed.
- App API: 32 passed.
- Desktop: 249 passed; the environment-gated representative acceptance is
  invoked by the coordinator; Svelte check and production build passed.
- Capability contract and generated roadmap checks passed.
- The exact-source coordinator completed all 16 required checks and emitted
  the receipt only after success.

## Nonclaims

This Slice does not accept Graph v2, a Typed Research Kernel, systematic-review
coverage, manuscript completeness, candidate or packaged artifacts, live user
projects, signing, notarization, promotion, publication, 1.19 retirement, or
release authorization. `PILOT-903` and the release program remain separate
future work.
