# Skill Quality Contract

This contract defines the minimum structure and behavioral clarity required for canonical academic skills in this repository.

## Required Sections

Every canonical skill under `skills/` should contain these sections:

| Section | Requirement |
|---------|-------------|
| `## Purpose` | State the concrete research task the skill solves in one or two sentences. |
| `## Inputs` | List expected inputs, accepted artifact paths, and behavior when input is missing. |
| `## Process` | Provide executable steps, not broad advice. |
| `## Output Contract` | Name the exact artifact paths and formats the skill must produce. |
| `## Quality Bar` | Define checkable standards for scholarly quality. |
| `## Common Pitfalls` | List likely failure modes and how to correct them. |

## Content Constraints

Each skill should also make these behaviors explicit:

| Constraint | Requirement |
|------------|-------------|
| Canonical artifact path | Use concrete `RESEARCH/[topic]/...` paths or registry-aligned artifact paths. |
| Task/stage relation | Keep the skill tied to its stage and Task ID contract. |
| Evidence handling | Explain how literature, data, citations, or project artifacts support claims. |
| Insufficient input behavior | Write a gap note or ask for missing prerequisites; do not fabricate. |
| Claim strength rules | Separate finding, interpretation, and implication. |
| No hallucinated citations/data | Never invent sources, datasets, sample sizes, statistics, or results. |
| Platform-neutral wording | Avoid hard-coding Codex, Claude Code, Gemini, or slash-command assumptions inside skill logic. |

## Validation

Use the audit script for the skill quality baseline:

```bash
python3 scripts/audit_skill_sections.py --output docs/maintainer/skill-quality-gap-report.md
python3 scripts/audit_skill_sections.py --strict
```

`--strict` returns non-zero until every canonical skill satisfies this contract. That gate is intentionally separate from the release validator until the staged migration is complete.

## Upgrade Order

Use `docs/maintainer/skill-quality-gap-report.md` to prioritize work:

1. Core stages: `A_framing`, `B_literature`, `F_writing`, `J_proofread`, `I_code`.
2. Design and synthesis: `C_design`, `D_ethics`, `E_synthesis`.
3. Submission, presentation, cross-cutting, and domain profile support.

For each batch, update source files under `skills/`, run the audit, then use
staged package validation to confirm the portable and plugin package shapes:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
python3 scripts/audit_distribution_payloads.py --root /tmp/qiongli-dist
```

Do not review or commit generated package mirrors. If the staged package looks
wrong, fix the canonical skill source, registry metadata, or materializer logic.
