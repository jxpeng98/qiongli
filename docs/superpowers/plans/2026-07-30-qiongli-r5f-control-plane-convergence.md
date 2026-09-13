# Qiongli R5F Control-Plane Convergence

Status: in progress; F0, F1, F2, the automated portion of F3, and the F4
automated harness and manual evidence recorder are implemented locally. F3
exact-window/manual qualification and a clean-worktree F4 packaged execution
remain open.

## Purpose

R5F closes the remaining product-surface gap between the Svelte App, the Rust
CLI, client plugins, and receipt-owned standalone Skills. It does not add a
second installer and does not make the UI or CLI resolve client paths.

The control model is:

- Client Integration cards own Codex and Claude Code plugin lifecycle.
- Advanced standalone content owns only receipt-managed Skills destinations.
- The Rust `DesktopService` owns discovery, verification, planning, and state.
- App and CLI return the same versioned snapshot and App-event vocabulary.
- Unmanaged or legacy content is observable but never adopted, overwritten, or
  removed implicitly.

## F0 — Ownership And Navigation

- Keep one Client Integrations navigation entry.
- Redirect the retired Workflow Content route to the advanced section.
- Link Overview directly to the merged section.
- Remove Codex and Claude Code plugin destinations from the standalone Skills
  selector so plugin installation has one owner.
- Keep the canonical embedded pack separate from every receipt-owned
  projection.

Local status: implemented. A focused component contract proves that the
standalone panel exposes only `qiongli-managed`, `current-project`, and an
explicit opaque `custom-folder` workflow. In the App, `current-project` is an
explicit registered-project selector rather than process-CWD inference, while
client plugins remain in the
Integration control. The retired Workflow Content and Model Backend routes
redirect during route loading, preserve diagnostic query parameters, and land
on their canonical Client Integrations sections without rendering a transient
second management surface. Host execution guidance and the only remaining
legacy credential cleanup action now live in Client Integrations, so a
bookmark cannot reopen a second execution model.

## F1 — Target-Scoped Managed Skills

- Extend the App contract with target-scoped verify, update, removal, and
  preserve-and-detach intents using the anonymous
  `skills-target-<sha256>` identity.
- Resolve the target identity to an exact registered receipt only inside Rust.
- Reject missing, ambiguous, unsafe, symlink-swapped, or unmanaged targets
  before mutation preview. Drifted targets reject update/removal and expose
  only a digest-bound registry detach that preserves every target byte.
- Add an opaque folder-selection event for a new custom destination; absolute
  paths must not enter ordinary App snapshots, logs, or frontend state.
- Make every displayed custom receipt-owned destination individually
  manageable.

F1 is required before Qiongli may claim that all installed standalone Skills
are manageable from the App.

Local status: implemented. App API schema 14 adds the opaque folder-selection
event, a path-free registered-project materialization intent, and target-scoped
verify/update/removal/detach intents. Rust resolves an exact
private registry entry from `skills-target-<sha256>`, re-approves the target,
re-verifies its receipt, rejects invalid, foreign, ambiguous, drifted, or
missing targets, and never returns the absolute path to the WebView. A
cross-restart lifecycle test proves that a registered custom target remains
verifiable and removable without retaining the folder picker result. The
merged panel exposes distinct accessible actions for every registered target,
including an anonymous eight-character suffix for multiple custom folders.
An installed destination now locks its receipt profile in the App:
installation is available only for a missing destination, update preserves the
installed profile, and read-only verification can explicitly confirm a drift.
Drifted content cannot be overwritten or deleted. Its only mutation is
“Preserve and detach”, which rechecks the anonymous target, profile, drift
state, and registry receipt digest before removing only Qiongli's private
ownership entry. The destination directory and every changed file remain
unchanged and become user-managed. A known preset path that still contains
those retained bytes is reported as `unmanaged`/`conflict`, not `missing`, so
the App cannot offer an installation that Rust would later reject. Mutation
controls follow the standalone Skills capability while receipt verification
remains read-only.

Schema 14 also removes `detected-codex` and `detected-claude-code` from the
standalone Skills preset contract. Host-owned Skills can therefore be managed
only with their Client Integration; neither a stale bookmark nor a handwritten
App intent can reopen the duplicate lifecycle.

Desktop no longer treats its process working directory as an implicit
`current-project` target. A packaged macOS App launched from Finder derives
project Skills targets only from Research Library registrations. The WebView
sends a `projectId`, Rust resolves the registered root, and App snapshots carry
only that project ID, the symbolic project target, and the opaque target ID.
CLI commands retain their intentional current-directory semantics. When the
CLI installs Skills from a registered project directory, Desktop links the
same receipt and target ID back to that registered project after restart
instead of projecting it as an anonymous custom target. Target-scoped
update/removal/detach previews use that same registered-project mapping, so
their confirmation target remains `<project>/.qiongli-skills` after restart
without making the private project root part of App state.

Registered-project materialization is also bound at the native transaction
boundary. Preview requires an active, ready Research Library registration and
records the exact library revision, project semantic revision, project ID, and
anonymous target ID. Confirmation revalidates all four values while holding
the project state lock; archived, unready, removed, moved, or stale project
destinations fail closed without writing Skills bytes. This is enforced by
Rust even when a stale or forged WebView intent bypasses disabled controls.

Folder selection, mutation preview, and confirmation keep the same
path-opaque boundary. The WebView receives only
`<user-home>/.qiongli-skills`, `<project>/.qiongli-skills`, or
`<custom-folder>` while Rust retains the approved path and target digest. The
App API rejects a managed Skills, CLI, or Zotero handoff preview that
substitutes an absolute path, so the confirmation window remains useful
without turning frontend state into a path authority.

## F2 — CLI And GUI Mutation Parity

- Keep `qiongli app snapshot`, `app verify-integrations`, and
  `app verify-skills` on the same `DesktopService` and versioned App-event
  contract used by the GUI.
- Define a serializable `ManagedOperationPlanV1` for cross-process CLI preview
  and apply. It binds product identity, selected target identities, expected
  receipts, preconditions, expiry, requested effects, approvals, and a
  deterministic plan digest.
- Require the apply command to recompute and match the reviewed plan digest.
- Persist neither GUI operation tokens nor bearer installation capability.
- Keep candidate/native release engineering commands separate from normal
  user-facing plugin and Skills lifecycle commands.

Local status: implemented. `ManagedOperationPlanV1` is a strict canonical JSON
artifact with a separate schema version, ten-minute expiry, product/content
identity, anonymous target identities, exact receipt or packaged-product
preconditions, requested effects, ordered approvals, a semantic digest, and a
self-excluding deterministic plan digest. It never contains a standalone
Skills path or a GUI operation token.

The normal user-facing CLI now provides:

- `qiongli app plan cli-install`;
- `qiongli app plan skills-reconcile --preset ... --profile ...`;
- `qiongli app plan skills-update|skills-remove|skills-detach --target-id ...`;
- `qiongli app plan integrations-install|integrations-reconcile|integrations-remove`
  `--target ...`;
- `qiongli app apply --plan ... --expected-plan-digest ...` with explicit
  operation approvals;
- `qiongli app verify-skills --target-id ...` on the same App-event path used
  by the GUI.

The R1 `qiongli content materialize --target ...` syntax is parser-retained
only for a stable migration error. It now returns
`managed-skills-plan-required` before config resolution or target inspection
and performs no write. Preset Skills use the reviewed App plan/apply contract;
a new custom destination is selected by the Desktop native folder picker so
its path never enters CLI output or a portable plan. Once registered, the
anonymous target remains verifiable, updatable, removable, and safely
detachable after drift from both CLI and GUI.

The root CLI help no longer advertises candidate/native payload release
engineering or candidate UI commands beside normal product workflows. Those
compatibility commands remain available under the explicitly labelled
`qiongli install --help` release-engineering section, which directs ordinary
CLI, Plugin, and standalone Skills lifecycle back to reviewed
`qiongli app plan/apply` operations.

Apply reloads a bounded absolute plan file, rejects noncanonical or unknown
fields, rechecks time, product identity, approvals, target identity, current
receipt or packaged installation evidence, and recomputes the reviewed
semantic plan before any write. Skills mutation uses the same materialize /
register / compensate and remove / unregister / restore transaction helpers as
the GUI. Plugin reconcile and removal reuse packaged-product preview, apply,
verification, and receipt-owned removal authority; a source process cannot
mint those plans.

GUI snapshots, CLI snapshots, and read-only verification invoke the same
`NativeDesktopService`. The GUI and CLI also call one shared
receipt-observation function for current, update-available, and drifted Skills
classification rather than maintaining parallel evidence checks.
The cross-process Skills lifecycle test now reads the real App snapshot after
CLI apply and removal, proving that the GUI observes the same anonymous target
as `current` and then `missing`; neither snapshot contains its absolute path.

Plugin lifecycle projection now exposes stable `nextAction` and
`ownershipState` enums instead of asking the WebView to interpret English
labels. Batch install, reconciliation, and receipt-owned removal controls are
enabled only for compatible selected lifecycle states. Update and repair share
one selection-bound reconciliation action: Rust rejects empty selections,
install-ready or conflicting targets, and any request that would expand beyond
the explicitly selected clients. The retired global repair and duplicate
update intents are rejected by both the App API and Rust parser. Conflict or
unavailable targets are removed from the default batch selection and retain a
causal inspect/refresh next step instead of exposing every generic mutation. A
source build's Verify action refreshes inventory and host observations without
claiming packaged byte verification or mutation authority.
CLI installation and reconciliation are also separate mode-bound plans:
`integrations-install` requires at least one selected missing target, while
`integrations-reconcile` requires at least one selected receipt-owned target
that needs repair and rejects missing targets. Native Desktop intent handling
rechecks the same lifecycle preconditions, so a handwritten install intent
cannot bypass disabled GUI controls.
Known client versions below the supported floor now project one causal
`upgrade-client` next action instead of retaining a misleading install/current/
repair action. Install and reconciliation reject that state in the WebView,
native Desktop intent handler, and cross-process CLI plan/apply preconditions.
An unsupported target with no managed installation is not selectable. An
existing receipt-owned installation remains selectable only for read-only
verification and receipt-owned removal, so the user can upgrade the client or
cleanly unwind Qiongli without entering a dead end. Native snapshot validation
and the App API reject contradictory compatibility, connection, registration,
ownership, and next-action combinations.
The App snapshot contract also requires exactly one canonically ordered Codex
and Claude Code identity and matching trust/mutation authority. Client
Integrations resolves each card by its explicit target rather than array
position, preventing a malformed or reordered snapshot from binding a
selection to the wrong host.
While a native Integration or standalone Skills request is in progress, the
App locks the selected clients, destination, and profile in addition to
disabling mutation buttons. The busy region remains announced, preventing the
visible operation scope from diverging from the request already sent to Rust.

The packaged App can now express CLI installation through the same plan/apply
contract. The installed `~/.local/bin/qiongli` receives a private schema-2
install receipt bound to the exact source App canonical executable, desktop
manifest, and product-control digest. The detached CLI revalidates that signed
App authority before plugin planning. A legacy receipt is offered a
no-binary-change authority upgrade rather than being reported as fully current.

Every confirmable Integration preview now carries the exact symbolic
Qiongli-managed source and client registry destination. The shared native
preview validator and App API schema reject a confirmable install, update,
repair, or removal preview without this target evidence, so the execution
dialog can identify where the operation will occur without exposing a private
absolute path.

The product orchestration boundary is host-only. App API schema 14 and the Rust
App intent parser reject the retired direct-model test and continue intents.
Persisted legacy direct-model checkpoints remain inspectable and cancellable
for cleanup, but cannot advertise pause, resume, recover, or continue. Current
host-driven checkpoints retain only state transitions supported by their
causal status. The CLI exposes legacy backend state only for read-only
diagnosis and cleanup; it cannot enable, test, or select that backend.
The underlying native Desktop service is also fail-closed: direct-backend
enablement, credential replacement, connection testing, and project queries
return `host-driven-execution-required`. The historical adapter is reachable
only behind a test-only experiment switch, while credential removal remains a
normal explicit migration-cleanup action.

## F3 — Product-Surface Qualification

- Verify compact and large layouts at exact `375`, `768`, `1024`, and `1440`
  widths without wrapped status capsules.
- Verify notification banners do not alter layout, can be dismissed, expire
  after bounded intervals even when the pointer rests over them, and pause only
  while keyboard focus remains inside for deliberate inspection.
- Verify confirmation dialogs show the native destination and execution phase
  while keeping the initiating page stable.
- Verify CLI PATH diagnostics against login-shell and GUI-process environments
  without rewriting shell profiles.
- Complete the remaining R5E focus order, contrast, reduced-motion, restart,
  representative fixture, and authoritative source-state gates.

Automated status: the notification is a fixed, non-layout-shifting top-right
banner with explicit dismissal and 5/8/12-second bounded lifetimes. Keyboard
focus pauses its timer; pointer hover does not. The confirmation boundary owns
Escape, backdrop dismissal, busy-state locking, focus trapping, body-scroll
locking, and exact focus restoration without relying on the previous
third-party dismiss-layer teardown. Semantic z-index tokens keep the blocking
confirmation boundary above transient banners. Status capsules and secondary
tag capsules are single-line and ellipsize rather than wrapping; the shared
status capsule also participates in flex shrinking, so a long localized state
cannot force a narrow row beyond the viewport. Codex and Claude Code tabs
implement Arrow, Home, and End navigation with focus following the active tab.
Desktop
surfaces use a tested 10/11px minimum compact typography scale instead of
shrinking dense copy to 7–9px. Long update reason codes and Zotero artifact
evidence are available through explicit disclosure controls rather than
occupying the default dashboard. Orchestrator cancellation uses an in-card,
reversible confirmation disclosure instead of a blocking browser dialog.
Retired Model Backend, duplicate host-content, and direct-run translation keys
plus the unused Workflow Content feature descriptor have been removed so
dormant frontend metadata cannot re-advertise the old product model.
Academic Graph recovery states now include a direct Research Library action.
If the interactive Cytoscape renderer cannot mount, its deterministic fallback
keeps relations keyboard-selectable and synchronized with the exact evidence
inspector rather than degrading to node-only inspection. The adapter keys
element replacement to complete rendered element data, including labels,
risk, confidence, relation, and topology metadata, while reusing the same
immutable layout identity for O(1) selection/focus-only updates.
CLI login-shell testing emits and accepts one fixed command marker, so startup
messages containing path-like text cannot be mistaken for `command -v`
evidence. A CLI install preview or explicit CLI refresh clears its earlier test
result before presenting new evidence.
Read-only CLI, Integration, and Zotero snapshot probes preserve an approved
process-local custom Skills selection; only the true App reconnect boundary
clears it, matching the native service lifetime and preventing a stale target
after restart.
Registered-project Skills discovery resolves all usable project roots from one
validated Research Library read rather than reloading the complete library for
each project. The App snapshot remains O(N) across the bounded 512-project
library. The managed-details disclosure renders installed, exceptional, and
currently selected project targets instead of eagerly creating a row for every
missing project; the project selector still exposes the complete eligible
set.

The automated baseline currently passes:

- 39 Desktop files / 185 tests, with no Svelte runtime warnings;
- 32 App API contract tests;
- 160 `qiongli` library tests;
- 31 native CLI integration tests;
- 4 fail-closed R5F manual-receipt contract tests;
- 162 `qiongli-project` tests;
- 32 native UI tests;
- the packaged acceptance example fixture;
- zero Svelte diagnostics, Rust formatting, shell syntax, diff checks, and the
  static production build.

Manual packaged qualification matrix:

1. At `375`, `768`, `1024`, and `1440` window widths, visit Overview, Client
   Integrations, Advanced standalone content, About, Research Library,
   Captures, and Academic Graph. Record any horizontal page overflow, wrapped
   capsule, clipped primary action, or unreachable control. At compact widths,
   the primary navigation must remain one horizontal scroll row, automatically
   reveal the active route, and keep language, runtime state, and Refresh in
   one compact toolbar. Page-header descriptions must remain accessible while
   their visual copy is bounded, and Research Library's low-frequency import
   and migration controls must close on selection, outside click, or Escape.
2. Trigger success, warning, and failure notices. Confirm the page does not
   move; each banner is in the top-right safe area, closes immediately with its
   button, expires without pointer interaction, and does not survive a later
   notice.
3. Open an install/remove confirmation from the invoking button. Confirm Cancel
   receives initial focus, Tab/Shift+Tab remain inside, Escape/backdrop close
   only while idle, the busy state cannot be dismissed, and focus returns to
   the exact invoking control.
4. In About, verify a current 2.x CLI, an old 1.x CLI, a shadowing mise shim,
   and a GUI process without `~/.local/bin` in `PATH`. `Test in new shell` must
   distinguish active, missing, shadowed, and version-mismatch states without
   editing `.zshrc`.
5. In Client Integrations, confirm source-read-only builds retain Verify but
   disable install/update/repair/remove. In the accepted App, exercise Codex
   and Claude Code install, verify, update/repair, host-action guidance,
   restart, and receipt-owned removal while checking unmanaged canaries remain.
   With a below-floor client fixture, confirm the only next step is Upgrade
   client, install/repair remain unavailable, an uninstalled target cannot be
   selected, and an existing Qiongli-managed target can still be verified or
   removed.
6. For standalone Skills, exercise Qiongli managed, an explicitly selected
   registered project, and a custom folder. Confirm an installed profile is
   locked and every custom
   target has its own anonymous verify/update/remove actions. Introduce a
   controlled drift, confirm verification reports that causal state, and use
   “Preserve and detach”. The target bytes and user canary must remain
   unchanged, the destination must disappear from the private managed registry,
   and restart must not recover the detached ownership or retain a folder path
   or process-local preview. A Finder-launched App must not invent a
   current-project destination from its process working directory. A
   receipt-owned project target installed by the CLI from a registered project
   must reappear under that exact project in Desktop with the same opaque
   target ID. Its update, removal, and preserve-and-detach confirmations must
   continue to display `<project>/.qiongli-skills`, never `<custom-folder>` or
   an absolute path, after restart. Archive, make unready, and revise a
   selected project after its preview; confirmation must fail causally and
   must not write the target.
7. On Academic Graph, validate representative empty, sparse, connected, risk,
   revision-comparison, path-finding, portfolio, and bounded-large fixtures.
   Confirm keyboard selection, minimap, reduced motion, source-artifact
   opening, and restart rebuild from authoritative project state. Empty and
   unrecognized states must open their source evidence and expose Research
   Library/rebuild actions. A failed bounded filter query must preserve the
   verified projection and offer a local retry rather than replacing the whole
   workspace with a load failure. Force the interactive renderer fallback and
   confirm both nodes and relations remain keyboard-selectable and open the
   exact evidence inspector.

The fail-closed recorder lists these observations with:

```bash
pnpm acceptance:r5f:manual-record -- --list-gates
```

It accepts a complete matrix only when the packaged-product receipt, the R5D
Zotero automated receipt, and the completed R5D Zotero manual receipt all name
the same source commit and canonical executable. The resulting receipt binds
all input hashes and the versioned manual-gate contract; it refuses missing,
duplicate, unknown, stale-product, or already-recorded evidence and remains
explicitly non-publishing.

Next local batch: commit the intended source, generate the clean signed
non-publishing package, and execute this matrix against its isolated manual
home.

## F4 — Packaged Acceptance

Use an isolated home and a signed, non-publishing macOS package to prove:

1. Qiongli CLI 1.x is replaced by the packaged 2.x CLI and the GUI process
   observes the same executable after refresh.
2. Codex and Claude Code plugin install, verify, update, repair, activation
   guidance, restart, and receipt-owned removal preserve unmanaged content.
3. Qiongli-managed, registered-project, and custom standalone Skills complete
   install, verify, update, drift detection, byte-preserving detach, and
   receipt-owned removal where the receipt is still current.
4. App and CLI snapshots report the same target identities and causal states.
5. Restart preserves receipts and does not preserve process-local previews.
6. Academic Graph and Zotero manual gates remain bound to their exact packaged
   artifacts and are not inferred from source-build tests.

Local harness status: implemented. The packaged acceptance example now creates
a separate isolated control-plane home, installs the CLI through
`ManagedOperationPlanV1`, validates the schema-2 App authority, and then uses
only the installed CLI to plan/apply Codex and Claude Code reconciliation and
receipt-owned removal. It installs Qiongli-managed, current-project, and custom
standalone Skills, verifies their opaque target identities, updates a valid
older-pack receipt, detects and recovers a controlled owned-file drift, removes
all three destinations, and proves unmanaged plugin canaries survive. Plan and
result output is checked for isolated-home path leakage.

The example compiles, the shell entry is syntactically valid, and the Rust/App
API contract gates pass. The full signed non-publishing execution remains open
until the intended source is committed because
`desktop:macos:acceptance` deliberately rejects a dirty worktree; a source
build must not be recorded as exact packaged-product evidence. After the
packaged run, the R5D recorder must be completed first; the R5F recorder then
binds the responsive, notification, dialog, CLI/PATH, Integration, standalone
Skills, Academic Graph, and Zotero observations to that exact product.

## Exit Gate

- every plugin and receipt-owned Skills destination has exactly one visible
  owner and one causal next action;
- App and CLI read and verify through one Rust service and one event contract;
- CLI mutations use reviewed, digest-bound plans rather than process-local UI
  tokens;
- confirmable Integration operations identify their symbolic source and
  registry destination before approval;
- custom Skills are target-scoped and manageable without exposing paths to the
  frontend;
- drifted Skills always have a safe exit: update/removal fail closed, while a
  reviewed detach removes only the exact digest-bound ownership record and
  preserves the target tree;
- product orchestration is host-driven; legacy direct-model checkpoints are
  cleanup-only and cannot be reactivated;
- unmanaged and legacy bytes remain preserved;
- the signed macOS acceptance matrix passes, including the remaining R5E and
  R5D manual gates;
- no product status equates installation, host activation, MCP attachment, or
  live connection without authoritative evidence.
