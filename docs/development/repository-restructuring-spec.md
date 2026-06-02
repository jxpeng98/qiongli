# Repository Restructuring Spec

Date: 2026-06-02
Base branch: `dev`
Base commit: `fa6ce5f2f9100a55c29bc56545a0c7e27b619e06`

## Summary

Qiongli should move to a moderate hybrid layout that separates canonical
academic content, installable package shells, runtime code, tooling, and
quality assets without changing user-visible package names.

The target design keeps the source-only development contract introduced on
`dev`: maintainers edit canonical source, CI and release automation materialize
self-contained package payloads into staging trees, and generated mirrors stay
out of source commits.

The central change is to create a `content/` boundary for academic workflow
source and a `packages/` boundary for every installable or publishable package.
The legacy `research_skills` package remains as a deprecated compatibility
shim during the migration window.

## Current State

The latest `dev` tree has 777 tracked files. The largest top-level groups are:

- `tests/`: 88 files
- `skills/`: 88 files
- `templates/`: 72 files
- `release/`: 72 files
- `subjects/`: 69 files
- `docs/`: 66 files
- `scripts/`: 63 files
- `qiongli-workflow/`: 39 files
- `packages/`: 26 files
- `bridges/`: 26 files
- `guides/`: 24 files
- `evals/`: 24 files

Generated distribution outputs are already untracked. The clean checkout does
not track `qiongli/payload/`, `packages/npm-qiongli/payload/`,
`packages/npm-qiongli/python-runtime/`, `plugins/qiongli/skills/qiongli-workflow/`,
or package mirrors under `qiongli-workflow/`.

Recent `dev` changes hardened staged materialization and preflight behavior.
The restructuring must preserve that source-only model.

## Goals

1. Make the root directory smaller and easier to scan.
2. Put all canonical academic workflow content under one source boundary.
3. Put all installable or publishable package shells under `packages/`.
4. Keep user-visible names stable: Python package `qiongli`, npm package
   `qiongli`, plugin name `qiongli`, workflow package name `qiongli-workflow`.
5. Deprecate `research_skills` without breaking existing imports or console
   scripts during the migration window.
6. Keep generated outputs untracked and materialized only in staging or
   explicit maintenance flows.
7. Use tests and documentation updates to make the new layout enforceable.

## Non-Goals

1. Do not rewrite Git history to remove old generated outputs.
2. Do not remove legacy CLI aliases in this restructuring.
3. Do not change the public install commands for PyPI, npm, Codex, Claude, or
   Gemini packages.
4. Do not convert staged materialization back to in-place release publishing.
5. Do not merge `tests/` into package directories. Tests cover cross-package
   contracts and should stay top-level.

## Target Topology

```text
/
  content/
    workflow/
      SKILL.md
      VERSION
      agents/
      references/
      workflows/
    skills/
    templates/
    standards/
    roles/
    venue-profiles/
    subjects/
    schemas/
    skills-core.md
    skills-summary.md

  packages/
    python-qiongli/
      src/
        qiongli/
        research_skills/
    npm-qiongli/
    qiongli-plugin/
    qiongli-literature-mcpb/

  tooling/
    scripts/
    pipelines/
    install/
    release/

  evals/
    cases/
    rubrics/
    runner/
    suites/

  tests/
  docs/
  .github/
  README.md
  README_CN.md
  CHANGELOG.md
  LICENSE
  pyproject.toml
  package.json
  package-lock.json
  uv.lock
```

Root metadata stays at the repository root when external tools expect it
there. This includes `pyproject.toml`, `package.json`, lockfiles, readmes,
license, changelog, and `.github/`.

## Directory Contracts

### `content/`

`content/` is the only editable source for academic workflow content. Feature
PRs that change skills, templates, contracts, roles, subjects, venue profiles,
workflow references, or content schemas should touch this tree.

The source-only development docs should describe `content/` as the canonical
source and package mirrors as generated outputs.

### `packages/`

`packages/` contains all installable or publishable package shells:

- `packages/python-qiongli/` owns Python runtime source for the `qiongli`
  package and compatibility shims.
- `packages/npm-qiongli/` keeps the npm wrapper and package metadata.
- `packages/qiongli-plugin/` owns Codex, Claude, Gemini, and agent plugin
  manifests and command entrypoints.
- `packages/qiongli-literature-mcpb/` keeps the MCPB package.

Generated payload directories may still appear under packages during
materialization, but they remain ignored and should not be committed.

### `tooling/`

`tooling/` groups maintainer-facing automation:

- `tooling/scripts/` for Python and shell maintenance scripts.
- `tooling/pipelines/` for paper-type and workflow pipeline descriptors.
- `tooling/install/` for installer manifests and install support assets.
- `tooling/release/` for release automation docs, templates, receipts, and
  rollback material.

CI scripts can continue to call stable wrapper entrypoints while the internal
files move. A later cleanup can remove wrappers after all workflows and docs
have been updated.

### `evals/`

`eval/` and `evals/` should merge into one `evals/` tree:

- `evals/cases/` for shared evaluation cases.
- `evals/rubrics/` for scoring rubrics.
- `evals/runner/` for evaluation runner code.
- `evals/suites/` for existing academic quality, controller mode, and subject
  specialization suites.

### `tests/`

`tests/` remains top-level because it verifies cross-package behavior:
source-tree contracts, materialization, package artifacts, release automation,
provider bridges, docs, and CLI behavior.

### `docs/`

`docs/` remains top-level as the VitePress documentation site. The current
root `guides/` tree should merge into `docs/guide/`, `docs/advanced/`, or
`docs/maintainer/` depending on audience.

## Migration Map

| Current path | Target path | Notes |
| --- | --- | --- |
| `qiongli-workflow/` | `content/workflow/` | Keep installed package name `qiongli-workflow`; only source path changes. |
| `skills/` | `content/skills/` | Update registry, audits, docs, and materializers. |
| `templates/` | `content/templates/` | Keep `CLAUDE.project.md` exclusion behavior in package materialization. |
| `standards/` | `content/standards/` | Update validators and docs. |
| `roles/` | `content/roles/` | Update materializer copy roots. |
| `venue-profiles/` | `content/venue-profiles/` | Preserve installed mirror path. |
| `subjects/` | `content/subjects/` | Preserve subject package materialization behavior. |
| `schemas/` | `content/schemas/` | Content schema validation follows content source. |
| `skills-core.md` | `content/skills-core.md` | Preserve installed mirror file. |
| `skills-summary.md` | `content/skills-summary.md` | Preserve installed mirror file. |
| `qiongli/` | `packages/python-qiongli/src/qiongli/` | Public Python package name stays `qiongli`. |
| `research_skills/` | `packages/python-qiongli/src/research_skills/` | Deprecated compatibility shim. |
| `bridges/` | `packages/python-qiongli/src/qiongli/bridges/` | Runtime/provider code belongs with Python package. |
| `packages/npm-qiongli/` | `packages/npm-qiongli/` | Keep path stable unless a later package rename is approved. |
| `packages/qiongli-literature-mcpb/` | `packages/qiongli-literature-mcpb/` | Keep path stable. |
| `plugins/qiongli/` | `packages/qiongli-plugin/` | Plugin package source moves under packages. |
| `.agent/` | `packages/qiongli-plugin/platforms/agent/` | Move only after root discovery checks pass. |
| `.gemini/` | `packages/qiongli-plugin/platforms/gemini/` | Move only after root discovery checks pass. |
| `scripts/` | `tooling/scripts/` | Keep temporary root wrappers for CI and docs. |
| `pipelines/` | `tooling/pipelines/` | Pipeline descriptors are maintainer tooling. |
| `install/` | `tooling/install/` | Installer manifest is support data. |
| `release/` | `tooling/release/` | Release assets are operational artifacts. |
| `eval/` | `evals/runner/`, `evals/rubrics/`, `evals/cases/` | Merge with existing `evals/`. |
| `guides/` | `docs/guide/`, `docs/advanced/`, `docs/maintainer/` | Remove duplicate docs entrypoint. |

## Materialization Data Flow

The new flow should be:

```text
content/ + packages/* source + tooling/materializers
  -> staging tree created by materialize_distribution_payloads.py
  -> package payloads and plugin skill mirrors inside staging tree
  -> package tests, audits, preflight checks, release artifacts
```

The source checkout should remain output-free. A new runtime module,
`qiongli.source_layout`, should own logical source roots and physical path
resolution. Tooling scripts should import that module instead of hard-coding
content, package, and staging paths.

The generated output guard should continue to reject changes to package mirrors
and generated payload directories. After the move, it should reject generated
paths under the new package locations as well.

## Deprecation Strategy for `research_skills`

`research_skills` should become a compatibility shim, not a parallel runtime.

1. Move the shim to `packages/python-qiongli/src/research_skills/`.
2. Re-export the matching `qiongli` modules.
3. Keep console scripts `research-skills`, `rsk`, and `rsw` during the
   migration window.
4. Document `qiongli` and `ql` as the preferred names.
5. Add tests that imports from `research_skills` still resolve to equivalent
   `qiongli` behavior.
6. Remove the shim only in a separate breaking-change release with explicit
   upgrade notes.

Warnings should be introduced cautiously. CLI aliases used in automation should
not print noisy warnings to stdout. Import-time warnings may be acceptable only
after checking tests and downstream expectations.

## Phased Implementation Plan

### Phase 0: Layout Constants and Guardrail Tests

Add a layout abstraction before moving files. It should expose logical roots:
workflow source, skills source, templates, standards, roles, venues, subjects,
schemas, Python package root, npm package root, plugin package root, tooling
root, and eval roots.

Add tests that assert:

- canonical sources are found through the abstraction,
- generated output roots are not tracked,
- materialization to `--out` does not write back to the source checkout,
- root wrappers still invoke moved tooling entrypoints.

### Phase 1: Move Canonical Content to `content/`

Move workflow content, skills, templates, standards, roles, venue profiles,
subjects, schemas, and top-level skill summary files into `content/`.

Update:

- materializers,
- audits,
- subject catalog loading,
- docs,
- source-tree tests,
- plugin and package artifact tests,
- release and CI references.

Acceptance criteria:

- source-level tests pass,
- staged materialization produces the same installed package shape,
- package contract tests pass against the staged tree,
- no generated output directories are tracked.

### Phase 2: Unify Runtime and Package Sources Under `packages/`

Move Python runtime code into `packages/python-qiongli/src/qiongli/`.
Move `research_skills` into `packages/python-qiongli/src/research_skills/` as
a compatibility shim. Move `bridges/` into `qiongli.bridges`.

Update `pyproject.toml` package discovery and package data so the published
PyPI package remains named `qiongli` and still includes the materialized
payload when built from a staging tree.

Acceptance criteria:

- `python -m qiongli.cli --help` or equivalent CLI tests pass,
- legacy console scripts still work,
- imports from `research_skills` still pass compatibility tests,
- provider bridge tests pass from the new package path.

### Phase 3: Move Plugin Sources Under `packages/qiongli-plugin/`

Move `plugins/qiongli/` into `packages/qiongli-plugin/`. Move platform-specific
agent and Gemini assets into that package only after root discovery checks pass.

Update plugin materializers and tests to materialize the same release payload
shape. The installed plugin may still contain `skills/qiongli-workflow`; that
is an artifact shape, not the source checkout shape.

Acceptance criteria:

- Codex, Claude, and Gemini plugin manifests pass version alignment tests,
- plugin package materialization produces a real portable skill copy,
- marketplace artifact validation still passes.

### Phase 4: Consolidate Tooling, Evals, and Guides

Move `scripts/`, `pipelines/`, `install/`, and `release/` into `tooling/`.
Keep root wrapper scripts for the commands used by CI, docs, release workflows,
and common maintainer habits. Wrapper removal should be a later cleanup after
docs and workflows have converged.

Merge `eval/` into `evals/`. Move `guides/` into the VitePress docs tree.

Acceptance criteria:

- release automation tests pass,
- docs tests pass,
- evaluation tests pass,
- `npm run validate` still reaches the expected scripts through wrappers or
  updated package scripts.

### Phase 5: Documentation and Cleanup

Update all English and Chinese docs that mention old source paths. Add a
repository structure page that describes:

- `content/` as canonical academic source,
- `packages/` as installable package shells,
- `tooling/` as maintainer automation,
- `evals/` as quality evaluation assets,
- generated payloads as staged outputs only.

Remove obsolete references to direct edits in old root content directories.
Keep deprecation notes for `research_skills` and legacy aliases.

## Verification Matrix

Run focused checks after each phase:

```bash
python -m unittest tests.test_distribution_source_tree
python -m unittest tests.test_materialize_distribution_payloads
python -m unittest tests.test_distribution_payloads
python -m unittest tests.test_plugin_distribution_contract
python -m unittest tests.test_npm_package_contract
python -m unittest tests.test_release_automation
python -m unittest tests.test_distribution_materialization_docs
npm run validate
```

When a phase changes Python packaging, also run a build from a staged tree:

```bash
python scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
bash scripts/pypi_preflight.sh --root /tmp/qiongli-dist
bash scripts/npm_preflight.sh --root /tmp/qiongli-dist
```

During Phase 4, update these command paths to either root wrapper paths or the
new `tooling/` paths before running the checks. The behavior they verify should
remain the same.

## Risks and Mitigations

| Risk | Mitigation |
| --- | --- |
| Materializer reads old root paths after `content/` move. | Introduce layout constants before moving files and update tests first. |
| Python package build loses payload data. | Validate builds only from staged materialized trees and keep package data tests. |
| External imports depend on `bridges` or `research_skills`. | Keep compatibility shims and deprecate separately from the move. |
| CI breaks because scripts move. | Keep root wrappers while workflows and docs migrate. |
| Plugin manifests assume old `plugins/qiongli` path. | Move plugin tests with the package source and validate staged artifacts. |
| Docs drift across English and Chinese pages. | Update paired docs in the same phase and keep doc tests for key path tokens. |
| Generated outputs accidentally become tracked again. | Update `.gitignore`, generated path classification, and source-tree tests. |

## Acceptance Criteria for the Restructure

The restructuring is complete when:

1. Maintainers edit academic content under `content/`.
2. All package sources live under `packages/`.
3. `research_skills` is a documented compatibility shim.
4. `eval/` and `evals/` are unified under `evals/`.
5. Maintainer automation is grouped under `tooling/` with stable wrappers for
   common commands.
6. Staged materialization produces the same installable package shapes as
   before the restructure.
7. Generated outputs remain untracked in the source checkout.
8. English and Chinese documentation describe the new layout.
9. CI, package contract tests, release preflight tests, and docs tests pass.

## Review Focus

Reviewers should focus on:

- whether the `content/` boundary is complete enough for all canonical source,
- whether moving Python runtime code into `packages/python-qiongli/` is worth
  the packaging churn,
- whether `bridges/` should move with Python runtime in Phase 2 or remain
  top-level until a separate provider-runtime cleanup,
- whether root wrappers for moved scripts should be permanent or temporary,
- whether `.agent/` and `.gemini/` can move without breaking local platform
  discovery.
