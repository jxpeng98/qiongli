# Qiongli R5G Project-Centered Workspace

Status: in progress. P0–P4 are locally implemented and accepted. P5 local
qualification passes; the clean-commit packaged macOS acceptance remains.

## Purpose

R5G turns the existing Research Library, Captures, Academic Graph, Timeline,
and host-driven Orchestrator surfaces into one project-centered workspace. It
also makes the already accepted Plugin-first ownership model explicit in the
Client Integrations presentation.

R5G does not add a model backend, a general-purpose editor, a second installer,
or WebView path authority. Codex or Claude Code continues to own model
conversation and execution. Rust continues to own registered project roots,
artifact validation, receipts, installation, and mutation authority.

## Current diagnosis

The accepted R5C–R5F services expose the required project, graph, continuity,
and integration state, but the Desktop presentation still has three broken
continuity boundaries:

1. Research Library opens a project root in an external application instead
   of entering a Qiongli project workspace.
2. Academic Graph inspection exposes metadata and source anchors, but its only
   artifact action opens the underlying file externally.
3. Research Library, Captures, Academic Graph, Timeline, and Orchestrator each
   own an independent project selector. A project choice is therefore not a
   stable App context or deep link.
4. Client Integrations correctly owns host Plugin lifecycle in Rust, but its
   equal-weight `Skills` status and the adjacent standalone Skills panel make
   one bundled component appear to be a second required installation.

## Product contract

### Project context

- One process-local project workspace context is derived only from the
  registered Research Library snapshot.
- The URL `project` query parameter is the shareable/deep-link form of that
  opaque identity.
- A valid URL identity takes precedence; otherwise the current valid context
  persists across routes; otherwise the first active usable project is chosen.
- No project root, artifact absolute path, or folder-picker result enters the
  context.
- Research Library, Captures, Academic Graph, Timeline, and Orchestrator use
  this one context. Portfolio mode remains an explicit cross-project view.
- Project navigation preserves the shared source-fixture transport parameter
  but drops route-local `capture`, `entity`, filter, and cursor parameters.

### Project workspace

The project workspace contains:

1. Overview — registered identity, research question, thesis, evidence
   position, coverage, risks, and next priorities.
2. Artifacts — bounded, registered project artifacts; introduced by P2.
3. Captures — Inbox, Outbox, Conflicts, and Coverage.
4. Academic Graph — project graph, exploration, evidence inspection, and
   source drill-down.
5. Timeline — project activity, revision history, and resolution history.
6. Run in Client — revision-bound Codex or Claude Code handoff and persisted
   checkpoints.

The primary project action enters this internal workspace. Revealing the
project root in Finder remains an explicit secondary action.

### Artifact viewer

P2 adds a read-only `read-project-artifact` App intent. The request binds:

- `projectId`;
- expected project semantic revision;
- a recognized relative artifact identity;
- optional source anchor; and
- a bounded response budget.

Rust resolves the registered root, rejects traversal and symlink escape,
revalidates the project revision, permits only recognized portable artifacts,
and returns a digest-bound, path-redacted representation. Markdown, CSV, JSON,
and JSONL receive strict format metadata and a bounded UTF-8 source window.
Unsupported, non-text, oversized, or invalid content is not injected into the
WebView.

The reusable viewer is a non-modal details rail with keyboard-equivalent close,
focus restoration, source metadata, and truncation evidence. Academic Graph
retains an explicit secondary external reveal action outside the viewer.
Graph selection, current-revision Capture artifact anchors, and the Artifacts
inventory can open the same viewer. Timeline links exact graph and Capture
identities into those project surfaces without inventing historical file
bytes.

### Plugin and Skills ownership

For Codex and Claude Code, one Host Integration contains:

- Plugin source;
- bundled workflow Skills;
- Lite/Full MCP bridge as supported;
- registration/marketplace evidence; and
- host-owned activation.

Installing the Host Integration installs the supported bundled Skills. There
is no separate host Skills installation action.

Standalone Skills is an optional advanced projection from the same canonical
embedded pack. It remains limited to the existing receipt-owned
Qiongli-managed, registered-project, and opaque custom-folder destinations.
The UI must state that it is not required for a host with its Qiongli Plugin
installed.

## P0 — Contract and information architecture

- Freeze this plan and add the R5G roadmap section.
- Keep existing URLs compatible while adding project-bound deep links.
- Define one project context owner, navigation model, and selection precedence.
- Freeze the read-only Artifact Viewer and Plugin/Skills presentation
  boundaries before adding native intents.

Gate: plan and roadmap agree with R4 host-driven execution, R5E graph
authority, and R5F control-plane ownership.

## P1 — Shared project context and navigation

- Add one root-provided project workspace context.
- Add a compact project context bar with a project selector and project-local
  navigation.
- Preserve the selected project across Research Library, Captures, Academic
  Graph, Timeline, and Orchestrator.
- Make Research Library's primary action enter the internal workspace and keep
  Finder reveal secondary.
- Make Timeline enter project activity when reached from project navigation.
- Preserve Portfolio as an explicit cross-project mode.
- Add pure URL/selection tests, component keyboard/focus tests, and exact-width
  layout contracts.

Gate: selecting a project once changes every project route and the URL without
exposing a root path. Reloading a deep link restores the same project.

### P1 implementation record — July 30, 2026

The root Svelte layout now owns one `ProjectWorkspaceState` derived only from
registered Research Library identities and the path-redacted `project` query
parameter. Global navigation contains only Overview, Research Library,
Portfolio, Client Integrations, and About. A responsive project context bar
owns Overview, Captures, Academic Graph, Timeline, and Run in Client.

Research Library, Captures, Academic Graph, Timeline, and Orchestrator no
longer own independent project state. Research Library enters the internal
workspace and presents Finder reveal as a secondary action. Graph and Captures
no longer repeat a second project selector, Orchestrator displays the bound
project rather than selecting another local scope, and Timeline defaults to
project activity when entered from the project navigation.

Pure selection and URL tests, source architecture contracts, all 191 Desktop
tests, zero-warning Svelte diagnostics, and the static production build pass.
A local source-fixture browser check at the minimum-width and wide layouts
confirmed no body/main overflow, single-line status, Research Library → Graph
context retention, project-switch URL replacement, Timeline project-activity
inheritance, and zero console warnings. Exact packaged width qualification
remains a P5 gate.

## P2 — Bounded artifact projection

- Add strict App API request/event schemas for artifact reading.
- Implement recognized-artifact resolution, revision binding, size limits,
  content digesting, typed parsing, and path/symlink safety in Rust.
- Add source-build fixtures without weakening packaged authority.
- Add a reusable internal Artifact Viewer details rail.

Gate: known Markdown, CSV, JSON, and JSONL fixtures render internally at an
exact anchor; stale revision, traversal, unregistered, symlinked, oversized,
and unsupported requests fail closed.

### P2 implementation record — July 30, 2026

App API schema v15 adds the strict `read-project-artifact` intent and
`project-artifact-read` event. Rust owns a whitelist of the project manifest,
eight canonical semantic artifacts, and graph semantic links; it revalidates
the exact semantic revision before and after a bounded read, rejects path and
link substitution, computes the complete-source digest, and returns no host
path.

The Desktop now has a project-local Artifacts route and one reusable,
line-numbered viewer with anchor highlighting, truncation evidence, an
accessible read-only content region, Escape close, and focus restoration.
Academic Graph and current-revision Capture evidence use the same projection.
Structured `field:` anchors and bounded `line:`/`row:` anchors resolve to the
corresponding source line instead of degrading to an unanchored full-document
preview.
Invalid UTF-8, stale revisions, unsupported paths, entity/projection mismatch,
and drift fail closed.

## P3 — Graph and continuity drill-down

- Make the internal viewer the primary Academic Graph artifact action.
- Preserve graph selection, zoom, filters, and focus when opening or closing
  content.
- Link exact Capture and Timeline evidence to the same viewer.
- Add reverse links from recognized artifact anchors to exact graph entities
  when authoritative identities exist.
- Keep accessible node/relation inventories synchronized with the viewer.

Gate: Graph → artifact → Graph and Timeline/Capture → artifact journeys remain
inside the App and preserve exact project and source identity.

### P3 implementation record — July 30, 2026

Internal preview is now the primary Academic Graph source action while external
reveal remains secondary. The viewer is keyed to exact projection, entity,
project revision, artifact, and source anchor, so opening or closing it does
not reset graph focus, filters, or selection.

Timeline project, graph-node, graph-edge, and Capture identities now deep-link
to the exact project surface. Graph `entity` links restore the selected node or
edge; Capture `capture` links open the exact inbox record. Project navigation
retains the shared project and fixture context while discarding route-local
entity and Capture parameters. Historical timeline records do not claim
artifact bytes that the project store does not retain.

## P4 — Host Integration presentation

- Present one installable Host Integration package per client.
- Nest bundled Skills, MCP, registration, and activation as component evidence.
- Rename the advanced panel to Standalone Skills and state its optional scope.
- Remove any text or visual hierarchy that implies a second Codex/Claude Skills
  installation.
- Keep all existing R5F receipt, preview, plan/apply, and unmanaged-content
  protections unchanged.

Gate: a fresh user can identify one host install action, understand what it
includes, and cannot install a duplicate host Skills projection.

### P4 implementation record — July 30, 2026

Each detected client now presents one Qiongli Host Integration package. Plugin
source, bundled workflow Skills, registration/marketplace, MCP bridge, and host
activation are nested status evidence rather than equal install choices.
Detached or legacy Skills without the current Plugin source are reported as
attention evidence and never make the Host Integration appear installed.
The adjacent advanced projection is renamed Standalone Skills and explicitly
states that it is optional and must not duplicate the host-bundled Skills.
Existing receipt, selection, preview, and plan/apply authority is unchanged.

## P5 — Cross-surface qualification

- Verify deep links, reload, restart, archived/missing projects, stale
  revisions, and project removal.
- Verify keyboard order, visible focus, details-rail focus restoration,
  reduced motion, screen-reader names, and accessible graph inventory parity.
- Verify `375`, `768`, `1024`, and `1440` widths without wrapped status
  capsules, clipped navigation, or horizontal page overflow.
- Run packaged macOS acceptance against representative migrated and native 2.x
  projects.

Gate: one clean committed App passes the project, artifact, graph,
integration, restart, and accessibility acceptance ledger.

### P5 local qualification record — July 30, 2026

Local fixture journeys cover project restoration, exact Graph and Capture deep
links, in-App artifact preview, route-local query cleanup, and the nested Host
Integration presentation. Browser checks cover eight project and integration
routes at exact `375`, `768`, `1024`, and `1440` content widths: all 32
route-width combinations have no page overflow, wrapped status capsules, or
clipped active project navigation, and the browser log has no runtime errors.
At `375`, keyboard focus enters the bounded Artifact Viewer, Escape closes it,
and focus returns to the invoking preview control.

All 32 App API tests, 199 Desktop tests, 167 Rust project tests, 161 Rust
desktop tests, 31 native CLI integration tests, and 4 manual-receipt contract
tests pass. Zero-warning Svelte diagnostics, Rust formatting and checking, the
static production build, diff checks, and an ad-hoc local macOS App build also
pass.

The product-controlled acceptance harness now invokes the packaged CLI to
rebuild a project graph, read an exact graph artifact through the same
`project-artifact-read` App event contract, prove anchor matching and path
redaction, and reject a stale project revision. Its receipt exposes the
mandatory `project_artifact_internal_projection` check and continuity counts
for the successful view, anchor match, and stale-revision rejection. The
remaining P5 gate requires committing the intended source and running that
non-publishing macOS acceptance workflow against the exact commit.

## Execution order

```text
P0 contract
  -> P1 project context
  -> P2 artifact projection
  -> P3 graph and continuity drill-down
  -> P4 integration presentation
  -> P5 packaged acceptance
```

P4 can be implemented alongside P2/P3 after P1 establishes the shared
presentation shell, but it cannot change the R5F ownership contract.

## Explicit non-goals

R5G does not:

- invoke a model from the Qiongli App;
- run Codex or Claude CLI as a hidden model backend;
- add arbitrary filesystem browsing to the WebView;
- edit canonical project artifacts directly in P1–P3;
- infer graph relationships, source anchors, or authorship;
- duplicate Plugin or standalone Skills lifecycle authority; or
- claim external-open fallback is equivalent to internal artifact inspection.
