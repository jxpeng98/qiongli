# Qiongli 2 Community Alpha Zero-Cost Distribution Note

Status: R3P-A through R3P-D repository implementation complete on the rolling
branch; the first exact-head promotion, protected authorization, offline
signing, and publication run are pending

Decision date: July 17, 2026

Target release: `v2.0.0-alpha.1`

Target branch: `feat/2x-native-alpha1`

Architecture authority:
`docs/architecture/decisions/0215-community-alpha-distribution-boundary.md`

## Decision

Qiongli may distribute an explicitly labelled Community Alpha for macOS arm64,
Windows x86_64, and Linux x86_64 without purchasing Apple Developer Program or
Windows Authenticode credentials. This is a pre-release testing lane, not a
platform-trusted production release.

The free lane does not weaken Qiongli's own release trust. Every distributed
artifact must still be bound to one exact source revision by the existing
detached Ed25519 release authority, target identity, SHA-256 inventory, SBOM,
provenance, release notes, and an explicitly authorized publication receipt.
Raw CI artifacts remain non-publishing evidence.

The paid production lane remains part of the roadmap. A later release may use
Developer ID and notarization on macOS and trusted Authenticode with an RFC
3161 timestamp on Windows without changing the Community Alpha history.

## Distribution Matrix

| Target | Community Alpha platform trust | First-install artifact | User-visible limitation |
|---|---|---|---|
| macOS arm64 | ad-hoc code signature; not Developer ID signed or notarized | drag-to-Applications DMG | Gatekeeper requires the per-app **Open Anyway** flow |
| Windows x86_64 | no Authenticode publisher signature | portable application ZIP | SmartScreen may warn; Smart App Control or enterprise policy may block execution |
| Linux x86_64 | no central operating-system publisher identity; optional embedded AppImage GPG signature | Type 2 AppImage | executable permission and supported AppImage/window facilities are required |

The macOS update ZIP remains separate from the first-install DMG. The Windows
ZIP contains the complete portable directory; an untrusted self-signed root
certificate is never installed. The Linux AppImage may carry an embedded GPG
signature, but the bundled Qiongli verifier remains authoritative for update
selection and installation.

No target requires the user to install Rust, Python, Node.js, Cargo, npm, pip,
GPG, Sigstore, or a package manager. Tools used only by the release pipeline
are build-time dependencies and are not shipped as runtime prerequisites.

## Mandatory Release Set

One Community Alpha release set contains, at minimum:

- the macOS arm64 DMG and `.app.zip` produced from the same ad-hoc-signed App;
- the Windows x86_64 portable ZIP with all required executables together;
- the Linux x86_64 AppImage and companion CLI artifact when the AppImage is not
  the CLI forwarding surface;
- a sorted SHA-256 inventory covering every public asset;
- detached Qiongli release/update signatures and the public authority record;
- target-specific metadata that binds version, channel, capability profile,
  operating system, architecture, installer kind, size, and digest;
- a CycloneDX SBOM, SLSA provenance, and truthful release notes; and
- one final Community Alpha publication receipt bound to the exact release set.

The publication receipt must state the distribution class and platform trust
for every target. It must not claim Developer ID, notarization, Authenticode,
SmartScreen reputation, enterprise-policy compatibility, or Linux distribution
endorsement when those properties were not observed.

## Security Boundary

- Platform trust and Qiongli release trust are independent. Waiving paid
  platform trust never waives release-envelope or update-metadata verification.
- Community Alpha assets never enter the Stable stream. A preview update stream
  may expose them only after its signed schema carries the Community Alpha
  distribution class and the client displays the associated warning.
- Release and launch-grant private keys stay outside the repository and shipped
  binaries. CI receives them only through a protected GitHub Environment or an
  external signer; ordinary repository `.env` files are not key storage.
- The updater rejects an unknown key, target, digest, size, channel,
  distribution class, expired generation, downgrade, or replay before staging.
- Users may use the operating system's bounded per-app override on macOS or the
  normal SmartScreen continuation when Windows offers it. Documentation must
  not instruct users to disable Gatekeeper, disable Smart App Control, weaken
  enterprise policy, or import a self-signed root certificate.
- A Windows device that blocks the unsigned binary is unsupported by the free
  Community Alpha. The release notes must state this before download.

## Acceptance Gate

Publication requires all of the following:

1. one clean exact source revision and successful required Native CI;
2. final artifact regeneration outside the source checkout;
3. target-native packaged startup evidence for macOS, Windows, and Linux;
4. macOS DMG mount/layout, copied-App startup, update ZIP, and rollback checks;
5. Windows complete-directory extraction and launcher/CLI startup checks without
   claiming that Smart App Control permits the unsigned build;
6. Linux AppImage integrity, executable startup, and supported runtime-facility
   checks;
7. verification of the final SHA-256 inventory, authority, detached signatures,
   SBOM, provenance, metadata, and release notes;
8. installation instructions and warnings reviewed in English and Chinese; and
9. explicit maintainer authorization for this exact Community Alpha release
   set before tag creation, asset upload, or update-stream mutation.

Passing Community Alpha acceptance is not production-signing acceptance.
Windows and Linux interactive evidence may be gathered on controlled machines
or VMs, but it cannot be inferred only from cross-compilation.

## Execution Slices

To keep the release tail short, implement this decision in four bounded batches:

- `R3P-A`: add the `community-alpha` distribution class, schema rules,
  fail-closed authorization input, and release-note labels;
- `R3P-B`: promote exact-head macOS ad-hoc, Windows unsigned portable, and Linux
  AppImage outputs into one candidate without reusing raw CI artifacts;
- `R3P-C`: bind all target assets into the existing Ed25519 metadata,
  checksums, SBOM, provenance, and Community Alpha ledger;
- `R3P-D`: run target-native acceptance, publish one GitHub Pre-release after
  explicit authorization, and record the immutable release receipt.

R3P-A is implemented in `qiongli-platform::distribution`. Its bounded,
canonical, unknown-field-rejecting records close the Community Alpha class,
the macOS arm64 / Windows x86_64 / Linux x86_64 target matrix, platform-trust
claims, warning codes, raw-CI prohibition, and exact release-set authorization.
The verified release-set capability is produced only when the policy set,
source revision, release-set digest, protected GitHub Environment, workflow
run, actor, and authorization time window all match.

The authorization JSON is a protected-workflow input, not a freestanding
cryptographic identity claim. The final workflow must construct and compare it
against trusted GitHub runtime context; copying the JSON to another run or
release set does not authorize publication. R3P-C still has to bind the actual
asset inventory and Qiongli signatures, and R3P-D still has to consume the
verified capability before any external mutation.

Local R3P-A verification on July 17, 2026 passed 11 focused distribution tests,
all 86 then-current `qiongli-platform` tests, and `clippy -D warnings`. These
are repository implementation results, not target-native acceptance or
public-release proof.

R3P-B is implemented as a separate read-only exact-head workflow and shared
Rust promotion contract. It runs when the implementation reaches `2.x` and
also retains a future manual entry point. It accepts only the explicit current
remote `2.x` HEAD,
rebuilds on macOS arm64, Windows x86_64, and Linux x86_64 in one workflow run,
records public asset and package/acceptance evidence digests, and aggregates
only that run's three promotion inputs. The macOS signing boundary has a
distinct `--community-alpha` mode; it is not the `--test-only-ad-hoc` artifact.

Local R3P-B verification passed six focused promotion tests, all 92 current
`qiongli-platform` tests, affected Clippy gates, YAML parsing, shell syntax,
and an isolated macOS promotion fixture. A Linux promotion request on the macOS
host failed before output creation as required. The workflow has not yet run on
the merged `2.x` head, so no real three-platform candidate is claimed.

The existing production-signed macOS workflow and future Windows Authenticode
work remain intact. They are follow-up hardening rather than blockers for the
free Community Alpha.

R3P-C uses the checked-in public Alpha authority and the Rust
`native_community_alpha_release` tool. The tool rehashes the candidate and
target receipts, emits a sorted inventory, CycloneDX 1.6 SBOM, SLSA provenance,
bilingual release notes, and an Ed25519-signed integrity record. Only public
keys are stored in the repository or embedded in binaries.

R3P-D uses the required-reviewer `community-alpha-publication` Environment on
the `2.x` branch. Its job has read-only repository permission and emits only a
short-lived authorization bound to the exact source, workflow run, actor, and
release-set digest. Final signing and GitHub pre-release creation occur on the
maintainer machine, so GitHub never receives the private release key.

## Nonclaims

This decision does not claim that the three final candidates have already been
promoted or accepted, that Windows Smart App Control will allow the unsigned
binary, that macOS Gatekeeper will open the app without user action, that all
Linux distributions support the AppImage, or that a tag, GitHub Release, or
public update endpoint has been created.
