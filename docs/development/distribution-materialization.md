# Distribution Materialization

Qiongli keeps one editable source tree and materializes installable package
payloads from that source. Development changes should happen in canonical
source paths. Generated outputs exist to make PyPI, npm, plugin, and
subject-specific installs self-contained.

## Canonical source

Edit these files when changing the academic workflow:

- `qiongli-workflow/` for the package shell, `SKILL.md`, `VERSION`,
  workflows, references, and agents.
- `skills/` for reusable academic skills.
- `templates/` for output templates and task packet templates.
- `standards/` for machine-readable contracts and policies.
- `roles/` for agent role definitions.
- `venue-profiles/` for venue metadata.
- `subjects/` for subject catalogs, subject skills, overlays, and
  subject-specific venue profiles.
- `skills-core.md` and `skills-summary.md` for top-level skill references.

## Repository structure

The repository intentionally separates the clean checkout source tree from
installable package shapes.

Clean checkout `qiongli-workflow/` shape:

- `qiongli-workflow/SKILL.md`
- `qiongli-workflow/VERSION`
- `qiongli-workflow/agents/`
- `qiongli-workflow/references/`
- `qiongli-workflow/workflows/`

The clean checkout does not contain `qiongli-workflow/templates/`,
`qiongli-workflow/standards/`, `qiongli-workflow/roles/`, or
`qiongli-workflow/venue-profiles/`. Those package mirror directories are
created only by materialization commands when a self-contained skill package
is needed.

Treat root-level directories as the source of truth for duplicated domains:
`templates/`, `standards/`, `roles/`, `venue-profiles/`, and `skills/`.
Generated package copies stay out of the repository checkout and are
materialized only for validation or packaging.

## Generated and materialized outputs

Do not edit materialized copies directly. They are generated from canonical
source during local checks, CI, release staging, and package publishing.

- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`
- `plugins/qiongli/skills/qiongli-workflow/`

In-place materialization can also create ignored package mirror paths under
`qiongli-workflow/`, including `qiongli-workflow/skills/`,
`qiongli-workflow/templates/`, `qiongli-workflow/standards/`,
`qiongli-workflow/roles/`, `qiongli-workflow/venue-profiles/`,
`qiongli-workflow/skills-core.md`, and `qiongli-workflow/skills-summary.md`.
These paths are not part of the clean checkout source tree.

Feature PRs should not commit generated outputs. GitHub Actions may materialize payloads in a temporary workspace to validate packaging. Release automation may materialize payloads in a staging workspace before building artifacts.

## Source-only feature development

Normal feature PRs should update canonical source, tests, and documentation
only. Only canonical source belongs in review; package mirror directories are
not review targets, and generated payload diffs should be treated as a signal
that the source change was made in the wrong place or that local cleanup is
needed.

When a feature affects installable packages, validate the package contract by
materializing into a staging directory and running package tests against that
staged tree. Do not copy those staged files back into the source checkout.

## Output-free checkout

Generated outputs are intentionally untracked. A clean checkout remains
output-free, while release artifacts keep the same installed structure by
materializing payloads during local checks, CI, release staging, and package
publishing.

If local maintenance creates in-place outputs, preview cleanup first:

```bash
python scripts/clean_generated_outputs.py --dry-run
```

Then remove only ignored generated output paths:

```bash
python scripts/clean_generated_outputs.py --apply
```

## Adding a new skill

Use the root `skills/` tree as the source of truth.

1. Create the skill file under `skills/<stage>/<skill-id>.md`.
2. Add its metadata to `skills/registry.yaml`.
3. If the skill should appear in subject installs, reference its id from
   `subjects/catalog.yaml` in the relevant subject group.
4. Add or update tests for the registry, section contract, and subject
   materialization behavior.
5. Run materialization checks locally if the change affects installable
   packages.

Do not edit materialized copies under `packages/python-qiongli/src/qiongli/payload/`,
`packages/npm-qiongli/payload/`, `packages/npm-qiongli/python-runtime/`,
`plugins/qiongli/skills/qiongli-workflow/`, or mirrored directories under
`qiongli-workflow/`.

## Adding or extending a subject package

Use `subjects/` as the source of truth for subject-specific behavior.

1. Add or update the subject definition in `subjects/catalog.yaml`.
2. Put subject-specific skills in `subjects/<subject-id>/skills/`.
3. Put subject-specific overlay content in `subjects/<subject-id>/overlays/`.
4. Add venue profiles under `subjects/<subject-id>/venue-profiles/` when the
   subject needs target-journal or conference guidance.
5. Use `complete` coverage for full subject installs and `focused` coverage
   for slim subject packages.
6. Validate one subject package manually when changing catalog semantics with
   `python scripts/materialize_subject_package.py`:

```bash
python scripts/materialize_subject_package.py \
  --subject <subject-id> \
  --coverage complete \
  --source . \
  --out /tmp/qiongli-subject-complete

python scripts/materialize_subject_package.py \
  --subject <subject-id> \
  --coverage focused \
  --source . \
  --out /tmp/qiongli-subject-focused
```

Subject packages are outputs of the materializer, not independent source
trees. If a materialized subject package looks wrong, fix the source catalog,
skill, overlay, template, or materializer logic.

## Local validation

For ordinary feature work, run source-level tests first:

```bash
python -m unittest tests.test_subject_catalog tests.test_subject_materializer
```

If the change affects packaging, materialize into a local staging workspace:

- `python scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force`
- `python scripts/materialize_distribution_payloads.py --target plugin --out /tmp/qiongli-plugin --force`

Use `python scripts/materialize_distribution_payloads.py --target all --in-place`
only for explicit release or maintenance work. Feature PRs should prefer
`--out` so generated files are not committed back to feature branches.
