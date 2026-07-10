# Repository Structure

This page is the current source-layout contract for maintainers.

```text
/
  content/                  canonical academic and runtime-contract source
    workflow/               source for generated qiongli-workflow package
    distribution/           plugin metadata source for generated plugin payloads
    skills/                 internal skill specs
    templates/              reusable artifact templates
    standards/              contracts, capability maps, policies
    mcp-contracts/          runtime capability registry, schemas, and fixtures
    roles/                  functional-agent role configs
    subjects/               subject catalog and overlays
    schemas/                JSON/YAML schemas
    venue-profiles/         venue profile data

  packages/
    python-qiongli/         Python package source and compatibility shims
    npm-qiongli/            npm wrapper package source
    qiongli-literature-mcpb/ MCPB package source

  tooling/
    scripts/                real maintainer script implementations
    pipelines/              paper-type DAG descriptors
    install/                installer manifests and support assets
    release/                release docs, receipts, rollback assets

  evals/                    eval cases, rubrics, runner assets
  tests/                    cross-package regression tests
  docs/                     VitePress documentation
  scripts/                  stable wrapper entrypoints
```

## Generated Artifact Shapes

These paths may appear during staging or local maintenance, but they are not
canonical source:

- `qiongli-workflow/`
- `plugins/qiongli/`
- `plugins/qiongli-next/`
- `.agent/`
- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`
- `packages/qiongli-plugin/`
- `packages/qiongli-next-plugin/`

Use staged materialization to produce them:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

## Compatibility Boundaries

- Root `scripts/` remains stable for CI, docs, and user habits. Edit
  `tooling/scripts/` unless the wrapper contract itself changes.
- `content/mcp-contracts/` is the canonical MCP runtime-contract boundary. It
  is not an academic standard, and repository-only RC1 engineering policy must
  remain under `tooling/quality/` rather than being materialized from it.
- `research_skills` remains as a deprecated Python compatibility shim under
  `packages/python-qiongli/src/research_skills/`.
- Root `.agent/` is generated from `content/workflow/` and
  `content/distribution/plugins.yaml`.
- Root `qiongli-workflow/` is generated from `content/workflow/` plus synced
  content mirrors.
- `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`,
  and `packages/qiongli-next-plugin/` are generated plugin payload shapes, not
  source directories.
