# Subject Packaging Model

Qiongli uses one canonical workflow, a stable full local runtime, and several installable subject packages for compatibility and release artifacts. This page explains the difference between project-level subject guidance, `core`, specialized packages, coverage modes, composites, and local customization from both the user and developer perspectives.

## User Model

For normal CLI/local plugin use, the installed package should remain a stable core/full runtime:

```bash
qiongli install --target all
```

Per-project subject behavior lives in `.qiongli/guidance_manifest.yaml`, not in a different client install. Use:

```bash
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project set-venue journal-of-finance --project-dir .
qiongli project set-method-lens event-study --project-dir .
qiongli project status --project-dir .
```

If the manifest is missing, Qiongli runs with implicit `active_subject: auto`: it uses core guidance, infers temporary subject and method lenses from the current task, and writes auditable proposals before changing project-local state. Persistent subject changes come from explicit project commands or accepted guidance proposals.

The installed skill directory is still `qiongli-workflow`, and each client install has one active package at a time. Subject-specific installs remain important for Desktop ZIPs, focused packages, compatibility testing, and release payloads.

| Situation | Command | Meaning |
|---|---|---|
| Normal CLI/local plugin use | `qiongli install --target all` | Stable full runtime shared across projects |
| A project should use finance guidance | `qiongli project set-subject finance --project-dir .` | Writes project-local subject context |
| No project subject is configured | No command required | Implicit `active_subject: auto` with core guidance and temporary inference |
| Focused Desktop/Web ZIP | `qiongli install --subject economics --coverage focused --target all` | Deliberately slim package under upload limits |
| Compatibility or release payload testing | `qiongli install --subject economics --target all` | Full framework plus economics specialization as an installed package |
| Official cross-disciplinary ZIP/package | `qiongli install --subject economics-accounting --target all` | Full framework plus the economics/accounting composite layer |
| Local generated package rules | `python3 scripts/materialize_subject_package.py --custom-dir <path> ...` | One generated package with local overlays, profiles, or skills |

### Core

`core` is the default subject. It is the general Qiongli workflow for users who do not want to choose a discipline up front.

It provides:

- shared workflow contracts and task IDs
- shared templates, standards, references, and roles
- canonical generic skills
- broad domain and venue profile availability in `complete` coverage

Use `core` when the project is general, exploratory, or not yet clearly tied to one discipline. In normal CLI/local plugin use, this is also the stable installed runtime used with project-level guidance.

### Specialized Subjects

A specialized subject, such as `economics`, `political-economy`, or `geoeconomics`, keeps the same Qiongli workflow but adds a discipline layer. It does not fork the workflow contract.

It can add:

- selected domain and venue profiles
- append overlays for generic skills
- section replacements for specific quality bars or pitfalls
- a small number of subject-specific skills

The default CLI/npm behavior is `coverage=complete`. Therefore:

```text
qiongli install --subject economics
= full core framework + economics overlays + economics-specific skills
```

It is not a reduced package unless you explicitly choose `--coverage focused`.

For everyday project work, prefer `qiongli project set-subject <subject>` over reinstalling a specialized package. Use `qiongli install --subject ...` when you are preparing a Desktop ZIP, deliberately testing a focused or complete package, validating release payloads, or maintaining compatibility with an install surface that needs one active subject package.

### Coverage Modes

`complete` and `focused` answer different user needs.

| Coverage | Best for | Package shape |
|---|---|---|
| `complete` | Users who want the full framework and subject expertise | all core generic skills plus selected subject overlays and subject-specific skills |
| `focused` | Desktop/Web ZIPs, upload limits, deliberate slim installs | selected subject profiles, selected effective skills, and subject-specific skills |

If unsure, use `complete`.

### Composite Subjects

Composite subjects are official cross-disciplinary packages, such as `economics-accounting`.

`composes` is metadata, not automatic union. It records that an official composite subject was designed from component subject expectations, but the composite still declares its own ordered groups, profile selection, overlays, and subject-specific skills.

They are not automatic unions of multiple subjects. A composite must define its own ordered groups, profile selection, overlays, and subject-specific skills. This keeps the package coherent and avoids accidental conflicts between disciplines.

Use a composite when the research question is inherently cross-disciplinary and a single subject would miss important norms.

Use `political-economy` and `geoeconomics` as separate subjects when you want those domains as the active package. They are not a composite pair and should not be installed together in one Desktop/Web ZIP.

### Local Customization

Local customization is for user-specific or lab-specific rules that should not immediately become canonical source. It is applied through the Python materializer:

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

A custom directory may include:

- `subject.yaml`
- `overlays/skills/*.md`
- `skills/registry.yaml`
- `skills/*.md`
- `domain-profiles/*.yaml`
- `venue-profiles/*.yaml`

Customizations only affect the generated output. They do not write back to the canonical repository.

For ordinary project-local guidance, prefer `.qiongli/guidance_manifest.yaml`, `.qiongli/local_guidance.md`, and `.qiongli/guidance.d/*.md`. The materializer path is for generated packages and release-style payloads.

## Developer Model

The source tree is layered. Generic workflow behavior stays canonical; subject depth is added through overlays and selected subject assets.

| Layer | Source location | Purpose | Enters `core`? |
|---|---|---|---|
| Workflow contract | `qiongli-workflow/`, `standards/`, `references/` | shared task IDs, outputs, quality gates | yes |
| Generic skills | `skills/*/*.md` | canonical cross-disciplinary capability cards | yes |
| Generic profiles | `skills/domain-profiles/*.yaml`, `venue-profiles/*.yaml` | reusable domain and venue metadata | materialized into core packages |
| Subject catalog | `subjects/catalog.yaml` | ordered subject groups and selected assets | no direct runtime behavior without materialization |
| Subject overlays | `subjects/<subject>/overlays/skills/*.md` | append or replace declared skill sections | no |
| Subject skills | `subjects/<subject>/skills/*.md` | small number of discipline-specific skills | no |
| Composite subject | `subjects/<composite>/...` | official cross-disciplinary package | no |
| Local custom layer | `--custom-dir` | user/lab-specific generated package layer | no |

### What Belongs In Core

Put content in `core` when it is broadly valid across many disciplines:

- shared research workflow contracts
- generic task definitions
- cross-disciplinary quality gates
- reusable writing, literature, design, code, and compliance skills
- profiles or templates that many subjects can reasonably reuse

Core should remain stable, general, and useful when the user has not chosen a subject.

### What Belongs In A Subject

Put content in a subject when it meaningfully changes discipline behavior:

- method-specific diagnostics
- discipline-specific failure modes
- venue norms and evidence standards
- field-specific data, construct, or identification requirements
- stricter quality bars for selected skills
- a skill that only makes sense inside that discipline

Subject-specific content should usually be an overlay or profile update first. Create a new subject-specific skill only when the workflow really needs a distinct expert capability.

### What Must Not Happen

Avoid these patterns:

- copying a generic skill into `subjects/<subject>/skills/`
- creating `econ-*` versions of generic skills just to change examples
- using `replace_sections` without declaring exact section names
- changing workflow contracts for a single discipline unless the contract is truly general
- treating a composite subject as a blind union of two subject registries
- writing local customizations back into canonical source by accident

### Promotion Rule

Subject content can move toward core only after it proves general value.

Use this rule:

```text
If the rule is useful across disciplines, promote it to core.
If the rule is true only in one discipline, keep it in the subject.
If the rule is local to a lab, project, or user, keep it in custom-dir.
```

### Adding Or Deepening A Subject

Each official subject should provide:

- ordered catalog groups in `subjects/catalog.yaml`
- selected domain profiles and venue profiles
- overlays for generic skills that need discipline constraints
- minimal subject-specific skills only where overlays are insufficient
- materializer tests for `complete` and `focused` coverage
- release validation for Desktop/Web file budget when the subject has a ZIP
- documentation examples for install, switching, and customization

This structure keeps subject packages specialized without fragmenting the canonical workflow.
