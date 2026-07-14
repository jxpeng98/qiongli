# Qiongli 2.0 R3F Native Desktop Manager Design

Date: 2026-07-14

Status: frozen for implementation

Roadmap slice: `R3F / UI-201` plus the read-only, service-backed portion of
`UI-202`

Scope: one native desktop shell in the canonical Qiongli executable

## Outcome

R3F adds the first Rust-native Qiongli desktop manager without weakening the
accepted service, state, installer, or host-trust boundaries. The canonical
binary accepts `qiongli ui` and opens one native window. The window reads the
same embedded content, redacted configuration, Lite MCP contract, doctor, and
Codex and Claude Code discovery services used by the existing product.

R3F is a blocking UI prototype, not an installer or release. It exposes
welcome/system, Skills, MCP, Providers, Integrations, and Diagnostics views.
Refresh and preview requests cross a typed service boundary. Production config
write, secret capture, MCP process launch, plugin install, remove, update, and
rollback remain unavailable until their signed or secure service paths exist.

## Accepted Toolkit Boundary

ADR 0202 remains authoritative. R3F pins `eframe`, `egui`, and
`egui_kittest` `0.35.0`, which is the current reviewed release on 2026-07-14.
The crate requires the native `wgpu` renderer and AccessKit integration. It
does not enable eframe persistence, a web target, a webview, JavaScript, or a
second frontend toolchain.

The desktop entry remains a mode of `apps/qiongli`; there is no second product
binary. `qiongli-ui` is a library containing presentation state, immutable
view models, typed intents and events, stock egui views, and the eframe app.
The application composition root supplies the concrete service adapter.

Primary toolkit references:

- <https://docs.rs/eframe/0.35.0/eframe/>
- <https://docs.rs/eframe/0.35.0/eframe/trait.App.html>
- <https://docs.rs/egui_kittest/0.35.0/egui_kittest/>

## Dependency Direction

```text
apps/qiongli
  +-> qiongli-ui
  +-> qiongli-content
  +-> qiongli-config
  +-> qiongli-runtime
  +-> qiongli-platform

qiongli-ui
  -> eframe / egui / zeroize only
```

`qiongli-ui` does not depend on config, content, runtime, platform, the app, or
concrete filesystem paths. Its only non-UI dependency is `zeroize`, used for
transient public-form input. The app adapter converts existing redacted service
objects into UI-owned enums and counts. No eframe or egui type crosses back
into a service crate.

## Immutable Desktop Snapshot

The service returns a bounded `DesktopSnapshotV1` containing only display-safe
data:

- product version, current operating system and architecture;
- embedded pack ID, content version, entry count, and the three profile IDs;
- Lite MCP availability and the canonical public tool count;
- config state, revision, default profile, secret-store state, and five
  redacted provider states;
- Codex and Claude Code local state represented by typed source, marketplace,
  direct-package, and registration states;
- fixed doctor checks, blocking flags, and remediation codes; and
- capability flags showing which preview or apply paths are unavailable.

The snapshot contains no absolute path, home directory, secret reference,
email address, API key, environment value, raw config bytes, signing material,
or host registry document. Discovery failures become fixed reason codes.
Missing HOME is a diagnosable integration state and does not prevent the
desktop shell from opening.

Snapshot collection is read-only. It must not create Qiongli, Codex, Claude,
eframe persistence, or temporary state.

## Views And Navigation

The fixed navigation order is:

1. Overview
2. Skills
3. MCP
4. Providers
5. Integrations
6. Diagnostics

Navigation uses stock selectable controls with stable accessible names and a
visible selected state. The layout is responsive: a persistent side rail is
used at normal desktop widths and a compact top selector is used below the
minimum two-column width. Every view remains reachable by keyboard.

The visual language is academic and restrained: flat surfaces, high contrast,
system-provided fonts, an 8-point spacing rhythm, one muted gold accent, no
decorative shadows, no structural emoji, and no custom motion. System light or
dark theme, platform scaling, focus rendering, and reduced-motion behavior are
preserved rather than overridden.

### Overview

Shows product and target identity, content/config health, Lite MCP readiness,
the two local integration summaries, and a prominent truthful alpha notice.

### Skills

Shows the verified embedded pack, entry count, and supported profile table.
Materialization remains described but disabled because the UI has no approved
target-selection service in R3F.

### MCP

Shows the native Lite MCP command shape, transport, public tool count, and
runtime-dependency statement. Launch testing remains unavailable from UI.

### Providers

Shows all five providers in canonical order using redacted readiness. A
provider preview form exercises labelled input, validation, feedback, and the
typed intent boundary. R3F accepts only a public contact-email preview; it
never asks for or stores an API key. The production adapter returns a fixed
source-build apply-unavailable result and never writes configuration.

### Integrations

Shows Codex and Claude Code state, symbolic locations, activation ownership,
and unsupported Desktop/cloud boundaries. An install-preview request returns a
typed unavailable event until a production launch grant exists. It never calls
a client or edits host state.

### Diagnostics

Shows ordered doctor checks, recovery state, fixed remediation codes, and a
Refresh action. It does not render private errors or raw service documents.

## Typed Intents, Events, And Confirmation

The UI can issue only bounded intents:

- `Refresh`;
- `PreviewProviderPublicSetting` for a provider and transient public email;
- `PreviewIntegration` for Codex or Claude Code; and
- `ConfirmOperation` or `CancelOperation` for an opaque service-issued token.

Intents carrying transient user input do not implement `Debug`, serialization,
or logging. Events contain only fixed result codes and display-safe operation
previews. A preview states whether confirmation is permitted. The production
R3F adapter always sets apply to unavailable; the generic confirmation flow is
tested with a fake service so later service work need not redesign the UI.

UI callbacks may update navigation and transient form state, then send an
intent. They may not access the filesystem, environment, config store, network,
process API, host client, clipboard, secret store, or installer directly.

## Accessibility And Test Contract

R3F uses stock egui headings, labels, buttons, selectable controls, tables,
text inputs, progress feedback, and dialogs. Every interactive control has a
stable visible label. Inputs use `labelled_by`; errors are adjacent and do not
rely on color. Disabled actions remain visibly and semantically disabled.

`egui_kittest` tests the AccessKit tree without opening a real window:

- every navigation destination and critical control is named and reachable;
- keyboard/click navigation reaches all six views;
- provider input produces fixed validation or preview feedback without echoing
  the submitted value;
- preview dialogs cancel and confirm through typed intents;
- loading, empty, ready, error, blocked, and recovery states render;
- 100%, 150%, and 200% scales retain the critical controls; and
- narrow and normal desktop widths retain navigation and content.

Manual VoiceOver, Narrator, Linux assistive-technology, packaged startup, and
target-native clean-machine receipts remain blocking gates before a desktop
target is publicly advertised.

## Failure And Security Rules

- renderer startup failure returns the fixed CLI reason code
  `desktop-ui-start-failed` without an absolute path or driver detail;
- an invalid embedded pack still prevents every product mode from starting;
- config or host discovery failure degrades only its bounded card and doctor
  state;
- no background thread, network request, or subprocess is started in R3F;
- no persistence feature is enabled, so closing the prototype writes no UI
  state; and
- custom widgets are prohibited in this slice unless stock egui cannot express
  the required semantic role.

## Non-claims

R3F does not prove or provide:

- production config or secret mutation;
- MCP launch or process supervision from the UI;
- plugin composition, installation, client activation, removal, or rollback
  from the UI;
- eframe state persistence;
- packaged `.app`, `.exe`, MSI, DMG, AppImage, Flatpak, or system launcher;
- target-native manual screen-reader acceptance;
- Claude Desktop, cloud/web, public marketplace, updater, signing, or release;
- Full MCP, project writes, agents, ToolHost, or orchestrator execution.

## Exit Gate

R3F closes when:

1. the pinned native UI dependencies build on the Tier 1 CI matrix with
   AccessKit and wgpu and without a web frontend or persistence feature;
2. `qiongli ui` is a real canonical-binary mode;
3. the six views render from a read-only real-service snapshot;
4. typed preview and confirmation boundaries pass fake-service tests while the
   production adapter keeps apply unavailable;
5. AccessKit, keyboard, responsive-width, and scale tests pass;
6. snapshot collection is redacted, path-free, and side-effect-free;
7. local native, Lite compatibility, and Windows MSVC gates pass; and
8. exact-head CI and the rolling Draft PR preserve the stated non-claims.
