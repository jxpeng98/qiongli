# Qiongli R3N Alpha.1 Desktop Application Closure Execution Plan

Status: Batch 1 complete; Batch 2 ready

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

- [ ] Add a bounded asynchronous Lite MCP self-test service using the canonical
  registry and dispatcher.
- [ ] Report initialize, exact tools list, offline dispatch, redacted provider
  readiness, and discovered client-registration state.
- [ ] Add timeout/cancel handling and keep network/mutation out of the default
  test.
- [ ] Decouple Codex and Claude Code discovery from signed-candidate apply
  authority in source and packaged sessions.
- [ ] Distinguish not discovered, discovered unmanaged, managed, drifted, and
  candidate-required states with fixed remediation.
- [ ] Test isolated adapters and record real-client evidence when supported
  clients are available.

**Checkpoint:** The MCP button produces actionable evidence, and the
Integrations page discovers local Codex and Claude Code even in an ordinary
source session.

## Batch 3 — Add The Desktop Application Entry

- [ ] Extract one reusable native app composition entry from the current
  `qiongli ui` command path.
- [ ] Make desktop/no-argument activation open the UI while preserving every
  explicit CLI command and machine-readable output contract.
- [ ] Add only the minimum platform launcher boundary needed to avoid a
  persistent console window on Windows; keep all product logic in shared Rust
  services.
- [ ] Add product name, version, icon, application identifier, license, and
  startup-error metadata.
- [ ] Test CLI/UI dispatch, repeated launches, invalid embedded content, missing
  config home, renderer failure, and no-language-runtime startup.

**Checkpoint:** A development application bundle can be opened without typing
`qiongli ui`, and CLI commands behave exactly as before.

## Batch 4 — Package The Three Desktop Targets

- [ ] Produce a macOS `.app` and distributable archive/DMG with Finder launch.
- [ ] Produce a Windows desktop GUI executable and portable package with
  Explorer launch and no persistent console window.
- [ ] Produce a Linux AppImage with icon and desktop metadata.
- [ ] Bind each desktop artifact to the R3M product, version, source,
  executable, and embedded-pack identity without weakening the three-file
  portable candidate contract.
- [ ] Add CI jobs which assemble and structurally inspect all three packages;
  upload only explicitly non-publishing artifacts until signing gates pass.
- [ ] Document target-specific installation, CLI access, removal, trust
  prompts, and unsupported architectures truthfully.

**Checkpoint:** CI produces one desktop-activatable artifact for macOS,
Windows, and Linux from the same exact source and embedded pack.

## Batch 5 — Reaccept Alpha.1

- [ ] Run packaged clean-machine startup and interactive window acceptance for
  macOS, Windows, and Linux without Rust, Python, Node, Cargo, npm, or pip.
- [ ] Run keyboard, scale, basic screen-reader, settings persistence, Skills
  lifecycle, MCP self-test, and discovery journeys in the packaged app.
- [ ] Run real Codex and Claude Code discovery plus candidate-backed install,
  diagnose, and remove on supported isolated clients.
- [ ] Apply external macOS signing/notarization, Windows Authenticode, and
  signed Linux release metadata only through maintainer-controlled boundaries.
- [ ] Regenerate the signed candidate, desktop artifact descriptors, release
  notes, checksums, and readiness receipt from the final exact head.
- [ ] Require exact-head Native CI and every recorded publication gate before
  moving PR #63 from Draft or creating `v2.0.0-alpha.1`.

**Checkpoint:** The readiness receipt proves a usable cross-platform desktop
application, not only a CLI-mode startup preflight.

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

Batch 4 adds target package build/inspection jobs. Batch 5 adds the slower
packaged clean-machine and real-client acceptance once, against the final
candidate rather than after every implementation commit.

## Completion Definition

R3N and R3 are complete only when the five reported failures are closed in a
packaged application, all three OS-family artifacts have clean-machine launch
evidence, source discovery is truthful, state mutation remains typed and
approved, and the final release ledger binds the exact accepted head. Until
then, R3M remains accepted technical evidence but Alpha.1 publication remains
blocked.
