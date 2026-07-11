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
    qiongli-native/         canonical Rust-native 2.x workspace and product app
    python-qiongli/         Python package source and compatibility shims
    npm-qiongli/            npm wrapper package source
    qiongli-literature-mcpb/ MCPB package source

  tooling/
    architecture/           machine-readable native decision inventory
    quality/                repository-only source policy and debt baseline
    scripts/                real maintainer script implementations
    pipelines/              paper-type DAG descriptors
    install/                installer manifests and support assets
    release/                release docs, receipts, rollback assets

  evals/                    eval cases, rubrics, runner assets
  tests/                    cross-package regression tests
  docs/                     VitePress documentation
    architecture/decisions/ accepted and superseded native ADRs
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
- `packages/qiongli-native/` is the only Qiongli 2 native workspace. It owns
  the single `apps/qiongli` product executable; native service crates must stay
  below that workspace and must not be duplicated into generated plugins.
- `research_skills` remains as a deprecated Python compatibility shim under
  `packages/python-qiongli/src/research_skills/`.
- Root `.agent/` is generated from `content/workflow/` and
  `content/distribution/plugins.yaml`.
- Root `qiongli-workflow/` is generated from `content/workflow/` plus synced
  content mirrors.
- `docs/architecture/decisions/` is the reviewed source of Qiongli 2
  architecture decisions; `tooling/architecture/` contains validation records,
  not runtime payloads or marketplace metadata.
- `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`,
  and `packages/qiongli-next-plugin/` are generated plugin payload shapes, not
  source directories.
