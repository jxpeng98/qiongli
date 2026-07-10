# Architecture

Qiongli uses a hybrid repository layout: canonical academic content, runtime
code, package shells, and maintainer tooling live in separate source
boundaries, while release and install payloads are materialized as generated
outputs.

## Source Boundaries

| Boundary | Editable source | Responsibility |
|---|---|---|
| Academic content | `content/workflow/`, `content/skills/`, `content/templates/`, `content/standards/`, `content/roles/`, `content/subjects/`, `content/schemas/`, `content/venue-profiles/` | Workflow package source, academic contracts, skills, templates, roles, subjects, schemas, and venue profiles |
| Runtime capability contracts | `content/mcp-contracts/` | Versioned MCP capability registry, runtime input/output schemas, compatibility aliases, and smoke fixtures; not an academic-standard or RC1-policy boundary |
| Plugin distribution metadata | `content/distribution/plugins.yaml` | Stable and prerelease plugin names, descriptions, prompts, keywords, and platform enablement |
| Python runtime | `packages/python-qiongli/src/` | `qiongli`, deprecated `research_skills` shim, bridge adapters, CLI/runtime code |
| Package shells | `packages/npm-qiongli/`, `packages/qiongli-literature-mcpb/` | Publishable npm and MCPB package sources |
| Maintainer tooling | `tooling/scripts/`, `tooling/pipelines/`, `tooling/install/`, `tooling/release/` | Automation, pipeline descriptors, installer manifests, release assets |
| Quality assets | `evals/`, `tests/` | Evaluation cases/runners and cross-package regression tests |
| Documentation | `docs/` | VitePress docs and maintainer guidance |

Root `scripts/` files are compatibility wrappers. Keep user-facing commands and
CI references stable there, but edit script implementations under
`tooling/scripts/`.

Root `qiongli-workflow/`, `plugins/qiongli/`, `plugins/qiongli-next/`,
and `.agent/` are generated artifact shapes. Edit workflow content
under `content/workflow/` and plugin metadata under
`content/distribution/plugins.yaml`.

## Layer Model

| Layer | Primary editable source | Responsibility |
|---|---|---|
| Contract | `content/standards/research-workflow-contract.yaml` | Task IDs, artifacts, quality gates |
| Capability Map | `content/standards/mcp-agent-capability-map.yaml` | Runtime routing, MCP and skill requirements |
| Functional Agents | `content/roles/` | Ownership, quality thresholds, tone |
| Internal Skill Specs | `content/skills/` | Reusable execution behavior |
| Pipelines | `tooling/pipelines/` | Step sequencing and handoffs |
| Client entry UX | `content/workflow/workflows/`, `content/distribution/plugins.yaml` | Portable workflows and generated platform command surfaces |
| Runtime | `packages/python-qiongli/src/qiongli/` | CLI, installers, orchestration, providers |
| Distribution | materialized staging tree | `qiongli-workflow/`, plugin payloads, npm payload, Python payload |

## Stable User-Facing Entry Modes

| Entry mode | Best for | Stable entry |
|---|---|---|
| CLI install/upgrade | Installing and upgrading assets | `qiongli`, `ql`, `research-skills`, `rsk`, `rsw` |
| Script entrypoints | CI, release, local maintenance | `scripts/*.py`, `scripts/*.sh` wrappers |
| Orchestrator CLI | Task planning, execution, validation | `python3 -m qiongli.bridges.orchestrator ...` |
| Portable skill package | Cross-client distribution surface | generated `qiongli-workflow/` |
| Plugin package | Codex/Claude plugin distribution | generated `plugins/qiongli/` |

## Dependency Direction

Treat the system as a one-way graph:

1. `content/standards/` and `content/mcp-contracts/`
2. `content/roles/` and `content/skills/`
3. `content/templates/`
4. `tooling/pipelines/`, `content/workflow/workflows/`, and plugin metadata
5. Python and Rust runtime packages
6. materialized distribution payloads

Generated payloads must not become hidden sources of truth. If a generated
plugin directory disagrees with `content/`, `content/distribution/plugins.yaml`,
or the MCPB runtime package, fix the source and materialize again.

For exact directory responsibilities, see
[Repository Structure](/development/repository-structure).
