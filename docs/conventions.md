# Qiongli Framework - Conventions

## Terminology

- `content/` is the canonical academic source tree.
- `packages/` contains installable or publishable package sources.
- `tooling/` contains maintainer automation and operational assets.
- Root `scripts/` is a stable wrapper layer; edit implementations in
  `tooling/scripts/`.
- Root `qiongli-workflow/`, `plugins/qiongli/`, `plugins/qiongli-next/`,
  `.agent/`, and `.gemini/` are generated distribution shapes; edit their
  sources in `content/` and `content/distribution/plugins.yaml`.

## Edit Order

When a change spans multiple layers, apply it in this order:

1. `content/standards/` for contract or routing truth.
2. `content/roles/` and `content/skills/` for responsibility or execution
   behavior.
3. `content/templates/` for stable structured outputs.
4. `tooling/pipelines/`, `content/workflow/workflows/`, and
   `content/distribution/plugins.yaml` for sequencing, entry UX, or plugin
   metadata.
5. `packages/python-qiongli/src/qiongli/` only if runtime execution must
   change.
6. Generated payloads only through staged materialization.

## Where To Put Changes

| If the change is mainly... | Put it here |
|---|---|
| Artifact paths, task outputs, quality gates | `content/standards/research-workflow-contract.yaml` |
| Runtime routing, MCP requirements, skill requirements | `content/standards/mcp-agent-capability-map.yaml` |
| Functional ownership, thresholds, tone | `content/roles/` |
| Reusable task behavior | `content/skills/` |
| Reusable markdown/table structure | `content/templates/` |
| Subject catalog or subject overlays | `content/subjects/` |
| Domain or venue profile data | `content/skills/domain-profiles/`, `content/venue-profiles/` |
| Pipeline sequencing | `tooling/pipelines/` |
| Maintainer automation | `tooling/scripts/` |
| Public script entrypoint compatibility | root `scripts/` wrapper, only when compatibility changes |
| Python runtime, CLI, installer, bridges | `packages/python-qiongli/src/qiongli/` |
| Plugin manifests, prompts, keywords, or platform enablement | `content/distribution/plugins.yaml` |
| Plugin command wrappers, MCP bundle manifest, or platform entry files | `tooling/scripts/build_plugin_artifacts.py` |
| npm package wrapper | `packages/npm-qiongli/` |
| Evaluation cases, rubrics, runners | `evals/` |

## Skill Admission Rules

Create a new internal top-level skill only when all four conditions hold:

1. It consumes typed inputs and produces typed outputs.
2. It owns at least one stable artifact path under `RESEARCH/[topic]/`.
3. It is worth direct pipeline or task-level dependency wiring.
4. It carries distinct failure modes, review expectations, or quality-gate
   value.

Otherwise prefer extending an existing skill, template, provider adapter, role,
or pipeline step.

## Generated Output Rule

Do not edit generated payloads directly. Use:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

Generated paths are ignored and guarded, including:

- `qiongli-workflow/`
- `plugins/qiongli/`
- `plugins/qiongli-next/`
- `.agent/`
- `.gemini/`
- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`
- `packages/qiongli-plugin/`
- `packages/qiongli-next-plugin/`

## `research_skills`

`research_skills` is a deprecated compatibility shim under
`packages/python-qiongli/src/research_skills/`. Keep it working during the
migration window, but use `qiongli` for new imports and docs.
