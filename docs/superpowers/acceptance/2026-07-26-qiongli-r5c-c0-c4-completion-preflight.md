# Qiongli R5C C0-C4 Completion Preflight

Status: passed for C0-C4 current-head evidence; not an R5C completion record

Date: July 26, 2026

Publication allowed: false

## Purpose and boundary

This preflight assembles the C0-C4 portion of the R5C completion ledger while
C5 still waits for independently authenticated Codex and Claude Code live-host
receipts. It does not mark C5 or R5C complete and does not replace the C5
package-bound acceptance record.

The review distinguishes three identities:

| Identity | Value |
|---|---|
| Preflight source HEAD | `24eb9359aab82b78db0560e2fd4702278c1321f5` |
| Accepted product source | `1673e1f6c1eb933c8033b6981df883b67d19c8d1` |
| Accepted packaged binary SHA-256 | `1d103190d712e61cb0019f66b038b7ba784d832a9e36f745c797a09e127dea05` |
| Product acceptance receipt SHA-256 | `b163f413b7032a8ec1e1a5ac68a68b0cef15ad1d861050851f26a0525ae2998e` |
| Prepared host fixture receipt SHA-256 | `1cc7e8a502f717d0a9e525a5a0068718ff2cee05e4787c7d9490c64a41317e45` |

All post-product changes in this preflight head are outside the accepted
C0-C4 App, App API, Desktop, native Desktop, and project-domain source paths.
They prepare and tighten C5 acceptance contracts, schemas, scripts, and
records. The accepted App is not rebuilt or silently replaced by this review.

## Commit ancestry

All 26 implementation and product commits named by the C0-C4 plans and the
accepted C5 product receipt are ancestors of the preflight source HEAD.

| Stage | Accepted implementation commits |
|---|---|
| C0 | `5bd8606c` |
| C1 | `cf557e85`, `a7c37eaa`, `1f1145e3`, `27bcf5c9` |
| C2 | `aa0adcb8`, `50df353d`, `b1f3ca89`, `5844c416`, `8b156970` |
| C3 | `8e9913cf`, `411bd855`, `7caa9d28`, `89f77523`, `863bfbc9` |
| C4 | `51c53b1a`, `d2acf67a`, `8c393cc8`, `1d9e5a73`, `fd00d61c`, `d3bf7cd5`, `8cd3fa2b`, `627740d9`, `fd1e166a`, `dedfc7b4` |
| Accepted product | `1673e1f6` |

The only project-domain changes after the C3 closing commit are the accepted
C4 native continuity read and acknowledgement-preview work in `d2acf67a` and
`8c393cc8`. No C4 App API, Desktop, or native Desktop source changed after
`dedfc7b4`, and none of the C0-C4 owned product paths changed after the
accepted product source.

## Current-head focused gates

The following commands ran from the clean preflight source HEAD:

| Gate | Result |
|---|---|
| `pnpm --dir packages/qiongli-app-api check` | passed |
| `pnpm --dir packages/qiongli-app-api test` | 19 passed |
| `pnpm --dir packages/qiongli-desktop check` | 0 errors, 0 warnings |
| `pnpm --dir packages/qiongli-desktop test` | 27 files, 116 tests passed |
| `pnpm --dir packages/qiongli-desktop build` | production build passed |
| `cargo test --locked --manifest-path packages/qiongli-native/Cargo.toml --package qiongli-project` | 156 passed |
| `cargo test --locked --manifest-path packages/qiongli-native/Cargo.toml --package qiongli --lib` | 122 passed |
| Rust formatting check | passed |
| Workspace all-target/all-feature Clippy with `-D warnings` | passed |
| Workspace all-target/all-feature `cargo check` | passed |
| `git diff --check` | passed |

No broad cybersecurity scan or unrelated legacy runtime suite was run.

## Requirement ledger

### C0 — Native package and current-product baseline

Evidence owner:

- the schema-2 packaged-product receipt bound to source `1673e1f6`;
- the exact copied App binary digest above; and
- the accepted manual isolated-home install/restart observations in the C5
  acceptance record.

The receipt reports all C0 checks true, including embedded authority, product
control, complete inventory, current Codex and Claude Code install/verify/
remove, registration repair, packaged restart, empty-`PATH` startup, Skills
materialize/verify/refresh, Lite MCP self-test, provider Keychain lifecycle,
and isolated 1.x-to-2.x migration.

Preflight result: **proved for the accepted macOS engineering package**.
Developer ID, notarization, publication, and other platforms remain explicit
nonclaims.

### C1 — Durable capture delivery

Evidence owner:

- strict delivery contracts and private atomic ledger in the C1 commits;
- current `capture_delivery`, `capture_delivery_storage`, and
  `capture_delivery_service` tests inside the 156-test project suite; and
- packaged continuity counts for four delivery records, one retry, one
  acknowledgement replay, and one duplicate suppression.

The current tests cover immutable identity, queue/delivery/acknowledgement
transitions, stale CAS, replay, restart recovery, wrong-project and
wrong-revision conflict, cancellation, lock contention, corruption, bounded
storage, and path-redacted CLI projection.

Preflight result: **proved**.

### C2 — Assignment and conflict reconciliation

Evidence owner:

- strict assignment, comparison, disposition, plan, selection, and receipt
  contracts in the C2 commits;
- current assignment, resolution, storage, repository-inbox, interruption,
  replay, and exact-lineage tests in the project suite; and
- packaged continuity counts for one assignment, one resolution, and five
  explicit resolution items.

The current tests cover direct, rebound, duplicate, rejected, stale, archived,
divergent, and unbound paths; every frozen academic disposition; digest-bound
approval; atomic interruption recovery; acknowledgement after academic
commit; and preservation of canonical project authority.

Preflight result: **proved**.

### C3 — Incremental Portfolio continuity

Evidence owner:

- the five accepted C3 implementation commits;
- current catalog storage, incremental reconcile, bounded query, timeline,
  cancellation, corruption, deletion, and rebuild tests; and
- packaged continuity observations for three matching projects, eight lineage
  matches, 33 timeline events, one archive/restore, one derived deletion, and
  two full rebuilds.

The current tests prove incremental/full byte equivalence for the same
revision, stale-evidence fail-closed behavior, deterministic bounded queries,
restart recovery, cancellation without partial publication, and deletion of
derived state without canonical project changes.

Preflight result: **proved**.

### C4 — App API and Desktop continuity

Evidence owner:

- accepted C4.1-C4.5 commits and their manual bilingual/browser matrix;
- unchanged C4-owned source paths since `dedfc7b4`;
- current 19-test strict App API gate and 116-test Desktop interaction gate;
- current Desktop production build; and
- current native Desktop and project tests.

The evidence covers strict App API v5 decoding, current native previews and
bindings for every mutation, causal-state parity, bounded Portfolio and
Timeline operations, cancellation, restart invalidation, truthful unknown/
stale/conflicted states, independent Chinese and English catalogs, keyboard
and dialog focus, reduced motion, narrow layout, and private-data exclusion.

Preflight result: **proved**.

## Remaining completion dependency

C5 is still incomplete because the existing authenticated system Hosts have
not yet been verified against matching current 2.x registrations and no live
Codex or Claude Code receipt exists. Isolated Plugin, Skill, registration,
restart, and MCP health checks are necessary but are not substituted for a
real revision-bound handoff.

After both receipts pass, the final review may reuse this preflight only after
checking that:

1. the C0-C4 owned paths and accepted product identity have not changed;
2. both `system-existing` Host receipts match the exact fixture, product,
   binary, source, isolated Plugin digest, and current system registration;
3. all four rejection probes preserve checkpoint state in each Host;
4. App and copied CLI parity still holds after both acceptance runs; and
5. C5 and the parent R5C plan are updated without expanding any distribution
   claim.
