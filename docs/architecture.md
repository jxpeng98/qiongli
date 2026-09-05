# Architecture

Qiongli 2 is a Rust-native product moving to standalone CLI/Plugin/Skills/MCP
delivery without a Qiongli App window. The current package still combines the
CLI with Tauri 2 / Svelte 5 presentation and App-based installation trust;
independent CLI build/package acceptance remains open. Existing desktop support,
embedded content and Zotero Companion are retained during the split.

## Decision Boundary

Accepted decisions under `docs/architecture/decisions/` govern the 2.x line.
ADR 0218 supersedes ADR 0217's App-owned ACP default: users work in their chosen
External Hosts, which own models, authentication and private conversations.
This retains ADR 0211's Host-owned authentication and execution boundary.
Qiongli owns deterministic content, projects, tools, task/candidate/checkpoint
state, installation receipts and release identity. Same-device collaboration
precedes any optional cross-device synchronization.

ADR 0210 still owns the retained Tauri/Svelte presentation. Existing ACP/All Chat
source and schemas are preserved as deferred development work. ADR 0218 does not
qualify a standalone package or retire published GUI support. Earlier accepted
ADRs and evidence are not rewritten; implementation must preserve package trust,
preview/approval/CAS and recovery while removing the mandatory App dependency.

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
5. Selected External Hosts consume Plugin/Skills and native CLI/MCP contracts;
   their private models and conversations remain outside Qiongli.
6. Existing task/handoff/checkpoint owners will coordinate local candidates and
   exact-digest review. Retained All Chat observations do not grant project authority.
7. The Zotero Companion is reached only through the bounded loopback client;
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
