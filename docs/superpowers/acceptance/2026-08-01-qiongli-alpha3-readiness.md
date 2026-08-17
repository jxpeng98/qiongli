# Qiongli 2.0.0-alpha.3 Acceptance Ledger

Status: exact first-usable evidence retained; release candidacy reopened by an
essential Full MCP route defect, release qualification remains open, and
publication was explicitly rejected

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
inference, a Dataset graph node, unrestricted Full MCP mutation, production OS
publisher trust, or support for an unaccepted target or architecture. The only
public Full MCP project write is the previewed, digest-bound, explicitly
approved `qiongli_project_capture_apply` operation.

## Target claims

| Target | Candidate package | Permitted claim before acceptance | Required owner receipt | State |
|---|---|---|---|---|
| macOS arm64 | Community Alpha DMG and signed update ZIP | none | A6 package, A7 CLI/Hosts/update | Exact package built; release claims open |
| Windows x86_64 | unsigned portable directory | none | A6 native startup and package | Exact package built; release claims open |
| Linux x86_64 | AppImage and portable CLI archive | none | A6 native startup and package | Exact package built; release claims open |

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
| Native release executable | 28,590,464 B local snapshot | 29,360,128 B (28 MiB) | A5 exact-head measurement | No-Go for release: 29,428,960 B; 68,832 B over |
| macOS application | 30,520 KiB Alpha 2 receipt | 32 MiB | A6 exact-package measurement | Accepted: 31,970,161 manifested B |

The exact candidate makes both measurements final. The App remains within its
budget, but the native CLI is a release No-Go until the increase is explained,
reduced, or a replacement budget is explicitly reviewed. New runtime
dependencies receive no automatic size allowance.

## Evidence ownership

| Claim or gate | Evidence owner | Required evidence | State |
|---|---|---|---|
| Frozen version, claims, targets, update and size contract | A0 | this ledger and completion plan | Accepted in A0/A1 checkpoint |
| Fail-closed Host and source quality | A1 | Rust/frontend tests, format, Clippy, bundle contract | Accepted in A0/A1 checkpoint |
| Academic Graph and project truth | A2 | canonical coverage, stale-state, bounded fixtures | Accepted in A2 checkpoint |
| Native CLI, Plugin, Skills and Host lifecycle | A3 | install/verify/repair/remove/restart receipts | Accepted locally; A7 live receipts open |
| Version-generic release chain | A4 | release-policy and metadata tests | Accepted locally; exact CI owned by A5 |
| Exact source and required CI | A5 | clean commit and same-commit CI result | Accepted for `cced6082`; Native CI run `31438158969` |
| Exact packages and native targets | A6 | R5D/R5E/R5G and target-native receipts | Automated package and fresh-target inputs accepted for `cced6082` in run `31439930097`; CLI size and manual claim receipts remain open |
| Live Hosts and upgrade/rollback | A7 | revision-bound Codex, Claude Code and update receipts | Partial local Codex evidence only; Claude and update/rollback open |
| Supply-chain authorization and publication | A8 | independent trust verification and immutable release | Rejected for `cced6082`; forbidden until A6-A7 are accepted |
| Public-download smoke and observation | A9 | public artifact verification and observation ledger | Open |

## Publication authority

Only A8 may authorize and publish `v2.0.0-alpha.3`, and only after A0 through A7
are accepted for the exact same commit and packages. Local builds, raw CI
artifacts, isolated Host runs, or draft release assets are evidence inputs, not
publication authorization.

## Current candidate audit

The frozen product candidate is
`cced60826ac4d7dad596669103a7e15b61868e81`, which contains the merged App
first-use and native Zotero verticals. Native CI run `31438158969` passed that
exact source, including the packaged macOS product-control vertical and native
Linux, macOS, and Windows jobs.

Community Alpha promotion run `31439930097` verified the same source, rebuilt
all three targets, and completed non-publishing aggregation. On August 11,
2026, the maintainer rejected its protected publication approval. The run's
overall `failure` conclusion is therefore the intended authorization result;
all five pre-publication jobs succeeded and no tag or GitHub Release was
created.

On August 12, exact-package R5D Zotero automation passed against this source
with Companion `0.3.0` endpoint `2`. An authenticated system-profile Codex
`0.146.0` transaction also completed the revision-bound candidate, independent
review and checkpoint path; its redacted receipt SHA-256 is
`3dd8c2c6b2aa07da6de7b2855ea4e8993c4bf7653525500a5fd9db2ffc4b2e0a`.
The run exposed that Full MCP `qiongli_orchestrator_route` incorrectly returned
the Marketplace Lite upgrade result, so the successful transaction required an
explicit route bypass. Claude Code `2.1.222` passed isolated install/MCP checks,
but live model execution was not authorized. These are partial qualification
inputs, not an accepted A7 transition.

## Current exact internal candidate receipt

| Identity | Accepted value |
|---|---|
| Product source | `cced60826ac4d7dad596669103a7e15b61868e81` |
| Exact-head CI | Native CI run `31438158969`: success |
| Promotion | run `31439930097`: exact-head verification, three fresh targets, and aggregation succeeded; authorization rejected |
| Review receipt | [PR #122 exact-candidate evidence](https://github.com/jxpeng98/qiongli/pull/122#issuecomment-5247046894) |
| Candidate set SHA-256 | `47ef8d95449472bb6f01ed91d90364f729270bec29dd8961a481d37f757cc182` |
| Candidate state | `fresh-three-target-nonpublishing-candidate`; `publication_allowed=false` |
| Release reuse | Reopened by the Full/Lite route P0; any fixed product source requires new A5/A6 exact-head and package evidence |
| Packaged App | 31,970,161 manifested B; within the 32 MiB budget |
| Packaged native CLI | 29,428,960 B; 68,832 B over the 28 MiB release budget |
| Zotero Companion | version `0.3.0`; endpoint `2`; XPI SHA-256 `8c404a47b3e05d90ba9d065343c3fb27e9e50cd087cdfa91c118c88840ac4652` |

| Candidate asset | Bytes | SHA-256 |
|---|---:|---|
| `Qiongli-2.0.0-alpha.3-macOS-arm64.zip` | 10,273,795 | `d756e481d6d1738139720d5aa5bf12120d29328c0113302f77834904cb3af881` |
| `Qiongli-2.0.0-alpha.3-macOS-arm64.dmg` | 10,388,330 | `535386505702c529f3b4a52521d260bafc9c0dfc4abac9374613efbc295ba571` |
| `Qiongli-2.0.0-alpha.3-Windows-x64.zip` | 26,594,124 | `a84a443fd7361838970b8f16581f7fa0a0edf704a6d21849de38437948a572a7` |
| `Qiongli-2.0.0-alpha.3-Linux-x64.AppImage` | 11,495,928 | `9c2a454ad9a9a7f35996a000d042704e62431a603fffad290118beab3189b387` |
| `Qiongli-2.0.0-alpha.3-Linux-x64.zip` | 38,101,145 | `552e3fcd3fff1422dc70a79047d29437053344937b02ada8b94bd0f430ba8fe0` |

This receipt retains the automated evidence that originally established
`Internally usable` for the exact product source. The newly reproduced Full/Lite
route P0 reopens that transition for release use. The native CLI size budget,
manual/target receipts and A7-A9 remain open. The current product fix is not a
replacement candidate; it requires new A5/A6 evidence if Alpha 3 qualification
resumes.

## Delivery evidence transitions

The roadmap owns the development state machine; this ledger records only
accepted state transitions. It does not copy local command output or create a
new receipt when an existing source, package, Zotero, target, Host, or release
receipt already owns the claim.

| Transition | Ledger records | Reopen condition |
|---|---|---|
| Focused green -> Exact-head green | PR head SHA, required run, conclusion | any new product/package input commit |
| Exact-head green -> Package accepted | exact clean source, packaged-product receipt, Zotero receipt, product checks, `publication_allowed=false` | integrated spine or receipt input changes |
| Package accepted -> Internally usable | accepted package identity and known automated P0 list | expired/replaced package or a new essential-path P0 |
| Internally usable -> Release-qualified | only A6-A7 target, real-Host, manual claim, update, and rollback evidence | source, version, target, client, digest, metadata, or journey changes |
| Release-qualified -> Authorized | A8 exact-set authorization identity | any candidate member changes |
| Authorized -> Observed | A9 public URL, digest, startup/update result, and rollback decision | published asset or channel changes |

The `cced6082` candidate reached automated `Internally usable` readiness before
the Full/Lite route P0 was reproduced. That transition is now reopened for
release use, while its exact evidence remains valid historical input. A6-A9
public-claim work remains outside the P0 development loop unless release
qualification resumes or it reproduces an essential-path failure.

## Historical internal first-usable automated receipt

The internal-use gate is accepted for exact source
`ba33301412de1c6919bf35d69a1312825f6c069d`:

| Gate | Result |
|---|---|
| Exact-head CI | Native CI run `31283065849`: 10/10 passed |
| Three-target rebuild | Promotion run `31284047249`: macOS, Windows, Linux, and aggregate passed |
| Packaged macOS product | 26/26 Plugin, Skills, CLI, MCP, restart, migration, continuity, and Zotero-binding checks passed |
| Zotero automated lifecycle | 13/13 identity, state, search, approved-write, replay, duplicate, shutdown, removal, and fallback checks passed |
| Companion identity | XPI SHA-256 `77fff3a2841571a7f15b519b753f6b20eaf4c93492fea59c3b01cdfd8ca0c17c`; endpoint `2` |
| Publication | `false`; protected authorization remains waiting |

This receipt permitted blocker-only internal dogfooding for its exact source.
It is retained as historical evidence and does not qualify the current task's
changed source or satisfy the manual Zotero, visual, real system-profile Host,
update/rollback, or public release gates.

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
A7 still requires independent exact-candidate current-client and revision-bound
handoff receipts. The August 12 Codex receipt is partial because the route P0
required a bypass; Claude live evidence remains absent.

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
| Current qualification follow-up | isolated Codex `0.146.0` and Claude Code `2.1.222` passed; authenticated Codex handoff completed with a route bypass; Claude live execution not authorized |
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
