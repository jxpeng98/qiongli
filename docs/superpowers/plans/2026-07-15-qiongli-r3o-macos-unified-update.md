# Qiongli R3O macOS Unified Update Execution Plan

Status: Batch 2A implemented and locally accepted; Batch 2B next

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
  warnings denied. Cross-compiling the config library for Windows also passes;
  full Windows cross-Clippy remains a target-toolchain limitation on macOS and
  continues to run in target-native CI.

## Batch 2B — Download Exact Bytes Into Private Staging

- [ ] Implement bounded HTTPS archive fetch with timeouts, response-size
  limits, redirect revalidation, and the verified allowlisted host policy.
- [ ] Stream the archive into owner-private staging while checking exact size
  and SHA-256; remove incomplete or mismatched bytes.
- [ ] Add explicit cancellation and concurrency semantics without persisting a
  device identifier or making the staged archive executable.
- [ ] Test with an isolated fixture server plus offline, redirect, timeout,
  oversized, corrupt, stale-generation, and concurrent-check cases.

**Checkpoint:** CLI can truthfully report and download one verified newer
macOS update without changing the installed application.

## Batch 3 — Stage And Replace The macOS Application

- [ ] Verify the downloaded desktop manifest chain, archive layout, source,
  resource pack, Developer ID, Team ID, notarization/staple, and Gatekeeper
  result before execution.
- [ ] Add a service-free bundled native update helper that accepts only a
  transaction ID and owner-private journal.
- [ ] Add same-filesystem staging, old-app backup, fixed startup preflight,
  process-exit handoff, atomic activation, fixed-token relaunch, and bounded
  health commit.
- [ ] Fail closed for symlink/alias/mount substitution, non-owned staging,
  cross-device activation, protected/elevation-requiring locations, low disk,
  concurrent updater, and unknown journal state.
- [ ] Inject interruption before and after every state transition and prove
  restoration of the complete last-known-good application.

**Checkpoint:** A signed fixture can update and rollback the packaged app with
an empty `PATH`, without a shell, language runtime, package manager, or
privileged daemon.

## Batch 4 — Reconcile Receipt-Owned Product Content

- [ ] Inventory only supported Qiongli 2 Skills, Lite MCP/plugin, Codex, and
  Claude Code receipts; reject drifted, ambiguous, future-schema, or unmanaged
  destinations.
- [ ] Ask the staged new runtime to deterministically prepare replacements from
  its exact embedded pack before the active app changes.
- [ ] Bind every prepared operation to old/new product, pack, destination,
  receipt, plan, and content digests.
- [ ] Activate prepared operations with the application transaction and
  compensate them in reverse order on failure.
- [ ] Prove config, secrets, research data, unrelated host content, unmanaged
  files, and 1.x canaries remain byte-identical.

**Checkpoint:** A successful update leaves the application and every installed
managed surface on one new identity; rollback leaves all of them on one old
identity.

## Batch 5 — Add Desktop Update Experience

- [ ] Add display-safe update state, stream, progress, cancellation, available
  version, recovery, and fixed remediation models to the UI boundary.
- [ ] Add the Overview Update card with Stable/Beta selection, Check, Download
  and install, and recovery actions backed only by typed services.
- [ ] Preserve keyboard order, accessibility labels/status, scale, restart
  persistence, and render-loop responsiveness.
- [ ] Run packaged journeys for current, available, offline, corrupt, expired,
  read-only location, cancellation, failed health check, rollback, and restart.

**Checkpoint:** A user can safely understand and control update from the
double-clicked macOS application without Terminal.

## Batch 6 — Close Alpha.1 Publication

- [ ] Build the final exact-head macOS arm64 application and update fixture.
- [ ] Run production Developer ID signing, notarization, stapling, Gatekeeper,
  Finder launch, packaged UI, real-client, unified update, rollback, manual
  scale/contrast/VoiceOver, and clean-machine acceptance.
- [ ] Generate and sign immutable Beta stream metadata for the next eligible
  Qiongli 2 fixture/release; verify Stable rejects prerelease entries.
- [ ] Regenerate checksums, SBOM, provenance, candidate, release notes, update
  metadata, desktop descriptors, and the publication readiness receipt from
  one exact source and artifact set.
- [ ] Require exact-head Native CI and every macOS publication receipt before
  moving PR #63 from Draft or creating `v2.0.0-alpha.1`.

**Checkpoint:** The public ledger proves one usable, signed, notarized, safely
updateable macOS arm64 Alpha.1. Windows and Linux remain explicitly deferred.

## Fast Validation Loop

Batch-local tests run before the normal native workspace gate:

```text
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-platform native_update
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
