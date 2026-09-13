# Source and editing conventions

Qiongli 2 shares research guidance across CLI, Plugin/Skills and MCP. Keep each
behaviour in its existing owner so the entry points stay consistent.

| Source | Responsibility |
|---|---|
| `content/` | Research Skills, workflows, templates, subject guidance and public contracts |
| `packages/qiongli-native/` | Rust CLI, project services, Graph, MCP and native installation |
| `tooling/scripts/` | Maintainer automation, generation and packaging |
| Root `scripts/` | Stable wrappers around maintainer implementations |
| `docs/` and `docs/zh/` | English and Chinese documentation |

For a cross-layer change, establish the shared contract first, change the
implementation that owns it, then rebuild affected outputs. Do not edit
`qiongli-workflow/`, `.agent/`, generated Plugin trees, package payloads or installed
Host caches as source. The retained Python and npm product trees belong to the
1.x compatibility line; native packaging has separate owners.

Create a new Skill only for a distinct reusable task. Otherwise extend an existing
Skill, stage reference, template or subject profile. Keep stable task IDs, artifact
paths, citekeys and source anchors when revising guidance. A model's freedom to
choose a method does not remove approval or revision checks.

In user documentation, name the action first and explain its result. Distinguish
CLI installation, Plugin registration, session tools and online services. Use
current native commands, keep both languages aligned, and label historical guides
with their version. Update a generated document's source, then regenerate it.
Humanizer edits should improve flow while preserving requirements and limitations.

See [repository structure](/development/repository-structure),
[extension guidance](/advanced/extend-qiongli) and
[release policy](/maintainer/release-branch-policy).
