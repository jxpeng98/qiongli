# ADR 0209: macOS Unified Update And Qiongli 2-Only Boundary

- Status: Accepted
- Date: 2026-07-15
- Task ID: `ARC-209`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 Alpha.1 platform scope, update streams, replacement,
  managed-content reconciliation, and legacy boundary
- Supersedes in part: ADR 0204's 1.x import, compatibility-read, and 1.x
  rollback requirements; ADR 0207's three independently selectable updater
  channels; R3N's cross-platform Alpha.1 publication gate

## Context

Qiongli 2 is now a single Rust-native application whose executable, desktop
window, embedded Skills, Lite MCP, integration adapters, agents, and later
orchestrator ship as one versioned product. The 1.x updater had to update the
language package and then refresh separately distributed assets. Carrying that
split into Qiongli 2 would create two versions of one product and allow an old
runtime to install new content, or a new runtime to retain old content.

The first native alpha also does not need to make macOS, Windows, and Linux
publication readiness advance in lockstep. Current exact-package evidence is
strongest on macOS arm64. Requiring the other interactive and signing journeys
before the first alpha delays feedback without improving the macOS artifact.

ADR 0204 was accepted while 1.x remained a recovery target and therefore
specified import and compatibility behavior. The product direction has since
changed: the Rust line starts at `2.0.0-alpha.1`, owns only its v2 namespace,
and will not act as a migration or repair tool for 1.x.

## Decision

### Alpha.1 platform scope

`v2.0.0-alpha.1` is published only for macOS arm64. Its release gate includes
the signed and notarized application, Finder launch, the packaged UI journeys,
real supported-client journeys, unified update acceptance, and a final
exact-head release ledger on macOS arm64.

Windows and Linux builds may continue in CI as non-publishing engineering
artifacts. Their compilation and structural package checks may catch portable
code regressions, but Windows/Linux interactive acceptance, platform signing,
and publication are moved to a later alpha. A passing cross-compile or package
inspection is never represented as target-native publication acceptance.

### One application update

Qiongli 2 exposes one `update` capability through the desktop application and
CLI. It replaces both 1.x `selfupdate` and `upgrade` semantics:

1. resolve signed update metadata for the selected stream;
2. select only a strictly newer compatible complete application;
3. download and verify the exact macOS application archive;
4. verify Qiongli release metadata, digest, Developer ID, notarization, Team ID,
   product identity, target, and minimum updater contract;
5. stage and startup-check the new application without changing the active
   installation;
6. use the new executable and embedded resource pack to prepare reconciliation
   for every destination already owned by a Qiongli 2 receipt;
7. exit the running application and let the bundled native update helper swap
   the complete application plus prepared managed-content transactions;
8. relaunch and enter a bounded health window; and
9. retain the previous verified application and transaction journal until the
   health window commits, otherwise restore both.

The shipped user does not need Rust, Python, Node.js, Cargo, pip, npm, Homebrew,
or another package manager. The helper is a bundled native executable and may
perform only the fixed staged-replacement protocol. It does not become a second
runtime or contain product services.

The updater never updates an executable and Skills independently. A version is
accepted only as one complete application identity. After the application swap,
the new embedded content is rewritten only into destinations whose current
Qiongli 2 receipts prove ownership. Unmanaged paths, user files, unrelated
plugins, provider configuration, secrets, and research data are preserved.

### Stable and beta update streams

The user selects exactly one update stream:

- `stable` accepts only stable Qiongli 2 artifact identities;
- `beta` is the opt-in preview stream and accepts Qiongli 2 alpha, beta, and
  stable artifact identities.

This two-stream selection supersedes the independently selectable updater
channels in ADR 0207 but does not change artifact identity. Every artifact
still truthfully declares `alpha`, `beta`, or `stable` according to SemVer. The
beta stream aggregates eligible signed artifact records so Alpha.1 can update
to later alphas before a beta exists. Selection uses SemVer precedence, never
publication time or a mutable filename. A stable installation may explicitly
switch to beta; beta may switch to stable only when an eligible stable version
is strictly newer. Stream changes are previewed and persisted in v2 state.

Alpha and beta application builds default to the beta stream. Stable builds
default to stable. Update checks are user-initiated in Alpha.1; automatic
background installation is not enabled.

### Signed metadata and replacement authority

Each stream publishes a canonical signed manifest generation. A macOS update
entry binds at least:

- the complete artifact identity and exact SemVer;
- stream and monotonic metadata generation;
- source commit and minimum compatible updater version;
- signed/notarized application archive URL, filename, size, and SHA-256;
- desktop-package manifest and signing-receipt SHA-256;
- expected Apple Developer Team ID;
- publication, validity, and expiry times; and
- the embedded resource-pack identity used for reconciliation.

The embedded release authority verifies metadata before selection. The updater
rejects an untrusted key, stale generation, expiry, wrong stream, wrong target,
wrong profile, non-HTTPS source, unexpected redirect host, digest mismatch,
platform-signature mismatch, or non-increasing version. No metadata field,
environment variable, MCP request, model output, or plugin may provide an
arbitrary executable path or command.

Application replacement is allowed only when the current installation's parent
is safely writable by the current user. Alpha.1 fails closed with fixed manual
remediation when elevation would be required; it does not prompt for an
administrator password or install a privileged daemon.

### Qiongli 2-only boundary

Every Rust-native product, state, update, install, receipt, and recovery path is
Qiongli 2 or newer. Both the running version and update target must have SemVer
major version at least `2`. The special prerelease `2.0.0-alpha.1` is therefore
inside the boundary.

Qiongli 2 does not discover, read, import, migrate, rewrite, delete, repair,
upgrade, downgrade to, or take ownership of any Qiongli 1.x package, config,
plugin, Skill, MCP registration, receipt, or update feed. Existing 1.x bytes may
coexist on disk, but they are outside the Qiongli 2 product contract. Installing
Qiongli 2 is a fresh installation; removing it removes only receipt-owned v2
bytes. A 1.x user who wants Qiongli 2 installs it explicitly and configures it
as a new product.

The secure v2 namespace, atomic persistence, secret separation, and transaction
primitives of ADR 0204 remain in force. Only its legacy import,
compatibility-read, legacy cleanup, and rollback-to-1.x requirements are
superseded.

## Alternatives considered

### Keep software self-update and content upgrade separate

Rejected because it permits runtime/content skew and recreates the packaging
constraints of the Python line.

### Preserve alpha, beta, and stable as three user-visible update choices

Rejected for the application UX. A single opt-in preview stream can deliver
successive alphas and betas while signed artifact identities still preserve the
exact maturity level.

### Automatically import 1.x state

Rejected because it expands the updater into a cross-runtime migration engine,
couples the new product to frozen formats, and weakens the clean Qiongli 2
ownership boundary.

### Block Alpha.1 until three-platform interactive acceptance

Rejected. It hides useful macOS feedback behind unrelated platform readiness.
Later platform publication still requires its own native acceptance and trust
evidence.

### Replace the running application in place

Rejected because interruption can destroy the only known-good application and
leave managed content at a different version.

## Consequences

- Alpha.1 has a smaller, truthful support matrix: macOS arm64 only.
- The updater becomes an Alpha.1 blocker rather than a post-alpha feature.
- Update archives are larger than a content-only refresh, but the runtime and
  embedded product content cannot drift.
- The beta stream remains simple for users while artifact and release ledgers
  retain exact alpha/beta/stable identities.
- Qiongli 2 development and support do not carry 1.x parsing, migration, or
  downgrade code.
- Windows and Linux remain build-visible but cannot be advertised until their
  deferred gates pass.

## Security and privacy

- Update checks send only the selected stream and normalized macOS arm64
  identity; no stable device ID, project path, provider value, secret, client
  config, or research data is sent.
- Metadata and payload verification happen before staging or execution, and
  Apple platform verification happens again after extraction.
- Staging, journal, and backup directories are owner-private and reject
  symlinks, aliases, traversal, mount substitution, and out-of-root writes.
- Reconciliation uses only verified Qiongli 2 receipts and new embedded bytes;
  it does not enumerate legacy or arbitrary plugin directories.
- Logs and UI expose fixed reason and remediation codes, public versions, and
  digests, never private paths, authorization headers, credentials, or signing
  material.

## Rollback

Before activation, the updater retains the exact current application and a
verified snapshot or compensating journal for every managed destination that
will change. Failure before commit removes staging and leaves active bytes
untouched. Failure during swap or the health window restores the application
and every already-applied managed transaction, then verifies the old receipts.

Rollback never crosses below major version 2, switches to a 1.x runtime, or
reads 1.x state. A manual rollback requires a retained verified Qiongli 2
artifact authorized by signed metadata. If rollback cannot be proven complete,
the updater stops in recovery-required state and performs no further writes.

## Acceptance tests

1. Stable accepts only a strictly newer stable 2.x artifact; beta accepts a
   strictly newer alpha, beta, or stable 2.x artifact.
2. Current or target versions below major 2, equal/downgrade versions, channel
   mismatches, stale generations, expiry, bad signatures, wrong Team ID, wrong
   target, non-HTTPS URLs, and payload drift fail before staging.
3. An isolated macOS arm64 application checks, downloads, verifies, stages,
   swaps, relaunches, and commits an update with an empty `PATH` and no language
   or package-manager runtimes.
4. Receipt-owned Skills, MCP, Codex, and Claude Code content is regenerated from
   the new embedded pack; unmanaged and unrelated bytes remain unchanged.
5. Fault injection before and after every application and managed-content swap
   restores one complete old or new version, never a mixed version.
6. A read-only or elevation-requiring application location fails closed with
   fixed remediation and no partial write.
7. Legacy 1.x canaries in config and integration locations remain unread and
   byte-identical across check, update, failure, rollback, and removal.
8. The final Alpha.1 ledger binds the exact signed/notarized macOS arm64
   archive, signed update manifest, update journey, packaged UI journey,
   real-client journey, and exact source head.
