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

## Generated outputs

Do not edit materialized copies directly. They are generated from canonical
source during local checks, CI, release staging, and package publishing.

- `qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`
- `plugins/qiongli/skills/qiongli-workflow/`
- `qiongli-workflow/skills/`
- `qiongli-workflow/templates/`
- `qiongli-workflow/standards/`
- `qiongli-workflow/roles/`
- `qiongli-workflow/venue-profiles/`
- `qiongli-workflow/skills-core.md`
- `qiongli-workflow/skills-summary.md`

Feature PRs should not commit generated outputs. GitHub Actions may materialize payloads in a temporary workspace to validate packaging. Release automation may materialize payloads in a staging workspace before building artifacts.

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

Do not edit materialized copies under `qiongli/payload/`,
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

If the change affects packaging, run the generated payload audit in a local
staging workspace. Future automation should use the unified materialization
entrypoint so generated files are not committed back to feature branches.
