# ADR 0210: Tauri And Svelte Desktop Presentation

- Status: Accepted
- Date: 2026-07-19
- Task ID: `ARC-210`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 production desktop presentation, frontend build,
  typed application API, accessibility, and package closure
- Supersedes: ADR 0202's selection of `egui`, `eframe`, AccessKit, and the ban
  on a webview presentation layer

## Context

The R3 native application proved that Qiongli's Rust services, deterministic
embedded content, configuration, installer plans, receipts, and CLI can ship as
one product. It also exposed a presentation limitation: the current immediate-
mode UI does not provide the information hierarchy, readable content density,
component maturity, or interaction clarity required for client integration and
the research-workspace visualizations planned for R4.

R4-0 must repair today's product before adding the article workspace. Overview,
Workflow Content, and Client Integrations need truthful state, accessible
feedback, explicit operation previews, and packaging behavior that is identical
between local candidates and release artifacts. The frontend must remain a thin
consumer of Rust application services so that a later UI change does not move
research or installation authority into presentation code.

## Decision drivers

- a mature accessible component and testing ecosystem;
- a production path for knowledge graphs, evidence maps, and dense tables;
- one canonical Rust service, CLI, state, installer, and resource-pack model;
- a small typed and auditable command/event boundary;
- no Node.js or frontend toolchain required on an installed user's machine;
- target-native packaging and accessibility evidence before the old UI retires;
- modular frontend code that can be replaced without migrating product data.

## Decision

Qiongli 2 will use Tauri 2 as the desktop shell and Svelte 5 with SvelteKit's
static SPA mode as the production presentation stack. The operating system's
webview is the renderer. Node.js, pnpm, Vite, and SvelteKit are build-time tools
only and are not shipped as language-runtime bootstraps.

The canonical `qiongli` executable remains the product entry point. Tauri is
integrated into that executable rather than becoming a second domain runtime.
The CLI and desktop call the same Rust services. The existing `egui` frontend
is frozen during R4-0 and is removed only after packaged Tauri/Svelte acceptance
passes; no new product behavior may be implemented only in the old UI.

Presentation and authority are separated as follows:

1. Rust owns configuration, project artifacts, embedded content, filesystem
   discovery, install plans, trust checks, confirmation, receipts, recovery,
   updates, diagnostics, and all writes.
2. A versioned `qiongli-app-api` exposes allowlisted commands, redacted data
   transfer objects, operation tokens, and lifecycle events. It is independent
   of Svelte components and mechanically validated on both sides of IPC.
3. Svelte routes own navigation, local form state, rendering, focus restoration,
   and user intent. They do not parse or rewrite Qiongli state or client files.
4. Project-local portable research artifacts remain the cross-host source of
   truth. Conversation sessions are evidence inputs, not the canonical article
   record, and the frontend cannot silently observe unrelated host sessions.

The frontend dependency baseline is deliberately conventional: SvelteKit with
`adapter-static`, TypeScript, Vite, Tailwind CSS with semantic CSS variables,
Bits UI for accessible headless primitives, Lucide for icons, TanStack packages
for server state/forms/tables/virtualization where those problems appear, Zod
for runtime DTO validation, and Cytoscape for R4A graph views. Dependencies are
exactly resolved in the workspace lockfile; additions require a concrete use
case and ownership review.

R4-0 initially migrates Overview, Workflow Content, and Client Integrations.
Every mutating integration journey must show what Qiongli content will be
installed, where it will be installed, and whether the current build has the
authority to confirm the operation. Detection of a host executable version is
reported separately from the presence and health of Qiongli-managed content.

## Alternatives considered

### Continue expanding `egui`

Rejected for R4 production work. The current implementation can remain a
temporary fallback, but closing its content hierarchy, form, table, graph, and
accessibility gaps would require maintaining Qiongli-specific UI primitives.

### Vue 3 in Tauri

Vue has a mature ecosystem and would satisfy the shell requirements, but the
team has selected Svelte's smaller component model for this product. A Vue
intermediate implementation would add migration work without validating a
different service boundary.

### React in Tauri

React has the broadest library ecosystem, but its application and state
conventions add more presentation structure than R4-0 needs. The graph and
table libraries Qiongli requires are available without making React the shell.

### A browser-served web application

Rejected as the canonical desktop product because filesystem installation,
receipts, secret storage, update replacement, and host integration require the
native trust boundary. A future read-only browser companion may consume a
separate authenticated service but cannot replace this decision implicitly.

### Fully custom Svelte components

Rejected. Qiongli will compose maintained accessible primitives and use custom
code for product-specific content, not recreate dialogs, menus, focus handling,
tables, virtual lists, or graph engines.

## Consequences

Positive consequences:

- the UI gains the browser accessibility model and a mature component/testing
  ecosystem while Rust remains the single product authority;
- R4A graphs and evidence maps can use established visualization engines;
- static frontend bytes can be embedded and versioned with the application;
- the typed API makes presentation replacement and CLI parity reviewable;
- UI modules remain smaller because domain operations do not move into routes.

Costs and limitations:

- development and release builds now require a pinned Node.js/pnpm frontend
  toolchain in addition to Rust;
- each target depends on its supported system webview and requires target-native
  rendering, accessibility, startup, and packaging evidence;
- IPC DTO compatibility and command permissions become a maintained contract;
- the transition temporarily carries both presentation dependency trees;
- browser security policy, asset embedding, and frontend supply-chain auditing
  are new release responsibilities.

## Security and privacy

- Tauri exposes an allowlist of application commands; there is no generic shell,
  process, filesystem, URL fetch, eval, or arbitrary path command.
- All command arguments are deserialized into bounded enums and structs and are
  revalidated in Rust. Frontend validation is usability, not trust.
- Content Security Policy denies remote code. Application scripts and styles
  are packaged locally, and navigation outside the application is denied unless
  a later typed policy explicitly permits it.
- DTOs contain redacted status and display-safe path evidence. Credentials,
  authorization material, raw private config, and research document contents
  are not emitted through global events or logs.
- Writes retain the existing preview, operation-token, confirmation, ownership,
  receipt, rollback, and recovery rules. A source build without release or local
  development authority must say so and remain read-only.
- The UI does not enumerate Codex, Claude Code, or cloud conversation history.
  Cross-host research capture requires explicit project artifacts or a future
  opt-in adapter with a documented consent and provenance boundary.

## Rollback

Until the R4-0 packaged gate passes, the old presentation may be built from the
same Rust service contracts as a development fallback. If Tauri/Svelte fails a
blocking package, accessibility, security, or authority test, pause frontend
expansion, keep the Rust services and `qiongli-app-api`, and restore the prior
presentation entry point. No config, receipt, project, or research-state format
may depend on Svelte, browser storage, or Tauri-specific types, so rollback does
not migrate product data.

After the gate passes and the old UI is removed, rollback means shipping a
previous verified complete Qiongli 2 application through the existing signed
application-replacement protocol, never mixing frontend assets with another
Rust runtime version.

## Acceptance tests

1. The static Svelte frontend builds reproducibly from the pinned lockfile and
   is included in the candidate/release application; installed startup requires
   no Node.js, pnpm, Vite dev server, browser download, or network access.
2. Overview, Workflow Content, and Client Integrations render loading, empty,
   ready, missing, unavailable, failed, preview, confirmation, cancelled, and
   completed states from the typed API without parsing host files in TypeScript.
3. Rust and TypeScript contract fixtures validate the same schema version,
   command names, enum values, redaction rules, and operation-token lifecycle;
   unknown or oversized input fails closed.
4. Every mutating UI action maps to an existing Rust service intent and produces
   the same preview, authority decision, confirmation, receipt, and recovery
   result as the CLI path.
5. Source-read-only and locally installable candidates are visibly distinct.
   Packaged-product authority is never inferred from an executable version or
   embedded content alone.
6. Keyboard-only tests navigate all migrated routes, operate filters and forms,
   accept and cancel confirmation, reach inline errors, and restore focus with
   no traps. Automated semantic checks reject unnamed controls and invalid
   dialog relationships.
7. Target-native receipts cover VoiceOver on macOS before Alpha publication;
   Windows and Linux require equivalent evidence before those targets are
   advertised. The interface remains usable at 100%, 150%, and 200% scaling,
   with visible focus, sufficient contrast, and reduced motion.
8. Content Security Policy, command allowlist, dependency audit, offline launch,
   and packaged asset-closure checks pass for the exact release candidate.

## Follow-up tasks

- `R4-0`: land the typed API, shell, three migrated journeys, authority split,
  tests, and packaged candidate evidence.
- `R4A`: add project workspace, research capture, and academic graph views only
  after the R4-0 gate passes.
- `QAT-401`: add target-native webview accessibility and offline startup
  receipts to release qualification.
- `PKG-401`: include the static frontend, lockfile identity, CSP, and asset
  closure in package manifests and release ledgers.

## Primary references

- [Tauri 2 frontend configuration](https://v2.tauri.app/start/frontend/)
- [Tauri 2 SvelteKit configuration](https://v2.tauri.app/start/frontend/sveltekit/)
- [Svelte package catalog](https://svelte.dev/packages)
- [Bits UI documentation](https://www.bits-ui.com/docs)
