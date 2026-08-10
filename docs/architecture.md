# Architecture

Qiongli 2 is one self-contained Rust-native product with a Tauri 2 / Svelte 5
desktop presentation. The packaged App carries the native CLI, embedded Skills,
Lite and Full MCP surfaces, managed Codex/Claude integration payloads, and the
Zotero Companion. It does not require a user-installed Python or Node runtime.

## Decision Boundary

Accepted decisions under `docs/architecture/decisions/` govern the 2.x line.
ADR 0210 supersedes the earlier AccessKit/egui presentation choice with Tauri
and Svelte; ADR 0211 keeps model authentication, conversation, and execution in
the supported Host. Qiongli owns deterministic content, project state, tools,
handoffs, installation receipts, and release identity.

A change that contradicts an accepted ADR needs a superseding ADR. Generated
payloads and historical migration plans cannot override the current decision
set.

## Editable Source Boundaries

| Boundary | Editable source | Responsibility |
|---|---|---|
| Academic content and contracts | `content/` | workflow, Skills, templates, roles, standards, Plugin metadata, MCP profiles and schemas |
| Native product | `packages/qiongli-native/` | App service, CLI, Lite/Full MCP, project state, embedded pack, integration and release runtime |
| App wire contract | `packages/qiongli-app-api/` | versioned TypeScript decoding of native snapshots, intents and events |
| Desktop presentation | `packages/qiongli-desktop/` | Svelte UI and typed transport adapter |
| Distribution companions | `packages/qiongli-lite-mcp/`, `packages/qiongli-*-mcpb/`, `packages/qiongli-zotero-companion/` | separately packaged MCP and Zotero delivery surfaces |
| Legacy 1.x | `packages/python-qiongli/`, `packages/npm-qiongli/` | maintained 1.x compatibility and migration evidence; not a 2.x runtime fallback |
| Maintainer tooling | `tooling/`; stable wrappers in `scripts/` | materialization, validation, packaging, acceptance and release automation |
| Evidence | `tests/`, `evals/`, `docs/superpowers/acceptance/` | focused regressions, evaluation assets and accepted receipts |

Root `scripts/` files are stable wrappers; edit their implementations under
`tooling/scripts/`. Edit canonical Plugin and Skill inputs under `content/`,
then materialize generated payloads. Do not edit `dist/`, installed client
directories, or generated plugin trees as source.

## Product Spine

1. `content/` defines academic behavior, public MCP contracts, and distribution
   metadata.
2. `qiongli-content` builds the deterministic resource pack consumed by the
   native executable.
3. Native services own configuration, project state, previews, approvals,
   mutations, CLI output, MCP dispatch, and Host integration.
4. The App API validates the native wire shape; Svelte renders it and returns
   typed intents through Tauri.
5. Plugin/Skills and MCP packages expose the same embedded contracts to Codex
   and Claude Code.
6. The Zotero Companion is reached only through the bounded loopback client;
   import-file export remains the safe fallback.

App, CLI, Full MCP, and Host handoff must use the same project services and
revision semantics. The frontend must not construct native plans, paths,
provider models, or readiness claims.

## MCP And Mutation Boundary

Lite MCP owns bounded provider, literature, planning, and Zotero-facing tools.
Full MCP adds registered-project and Academic Graph operations. The public Full
MCP includes one explicit project write,
`qiongli_project_capture_apply`; it re-previews the capture and requires the
matching plan digest plus `approve_filesystem_write=true`.

In-process ToolHost dispatch remains read-only and rejects that write. Release
notes must therefore distinguish “one bounded approval-bound capture write”
from unrestricted Full MCP or ToolHost mutation.

## Dependency Direction

Treat the product as a one-way graph:

1. canonical standards, Skills, MCP schemas and Plugin metadata;
2. native domain/project/runtime services;
3. App API and CLI/MCP adapters;
4. Svelte and Host presentation;
5. materialized packages and release evidence.

If two surfaces disagree, fix the highest shared owner and regenerate or adapt
downstream outputs. Do not add a second project format, provider registry,
release ledger, or product backend.

For exact directory responsibilities, see
[Repository Structure](/development/repository-structure).
