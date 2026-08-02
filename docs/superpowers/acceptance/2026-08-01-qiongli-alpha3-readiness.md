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
| Complete Svelte client | 1,921,802 B | 2,048,000 B | production bundle contract | Enforced; A3: 1,926,524 B |
| Client JavaScript | 1,588,144 B | 1,689,600 B | production bundle contract | Enforced; A3: 1,592,611 B |
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
| Academic Graph and project truth | A2 | canonical coverage, stale-state, bounded fixtures | Accepted in A2 checkpoint |
| Native CLI, Plugin, Skills and Host lifecycle | A3 | install/verify/repair/remove/restart receipts | Accepted locally; A7 live receipts open |
| Version-generic release chain | A4 | release-policy and metadata tests | Accepted locally; exact CI owned by A5 |
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

## A2 graph and project truth receipt

The canonical coverage registry enumerates the manifest, all eight supported
artifacts, and explicit semantic links exactly once. Every registered artifact
has one deterministic extractor; `stage_handoff.md` remains structural-only,
and unregistered prose or BibTeX cannot manufacture graph facts.

The native readiness projection now binds source freshness to project revision,
graph source digest, projection identity, and the last successful persisted
Portfolio contribution. App events, CLI snapshot/query/doctor output, and Full
MCP expose the same state. A changed project remains stale across restart until
the derived Portfolio graph is reconciled; missing, migrated, and rebuilt
sources retain their distinct outcomes.

| Gate | Result |
|---|---|
| Project graph suite | 173 tests passed |
| Coverage contract | 10 sources; 8 artifact extractors; structural handoff passed |
| Freshness lifecycle | fresh, stale across restart, missing, migrated and rebuilt passed |
| Continuity | portable, restart, archive/restore, migration and stale revision passed |
| Bounds | deterministic large graph and portfolio fixtures passed |
| App API and UI | App API 32 tests; Svelte 0 errors/warnings; graph UI 10 tests passed |
| CLI and Full MCP | copied CLI graph journey and real stdio Full MCP graph read passed |
| Rust quality | affected crates Clippy passed with `-D warnings` |
| Production client | 1,923,795 B total; 1,590,137 B JS; 236,184 B CSS; hard gate passed |

This receipt qualifies local A2 behavior only. Exact-commit CI and package
qualification remain owned by A5 and A6, and publication remains forbidden.

## A3 CLI, Plugin, Skills, and Host lifecycle receipt

The native CLI now has separate digest-bound preview/apply operations for
installation, supported login-profile PATH configuration, and remove or exact
predecessor restoration. The PATH operation owns one marker block and rejects
symlinked, oversized, non-UTF-8, or changed-after-preview profiles. Removal
rejects unowned, drifted, symlinked, and explicitly shadowed targets.

Codex and Claude Code activation commands no longer exist as frontend
constants. The native App snapshot supplies structured commands, restart
requirements, and the exact personal or user scope. Every refresh and
post-apply verification performs a fresh bounded Host probe; copied files or a
cached ready state cannot satisfy activation and MCP attachment.

| Gate | Result |
|---|---|
| Native CLI core | 19 focused tests; complete 174-test App library passed |
| Fresh login PATH | zsh and bash resolve the exact receipt-owned target without GUI PATH |
| Restore safety | managed update preserves and removal restores only an exact predecessor digest |
| Client coordination | Codex and Claude install, repair, remove/rollback and replay tests passed |
| Current real clients | isolated Codex `0.145.0` and Claude Code `2.1.220` clean-client journeys passed |
| Plugin bundles | deterministic, tamper-evident, runtime-independent and exact-removal tests passed |
| App API and UI | App API 32 tests; Desktop 242 tests; Svelte 0 errors/warnings |
| Rust quality | affected App and UI Clippy passed with `-D warnings` |
| Production client | 1,926,524 B total; 1,592,611 B JS; 236,184 B CSS; 85 files; hard gate passed |
| Installation authority | one bilingual native 2.x page; npm/Python/shell material labeled 1.x |

These isolated current-client runs qualify A3 compatibility behavior but are
not the revision-bound system-profile handoff receipts owned by A7. Exact-head
CI, exact packages, and publication authorization remain open.

## A4 version and release-engineering receipt

The active Community Alpha chain now derives product, candidate, release,
macOS package, Linux AppImage, update metadata, download URL, and release-note
identities from the workspace version or the exact requested tag. The native
version synchronizer updates all nine workspace lock entries, both embedded
Host plugin manifests, the canonical Skill registry, and workflow identity.
The embedded pack is regenerated from those exact inputs.

Alpha 1 supply-chain evidence remains an immutable historical fixture in one
explicit allowlist entry. The formerly Alpha 1-only update metadata generator
is now `native_update_metadata`, and the C5 live-host runbook reads the accepted
package version rather than carrying a milestone literal.

| Gate | Result |
|---|---|
| Native version sync | `2.0.0-alpha.3` applied to Cargo, all workspace lock entries, plugins, Skills, workflow, and embedded pack; second run changed 0 files |
| End-to-end tag contract | Cargo, lock, plugin manifests, content lock, release notes, and version-driven release sources passed; one drifted plugin version failed closed |
| Active Alpha 1 scan | no active workflow, example, script, or template assumption; one immutable historical evidence fixture allowlisted |
| Release policy tests | 92 focused Python tests passed |
| Rust release examples | generic update metadata 2 tests passed; Community Alpha release identity test passed |
| Promotion binding | automatic entry follows successful `Native CI`; manual entry verifies the named run, exact source, completion, success, and current remote `2.x` HEAD |
| Branch protection | live ruleset `18800504` active for `2.x`, strict required `Native CI` boundary and Linux/macOS/Windows Rust checks |
| Publication environment | `community-alpha-publication` exists with required maintainer review and a custom `2.x` branch policy |
| Target version availability | local tag, remote tag, and GitHub release `v2.0.0-alpha.3` absent before candidate preparation |

The bilingual Alpha 3 notes state platform trust limits, client version floors,
data and graph boundaries, conditional automatic-update behavior, known
non-claims, receipt verification, and rollback. This receipt does not replace
the exact-commit CI result, target packages, live-host receipts, or publication
authorization owned by A5 through A8.
