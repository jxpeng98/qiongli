# Qiongli 2.0.0-alpha.3 Acceptance Ledger

Status: in progress — publication is not authorized

Date opened: August 1, 2026

Target branch: `2.x`

Planning baseline: `b19661841eee9b8186f3288be3017d7f7704686a`

Target release: `v2.0.0-alpha.3`

## Frozen release claim

Alpha 3 may claim a deterministic Academic Graph v1 over supported canonical
research artifacts, bounded project continuity, App/CLI/MCP read parity, and
managed Codex and Claude Code integrations only where the target-native and live
Host receipts below are accepted.

It does not claim arbitrary-directory interpretation, prose or BibTeX fact
inference, a Dataset graph node, Full MCP mutation, production OS publisher
trust, or support for an unaccepted target or architecture.

## Target claims

| Target | Candidate package | Permitted claim before acceptance | Required owner receipt | State |
|---|---|---|---|---|
| macOS arm64 | Community Alpha DMG and signed update ZIP | none | A6 package, A7 CLI/Hosts/update | Open |
| Windows x86_64 | unsigned portable directory | none | A6 native startup and package | Open |
| Linux x86_64 | AppImage and portable CLI archive | none | A6 native startup and package | Open |

Cross-platform Host integration is not inferred from macOS evidence. A target
receives install, PATH, repair, remove, restart, Codex, or Claude Code claims
only after that journey has a target-native receipt.

## Update contract

The preferred contract is a signed Alpha 1 to Alpha 3 update with atomic
rollback. A7 owns its accepted upgrade and rollback receipt. If that receipt is
not accepted, release notes and update metadata must disable the automatic
update claim and document manual replacement; the release cannot silently fall
back to an unverified updater.

## Size budget

All values are uncompressed bytes unless stated otherwise. The automated client
limits leave less than seven percent headroom above the August 1 baseline.

| Artifact | Measured planning baseline | Alpha 3 maximum | Enforcement | State |
|---|---:|---:|---|---|
| Complete Svelte client | 1,921,802 B | 2,048,000 B | production bundle contract | Enforced; A1: 1,921,983 B |
| Client JavaScript | 1,588,144 B | 1,689,600 B | production bundle contract | Enforced; A1: 1,588,325 B |
| Client CSS | 236,184 B | 250,880 B | production bundle contract | Enforced |
| Largest JavaScript asset | 445,863 B | 471,040 B | production bundle contract | Enforced |
| Shared application shell | post-migration contract | 409,600 B | production bundle contract | Enforced |
| Client file count | 84 | 90 | production bundle contract | Enforced |
| Native release executable | 28,590,464 B local snapshot | 29,360,128 B (28 MiB) | A5 exact-head measurement | Provisional |
| macOS application | 30,520 KiB Alpha 2 receipt | 32 MiB | A6 exact-package measurement | Provisional |

The native and App limits become final only after a clean exact-HEAD Alpha 3
candidate is built. A larger exact candidate is a No-Go until the increase is
explained, reduced, or a replacement budget is explicitly reviewed. New runtime
dependencies receive no automatic size allowance.

## Evidence ownership

| Claim or gate | Evidence owner | Required evidence | State |
|---|---|---|---|
| Frozen version, claims, targets, update and size contract | A0 | this ledger and completion plan | Accepted in A0/A1 checkpoint |
| Fail-closed Host and source quality | A1 | Rust/frontend tests, format, Clippy, bundle contract | Accepted in A0/A1 checkpoint |
| Academic Graph and project truth | A2 | canonical coverage, stale-state, bounded fixtures | Open |
| Native CLI, Plugin, Skills and Host lifecycle | A3 | install/verify/repair/remove/restart receipts | Open |
| Version-generic release chain | A4 | release-policy and metadata tests | Open |
| Exact source and required CI | A5 | clean commit and same-commit CI result | Open |
| Exact packages and native targets | A6 | R5D/R5E/R5G and target-native receipts | Open |
| Live Hosts and upgrade/rollback | A7 | revision-bound Codex, Claude Code and update receipts | Open |
| Supply-chain authorization and publication | A8 | independent trust verification and immutable release | Forbidden |
| Public-download smoke and observation | A9 | public artifact verification and observation ledger | Open |

## Publication authority

Only A8 may authorize and publish `v2.0.0-alpha.3`, and only after A0 through A7
are accepted for the exact same commit and packages. Local builds, raw CI
artifacts, isolated Host runs, or draft release assets are evidence inputs, not
publication authorization.

## A1 local gate receipt

The following gates passed before the A0/A1 checkpoint was committed. This
receipt is superseded by the exact-commit A5 evidence and grants no publication
authority.

| Gate | Result |
|---|---|
| Rust format | `cargo fmt --all -- --check` passed |
| Rust check | locked workspace, all targets and all features passed |
| Rust Clippy | workspace, all targets and all features with `-D warnings` passed |
| Rust tests | locked workspace, all targets and all features passed |
| App API | 32 tests passed |
| Desktop | 241 tests passed; Svelte check reported 0 errors and 0 warnings |
| Production client | build and bundle-size contract passed |
| Host negative/positive matrix | unavailable, failed, timed out, action-required and observed transitions passed |

The real Codex and Claude Code clean-client tests remain intentionally ignored
inside the offline workspace suite. They are not counted as live Host evidence;
A7 still requires independent current-client and revision-bound handoff receipts.
