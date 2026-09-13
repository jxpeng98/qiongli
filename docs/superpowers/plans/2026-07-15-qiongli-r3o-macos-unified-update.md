# Qiongli R3O macOS Unified Update Execution Plan

Status: Batch 6A signing and metadata boundary implemented; external publication evidence next

Date: July 15, 2026

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

**Goal:** Ship a dependency-free, complete-application updater in
`v2.0.0-alpha.1` for macOS arm64 with Stable/Beta streams, transactional
managed-content reconciliation, rollback, and a strict Qiongli 2-only boundary.

**Design:**
`docs/superpowers/specs/2026-07-15-qiongli-r3o-macos-unified-update-design.md`

## Execution Policy

R3O stays in the existing rolling Draft PR. Work advances through short
checkpoints with focused Rust tests before the normal native gate. The updater
must not consume 1.x package metadata, state, registrations, or tests. Windows
and Linux compile/package checks may continue but their update executors and
interactive acceptance are not Alpha.1 blockers.

No production release manifest, tag, Developer ID signature, notarization, or
public asset is created until the final publication batch and maintainer
credentials are supplied through the accepted release boundary.

## Batch 1 — Freeze Scope And Verify Update Metadata

- [x] Record macOS arm64-only Alpha.1 publication scope and explicitly defer
  Windows/Linux interactive, signing, updater, and publication gates.
- [x] Accept the stable/beta update-stream model and Qiongli 2-only boundary in
  ADR 0209 without rewriting frozen ADR history.
- [x] Add a bounded canonical signed macOS update-entry contract in
  `qiongli-platform` using the existing embedded release authority.
- [x] Require exact Lite/macOS/aarch64/native-installer identity, HTTPS source,
  source commit, application/manifest/signing/content digests, Team ID,
  validity, and monotonic generation.
- [x] Test Stable versus Beta eligibility, Alpha.1 preview eligibility,
  current/target major >=2, strictly increasing SemVer, key windows, replay,
  expiry, canonical JSON, target mismatch, URL rejection, and redacted errors.

**Checkpoint:** Untrusted, legacy, stale, downgraded, or wrong-target metadata
cannot become a downloadable update; no network or filesystem mutation exists
in this batch.

Implemented evidence:

- `NativeUpdateStream` exposes exactly Stable and Beta. Stable accepts only a
  stable artifact; Beta accepts alpha, beta, or stable while every artifact
  retains its exact release-channel identity.
- The canonical signed manifest binds generation, Lite macOS arm64 native
  installer identity, source commit, minimum updater, signed/notarized archive,
  desktop manifest, signing receipt, embedded resource pack, Team ID, and
  validity interval.
- Verification reuses the native release key role and rejects legacy current,
  target, and minimum-updater versions; non-increasing SemVer; stale
  generation; key-window, signature, time, stream, channel, target, Team ID,
  HTTPS, and allowlisted-host failures before any download exists.
- Six focused update tests, platform Clippy with warnings denied, the 2.x
  boundary, frozen ADR validation, and the complete Rust workspace test suite
  pass. The full suite confirms that the existing bundle/archive tests dominate
  runtime, so later R3O batches retain the focused loop until checkpoint or
  publication gates.

## Batch 2A — Persist Preference And Check Signed Metadata

- [x] Add a versioned v2 update state document with selected stream, last
  accepted generation, last-known-good identity, and active transaction.
- [x] Default prerelease builds to Beta and stable builds to Stable; make a
  stream change an expected-revision transaction.
- [x] Add `qiongli update status`, `check`, and stream selection with stable
  JSON output. Human-readable control remains in the typed desktop UI rather
  than adding a second CLI output contract.
- [x] Fetch only the fixed Qiongli-owned Stable/Beta manifest endpoints with
  connect/request timeouts, identity encoding, no redirects, and a strict
  response-size limit; do not create a persistent device identifier.
- [x] Verify the fetched canonical signature, release authority, generation,
  stream, exact macOS arm64 identity, Team ID, version, archive host, size, and
  digests without writing state or starting a download.
- [x] Cover read-only status, revision-safe channel changes, signed check,
  missing release authority, active transactions, and strict CLI parsing with
  focused Rust tests.

**Checkpoint:** CLI can truthfully report state and identify one verified
current or newer macOS update without downloading bytes or changing the
installed application.

Implemented evidence:

- Update state is independent from global settings under the v2 config root,
  uses owner-private atomic Unix persistence, and rejects legacy versions,
  duplicate JSON keys, unknown fields, invalid digests, and stale revisions.
- Prerelease builds select Beta by default; only Stable and Beta are accepted,
  and stream mutation requires the caller's observed revision.
- `update check` has no caller-supplied URL. It uses fixed Qiongli endpoints,
  the embedded release authority and macOS Team ID, and returns
  `download: not-started` and `install: not-started` after verification.
- Focused config, platform, CLI, parser, and Clippy checks pass locally with
  warnings denied. Cross-target Windows Clippy for the complete `qiongli`
  application and all its targets also passes locally; target-native full
  workspace coverage remains in CI.

## Batch 2B — Download Exact Bytes Into Private Staging

- [x] Implement bounded HTTPS archive fetch with timeouts, response-size
  limits, redirect revalidation, and the verified allowlisted host policy.
- [x] Stream the archive into owner-private staging while checking exact size
  and SHA-256; remove incomplete or mismatched bytes.
- [x] Add explicit cancellation and concurrency semantics without persisting a
  device identifier or making the staged archive executable.
- [x] Cover exact private staging, corrupt, incomplete, oversized, redirect
  policy, stale revision, active transaction, and cancellation with focused
  deterministic tests.

**Checkpoint:** CLI can truthfully report and download one verified newer
macOS update without changing the installed application.

Implemented evidence:

- `qiongli update download --expected-revision <revision>` rechecks the fixed
  signed manifest, reserves a `Downloading` transaction, and streams no more
  than the signed byte count into a random owner-private staging directory.
- Redirects remain HTTPS-only and must stay on the exact GitHub release host
  allowlist. The final response must be HTTP 200 with identity encoding; any
  advertised length must equal the signed length.
- The partial file is never exposed under the signed archive name. Exact size,
  SHA-256, file sync, no-replace hard-link activation, and directory sync must
  all pass before the transaction becomes `Downloaded`.
- Failure removes partial/final bytes and clears the reserved transaction when
  safe. `qiongli update cancel --expected-revision <revision>` removes a
  Downloading/Downloaded staging transaction without touching the app.

## Batch 2C — Exercise The Network Fault Matrix

- [x] Add an isolated response/stream fixture layer that exercises the same
  manifest and archive validators without weakening production HTTPS policy.
- [x] Prove offline, refused connection, timeout, redirect loops, disallowed
  redirect hosts, compressed responses, missing/incorrect lengths, and read
  interruption return fixed path-redacted errors.
- [x] Add barrier-controlled concurrent download/cancel tests and confirm only
  one expected-revision reservation can own a staged transaction.

**Checkpoint:** Deterministic transport faults cannot leave executable,
unbounded, ambiguous, or orphaned update bytes.

Implemented evidence:

- Production manifest and archive responses now pass through isolated status,
  encoding, final-URL, and advertised-length validators before bounded reads;
  the production clients remain HTTPS-only with fixed endpoints/hosts.
- Deterministic response and stream fixtures cover redirects, redirect limits,
  disallowed hosts, compressed bodies, missing/incorrect lengths, oversized
  bodies, offline/refused/timeout reason codes, and interrupted reads.
- Every archive fetch failure removes its private transaction and leaves no
  partial or activated archive. Errors remain fixed reason codes without local
  paths.
- Barrier-controlled tests prove two callers using revision zero cannot both
  reserve a transaction, and a concurrent cancel wins without leaving bytes.
  The test also found and fixed a first-write directory initialization race so
  both callers now serialize through the revision lock.

## Batch 3A — Bind Downloaded macOS Package Evidence

- [x] Bind exact desktop-manifest and macOS signing-receipt filenames, HTTPS
  URLs, sizes, and SHA-256 digests into the signed update manifest.
- [x] Download the archive and both bounded evidence documents through the same
  fixed HTTPS/redirect policy into owner-private transaction staging.
- [x] Add offline `qiongli update verify --expected-revision <revision>` that
  revalidates the stored signed manifest, archive digest, desktop manifest,
  generic update-signing receipt, source commit, target identity, resource
  pack, Team ID, notarization, stapling, and Gatekeeper claims.
- [x] Add a distinct `Verified` transaction phase. Do not extract the archive,
  run platform tools, or claim the application is staged in this batch.

**Checkpoint:** A downloaded transaction advances to `Verified` only when its
three immutable files agree on one Qiongli 2 macOS arm64 release identity.

Implemented evidence:

- The signed update entry now binds the archive plus a versioned desktop
  manifest and generic macOS update-signing receipt as three distinct release
  assets. Sidecar hosts and redirects use the archive allowlist.
- Every file is streamed to a private partial path, checked for exact size and
  SHA-256, and activated without replacement. Any failure removes the complete
  transaction.
- Offline verification opens each staged file with no-follow final-component
  semantics, rechecks ownership/mode/size/digest, rejects duplicate receipt
  keys and unknown fields, and leaves the transaction at `Downloaded` on
  failure.
- `update verify` advances only `Downloaded -> Verified` through the expected
  revision. Output explicitly keeps `install: not-started` and does not claim
  extraction, code-signing execution, or application replacement.

## Batch 3B — Extract And Verify The macOS Application

- [x] Verify the downloaded desktop manifest chain, archive layout, source,
  resource pack, Developer ID, Team ID, notarization/staple, and Gatekeeper
  result before execution.
- [x] Extract into a new owner-private same-filesystem directory with fixed
  `/usr/bin/ditto`, reject links/special files/unexpected roots, and bind the
  extracted internal manifest and signed binary digests to Batch 3A evidence.
- [x] Run fixed-path `codesign`, `stapler`, and `spctl` adapters with bounded
  output and fixed path-redacted errors, then advance `Verified -> Staged`.

**Checkpoint:** Only one fully extracted and platform-verified Qiongli.app can
become the staged replacement candidate.

Implemented staging boundary:

- `qiongli update stage --expected-revision <revision>` accepts only a
  `Verified` transaction, re-verifies its signed manifest and all three
  immutable files, and advances only `Verified -> Staged`.
- The ZIP central and local headers are inspected before extraction. The
  validator rejects encryption, ZIP64, unsafe paths, links, special files,
  duplicate or unexpected entries, wrong sizes, local/central name drift, and
  any root other than the exact Qiongli application layout plus
  `_CodeSignature/CodeResources`.
- The fixed `ditto` adapter extracts without resource forks, extended
  attributes, quarantine, or ACL restoration into a new mode-0700 directory.
  The release archive shape uses matching `--norsrc`, `--noextattr`,
  `--noqtn`, `--noacl`, and `--keepParent` options so AppleDouble entries
  cannot enter the signed update contract.
- The extracted tree is walked without following links. It must contain only
  regular files and directories owned by the current user, exact internal
  manifest bytes, exact unsigned resource digests, and the post-signing
  launcher/canonical-binary digests bound by the generic signing receipt.
- Fixed-path `/usr/bin/codesign`, `/usr/bin/stapler`, and `/usr/sbin/spctl`
  adapters run with an empty environment, no shell, bounded output, a
  30-second timeout, fixed redacted failure codes, exact bundle identifier,
  Developer ID Application authority, and expected Team ID.
- A staging failure removes the partial application and leaves the transaction
  `Verified`. Successful staging still reports `install: not-started`; no
  application replacement, relaunch, or helper execution occurs in Batch 3B.

## Batch 3C — Replace And Roll Back The macOS Application

- [x] Add a service-free bundled native update helper that accepts only a
  transaction ID and owner-private journal.
- [x] Add same-filesystem staging, old-app backup, fixed startup preflight,
  process-exit handoff, atomic activation, fixed-token relaunch, and bounded
  health commit.
- [x] Fail closed for symlink/alias/mount substitution, non-owned staging,
  cross-device activation, protected/elevation-requiring locations, low disk,
  concurrent updater, and unknown journal state.
- [x] Inject interruption before and after every state transition and prove
  restoration of the complete last-known-good application.

**Checkpoint:** A signed fixture can update and rollback the packaged app with
an empty `PATH`, without a shell, language runtime, package manager, or
privileged daemon.

Implemented core boundary:

- Every desktop package now contains a third native executable,
  `qiongli-update-helper`. The desktop manifest, generic signing evidence,
  macOS signing/notarization entry point, and acceptance script bind its exact
  pre-signing or post-signing SHA-256 alongside the launcher and canonical
  runtime.
- Production signing emits a separate strict
  `*.signing.receipt.json` update sidecar with exactly the schema consumed by
  offline update verification. Ad-hoc signing deliberately does not emit this
  trust artifact.
- `qiongli update install --expected-revision <revision>` accepts only a
  `Staged` transaction. It re-verifies the immutable update evidence, validates
  the packaged application layout, runs the staged canonical runtime through
  `ui --startup-check`, creates an owner-private replacement journal and
  one-time health token, advances to `AwaitingExit`, and launches only the
  verified staged helper.
- The helper accepts one positional transaction ID and no paths, URLs,
  commands, runtimes, or shell input. It resolves the v2 state root itself,
  validates the fixed journal paths and helper digest, serializes replacement
  through an owner-only lock, waits for the initiating process to exit, then
  advances `AwaitingExit -> Activating -> HealthWindow`.
- Activation uses same-filesystem no-replace renames: the current application
  moves to a private sibling backup and the staged application moves to the
  exact prior location. A new canonical runtime must return successfully from
  the hidden fixed-token health command with matching binary, version, resource
  pack, transaction, and state identity before last-known-good is committed.
- A failed staged rename immediately restores the old application. A failed
  health check moves the failed new application out of the active location,
  restores the backup, clears the failed transaction, and leaves prior
  last-known-good metadata unchanged. Pre-activation timeout or validation
  failure returns the transaction to `Staged` and removes the stale contract so
  installation can be retried.
- A test-only checkpoint matrix injects interruption immediately before and
  after `Staged -> AwaitingExit`, `AwaitingExit -> Activating`,
  `Activating -> HealthWindow`, and `HealthWindow -> committed`, plus after
  each application rename. Before commit, the old application is restored or
  the transaction returns to a retryable `Staged`; after the durable health
  commit, the helper recognizes the committed state and keeps the new
  last-known-good application instead of rolling it back.
- `tooling/scripts/macos_alpha1_update_journey.sh` exercises the packaged
  ad-hoc-signed helper twice with an isolated HOME and empty `PATH`. The success
  journey proves the staged application inode becomes active and generation 2
  commits. The failed-health journey proves the original application inode is
  restored, generation 1 remains last-known-good, and transaction artifacts are
  removed. Its receipt is test-only and keeps Developer ID, notarization,
  Gatekeeper, clean-machine, network-selection, and publication gates open.

## Batch 4 — Reconcile Receipt-Owned Product Content

- [x] Inventory only supported Qiongli 2 Skills, Lite MCP/plugin, Codex, and
  Claude Code receipts; reject drifted, ambiguous, future-schema, or unmanaged
  destinations.
- [x] Ask the staged new runtime to deterministically prepare replacements from
  its exact embedded pack before the active app changes.
- [x] Bind every prepared operation to old/new product, pack, destination,
  receipt, plan, and content digests.
- [x] Activate prepared operations with the application transaction and
  compensate them in reverse order on failure.
- [x] Prove config, secrets, research data, unrelated host content, unmanaged
  files, and 1.x canaries remain byte-identical.

**Checkpoint:** A successful update leaves the application and every installed
managed surface on one new identity; rollback leaves all of them on one old
identity.

Implemented reconciliation boundary:

- Skills ownership is recorded only after an explicit CLI/UI materialization
  in an owner-private canonical `managed-content.json` registry. The entry
  binds the absolute target, Qiongli 2 version, profile, receipt, embedded pack,
  and content root. Registration failure compensates a new materialization or
  restores the prior verified materialization instead of deleting old content.
- The staged canonical runtime receives only the transaction ID, reloads the
  signed update evidence, verifies release-authority launch grants for Codex
  and Claude Code, and composes every replacement from its exact embedded pack
  and signed canonical binary. No shell, Python, Node, Rust toolchain, package
  manager, arbitrary command, or caller-selected update path participates.
- Reconciliation inventories only the explicit Skills registry and the fixed
  Codex/Claude Qiongli 2 source and registration receipts. Missing surfaces are
  skipped; legacy, mixed-version, drifted, conflicting, recovery-required, or
  unsupported state blocks preparation without changing active product bytes.
- A canonical transaction journal binds each destination and staging/backup
  path to old/new version, pack, receipt, content, and operation-plan digests.
  The Skills registry itself is a journaled operation, so successful updates do
  not leave new Skills bytes behind an old receipt index.
- The helper verifies the prepared journal digest before application handoff,
  activates the app and content with no-replace same-filesystem renames, and
  verifies the active identity during the new runtime health check. Any
  pre-commit failure compensates content in reverse order before restoring the
  old application; committed cleanup removes only verified backups/staging.
- `reconciliation-prepared` is a retryable install state. A retry reuses only an
  unchanged canonical journal with the expected target version and pack;
  cancellation verifies and removes external content staging before removing
  the update transaction.
- Focused and full Rust tests prove Skills directory and registry inode
  replacement/restoration, application helper failure checkpoints, exact
  Codex/Claude bundle composition, signed client grant verification, and
  byte-identical config, secret-reference, research, unmanaged, and 1.x
  canaries.

## Batch 5 — Add Desktop Update Experience

- [x] Add display-safe update state, stream, progress, cancellation, available
  version, recovery, and fixed remediation models to the UI boundary.
- [x] Add the Overview Update card with Stable/Beta selection, Check, Download
  and install, and recovery actions backed only by typed services.
- [x] Preserve keyboard order, accessibility labels/status, scale, restart
  persistence, and render-loop responsiveness.
- [x] Run deterministic desktop state journeys for current, available, offline,
  corrupt, expired, read-only location, cancellation, failed health check,
  recovery, and restart; retain the existing packaged helper success/rollback
  journey as the application-replacement evidence.
- [ ] Repeat the live-metadata and production-signed packaged journeys against
  the final exact-head candidate in Batch 6.

**Checkpoint:** A user can safely understand and control update from the
double-clicked macOS application without Terminal.

## Batch 6 — Close Alpha.1 Publication

- [ ] Build the final exact-head macOS arm64 application and update fixture.
- [x] Make the production signing output self-contained by carrying the exact
  desktop manifest beside the signed/notarized archive and strict update
  signing receipt.
- [x] Add a three-stage external-signing workflow that generates canonical
  Codex/Claude launch-grant preimages, accepts only detached public
  signatures, generates the canonical Beta manifest preimage, and verifies the
  final signed metadata without accepting private-key material.
- [ ] Run production Developer ID signing, notarization, stapling, Gatekeeper,
  Finder launch, packaged UI, real-client, unified update, rollback, manual
  scale/contrast/VoiceOver, and clean-machine acceptance.
- [ ] Generate and sign immutable Beta stream metadata for the next eligible
  Qiongli 2 fixture/release; verify Stable rejects prerelease entries.
- [ ] Regenerate checksums, SBOM, provenance, candidate, release notes, update
  metadata, desktop descriptors, and the publication readiness receipt from
  one exact source and artifact set.
- [x] Add the offline deterministic checksums/CycloneDX/SLSA evidence
  generator and fail-closed final publication-ledger validator. Preflight
  evidence is structurally unable to satisfy finalization.
- [ ] Require exact-head Native CI and every macOS publication receipt before
  moving PR #63 from Draft or creating `v2.0.0-alpha.1`.

**Checkpoint:** The public ledger proves one usable, signed, notarized, safely
updateable macOS arm64 Alpha.1. Windows and Linux remain explicitly deferred.

Implemented Batch 6A repository boundary:

- `macos_alpha1_sign_notarize.sh` now emits the exact canonical desktop
  manifest in every result directory. Production output therefore contains
  the signed/notarized archive, the strict update signing receipt, and the
  manifest that both records bind.
- `native_alpha1_update_metadata` implements `prepare-grants`,
  `prepare-manifest`, and `finalize`. The first two phases emit
  domain-separated canonical signing preimages. Only lowercase detached
  Ed25519 signatures return to the tool; no command accepts a seed, private
  key, password, Keychain export, arbitrary signing command, or publication
  destination.
- Finalization re-reads the production artifact set and public authority,
  verifies the release-key generation window, Beta metadata signature,
  production signing/notarization evidence, Codex and Claude Code launch
  grants, and Stable-stream rejection. It emits a canonical metadata response
  and a `publication_allowed: false` receipt.
- Deterministic tests complete the three stages with ephemeral test-only keys,
  prove strict signature-file parsing, and keep all production authority
  outside the product and repository workflow. Native CI also compares the
  manifest copied through the macOS signing boundary byte-for-byte.
- The exact operator commands and remaining external gates are recorded in
  `tooling/release/v2.0.0-alpha.1.md`.

Implemented Batch 6B repository boundary:

- `native_alpha1_release_evidence` provides separate
  `prepare-preflight`, `prepare-production`, and `finalize` commands.
  Production preparation re-verifies the signed/notarized application, public
  authority, signed Beta metadata, Codex and Claude Code launch grants,
  Stable-stream rejection, and the production-signed portable candidate.
- The tool parses `Cargo.lock` without network access or `cargo metadata`,
  writes a sorted SHA-256 asset manifest, canonical CycloneDX 1.6 SBOM,
  canonical in-toto/SLSA Provenance v1 statement, and one receipt binding all
  source, build, asset, and evidence digests.
- Finalization requires seven typed macOS acceptance/CI receipts and verifies
  every flat attachment against its size and SHA-256. Missing, false, stale,
  preflight, mismatched, modified, or unexpected evidence fails closed without
  creating an output directory.
- A successful ledger proves evidence completeness only. It remains
  `publication_allowed: false` and cannot create a tag, upload assets, change
  the Beta endpoint, or grant maintainer authorization.

## Alpha.1 Remaining Critical Path

All Alpha.1 updater implementation checkpoints through Batch 5 are complete.
Batch 6A provides the repository-side production signing and detached
metadata-signing boundary. Batch 6B provides deterministic supply-chain
evidence and the fail-closed final ledger. Public `v2.0.0-alpha.1` remains
blocked on the external execution portion of Batch 6: final exact-head
artifacts and embedded public authority, Developer ID signing/notarization,
detached production signatures, production-candidate regeneration, the seven
macOS acceptance/CI receipts, and final ledger assembly. Signing credentials
remain intentionally outside the repository and product.

## Fast Validation Loop

Batch-local tests run before the normal native workspace gate:

```text
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-platform native_update
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli update
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
```

The complete workspace test runs at checkpoint boundaries and before final
publication, not after each small updater implementation commit. Legacy Python
and Node test suites remain non-blocking and out of scope.

## Completion Definition

R3O is complete only when the packaged macOS arm64 app performs a signed
Stable/Beta Qiongli 2-only update, reconciles receipt-owned content, rolls back
all product bytes on injected failure, exposes the flow accessibly in desktop
and CLI, and binds final evidence to the exact published head. Documentation or
metadata verification alone does not make Alpha.1 update-ready.
