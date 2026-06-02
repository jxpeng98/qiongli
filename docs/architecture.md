# Architecture

Qiongli uses a hybrid repository layout: canonical academic content, runtime
code, package shells, and maintainer tooling live in separate source
boundaries, while release and install payloads are materialized as generated
outputs.

## Source Boundaries

| Boundary | Editable source | Responsibility |
|---|---|---|
| Academic content | `content/` | Workflow package source, internal skills, templates, standards, roles, subjects, schemas, and venue profiles |
| Python runtime | `packages/python-qiongli/src/` | `qiongli`, deprecated `research_skills` shim, bridge adapters, CLI/runtime code |
| Package shells | `packages/npm-qiongli/`, `packages/qiongli-plugin/`, `packages/qiongli-literature-mcpb/` | Publishable npm, plugin, and MCPB package sources |
| Maintainer tooling | `tooling/scripts/`, `tooling/pipelines/`, `tooling/install/`, `tooling/release/` | Automation, pipeline descriptors, installer manifests, release assets |
| Quality assets | `evals/`, `tests/` | Evaluation cases/runners and cross-package regression tests |
| Documentation | `docs/` | VitePress docs and maintainer guidance |

Root `scripts/` files are compatibility wrappers. Keep user-facing commands and
CI references stable there, but edit script implementations under
`tooling/scripts/`.

Root `qiongli-workflow/`, `plugins/qiongli/`, `.agent/`, and `.gemini/` are
generated artifact shapes. Edit their sources under `content/workflow/` or
`packages/qiongli-plugin/`.

## Layer Model

| Layer | Primary editable source | Responsibility |
|---|---|---|
| Contract | `content/standards/research-workflow-contract.yaml` | Task IDs, artifacts, quality gates |
| Capability Map | `content/standards/mcp-agent-capability-map.yaml` | Runtime routing, MCP and skill requirements |
| Functional Agents | `content/roles/` | Ownership, quality thresholds, tone |
| Internal Skill Specs | `content/skills/` | Reusable execution behavior |
| Pipelines | `tooling/pipelines/` | Step sequencing and handoffs |
| Client entry UX | `content/workflow/workflows/`, `packages/qiongli-plugin/platforms/` | Portable workflows and platform command surfaces |
| Runtime | `packages/python-qiongli/src/qiongli/` | CLI, installers, orchestration, providers |
| Distribution | materialized staging tree | `qiongli-workflow/`, plugin payloads, npm payload, Python payload |

## Stable User-Facing Entry Modes

| Entry mode | Best for | Stable entry |
|---|---|---|
| CLI install/upgrade | Installing and upgrading assets | `qiongli`, `ql`, `research-skills`, `rsk`, `rsw` |
| Script entrypoints | CI, release, local maintenance | `scripts/*.py`, `scripts/*.sh` wrappers |
| Orchestrator CLI | Task planning, execution, validation | `python3 -m qiongli.bridges.orchestrator ...` |
| Portable skill package | Cross-client distribution surface | generated `qiongli-workflow/` |
| Plugin package | Codex/Claude/Gemini plugin distribution | generated `plugins/qiongli/` |

## Dependency Direction

Treat the system as a one-way graph:

1. `content/standards/`
2. `content/roles/` and `content/skills/`
3. `content/templates/`
4. `tooling/pipelines/` and platform command sources
5. `packages/python-qiongli/src/qiongli/`
6. materialized distribution payloads

Generated payloads must not become hidden sources of truth. If a generated
directory disagrees with `content/` or `packages/`, fix the source and
materialize again.

For exact directory responsibilities, see
[Repository Structure](/development/repository-structure).
