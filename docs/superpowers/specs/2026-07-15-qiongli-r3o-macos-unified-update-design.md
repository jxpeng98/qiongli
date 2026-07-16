# Qiongli R3O macOS Unified Update Design

Status: accepted for Alpha.1 implementation

Date: July 15, 2026

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Outcome

R3O adds the minimum safe complete-application updater required by
`v2.0.0-alpha.1`. The first publication target is macOS arm64 only. A user can
check and install a newer Qiongli 2 application from the stable or beta stream,
then the new application reconciles every Qiongli-managed local surface from
its own embedded content without Rust, Python, Node, or a package manager.

The updater does not reuse the 1.x split between package `selfupdate` and asset
`upgrade`. It does not contain a 1.x importer or compatibility mode.

## Product Model

There is one version across the complete installed product:

```text
signed app archive
  -> canonical Rust runtime and desktop UI
  -> embedded Skills, Lite MCP, adapters, agents, and product metadata
  -> receipt-owned materializations derived from that exact embedded pack
```

An update is complete only after the application and all previously managed
materializations agree on the new identity, or after rollback restores the old
identity. A user may install or remove a managed surface independently, but an
updater cannot leave an installed surface on stale embedded bytes.

## Update Streams

`UpdateStream::Stable` accepts stable artifact identities only.
`UpdateStream::Beta` accepts alpha, beta, and stable artifact identities. The
second rule allows Alpha.1 to receive later prereleases while keeping only two
user-facing choices. Artifact metadata continues to identify the exact SemVer
release channel.

The selected entry must be strictly newer by SemVer and both current and target
major versions must be at least 2. Alpha/beta applications default to beta;
stable applications default to stable. Alpha.1 offers explicit check/install
and stream selection; scheduled background installation is deferred.

## Trust Chain

The update client reads a bounded canonical JSON document signed by the
embedded native release authority. The first contract verifies one selected
entry; the publication layer is responsible for constructing each stream's
monotonic manifest from immutable release records.

The signed entry binds:

- schema and monotonic generation;
- stable/beta update stream;
- exact Lite macOS arm64 native-installer artifact identity;
- source commit and minimum compatible updater version;
- exact signed/notarized `.app.zip` filename, HTTPS URL, byte size, and digest;
- exact desktop-package manifest and generic macOS update-signing receipt
  filenames, HTTPS URLs, byte sizes, and digests;
- embedded resource-pack digest;
- ordered Codex and Claude Code PluginBundle launch grants bound to the same
  product version, generation, signed canonical runtime, embedded pack,
  integration scope, mode, and validity window;
- expected Apple Developer Team ID; and
- validity interval.

The verifier performs no network or filesystem access. Download policy accepts
only HTTPS and a release-time allowlist; redirects are revalidated at every
hop. After download, the executor checks byte size and SHA-256 before extraction
and validates the archive against the bound desktop manifest. macOS then runs
Developer ID, designated-requirement, Team ID, notarization/staple, and
Gatekeeper checks on the staged application.

## Execution State Machine

```text
idle
  -> checking
  -> update-available | current | blocked
  -> downloading
  -> verified
  -> staged
  -> reconciliation-prepared
  -> awaiting-exit
  -> activating
  -> health-window
  -> committed | rolled-back | recovery-required
```

Every transition is journaled in the v2 state root with expected prior state
and public digests. Retrying an identical committed generation is a verified
no-op. A second updater cannot enter the mutation path while a transaction lock
is held.

The replacement implementation treats the health commit as the durable
decision point. An interruption before it restores the old application or
returns the verified new application to staging. If the new runtime commits
health but its success exit is lost, the helper re-reads the state, recognizes
the committed generation, and completes cleanup without reverting the new
last-known-good application. Test-only checkpoints cover both sides of every
state transition and both application renames; they are not exposed through
environment variables, CLI flags, journals, or release binaries.

## macOS Replacement Boundary

The running application cannot replace itself. The package therefore gains one
small native update helper. It accepts only a fixed transaction identifier and
resolves every staged path from the owner-private journal. It does not accept a
download URL, shell command, arbitrary destination, or plugin/model input.

The canonical runtime downloads, verifies, extracts, startup-checks, and
prepares reconciliation. The helper waits for the canonical process to exit,
renames the current application to the transaction backup, activates the
staged application in the same filesystem, applies prepared receipt-owned
transactions, and relaunches the new canonical runtime with a fixed health
token. The new runtime commits after bounded startup, embedded-pack, config,
and managed-receipt checks. Otherwise the helper restores the old application
and compensates managed writes.

Alpha.1 updates only a user-writable application location. A protected location
returns `update-install-location-not-writable` and instructions to replace the
application manually. Privileged helpers and password prompts are deferred.

## Managed Content Reconciliation

The preparation pass inventories only Qiongli 2 receipt registries already
owned by the config/content/platform services. For each valid current receipt,
it asks the new staged runtime to build a deterministic replacement from the
new embedded pack. It never infers ownership from a familiar directory name.

The transaction includes Skills materializations, Lite MCP/plugin bundles, and
Codex or Claude Code registrations that carry supported Qiongli 2 receipts.
Explicit Skills materializations are indexed in a canonical owner-private
registry, and that registry is updated as one of the compensated operations.
Each prepared operation binds old/new product version, embedded pack,
destination, receipt, content root, and plan digests. The helper activates
operations with same-filesystem no-replace renames after the application swap,
then reverses them before restoring the old application on any pre-commit
failure.

Config documents, selected stream, provider settings, secret references,
research projects, unmanaged host content, and 1.x bytes are not rewritten.
Unknown, drifted, future-schema, or ambiguous receipts block automatic update
with an explicit remediation instead of being overwritten.

## UI And CLI

Overview gains an Update card with current version, selected stream, last check
status, available version, and fixed remediation. Actions are:

- choose Stable or Beta;
- Check for updates;
- Download and install; and
- Retry recovery when a verified journal permits it.

The CLI gains equivalent machine-readable commands under `qiongli update`.
The UI receives display-safe typed events and opaque transaction tokens; it
does not fetch URLs, open archives, run platform verification, or write paths.

The staged application checkpoint is exposed as
`qiongli update stage --expected-revision <revision>`. It accepts only a
previously `Verified` transaction, performs no network access, and does not
replace or execute the downloaded application. The command revalidates the
immutable evidence, preflights both ZIP central and local headers, extracts
through fixed `/usr/bin/ditto`, verifies the exact internal manifest and
post-signing binary digests, then runs fixed `codesign`, `stapler`, and `spctl`
trust adapters before advancing to `Staged`.

The replacement checkpoint is exposed as
`qiongli update install --expected-revision <revision>`. From `Staged`, it asks
the verified staged canonical runtime to prepare a canonical content
reconciliation journal, then advances through `ReconciliationPrepared`. A
verified unchanged `ReconciliationPrepared` transaction is also accepted for
retry after a pre-activation helper failure. Installation performs a fixed
no-window startup preflight, writes an owner-private fixed-path replacement
journal and health token, advances to `AwaitingExit`, and starts the signed
`qiongli-update-helper` bundled inside the staged application. The helper
accepts only the transaction ID, waits for the initiating process to exit,
performs same-filesystem no-replace renames, and requires the newly activated
canonical runtime to commit a fixed-token application and managed-content
health check before removing old backups.

The desktop manifest and signing receipt bind three executable identities:
the thin desktop launcher, the canonical product runtime, and the native update
helper. The helper is signed before the outer application bundle and is never
resolved from `PATH`.

The macOS release archive must be built without resource forks, extended
attributes, quarantine metadata, ACLs, or AppleDouble paths. Only the exact
application payload, its internal desktop manifest, and
`Contents/_CodeSignature/CodeResources` are valid archive files.

## 2.x-Only Boundary

The updater rejects a current or target major below 2 before selection. The
application resolves only the v2 config root and receipt formats. No code path
searches PyPI/npm metadata, invokes the 1.x CLI, imports legacy state, upgrades
legacy plugins, or redirects host registrations from/to 1.x. Legacy bytes used
as test canaries must remain unread and unchanged.

## Delivery Batches

1. Land ADR, macOS-only roadmap, signed update-entry contract, stream and
   2.x-only verification tests.
2. Add v2 update preferences/state, fixed-endpoint CLI status/check/channel,
   and signed metadata verification without download or mutation.
3. Add the bounded HTTPS archive downloader, private exact-byte staging,
   cancellation/concurrency handling, and deterministic byte-integrity tests.
4. Exercise the production manifest/archive validators through isolated
   response and stream fault fixtures, including redirects, timeouts,
   interruption, and concurrent cancel, without weakening production HTTPS.
5. Bind/download the desktop manifest and generic signing receipt, then add
   offline expected-revision evidence verification and a distinct `Verified`
   transaction phase.
6. Extract the application through fixed macOS adapters, verify exact layout,
   Developer ID, Team ID, notarization/staple, and Gatekeeper, then advance to
   `Staged`. Implemented in Batch 3B.
7. Add the bundled helper, journaled A/B replacement,
   relaunch/health/rollback, state-transition fault injection, and a packaged
   ad-hoc-signed update/rollback journey. Implemented in Batch 3C.
8. Add receipt inventory and staged-runtime reconciliation for installed
   Skills, MCP, Codex, and Claude Code surfaces. Implemented in Batch 4.
9. Add the Overview Update card and packaged end-to-end journeys from Alpha.1
   to a later signed fixture, including offline, stale, corrupt, read-only, and
   rollback cases.
10. Generate production stream metadata and the exact-head macOS Alpha.1
   publication ledger after Developer ID signing and notarization.

## Deferred Work

- Windows and Linux update executors, interactive acceptance, signing, and
  publication;
- Intel macOS and universal application publication;
- privileged system-wide installation;
- scheduled background installation;
- delta/binary patches;
- enterprise mirrors or managed update policy; and
- all 1.x import, migration, compatibility, and rollback behavior.
