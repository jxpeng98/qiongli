# ADR 0202: Rust-Native UI And Accessibility Foundation

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201B`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 desktop shell and shared product-service boundary

## Context

Qiongli 2 needs a desktop application that manages skills, MCP servers,
providers, integrations, agents, orchestration, updates, diagnostics, and
recovery. The installed application must start without Python, Node.js, a Rust
toolchain, or a browser-runtime bootstrap. Its UI must call the same typed
application services as the CLI instead of reimplementing domain behavior.

The first native slice also needs an accessibility answer before UI breadth is
allowed to grow. A toolkit that renders successfully but cannot expose stable
roles, names, state, actions, focus, and keyboard order would make the desktop
promise unverifiable and expensive to replace later.

## Decision drivers

- one Rust-native dependency and packaging path across Tier 1 desktop targets;
- no bundled Node.js runtime, development server, or webview application layer;
- an existing native accessibility bridge rather than a Qiongli-specific one;
- deterministic, headless interaction tests for the shell and critical forms;
- explicit keyboard, scaling, contrast, and assistive-technology gates;
- a presentation layer that cannot own config, installer, provider, or
  orchestrator business logic.

## Decision

Qiongli 2 will use `egui` with the native `eframe` application framework and
its `wgpu` renderer for the first desktop implementation. `eframe`'s AccessKit
integration must remain enabled. `FND-201` will pin an exact reviewed release
in `Cargo.lock`; upgrades are normal dependency changes and require the UI
acceptance suite.

The desktop entry point is a mode of the canonical `qiongli` product binary
defined by ADR 0201. UI code owns view state, navigation, presentation, and
user intent only. Typed commands and immutable view models cross into shared
service crates. The UI must not parse or rewrite Qiongli config, host plugin
catalogs, credentials, or project state directly.

The shell uses semantic stock widgets wherever possible. Every custom widget
must provide stable IDs and AccessKit roles, accessible names, values, states,
relationships, and actions. Pointer-only behavior is prohibited. Destructive
or trust-changing operations use a preview and explicit confirmation supplied
by the same service command used by the CLI.

`UI-201` begins with a blocking prototype containing navigation, a provider
form, a skills/MCP table, progress and error feedback, a confirmation dialog,
and recovery status. UI expansion beyond that prototype is allowed only after
the acceptance tests below pass on the target-native runners and designated
assistive-technology checks.

## Alternatives considered

### Iced

Iced provides a typed Elm-style Rust architecture and native rendering, but its
current official material does not establish an integrated AccessKit path.
Adopting it would make Qiongli responsible for validating or building a toolkit
accessibility bridge before the shell exists. Rejected for the first slice.

### Slint

Slint has native packaging and AccessKit integration, but introduces another
declarative UI language and a different licensing and build-surface decision.
It remains a fallback if the accepted prototype fails, not a parallel UI stack.

### Tauri or another webview shell

A web UI can offer a mature component ecosystem, but it introduces a second
frontend toolchain and encourages UI/domain duplication. It also weakens the
single Rust-native build and zero-language-runtime audit. Rejected.

### Separate platform-native UIs

SwiftUI, WinUI, and a Linux-native toolkit could maximize platform fidelity,
but would create three implementations, three accessibility models, and three
release paths before service contracts stabilize. Rejected for 2.x.

## Consequences

Positive consequences:

- the production UI remains Rust-native and shares the product dependency
  graph;
- AccessKit supplies a cross-platform semantic bridge already integrated by
  the selected application framework;
- `egui_kittest` and service fakes can cover navigation and intent without a
  live provider or language runtime;
- one UI implementation can ship in the Tier 1 artifact matrix.

Costs and limitations:

- immediate-mode view code needs discipline to keep stable widget identity and
  focus order;
- native look and advanced text/table behavior may require carefully reviewed
  custom widgets;
- toolkit APIs are evolving, so dependency upgrades may be non-trivial;
- automated semantic checks do not replace manual screen-reader and keyboard
  acceptance on target operating systems.

## Security and privacy

- UI state receives redacted status objects; secret values are write-only and
  are never copied into view models, logs, diagnostics, screenshots, or crash
  reports.
- External links, host activation, network enablement, credential changes, file
  writes, updates, and removal operations require typed intent and policy
  checks in the service layer.
- The renderer cannot select arbitrary filesystem targets. File selection is
  normalized and revalidated by the service boundary before use.
- Accessibility labels must not expose secret values or hidden project data.
- Clipboard copy for sensitive values is off by default and, if later added,
  requires an explicit time-bounded design.

## Rollback

The UI is isolated behind typed service commands and view models. If the
prototype fails any blocking accessibility or packaging test, stop `UI-202`,
retain the shared service contracts, replace only the presentation crate, and
supersede this ADR. No project or global state migration may depend on egui or
eframe types. The CLI remains the recovery surface while a replacement UI is
qualified.

## Acceptance tests

1. A pinned native shell builds and launches on target-native macOS, Windows,
   and Linux runners without Python, Node.js, Cargo, or a webview application
   runtime in the shipped process tree.
2. The representative prototype exposes an expected AccessKit tree containing
   stable roles, names, values, states, focus, and actions for every interactive
   element; the test fails on unnamed or unreachable controls.
3. Keyboard-only tests complete onboarding, switch sections, edit and validate
   a provider form, inspect a skills/MCP row, accept or cancel a dialog, and
   reach recovery without pointer input or focus traps.
4. Automated UI tests cover loading, empty, success, validation-error,
   provider-error, partial-operation, cancellation, and rollback states using
   fake services.
5. Target-native manual receipts record VoiceOver on macOS, Narrator on
   Windows, and the selected Linux accessibility stack before that target is
   advertised.
6. The shell remains usable at 100%, 150%, and 200% scale, with visible focus,
   sufficient contrast, reduced-motion behavior, and no clipped critical
   action.
7. A dependency and payload audit confirms that no second web frontend,
   JavaScript package tree, language-runtime bootstrap, or direct state writer
   entered the desktop artifact.

## Follow-up tasks

- `FND-201`: pin `egui`, `eframe`, renderer, and test dependencies.
- `UI-201`: implement and qualify the blocking shell prototype.
- `UI-202`: add management journeys only after the prototype gate passes.
- `QAT-201`: include desktop startup and process-tree checks in clean-machine
  acceptance.
- `PKG-201`: record target-native packaging and launch receipts.

## Primary references

- [egui project documentation](https://github.com/emilk/egui)
- [eframe native application documentation](https://docs.rs/eframe/latest/eframe/)
- [AccessKit architecture and supported toolkit integrations](https://accesskit.dev/)
