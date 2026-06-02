# Repository Structure

This page is the current source-layout contract for maintainers.

```text
/
  content/                  canonical academic content source
    workflow/               source for generated qiongli-workflow package
    skills/                 internal skill specs
    templates/              reusable artifact templates
    standards/              contracts, capability maps, policies
    roles/                  functional-agent role configs
    subjects/               subject catalog and overlays
    schemas/                JSON/YAML schemas
    venue-profiles/         venue profile data

  packages/
    python-qiongli/         Python package source and compatibility shims
    npm-qiongli/            npm wrapper package source
    qiongli-plugin/         plugin manifests, commands, platform assets
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
- `.agent/`
- `.gemini/`
- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`

Use staged materialization to produce them:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

## Compatibility Boundaries

- Root `scripts/` remains stable for CI, docs, and user habits. Edit
  `tooling/scripts/` unless the wrapper contract itself changes.
- `research_skills` remains as a deprecated Python compatibility shim under
  `packages/python-qiongli/src/research_skills/`.
- Root `.agent/` and `.gemini/` are generated from
  `packages/qiongli-plugin/platforms/`.
- Root `qiongli-workflow/` is generated from `content/workflow/` plus synced
  content mirrors.
