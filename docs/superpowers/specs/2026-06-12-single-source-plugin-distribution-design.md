# Single-Source Plugin Distribution Design

## Goal

Make Qiongli easier to maintain by removing tracked plugin payload and wrapper
duplicates from the source tree. Daily feature work should edit one canonical
source for workflow behavior, skill content, distribution metadata, and runtime
code. Installable plugin shapes should be generated during validation, CI,
release staging, or publication.

The immediate motivation is the PR33 pattern: one routing and grill contract
change touched canonical `content/` files plus mirrored plugin skill files,
platform workflow files, and repeated manifest metadata. The functional behavior
was the same, but the review surface was much larger than the real change.

## Non-Goals

- Do not remove Codex, Claude, Gemini, npm, Python, CLI, or MCP distribution
  support.
- Do not weaken install validation. Generated artifacts must be validated more
  directly than the current checked-in mirrors.
- Do not keep thin wrapper files as editable source. If a wrapper is needed in a
  platform package, generate it from canonical workflow metadata.
- Do not make feature branches commit generated package payloads.
- Do not change academic workflow semantics as part of this architecture change.

## Current Problem

The repository already documents `content/` as the source of truth and has
materialization scripts. However, several installable or near-installable plugin
trees are still tracked as source:

- `packages/qiongli-next-plugin/skills/qiongli-workflow/**`
- `packages/qiongli-next-plugin/commands/**`
- `packages/qiongli-next-plugin/.codex-plugin/plugin.json`
- `packages/qiongli-next-plugin/.mcp.json`
- `packages/qiongli-next-plugin/mcp/**`
- `packages/qiongli-plugin/commands/**`
- `packages/qiongli-plugin/platforms/agent/workflows/**`
- `packages/qiongli-plugin/platforms/gemini/**`
- `packages/qiongli-plugin/.codex-plugin/plugin.json`
- `packages/qiongli-plugin/.claude-plugin/plugin.json`
- `packages/qiongli-plugin/gemini-extension.json`
- `packages/qiongli-plugin/.mcp.json`
- `packages/qiongli-plugin/mcp/**`

These trees duplicate content from:

- `content/workflow/`
- `content/skills/`
- `content/templates/`
- `content/standards/`
- `content/roles/`
- `content/subjects/`
- `packages/qiongli-literature-mcpb/server/`

The result is a hidden multi-source model. Tests assert that the mirrors are
installable, so feature PRs often update both canonical files and mirrored files.
That keeps artifacts current, but it makes every behavior change look like a
package rewrite.

## Target Architecture

Use a strict single-source model.

| Concern | Editable source | Generated output |
|---|---|---|
| Workflow skill shell | `content/workflow/` | `qiongli-workflow/` inside staged payloads |
| Internal skills | `content/skills/` | staged package `skills/` mirrors |
| Templates, roles, standards, venue profiles | `content/templates/`, `content/roles/`, `content/standards/`, `content/venue-profiles/` | staged skill package mirrors |
| Subject packages | `content/subjects/` | staged subject-specific packages |
| Plugin metadata | `content/distribution/plugins.yaml` | Codex, Claude, Gemini manifests |
| Platform command wrappers | derived from `content/workflow/workflows/*.md` and plugin metadata | staged `commands/*.md` or platform workflow wrappers |
| Literature MCP runtime | `packages/qiongli-literature-mcpb/server/` | bundled plugin MCP runtime |
| npm and Python payloads | canonical source plus Python runtime | staged package payloads |

`packages/qiongli-plugin/` and `packages/qiongli-next-plugin/` should no longer
be tracked source directories. If local or release workflows need those exact
directory shapes, the materializer creates them in a staging directory or an
ignored generated-output path.

## Distribution Metadata Source

Add a small canonical metadata file, for example:

`content/distribution/plugins.yaml`

It should define plugin families and platform variants without embedding copied
workflow content:

- stable plugin id: `qiongli`
- prerelease plugin id: `qiongli-next`
- display names
- descriptions
- keywords
- author, repository, license, category
- default prompts
- target platforms
- MCP server names and startup settings
- skill invocation name rewrites

The generator should derive each platform manifest from this file plus
`content/workflow/VERSION`. Version strings must not be hand-edited in each
platform manifest.

## Generated Wrappers

Thin command wrappers should be generated, not stored.

For each workflow in `content/workflow/workflows/*.md`, the generator can create
platform wrappers such as:

```text
Load the `<skill_name>` skill from this plugin, then follow
`skills/qiongli-workflow/workflows/<workflow>.md`.
```

The wrapper description should come from workflow frontmatter. If a platform
needs a different file location or invocation phrase, that rule belongs in the
generator or `content/distribution/plugins.yaml`, not in copied markdown files.

## Materialization Flow

Replace source-tree plugin packages with generated staging outputs:

1. Build a temporary materialization source from `content/`.
2. Materialize the core skill package.
3. Generate platform command wrappers from canonical workflows.
4. Generate Codex, Claude, Gemini, and Next manifests from
   `content/distribution/plugins.yaml`.
5. Copy the literature MCP runtime from `packages/qiongli-literature-mcpb/server/`
   into bundled plugin outputs.
6. Rewrite skill names and MCP server names for prerelease variants.
7. Validate the staged artifact.
8. Publish or package only from the staged artifact.

Feature PRs should use `--out /tmp/...` or another temporary staging directory.
Release commands may create ignored local generated outputs, but those outputs
must not be committed.

## Repository Boundary Changes

The following paths should be removed from tracked source and added to generated
output classification if any local workflow recreates them:

- `packages/qiongli-plugin/`
- `packages/qiongli-next-plugin/`

If the implementation needs a small non-generated source package for metadata,
use a neutral source-only path such as `content/distribution/` or
`tooling/distribution/`. Do not put installable plugin shapes under `packages/`
as editable source.

The generated-output guard should reject new edits to plugin payload shapes
unless a release override is active. Deletions of old generated mirrors should
be allowed during the migration.

## Test Strategy

Move tests from checked-in plugin package inspection to staged artifact
inspection.

Required test updates:

- Source tree test: assert `packages/qiongli-plugin/` and
  `packages/qiongli-next-plugin/` are not tracked.
- Metadata test: parse `content/distribution/plugins.yaml` and verify stable and
  prerelease plugin definitions.
- Manifest generation test: materialize to a temp directory and assert generated
  Codex, Claude, Gemini, and Next manifests match expected metadata and workflow
  version.
- Wrapper generation test: materialize to a temp directory and assert wrappers
  exist for every canonical workflow.
- MCP bundling test: assert plugin artifacts bundle MCP runtime files copied from
  `packages/qiongli-literature-mcpb/server/`.
- Routing contract test: assert generated artifacts contain routing and grill
  content copied from canonical `content/`.
- Release verification test: read generated staging paths, not deleted
  `packages/qiongli-*plugin` paths.
- Guard test: reject changed generated plugin artifact paths and allow canonical
  metadata/content edits.

This keeps install validation intact while removing duplicate source files from
review.

## Release And Install Behavior

Release automation should materialize plugin artifacts into a release staging
directory and build uploadable artifacts from there.

If a platform needs a Git-backed directory install, CI should publish a generated
artifact branch or release asset. The main development branch should not keep
that install shape as source.

Local maintainers should use:

```bash
python3 scripts/materialize_distribution_payloads.py --target plugin --out /tmp/qiongli-plugin --force
```

or:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

for validation. In-place generation remains an explicit release or maintenance
operation and should write only to ignored generated-output paths.

## Migration Plan

1. Add canonical distribution metadata under `content/distribution/`.
2. Teach plugin artifact builders to generate all platform manifests and command
   wrappers from canonical metadata and workflows.
3. Change materialization scripts so stable and prerelease plugin artifacts are
   produced only under staging or ignored generated-output paths.
4. Update `RepoLayout` and generated-output path classification to remove
   tracked plugin package source directories.
5. Update tests to validate staged artifacts instead of checked-in plugin
   mirrors.
6. Update release, preflight, postflight, and tag verification scripts to use
   materialized staging paths.
7. Delete tracked `packages/qiongli-plugin/` and `packages/qiongli-next-plugin/`
   trees.
8. Update docs to state that feature PRs must not include plugin payload or
   wrapper diffs.

## Risks

- Some marketplace flow may currently assume a checked-in plugin directory.
  Mitigation: validate the exact marketplace artifact from staging and, if
  required, publish generated artifacts outside the source branch.
- The first migration will delete many tracked files. Mitigation: keep the
  implementation mechanical, preserve validation coverage, and review generated
  artifact diffs in temporary staging rather than source.
- Metadata generation could accidentally diverge between platforms. Mitigation:
  use one metadata source and manifest generation tests for every supported
  platform.
- In-place release generation could accidentally be committed. Mitigation:
  expand generated-output guards and source-tree tests.

## Acceptance Criteria

- Daily content changes for workflow, skill, routing, grill, or Stage I behavior
  touch only canonical `content/` files and relevant tests/docs.
- No tracked files remain under `packages/qiongli-plugin/` or
  `packages/qiongli-next-plugin/`.
- Plugin manifests, command wrappers, skill payloads, and bundled MCP runtime are
  generated from canonical sources.
- Staged Codex, Claude, Gemini, Next, npm, Python, and CLI artifacts preserve
  current install behavior.
- Tests validate generated artifacts from temporary staging directories.
- Release scripts build from staged artifacts and no longer require checked-in
  plugin package mirrors.
- Generated-output guards prevent reintroducing plugin payload source drift.
