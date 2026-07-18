# Qiongli R3Q Native Product Control Plane Execution Plan

Status: Batch R3Q-A is complete; Batch R3Q-B is implemented and its complete
ad-hoc macOS packaged-product journey passes locally on the rolling branch.
The new exact-head GitHub job must pass before Checkpoint B is accepted.

Date: July 17, 2026

Target branch: `2.x`

Proposed rolling branch: `feat/2x-native-control-plane`

Proposed rolling PR: one Draft PR into `2.x`; do not open R4 in parallel

**Goal:** Turn the published Alpha.1 native components into one coherent App
and Rust CLI control plane that can discover supported clients, choose supported
paths, install and manage embedded Skills and native plugin/Lite-MCP surfaces,
configure literature providers, and recover from drift without Python, Node, or
a separately installed Rust toolchain.

**Roadmap:**
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

**Accepted legacy baseline:**
`tooling/migration/qiongli-1x-baseline-plan.json`

## Release And Field-Acceptance Basis

`v2.0.0-alpha.1` is an immutable GitHub Pre-release published from
`e984f01e7330f9c0c83bb66eb8a1f17b29d0b28d`. Its macOS App installs and starts,
but field acceptance found these product gaps:

- Overview is incomplete and owns settings that belong elsewhere;
- Software Update should move to About;
- Skills requires arbitrary folder selection instead of supported presets;
- Lite MCP health is coupled to client-registration status;
- OpenAlex and Semantic Scholar credentials cannot be saved from the App;
- Codex and Claude Code discovery is incomplete or misleading;
- the public App has no activation/candidate sessions and therefore cannot
  confirm integration installation;
- an unmanaged `qiongli` marketplace entry conflicts with the native Alpha
  registration;
- Global Settings and Providers duplicate literature configuration.

R3Q closes these gaps before R4 adds Full agent execution. R3Q does not mutate
the published Alpha.1 tag or assets and does not claim the Full orchestrator is
already executable.

## Design Inputs

These projects inform behavior and information architecture only. R3Q does not
add them as runtime dependencies or copy implementation without a separate
license and security review.

- `https://github.com/vercel-labs/skills`: canonical Skills store, global and
  project scopes, agent adapters, symlink/copy selection, list/update/remove;
- `https://github.com/block/goose`: shared CLI/Desktop provider and extension
  lifecycle plus OS credential storage;
- `https://github.com/modelcontextprotocol/inspector`: separate connect,
  initialize, list-tools, call-tool, timeout, and configuration evidence;
- `https://github.com/obra/superpowers`: explicit host-specific installation
  instead of pretending every Agent uses the same path;
- `https://learn.chatgpt.com/docs/build-skills`: current Codex user/project
  Skills locations and symlink behavior;
- `https://learn.chatgpt.com/docs/build-plugins`: current Codex personal
  marketplace and plugin distribution model.

## Execution Rules

1. Keep one rolling branch and Draft PR.
2. CLI and App call the same Rust product-control service.
3. Reuse existing materialization, candidate-source, activation, transaction,
   receipt, rollback, update, and redaction code before adding abstractions.
4. Add no new crate unless an independent security or reuse boundary is proven.
5. Preserve only four non-negotiable mutation rules:
   - do not overwrite or remove unmanaged bytes;
   - preview exact changes and obtain explicit approval before mutation;
   - never persist or display raw provider secrets outside the OS credential
     service;
   - every multi-resource change has verification and reverse compensation.
6. Do not require the legacy Python or Node suites. The accepted 1.x baseline is
   read as a capability oracle, not executed as a product dependency.
7. Use focused affected-package validation per batch. Run the full native
   workspace and packaged acceptance only at cohesive merge/release checkpoints.
8. Do not create a new public tag automatically. Any corrected field-test build
   requires a new immutable prerelease identity and explicit release approval.

## Target Architecture

```text
qiongli-ui / Rust CLI
        |
        v
ProductControlService
  |- ClientInventoryService
  |- DesiredStatePlanner
  |- ProductAuthorityVerifier
  |- InstallationCoordinator
  |- HealthAndRepairService
  |- LiteratureProviderService
        |
        +-- qiongli-content materializer
        +-- qiongli-config + SecretStore trait
        +-- qiongli-runtime Lite MCP
        +-- qiongli-platform Codex/Claude adapters
        +-- existing transaction/receipt/rollback/update services
```

The App layer converts button presses into typed intents and renders typed
results. It does not discover filesystem paths, invoke clients, edit configs,
or decide whether an installation is safe.

## 1.x Outcome-Parity Ledger

Add a checked-in machine-readable ledger under `tooling/migration/` with one
record for every accepted 1.x user outcome. Each record contains:

- stable capability ID;
- 1.x source and observed user outcome;
- target surfaces;
- 2.x disposition: `retain`, `replace`, `defer-to-r4`, or
  `retire-with-reason`;
- owning R3Q/R4 batch;
- implementation and acceptance evidence;
- explicit nonclaim where applicable.

Initial classifications:

| 1.x outcome | R3Q/R4 disposition |
|---|---|
| `target=auto/all` | R3Q multi-signal inventory and multi-select targets |
| default client Skills paths | R3Q versioned adapter path catalog |
| global/project Skills | R3Q canonical store plus target materialization |
| skills/plugin/both surfaces | R3Q desired-state profiles |
| copy/link modes | R3Q adapter decision; advanced override only if needed |
| subject and coverage selection | retain through embedded content profiles |
| local plugin install/remove | R3Q shared installation coordinator |
| standalone/bundled MCP registration | R3Q Lite-MCP integration lifecycle |
| provider setup and doctor | R3Q Literature Providers and secret service |
| install discovery/check | R3Q typed inventory and Product Doctor |
| upgrade/remove | retain through updater and receipt-owned lifecycle |
| CLI wrapper installation | retire; the native binary/App is the product |
| Full orchestrator doctor/run | defer to R4 native execution services |
| external Codex/Claude/Antigravity workers | optional R4 adapters, not core dependency |

A Rust-only repository test fails when an accepted 1.x install/setup/discovery/
doctor/update/remove/orchestration outcome has no ledger disposition.

## Batch R3Q-A — Capability Ledger And Client Inventory

Purpose: establish one truthful, testable answer to “what is installed, where,
why was it selected, and what can Qiongli safely do next?”

Implementation:

- [x] Add the 1.x outcome-parity ledger and strict schema/coverage validation.
- [x] Add `qiongli-platform::client_inventory` with typed client, surface,
  scope, path candidate, discovery evidence, ownership, and action readiness.
- [x] Resolve paths from adapter-owned ordered signals:
  - explicit supported environment/config override;
  - current official user and project paths;
  - Qiongli-managed receipts and manifests;
  - existing client configuration;
  - optional client App/CLI presence used only as discovery evidence;
  - observed legacy paths reported as legacy/unmanaged, never selected for
    replacement automatically.
- [x] Support current Codex user Skills and personal marketplace locations,
  Claude Code user/project Skills locations, Qiongli managed source roots, and
  explicit custom destinations.
- [x] Keep real paths private in services; UI models expose display-safe paths
  and fixed evidence/reason codes.
- [x] Replace directory-only `discover_client_config_root` classification with
  the inventory result while retaining symlink/reparse and unsafe-path checks.
- [x] Expose the same inventory through a read-only Rust CLI command and Desktop
  snapshot.

Primary files:

- `tooling/migration/qiongli-1x-product-parity.json`;
- `tooling/migration/qiongli-1x-product-parity.schema.json`;
- `packages/qiongli-native/crates/qiongli-platform/src/client_inventory.rs`;
- `packages/qiongli-native/crates/qiongli-platform/src/lib.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`;
- `packages/qiongli-native/apps/qiongli/src/command.rs`;
- `packages/qiongli-native/crates/qiongli-ui/src/model.rs`.

Focused acceptance:

- missing client, config-only client, App/CLI-only client, managed install,
  unmanaged legacy install, drift, conflict, unsafe path, env override, user
  scope, project scope, and simultaneous Codex/Claude fixtures;
- no test executes an untrusted discovered client binary;
- identical inventory classification through CLI and Desktop snapshots.

**Checkpoint A:** App and CLI report the same supported targets, selected paths,
evidence, ownership, and next safe action. No mutation is enabled yet.

Checkpoint evidence on July 17, 2026:

- `qiongli install inventory` emits schema-versioned, display-safe JSON for
  Codex and Claude Code using the same `CommandEnvironment::client_inventory`
  service consumed by Desktop snapshots;
- Desktop Integration cards expose ownership, next safe action, fixed evidence
  code, and the supported symbolic path inventory without storing a real path
  in the presentation model;
- focused fixtures cover missing, config-only, host-presence-only, environment
  override, user, project, custom, legacy/unmanaged, managed-current, drift,
  recovery, conflict, relative unsafe path, symlink, and simultaneous-client
  outcomes; no discovered binary is executed;
- `cargo test -p qiongli-platform` passed 102 library tests plus the parity
  ledger integration test;
- `cargo test -p qiongli-ui --lib` passed 18 tests;
- `cargo test -p qiongli --lib` passed 55 tests and
  `cargo test -p qiongli --test cli` passed 18 tests;
- affected-package all-target Clippy passed with warnings denied for
  `qiongli-platform`, `qiongli-ui`, and `qiongli`;
- this checkpoint remains strictly read-only: it creates no client config,
  Skills, marketplace, plugin-source, or receipt path.

## Batch R3Q-B — Packaged Product Authority And Desired State

Purpose: let the installed App safely use the native installation services that
Alpha.1 currently exposes only to candidate/test sessions.

Implementation:

- [x] Add a packaged-product verifier that binds the running executable/App,
  desktop manifest, release authority, embedded pack, version, target, and
  managed product root.
- [x] Derive one bounded in-memory local installation capability after startup
  verification; do not persist a bearer token or private signing material.
- [x] Remove the ordinary App's empty-session dead end. Candidate sessions stay
  available for candidate acceptance but are no longer the only product path.
- [x] Add a desired-state model for profile, target clients, Skills scope,
  plugin identity, Lite MCP, and activation expectation.
- [x] Compose existing content materialization, plugin source, registration,
  receipt, and rollback operations into one preview/confirm transaction.
- [x] Use `qiongli-next` for Alpha namespaced registrations where supported.
- [x] Classify an existing unmanaged `qiongli` entry as coexist/replace-required;
  never adopt or overwrite it implicitly.
- [x] Add verify, repair, remove, and reverse-compensation paths using existing
  receipt formats unless a genuinely new product-level receipt is required.

Primary files:

- `packages/qiongli-native/crates/qiongli-platform/src/product_control.rs`;
- existing `candidate_source.rs`, `candidate_install.rs`, `activation.rs`,
  `transaction.rs`, `codex.rs`, and `claude.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`;
- `packages/qiongli-native/apps/qiongli/src/application.rs`.

Focused acceptance:

- verified packaged App gains bounded authority;
- tampered/wrong-target/source-build App remains read-only;
- install, already-current, verify, drift repair, remove, mid-transaction
  failure, compensation, restart recovery, and unrelated-byte preservation;
- existing `qiongli` plus new `qiongli-next` coexistence;
- healthy supported target no longer returns
  `production-activation-session-unavailable`.

**Checkpoint B:** A normal packaged App can preview and apply a receipt-owned
Codex or Claude Code Lite installation without an externally injected session.

Implementation evidence on July 17, 2026:

- `qiongli-platform::product_control` verifies canonical desktop-manifest and
  product-control evidence against the running executable, embedded authority,
  embedded pack, source commit, target, fixed home, and managed product root;
- verification derives only target-bound Codex and Claude Code capabilities in
  memory. Product-control files contain signed public grants, never bearer
  tokens, launch private keys, or provider secrets;
- ordinary Desktop integration preview/apply now prefers this verified product
  path after candidate and acceptance sessions; source builds remain explicitly
  `source-build-read-only` and no longer report the removed empty-session code;
- Codex and Claude Code native adapters install the Alpha identity as
  `qiongli-next`. Observed legacy `qiongli` sources remain inspect-only and are
  preserved across install and remove;
- the coordinator previews, re-verifies, installs, verifies, repairs, removes,
  and compensates registration failure using existing source and activation
  receipts; changed client state after preview fails before product writes;
- desktop packaging can bind `.qiongli-product-control.json` into every exact
  target layout, and `native_product_control` emits a non-publishing external
  signing request then verifies two returned Ed25519 signatures before creating
  the canonical control and updated manifest;
- `cargo test -p qiongli-platform` passed 107 library tests plus the parity
  ledger test; `cargo test -p qiongli-ui --lib` passed 18 tests;
  `cargo test -p qiongli --lib` passed 56 tests; the focused CLI, activation,
  release-candidate, Codex bundle, and Claude bundle tests passed; affected
  all-target Clippy passed with warnings denied.

Packaged acceptance evidence on July 18, 2026:

- the macOS signing boundary now has an explicit
  `--preserve-signed-canonical` mode. A package carrying product control is
  rejected without it; the mode verifies the existing canonical signature,
  manifest/control digest, and control-to-canonical hash before signing the
  remaining App and DMG, then proves the canonical bytes did not change;
- `native_packaged_product_acceptance` generates test release/launch keys only
  in zeroizing memory, builds an authority-embedded product, ad-hoc signs the
  canonical runtime, exercises the public prepare/external-sign/finalize tools,
  composes the product-controlled package, and invokes the real App/DMG signing
  boundary;
- the isolated accepted App reported embedded authority and source commit,
  started through its launcher with an empty `PATH`, verified product control,
  and completed install, verify, already-current, and remove for both Codex and
  Claude Code while preserving legacy `qiongli` canaries;
- the canonical acceptance receipt recorded
  `accepted-ad-hoc-nonpublishing`, `publication_allowed: false`, and seven
  passing fixed checks. This is not Developer ID, notarization, human UI, or
  publication evidence;
- Native CI now contains a dedicated macOS job that repeats this journey on the
  exact commit and uploads only public non-publishing control/manifests and
  receipts. That job has not yet run for this local commit.

Checkpoint B remains pending only until the exact-head GitHub job succeeds.
Production release remains a separate external-key gate: sign the canonical
runtime with Developer ID first, externally sign both exact launch-grant
preimages, finalize/recompose product control, preserve the canonical signature
while signing/notarizing the App and DMG, then complete the R3Q-E human gates.

## Batch R3Q-C — Skills Manager And Integration Actions

Purpose: replace arbitrary path selection and status-only integration cards
with supported presets and outcome-oriented actions.

Implementation:

- [x] Add one canonical Qiongli Skills source managed from the embedded pack.
- [x] Add presets: Qiongli Managed, detected Codex, detected Claude Code,
  current project, and Custom Folder.
- [x] Let each adapter choose symlink or receipt-owned copy based on host and
  target support; display the selected method in preview.
- [x] Add `Install recommended`, `Install selected`, `Verify`, `Repair all`,
  `Update`, and `Remove` typed intents.
- [x] Split Integration cards into Client, Source, Skills, Registration,
  Activation, MCP attachment, and Overall sections.
- [x] Give every non-ready state one primary recovery action and optional
  inspection details.
- [x] Never label an already detected client as “not discovered” solely because
  registration is missing or conflicting.

Primary files:

- `packages/qiongli-native/crates/qiongli-content/` existing materializer;
- `packages/qiongli-native/crates/qiongli-platform/src/product_control.rs`;
- `packages/qiongli-native/crates/qiongli-ui/src/model.rs`;
- `packages/qiongli-native/crates/qiongli-ui/src/app.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop_contract.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`.

Focused acceptance:

- default install needs no folder picker;
- custom destination remains available and passes existing approval checks;
- simultaneous Codex/Claude install has one preview and all-or-compensate
  behavior;
- list/update/remove operates only on receipt-owned content;
- keyboard and AccessKit labels cover every action and recovery state.

**Checkpoint C:** A user can install Qiongli Lite for detected clients through
buttons, see exactly where it went, verify it, repair it, and remove it.

Checkpoint C evidence on July 18, 2026:

- Qiongli Managed is the no-picker default and materializes the embedded
  `skill-only` profile into a receipt-owned user location; current-project and
  Custom Folder remain explicit alternatives, while the detected-client
  presets route through the verified Codex/Claude plugin projection;
- the Alpha adapters select receipt-owned copies rather than symlinks so each
  installed plugin remains self-contained and dependency-free; the selected
  method and symbolic destination are visible before approval;
- packaged Codex and Claude Code installation now share one product-bound
  preview. The platform validates both targets before writing and compensates
  successful first-client writes if a later client apply fails;
- Install recommended, Install selected, Verify selected, Repair all, Update
  selected, and Remove selected are typed desktop intents. Update refuses
  unmanaged replacement, and removal begins with exact receipt verification;
- Integration renders Client, Source, Skills, Registration, Activation, MCP
  attachment, and Overall independently, retains supported-path evidence, and
  gives each card a primary action derived from readiness;
- focused evidence passed: six product-control tests including the two-client
  batch lifecycle, 19 UI/AccessKit tests, 57 application library tests, and
  affected all-target Clippy with warnings denied.

## Batch R3Q-D — Settings, Literature Providers, MCP, And About

Purpose: give every configuration concept one owner and make health output
causal rather than contradictory.

Implementation:

- [x] Make Overview a read-only product dashboard with recommended next action.
- [x] Add About with product/version/build/target/trust/update-channel details
  and move Software Update there.
- [x] Rename Providers to Literature Providers.
- [x] Move all provider enablement, OpenAlex/Crossref public fields, secret
  references, and offline credential-readiness tests out of Global Settings.
- [x] Add OS secret-store implementations behind the existing `SecretStore`
  trait, starting with macOS Keychain for packaged macOS acceptance.
- [x] Add masked save/replace/remove controls for OpenAlex and Semantic Scholar
  API keys; config stores only secret references.
- [x] Separate Lite MCP protocol health from client attachment/registration.
- [x] Test initialize, exact tools list, representative offline call, provider
  readiness, cancellation, and timeout as separate checks.
- [x] Keep Stable/Beta selection and unified product update behavior in About.

Primary files:

- `packages/qiongli-native/apps/qiongli/src/credential_store.rs`;
- `packages/qiongli-native/crates/qiongli-config/src/secret.rs`;
- `packages/qiongli-native/crates/qiongli-ui/src/model.rs`;
- `packages/qiongli-native/crates/qiongli-ui/src/app.rs`;
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`;
- affected config/runtime provider tests.

Focused acceptance:

- Global Settings and Literature Providers cannot issue conflicting writes;
- raw secrets are absent from config, UI debug output, logs, errors, receipts,
  diagnostics, and snapshots;
- credential replacement/removal survives restart and updates readiness;
- MCP reports Ready with registration Missing/Conflict as a separate advisory;
- About owns all update controls and state.

**Checkpoint D:** Settings have one owner, provider credentials work securely,
and MCP health no longer contradicts integration status.

Checkpoint D evidence on July 18, 2026:

- Overview is read-only and recommends the next product action; About owns the
  product identity, build, target, trust, Stable/Beta selection, and unified
  Software Update controls;
- Global Settings writes only the default profile. Literature Providers owns
  provider enablement, public contact fields, masked OpenAlex/Semantic Scholar
  credential lifecycle, and an explicit offline readiness test;
- packaged macOS sessions use Keychain through the `SecretStore` contract.
  Configuration persists opaque references only, while save, replacement,
  removal, restart, and revision-conflict compensation are covered by tests;
- Lite MCP renders its five bounded protocol checks independently from client
  attachment and registration advisories. Initialize, exact tool registry,
  representative offline dispatch, cancellation, and timeout remain distinct;
- focused evidence passed: 36 config tests, 47 runtime/MCP tests, 20 UI and
  AccessKit tests, 58 application library tests, 18 CLI tests, format, and
  affected all-target Clippy with warnings denied.

## Batch R3Q-E — Product Parity And Packaged Acceptance

Purpose: prove the App behaves as an installed product rather than a collection
of independently tested native components.

Implementation and evidence:

- [ ] Run the Rust-only parity-ledger coverage test.
- [ ] Run focused path, permission, receipt, rollback, secret, MCP, UI, and
  updater tests.
- [ ] Run format, full workspace check, full workspace Clippy, and full native
  workspace tests once on the exact proposed merge head.
- [ ] Build the exact macOS arm64 Community Alpha-class package from a clean
  source revision.
- [ ] On an isolated macOS user/config root, exercise startup, Inventory,
  Install recommended, Skills verify, Lite MCP self-test, provider save/remove,
  Codex registration, Claude Code registration, repair, removal, restart, and
  update-content preservation.
- [ ] Perform a human macOS pass for path labels, conflict recovery, keyboard
  traversal, scale, contrast, and VoiceOver basics.
- [ ] Update the roadmap, package README, user guide, release nonclaims, and PR
  ledger with only exact observed results.
- [ ] Decide separately whether to publish a corrected field-test prerelease or
  merge R3Q into the later R4 Alpha.2 line. Never replace Alpha.1 assets.

**Checkpoint E:** R3Q exit gates pass on the exact packaged artifact and the
rolling PR can become Ready.

## Immediate Next Coding Checkpoint

R3Q-D implementation and focused acceptance are complete on
`feat/2x-native-control-plane`. The next batch is R3Q-E: run the parity and
exact-head gates, build the exact macOS arm64 package, exercise its isolated
product journey, then record human-only acceptance separately. Do not promote
component-test evidence into packaged-product evidence.

## R3Q Exit Criteria

R3Q is complete only when:

- App and CLI share one native inventory and lifecycle service;
- supported default Skills paths require no manual folder browsing;
- the packaged App can obtain bounded installation authority from verified
  product evidence;
- one approved action installs Skills, native plugin source, Lite MCP, and
  registration for selected supported clients;
- existing unmanaged content is preserved and has a clear coexist/replace
  path;
- every installed resource is verifiable, repairable, removable, and covered
  by rollback/recovery evidence;
- Literature Providers securely manages supported credentials;
- MCP, client registration, activation, provider, and Full-runtime states are
  independent and actionable;
- the 1.x parity ledger contains no unclassified user outcome;
- the exact packaged macOS acceptance journey passes without Python, Node,
  Cargo, npm, pip, or a separately installed Rust toolchain;
- release notes still state that Full orchestration belongs to R4.
