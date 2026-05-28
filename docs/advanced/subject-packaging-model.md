# Subject Packaging Model

Qiongli uses one canonical workflow and several installable subject packages. This page explains the difference between `core`, specialized subjects, coverage modes, composites, and local customization from both the user and developer perspectives.

## User Model

The installed skill directory is always `qiongli-workflow`, and each client has one active package at a time. Changing the subject or coverage means rerunning `install` or `upgrade` with new flags.

| Situation | Install | Meaning |
|---|---|---|
| You are unsure what to choose | `qiongli install --target all` | Default `core / complete`: full general Qiongli framework |
| You know the work is economics | `qiongli install --subject economics --target all` | Full framework plus economics specialization |
| You want a slim economics package | `qiongli install --subject economics --coverage focused --target all` | Economics-selected profiles, overlays, and active effective skills |
| You need an official cross-disciplinary package | `qiongli install --subject economics-accounting --target all` | Full framework plus the economics/accounting composite layer |
| You want local custom rules | `python3 scripts/materialize_subject_package.py --custom-dir <path> ...` | One generated package with local overlays, profiles, or skills |

### Core

`core` is the default subject. It is the general Qiongli workflow for users who do not want to choose a discipline up front.

It provides:

- shared workflow contracts and task IDs
- shared templates, standards, references, and roles
- canonical generic skills
- broad domain and venue profile availability in `complete` coverage

Use `core` when the project is general, exploratory, or not yet clearly tied to one discipline.

### Specialized Subjects

A specialized subject, such as `economics`, keeps the same Qiongli workflow but adds a discipline layer. It does not fork the workflow contract.

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

## Developer Model

The source tree is layered. Generic workflow behavior stays canonical; subject depth is added through overlays and selected subject assets.

| Layer | Source location | Purpose | Enters `core`? |
|---|---|---|---|
| Workflow contract | `qiongli-workflow/`, `standards/`, `references/` | shared task IDs, outputs, quality gates | yes |
| Generic skills | `skills/*/*.md` | canonical cross-disciplinary capability cards | yes |
| Generic profiles | `skills/domain-profiles/*.yaml`, `qiongli-workflow/venue-profiles/*.yaml` | reusable domain and venue metadata | available to core |
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
