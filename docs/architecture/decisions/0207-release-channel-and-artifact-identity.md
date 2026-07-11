# ADR 0207: Release Channels and Native Artifact Identity

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201G`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 native release, artifact, and update identity

## Decision drivers

- Alpha, beta, and stable have different compatibility promises and must not be
  collapsed into a mutable `latest` stream.
- A user, host, updater, and release auditor must be able to identify exactly
  which product, version, channel, profile, platform, architecture, and package
  format an artifact serves before downloading or executing it.
- Native payloads require target-native build and startup evidence; a generic
  filename must never conceal the maintainer's current-host executable.
- Updates must authenticate metadata and payloads, resist downgrade and replay,
  and preserve a last-known-good installation.
- The frozen Python/Node-led 1.x release line must remain isolated from the
  Rust-native 2.x release and update system.
- Every advertised native artifact needs checksums, an SBOM, provenance, and
  the signatures appropriate to its target and distribution mechanism.

## Context

Qiongli 1.x publishes several language and plugin packages with related but not
identical version representations. Qiongli 2.x is a native product distributed
as target-specific desktop/CLI installers, local plugin or MCP artifacts, and
content-only packages. Reusing a single release asset name or a single
prerelease switch would make it possible for an alpha to reach stable users,
for a host to select an incompatible executable, or for an updater to replace a
working target build with an artifact assembled on a different machine.

Native self-update also changes the failure model. An in-place replacement can
leave the product unable to start, while unsigned or weakly identified metadata
can redirect a client to an older, wrong-profile, or wrong-target payload. The
release model therefore needs an explicit identity and signed channel contract
before the first `v2.0.0-alpha.1` artifact.

## Decision

### SemVer and independent channels

Qiongli 2 uses SemVer as the product version and defines exactly three release
channels:

- `alpha` accepts versions of the form `MAJOR.MINOR.PATCH-alpha.N`;
- `beta` accepts versions of the form `MAJOR.MINOR.PATCH-beta.N`;
- `stable` accepts versions with no prerelease identifier.

The channel is explicit metadata and must agree with the SemVer prerelease
identifier. A mismatched pair is invalid. Alpha, beta, and stable have separate
signed metadata, history, retention, and client selection. There is no implicit
fallback from one channel to another and no global mutable `latest` record.
Changing a client's selected channel is an explicit, previewed user or
administrator action.

Promotion does not relabel an existing artifact. It creates a new SemVer
release and a new artifact identity in the destination channel after that
channel's gates pass. Identical payload bytes may be reused only when their
digest and all channel-bound metadata remain truthful; signatures and release
receipts are still issued for the new identity.

### Canonical artifact identity

Every distributed artifact has a canonical identity tuple:

```text
(product, version, channel, profile, os, arch, installer_kind)
```

The fields mean:

- `product`: the stable product namespace, initially `qiongli`;
- `version`: the complete SemVer string, including prerelease sequence;
- `channel`: exactly `alpha`, `beta`, or `stable`;
- `profile`: the declared capability payload such as `skill-only`, `lite`, or
  `full`; for an executable package it is the signed launch-grant ceiling,
  while `remote` is not a local binary profile;
- `os`: the normalized target operating system;
- `arch`: the normalized target architecture;
- `installer_kind`: a closed, versioned vocabulary describing the delivery
  contract, such as a native installer, portable archive, plugin bundle, MCPB,
  or content archive.

The tuple appears in the filename, release manifest, update metadata, checksum
record, SBOM, provenance statement, signature subject, installation receipt,
diagnostics, and rollback record. A reusable canonical binary embeds product,
version, channel, target, and its compiled capability ceiling. Its package
manifest also supplies a signed launch grant binding the binary and
resource-pack digests, integration scope, and package profile. This is the
authoritative profile identity when identical binary bytes are reused by Lite
and Full packages. No consumer derives identity only from a filename or an
untrusted command argument.

`os=any` and `arch=any` are allowed only for a verified content-only artifact
that contains no executable, native library, install hook, or target-specific
launcher. A universal/fat native binary uses a distinct, truthful architecture
value and must be built and started on every architecture it advertises.

### Target-specific signed release sets

Each advertised OS/architecture/profile/installer-kind tuple is built as a
separate release artifact and receives target-native startup evidence.
Cross-compilation by itself is not acceptance evidence. Each release set
contains or links to:

- a cryptographic checksum manifest;
- the platform signature required by the target, including notarization or
  equivalent verification where applicable;
- an SBOM covering the shipped payload;
- build provenance binding source revision, build inputs, builder identity, and
  output digest; and
- a release receipt recording target-native install/start verification.

Two release artifacts may reuse a binary component digest only when each has a
separate signed manifest and launch grant. The runtime rejects a requested
profile above the grant even when the embedded binary has a higher compiled
ceiling.

The public release is incomplete, and the artifact must not be advertised, if
any required item is absent or fails verification.

### Signed update metadata and A/B replacement

The updater consumes canonical, signed channel metadata. Each metadata entry
binds the complete artifact identity, digest, size, minimum compatible updater,
publication/expiry information, and rollback policy. Metadata signatures are
verified before artifact selection; checksum and artifact/platform signatures
are verified before staging. Key IDs, threshold/rotation policy, and expiry are
part of the update trust contract, while private signing keys remain outside
the repository and build payload.

Self-update uses A/B slots or an equivalent staged replacement:

1. download and verify into the inactive slot;
2. run bounded integrity and startup checks against that slot;
3. atomically switch the active pointer;
4. retain the previous verified slot as last-known-good until the new version
   passes its health window; and
5. automatically switch back when startup or health verification fails.

An update never executes a partially downloaded artifact and never destroys the
last-known-good slot before a verified replacement exists.

### 1.x release-line isolation

The frozen 1.x PyPI, npm, plugin, Rust Lite, MCPB, and GitHub release identities
remain in their existing compatibility channel and metadata. They are not
inserted into a 2.x native alpha, beta, or stable feed. Likewise, native 2.x
artifacts are not published under a 1.x package identity or used as a silent
replacement for a 1.x dependency.

The 2.x updater does not automatically install, downgrade to, or take ownership
of a 1.x package, and a 1.x updater does not discover a 2.x native release.
Migration from 1.x to 2.x is an explicit installer transaction with its own
preview, state backup, compatibility checks, and rollback receipt. Repointing a
supported host registration to the accepted 1.x executable is a migration
rollback action, not a cross-channel self-update.

### No hidden current-host payloads

A generic archive, plugin, MCPB, marketplace record, or release asset must not
contain an undisclosed binary for the machine that happened to run the build.
Any payload containing native code has a target-specific identity, or a
truthfully declared and fully tested universal identity. Where a marketplace
cannot select OS and architecture, Qiongli publishes target-specific entries,
uses the local integration manager for variant selection, limits the entry to
content-only resources, or does not advertise local native support on that
surface.

## Alternatives considered

### One mutable `latest` feed with a prerelease flag

Rejected. It weakens promotion gates, makes rollback history ambiguous, and can
expose prereleases to stable clients through a metadata or client bug.

### Derive channel only from repository tags or filenames

Rejected. Consumers need signed, canonical metadata, and a filename does not
authenticate the selected product or target.

### Publish one generic plugin/archive containing the build-host binary

Rejected. It appears portable while failing on other operating systems or
architectures and gives marketplaces no truthful compatibility signal.

### Rely only on platform code signing

Rejected. Platform signatures do not by themselves bind Qiongli channel,
profile, update policy, SBOM, provenance, or every portable/content format.

### Replace the running installation in place

Rejected. A crash, power loss, disk-full condition, or bad startup can remove
the only working executable. A/B staging provides a testable recovery path.

### Put 1.x and 2.x in one update catalog

Rejected. The lines have different runtime, packaging, state, and rollback
contracts. Silent cross-major discovery would turn migration into an update and
bypass its safeguards.

## Consequences

- Release automation must build and validate a matrix rather than upload one
  ambiguous archive.
- Artifact names and manifests are longer, but support and diagnostics can
  identify the exact installed payload without inference.
- Channel promotion produces a new release record and cannot be implemented by
  moving a mutable alias.
- A/B slots require extra disk space and lifecycle management.
- Marketplace coverage may initially be narrower where the host cannot select
  target variants; compatibility claims remain honest.
- Supply-chain evidence becomes a blocking release input rather than optional
  post-publication documentation.
- 1.x remains independently supportable and cannot be accidentally retired by
  a 2.x updater change.

## Security and privacy

- Clients trust signed, expiring update metadata and verify the complete
  identity tuple, digest, size, and target before execution.
- The updater rejects unknown channels, rollback to a lower version unless an
  explicitly authorized signed rollback record permits it, expired metadata,
  replayed metadata generations, identity mismatches, and signature failures.
- Signing roles and keys are separated where practical between update metadata,
  release artifacts, and platform signing; rotation and revocation are tested
  without requiring an unsigned fallback.
- SBOM and provenance digests are bound by the release manifest so evidence
  cannot be silently swapped after publication.
- Update checks send only the selected channel and normalized artifact target
  required for resolution. They do not include project paths, research data,
  API keys, client configuration, or a persistent device identifier.
- Logs and receipts record public identities and verification outcomes, never
  signing secrets, provider credentials, or authorization headers.

## Rollback

On failed startup or health verification, the updater atomically selects the
last-known-good slot and records the failed identity and redacted reason. A
manual rollback is permitted only to an artifact retained in the selected
channel and authorized by valid signed metadata or a signed emergency rollback
record. Rollback never disables checksum, artifact signature, target identity,
or metadata verification.

If a channel publication is withdrawn, its metadata is superseded with a
signed revocation or replacement generation; already installed clients keep a
verified last-known-good slot. A compromised signing role triggers the key
rotation/revocation procedure and pauses publication for affected artifacts.
The response does not merge channels or direct clients to unsigned assets.

Rolling back this ADR means pausing 2.x native publication and update while a
replacement identity/trust design is accepted. It does not permit publishing a
generic hidden-target binary, using mutable unverified metadata, or mixing the
1.x and 2.x feeds.

## Acceptance tests

- Version/channel tests accept valid alpha, beta, and stable pairs and reject
  mismatches, unsupported prerelease labels, missing sequence numbers, and
  stable versions in prerelease feeds.
- Feed-isolation tests prove an alpha client sees only signed alpha metadata, a
  beta client sees only beta metadata, a stable client sees only stable
  metadata, and channel switching requires an explicit previewed action.
- Identity-schema tests require every tuple field and reject unknown product,
  profile, OS, architecture, or installer-kind values.
- Manifest consistency tests compare filename, embedded metadata, manifest,
  launch grant, checksum, SBOM, provenance, signature subject, and receipt
  identities and fail on any disagreement.
- Profile-grant tests reuse one canonical binary in Lite and Full packages,
  prove each artifact retains a distinct truthful identity, and reject any
  request above its signed ceiling.
- Content-only tests reject `os=any` or `arch=any` artifacts containing an
  executable, native library, install hook, or target-specific launcher.
- Native-matrix tests require target-native install and startup receipts for
  each advertised tuple and reject cross-compile-only evidence.
- Supply-chain tests verify checksums, platform/artifact signatures, SBOM,
  provenance, source revision, and manifest binding before publication.
- Update-security tests cover bad/rotated/revoked keys, insufficient signature
  threshold, expired or replayed metadata, wrong target/profile, truncated
  downloads, digest mismatch, unauthorized downgrade, and metadata rollback.
- A/B failure-injection tests cover interruption during download, staging,
  verification, pointer switch, first startup, and health window; a verified
  last-known-good slot remains bootable in every case.
- 1.x isolation tests prove 1.x package metadata never appears in 2.x feeds,
  2.x artifacts never appear in 1.x discovery, and migration requires a
  separate typed installer transaction.
- Marketplace/package scans fail any generic asset containing an undisclosed
  current-host binary or claiming an unverified target.
- Privacy tests prove update requests, logs, receipts, and crash fixtures omit
  credentials, project/research paths and data, and persistent device IDs.

## Follow-up tasks

- `REL-201`: add alpha parsing and independent alpha/beta/stable channel
  semantics to readiness, preflight, automation, postflight, notes, metadata,
  and validation tooling.
- `PKG-201`: define the native target matrix, canonical identity vocabulary,
  filenames, manifests, installer kinds, and target-native startup receipts.
- `PKG-202`: implement target signing/notarization, checksums, SBOM, provenance,
  and publication gates.
- `UPD-201`: define signed update metadata, key rotation/revocation, A/B slots,
  health checks, and the update/rollback failure matrix.
- `PLT-201` and `PLT-202`: carry artifact identity through install plans,
  receipts, managed markers, verification, removal, and migration rollback.
- `QAT-201`: add clean-machine and production process-tree audits for every
  advertised target artifact.
- `RLS-201`: publish `v2.0.0-alpha.1` only after channel isolation, target
  identity, supply-chain, startup, and rollback evidence pass.
