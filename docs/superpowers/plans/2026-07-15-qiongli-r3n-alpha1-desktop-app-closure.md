# Qiongli R3N Alpha.1 Desktop Application Closure Execution Plan

Status: Batch 5 macOS packaged UI journey accepted; external release gates remain

Date: July 15, 2026

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

**Goal:** Close the five observed desktop acceptance gaps and produce a useful,
dependency-free, double-clickable Lite Alpha.1 application on macOS, Windows,
and Linux.

**Design:**
`docs/superpowers/specs/2026-07-15-qiongli-r3n-alpha1-desktop-app-closure-design.md`

## Execution Policy

R3N stays in the existing rolling Draft PR. The batches below are short,
dependency-ordered checkpoints, not new branches or PRs. Each checkpoint must
leave the native workspace green and update the PR ledger with only observed
facts. R3M signing and install security contracts are reused rather than
redesigned.

## Batch 1 — Make Overview And Skills Operable

- [x] Add redacted editable global-settings models and typed load/preview/apply
  service operations.
- [x] Edit default profile, provider enablement, and supported public contact
  fields from Overview; keep secret values and references read-only.
- [x] Enforce expected revision, atomic write, reread verification, zeroized
  transient public-setting input, and fixed recovery feedback.
- [x] Add a native Skills folder picker and display-safe selected-destination
  state.
- [x] Reuse the existing materializer for preview/apply/verify and add
  receipt-owned remove without accepting model/MCP-provided paths.
- [x] Test restart persistence, stale revision, invalid config, symlink/path
  rejection, unrelated files, partial failure, and accessibility labels.

**Checkpoint:** A user can change supported global settings and materialize a
selected Skills profile from the window without direct UI filesystem access.

Implemented evidence:

- Overview edits default profile, provider enablement, and OpenAlex/Crossref
  public email replacements through a digest-bound preview and explicit
  client-config approval.
- Confirmation reopens the versioned store with the expected revision, retains
  existing secret references, commits atomically, and refreshes the redacted
  snapshot. A concurrent write fails with `revision-conflict`.
- Skills selection uses the native OS folder dialog. The service retains the
  real path privately while UI/debug events use redacted wrappers.
- Materialize and remove require digest-bound filesystem confirmation. Verify
  is read-only. Removal locks and verifies the receipt, moves the tree to a
  sibling quarantine, verifies it again, then removes only those managed bytes.
- Focused content, UI/AccessKit, desktop-service, host Clippy, Windows MSVC
  check/Clippy, complete native workspace tests, and the 2.x boundary pass.

## Batch 2 — Add MCP Health And Real Discovery

- [x] Add a bounded asynchronous Lite MCP self-test service using the canonical
  registry and dispatcher.
- [x] Report initialize, exact tools list, offline dispatch, redacted provider
  readiness, and discovered client-registration state.
- [x] Add timeout/cancel handling and keep network/mutation out of the default
  test.
- [x] Decouple Codex and Claude Code discovery from signed-candidate apply
  authority in source and packaged sessions.
- [x] Distinguish not discovered, discovered unmanaged, managed, drifted, and
  candidate-required states with fixed remediation.
- [x] Test isolated adapters with missing, discovered-unmanaged, managed,
  drifted, conflict, and recovery classifications.
- [x] Record release-grade real-client evidence when supported client binaries
  and their required validators are available.

**Checkpoint:** The MCP button produces actionable evidence, and the
Integrations page discovers local Codex and Claude Code even in an ordinary
source session.

Implemented evidence:

- `Run Lite MCP self-test` executes the embedded Marketplace Lite registry and
  the same `LiteMcpServer` dispatcher as stdio mode on a named worker thread.
  It checks initialize, exact ordered public tools, and the offline
  `qiongli_task_plan` route without starting a process or making a network or
  mutation request.
- The UI polls typed progress without blocking the render loop, disables
  duplicate starts, exposes cancellation, and enforces a five-second service
  deadline. Window/service drop also signals cancellation.
- Provider readiness and client registration are reported only as bounded
  counts, fixed statuses, reason codes, and remediation codes.
- Read-only client discovery recognizes a normal Codex `.codex` config root
  and the selected/default Claude Code config root without requiring a signed
  candidate. Candidate or grant sessions remain the only apply authority.
- Integrations render `client not discovered`, `discovered but unmanaged`,
  `managed`, `drifted`, `conflict`, `recovery required`, and `candidate required
  for install` independently rather than collapsing them into `missing`.
- UI/AccessKit and isolated desktop-service tests cover success, attention,
  cancellation, timeout, discovery, and authority separation. A real Claude
  receipt was attempted but the locally advertised executable was an invalid
  mise shim; the Codex receipt remains unavailable because the required Plugin
  Creator validator is not installed. Neither is claimed as passed evidence.

## Batch 3 — Add The Desktop Application Entry

- [x] Extract one reusable native app composition entry from the current
  `qiongli ui` command path.
- [x] Make desktop/no-argument activation open the UI while preserving every
  explicit CLI command and machine-readable output contract.
- [x] Add only the minimum platform launcher boundary needed to avoid a
  persistent console window on Windows; keep all product logic in shared Rust
  services.
- [x] Add product name, version, icon, application identifier, license, and
  startup-error metadata.
- [x] Test CLI/UI dispatch, repeated launches, invalid embedded content, missing
  config home, renderer failure, and no-language-runtime startup.

**Checkpoint:** A development desktop activation entry is available without
typing `qiongli ui`, and CLI commands behave exactly as before. OS package
assembly remains Batch 4.

Implemented evidence:

- The canonical `qiongli` executable now routes no-argument activation through
  a reusable desktop application composition entry. Every explicit command
  continues through the existing strict parser and output contracts, while
  `qiongli ui` remains available for terminal and host activation.
- `qiongli-desktop` is a cross-platform thin launcher. It resolves only the
  sibling canonical executable and invokes its UI mode; it contains no product
  services or embedded content. On Windows it is a GUI-subsystem executable
  and starts the canonical child with `CREATE_NO_WINDOW`, preventing a
  persistent console without creating a second product implementation.
- ADR 0208 records this target-specific exception while restoring frozen ADR
  0201 byte-for-byte to the accepted 2.x architecture baseline.
- Native window metadata now has the fixed application identifier
  `io.github.jxpeng98.qiongli`, package version and license, fixed startup error
  codes, and a Qiongli-specific RGBA window icon. The identifier is also bound
  to the eframe viewport rather than inheriting an egui default.
- Tests cover empty versus explicit dispatch, metadata, icon completeness,
  sibling-only launcher resolution, invalid embedded content, renderer
  failure, and two consecutive startup validations without a config home.
- Format, workspace check, workspace Clippy, all workspace test targets, and
  the 2.x change boundary pass. The loopback-only Zotero tests require a test
  environment that permits binding a local listener; they pass there after
  the restricted sandbox rejects the bind with `Operation not permitted`.
- The Windows MSVC desktop target checks successfully, and a native startup
  preflight succeeds with an empty `PATH`. On macOS the debug launcher is about
  1 MiB versus about 88 MiB for the canonical debug runtime, confirming that
  product bytes were not duplicated into the launcher.

## Batch 4 — Package The Three Desktop Targets

- [x] Add one deterministic package composer for macOS `.app.zip`, Windows
  portable ZIP, and Linux AppDir ZIP development artifacts.
- [ ] Produce a macOS `.app` and distributable archive/DMG with Finder launch.
- [ ] Produce a Windows desktop GUI executable and portable package with
  Explorer launch and no persistent console window.
- [x] Produce a Linux AppImage with icon and desktop metadata using a
  digest-pinned official Type 2 builder and reverse payload verification.
- [x] Bind each desktop artifact to the R3M product, version, source,
  executable, and embedded-pack identity without weakening the three-file
  portable candidate contract.
- [x] Add CI jobs which assemble and structurally inspect all three packages;
  upload only explicitly non-publishing artifacts until signing gates pass.
- [x] Document target-specific installation, CLI access, removal, trust
  prompts, and unsupported architectures truthfully.

**Checkpoint:** CI produces one desktop-activatable artifact for macOS,
Windows, and Linux from the same exact source and embedded pack.

Implemented foundation evidence:

- The desktop-package manifest is separate from the accepted R3M
  three-file portable candidate. It records `assembled-unpublished`, the exact
  source artifact and manifest hash, product source commit, canonical binary,
  launcher, embedded pack, application metadata, every package entry, and a
  deterministic entry content root.
- Package verification reparses the stored ZIP, validates the exact target
  layout, modes, hashes, resource identity, source binding, and application
  metadata, and rejects added, removed, reordered, or modified bytes.
- macOS receives `Qiongli.app` with `Info.plist`, a thin `Qiongli` launcher,
  the canonical `qiongli-cli`, ICNS icon, MIT license, and package manifest.
  Windows receives a GUI `Qiongli.exe`, canonical `qiongli-cli.exe`, external
  manifest, PNG icon, license, and package manifest. Linux currently receives
  a standard `Qiongli.AppDir` with `AppRun`, canonical CLI, desktop entry, PNG
  icon, license, and package manifest, which the Linux job finalizes into a
  real Type 2 AppImage.
- The package command stages bounded regular files in owner-private storage,
  verifies the resulting archive before writing output, cleans staging, and
  emits exactly the target archive, canonical manifest, and public receipt.
- Native CI runs package assembly as a parallel three-platform release matrix
  and uploads seven-day artifacts named explicitly as non-publishing. Exact
  AppDir-source matrix run `29414548962` passed macOS, Windows, and Linux on
  implementation head `33d91af8`.
- Linux finalization now requires the official Type 2 `appimagetool` asset with
  a source-controlled SHA-256 pin. It reverse-extracts the AppImage and a Rust
  finalizer verifies the tool, source ZIP/manifest/receipt chain, Type 2 magic,
  exact file set, modes, and hashes before emitting a separate non-publishing
  AppImage receipt.
- The thin desktop launcher accepts only normal window activation or one fixed
  `--startup-check` package preflight. CI uses the real packaged launcher with
  an empty `PATH`; arbitrary CLI input is not forwarded through the launcher.
- Focused layout, identity, tamper, source-binding, application asset, and
  launcher tests pass. The optimized integration fixture completes in about
  0.3 seconds instead of copying full debug binaries. A local macOS release
  smoke run produced and verified the expected six-entry `.app.zip`; because
  the implementation was uncommitted during that smoke run, it is mechanism
  evidence only and not exact-head release evidence.

Exact-head Native CI run `29419027524` passed all nine jobs for branch head
`d988070c` and PR merge candidate `ef253403`, including the Type 2 AppImage and
the real packaged startup entries on macOS, Windows, and Linux with an empty
`PATH`. A downloaded copy of that exact macOS artifact also opened the
no-argument `Qiongli 2` window from an isolated home and config root; macOS
Accessibility exposed the six navigation destinations and labelled Skills
controls. This local-machine result is not a clean-machine, keyboard, scale,
or human screen-reader acceptance. Normal Windows and Linux desktop activation,
production signing/notarization, and cross-platform clean-machine acceptance
remain Batch 5 gates.

## Batch 5 — Reaccept Alpha.1

- [ ] Run packaged clean-machine startup and interactive window acceptance for
  macOS, Windows, and Linux without Rust, Python, Node, Cargo, npm, or pip.
- [ ] Run keyboard, scale, basic screen-reader, settings persistence, Skills
  lifecycle, MCP self-test, and discovery journeys in the packaged app.
  - [x] Complete the macOS packaged settings-persistence, Skills lifecycle,
    MCP self-test, source-discovery, automated Tab-order, and keyboard-activation
    journey.
  - [x] Add exact-package macOS receipt/manifest verification, isolated
    empty-`PATH` startup, and request-only LaunchServices preflight evidence.
  - [ ] Complete manual macOS scale and VoiceOver acceptance.
  - [ ] Complete the corresponding Windows and Linux interactive journeys.
- [ ] Run real Codex and Claude Code discovery plus candidate-backed install,
  diagnose, and remove on supported isolated clients.
  - [x] Complete the non-publishing, ephemeral-test-signed macOS candidate
    journey against actual Codex and Claude Code clients.
  - [ ] Regenerate the same evidence from the final accepted source and
    production-signed candidate.
- [ ] Apply external macOS signing/notarization, Windows Authenticode, and
  signed Linux release metadata only through maintainer-controlled boundaries.
  - [x] Add an exact-package macOS signing/notarization entry point, a
    credential-free ad-hoc mechanism test, and a missing-credential
    production failure probe.
  - [ ] Run the macOS production path with the maintainer's Developer ID and
    `notarytool` Keychain profile against the final accepted source package.
- [ ] Regenerate the signed candidate, desktop artifact descriptors, release
  notes, checksums, and readiness receipt from the final exact head.
- [ ] Require exact-head Native CI and every recorded publication gate before
  moving PR #63 from Draft or creating `v2.0.0-alpha.1`.

**Checkpoint:** The readiness receipt proves a usable cross-platform desktop
application, not only a CLI-mode startup preflight.

Current non-publishing Batch 5 evidence:

- Actual Codex CLI `0.144.4` passed isolated Plugin Creator validation,
  personal Marketplace registration, install, list/enablement, cache receipt
  verification, empty-`PATH` Lite MCP, client remove, absence verification,
  and receipt-owned catalog removal.
- Actual Claude Code `2.1.209` passed strict validation, skills-directory
  discovery, local Marketplace registration, install, cache receipt
  verification, empty-`PATH` Lite MCP, uninstall, Marketplace removal, and
  absence verification.
- The tested client binaries and configuration roots were explicit and
  isolated. A Python environment with PyYAML was used only by the external
  Plugin Creator development validator; no Qiongli product path used Python.
- Those earlier separately composed local results did not close the
  candidate-bound gate. Final production evidence must still be regenerated
  from the final exact source alongside the signed publication ledger.
- Implementation head `5f421543` adds one candidate-bound external-client
  path. Its macOS aarch64 run used actual Codex CLI `0.144.4` and Claude Code
  `2.1.209`, verified each client cache against the candidate-materialized
  source receipt, launched the cached 12-tool Lite MCP with an empty `PATH`,
  and completed client plus Qiongli cleanup and absence checks. The digest-only
  receipt is
  `tooling/release/acceptance/v2.0.0-alpha.1-r3n-real-clients.md`.
- That receipt remains non-publishing and ephemeral-test-signed. It closes the
  implementation-head real-client mechanism, not the final production-signed
  candidate regeneration gate.
- Exact Native CI run `29421269995` passed all nine jobs for branch head
  `1ebca2be`; its macOS artifact was downloaded by artifact id `8345427437` and
  verified against the package archive, manifest, and receipt hashes.
- The packaged macOS arm64 App completed global-settings save and restart,
  343-entry Skills materialize/verify/remove, the offline 12-tool Lite MCP
  self-test, and Codex plus Claude Code `Discovered but unmanaged` refresh with
  an empty `PATH` and isolated state.
- macOS Accessibility exposed labelled roles and states. Real Tab traversal
  reached all six navigation controls, and Space activated the focused
  Diagnostics control. This automated evidence does not replace manual scale,
  VoiceOver, contrast, or human keyboard acceptance.
- The exact non-publishing evidence is recorded in
  `tooling/release/acceptance/v2.0.0-alpha.1-r3n-macos-packaged-ui.md`.
- Implementation commits `9c95ae67` and `1ae1fcaa` add a reusable macOS
  acceptance entry point, invoke it in the exact package job, and distinguish
  a LaunchServices request from process or displayed-window observation.
  Exact Native CI run `29438832633` passed all nine jobs at branch head
  `1ae1fcaa`; macOS artifact `8352644966` includes the path-redacted automated
  receipt for package source `62edf98b`, and the same downloaded artifact
  returned `request-accepted` through the optional local LaunchServices path.
  The bound evidence is
  `tooling/release/acceptance/v2.0.0-alpha.1-r3n-macos-preflight.md`.
- This closes the repeatable macOS automated-preflight mechanism only. It does
  not close clean-machine window observation, manual scale/VoiceOver/contrast,
  production signing, or the deferred Windows/Linux interactive gates.

## Fast Validation Loop

For Batches 1-3, run focused crate/application tests first, then the normal
native workspace gate. Packaging jobs begin in Batch 4 and remain target-
specific; they do not make every UI change wait for all package assembly.
Legacy Python and Node suites remain non-blocking.

Required checkpoint evidence:

```text
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
```

Batch 4 now has target package build/inspection jobs. Batch 5 adds the slower
packaged clean-machine and real-client acceptance once, against the final
candidate rather than after every implementation commit.

## Completion Definition

R3N and R3 are complete only when the five reported failures are closed in a
packaged application, all three OS-family artifacts have clean-machine launch
evidence, source discovery is truthful, state mutation remains typed and
approved, and the final release ledger binds the exact accepted head. Until
then, R3M remains accepted technical evidence but Alpha.1 publication remains
blocked.
