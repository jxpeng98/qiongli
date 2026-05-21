# Naming Policy

The external product name is **Qiongli** (`穷理`).

The complete system name is **Qiongli Zhengche** (`穷理证澈`).

The methodology and core evidence-governance layer is **Zhengche** (`证澈`).

## Naming Roles

| Scope | Name | Use |
|-------|------|-----|
| Public product, plugin display, docs title | Qiongli / `穷理` | Use for outward-facing discovery, marketplace copy, and general references. |
| Complete system, long-form introduction | Qiongli Zhengche / `穷理证澈` | Use when describing the full research workflow system. |
| Methodology, evidence ledger, citation risk, claim traceability | Zhengche / `证澈` | Use for the core method that keeps claims evidence-grounded and auditable. |
| Technical identifiers | `qiongli` for plugin/PyPI/CLI, `qiongli-workflow` for the portable skill | Use for manifests, install paths, release scripts, and package metadata. |
| Legacy compatibility aliases | `research-skills`, `research_skills`, `rsk`, `rsw` | Keep as compatibility entrypoints during the migration window. |

## Interpretation

`穷理` frames the public identity: scholarly inquiry that traces a question to its underlying logic and evidence.

`证澈` frames the method: make evidence, citations, assumptions, and reasoning clear enough to audit.

Together, `穷理证澈` means the system pursues research questions deeply while keeping the evidence chain transparent.

## Migration Rule

Use Qiongli identifiers for new public and technical surfaces:

- Repository: `https://github.com/jxpeng98/qiongli`
- Plugin ID: `qiongli`
- Portable skill ID: `qiongli-workflow`
- CLI: `qiongli`, `ql`
- Python distribution: `qiongli`

Keep legacy aliases (`research-skills`, `research_skills`, `rsk`, `rsw`) until downstream install, documentation, and user migration windows have settled. Any future alias removal must be a separate breaking-change release with explicit upgrade notes.
