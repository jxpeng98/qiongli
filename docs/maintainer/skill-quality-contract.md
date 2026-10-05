# Skill Quality Contract

This contract defines the minimum structure and behavioral clarity required for canonical academic skills in this repository.

## Required Sections

Every canonical skill under `content/skills/` should contain these sections:

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
| Platform-neutral wording | Avoid hard-coding a specific client name or slash-command assumption inside skill logic. |

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

For each batch, update source files under `content/skills/`, run the audit, then use
staged package validation to confirm the portable and plugin package shapes:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
python3 scripts/audit_distribution_payloads.py --root /tmp/qiongli-dist
```

Do not review or commit generated package mirrors. If the staged package looks
wrong, fix the canonical skill source, registry metadata, or materializer logic.

## Quality evidence levels

Report these separately; passing one does not imply passing another:

| Level | Evidence owner | What it establishes |
|---|---|---|
| Structure | `audit_skill_sections.py` | Required headings and textual markers are present; not semantic completeness. |
| Tool and data contracts | Focused native/service/package checks | Specified behavior and permission, compatibility and data-loss negatives under those inputs. |
| Actual Host behavior | Isolated installed-Plugin captures | Observed routing, tool calls, completion and recovery for the exact Host, model, candidate and task. |
| Answer quality | Complete answer/source review | Named judgments of source support, numeric accuracy, claim strength and requested scope. |

Use the existing [research journey evaluation](../../evals/research_journey/README.md)
and its fixed three-flow Plugin baseline for public-paper explanation, a supplied-source
paragraph and fresh-session saved-document continuation. The offline adapter reuses
whole-answer span review and Evaluation Truth V1; it never installs a Plugin,
launches a model or approves a project write. Freeze task/source/candidate bindings
before observations and retain failed and unattempted cases in the denominator.
Keep review criteria separate from model prompts. A changed prompt, source or
guidance needs a new capture; do not relabel the old answer as a repaired result.

Use actual trace token counts, tool calls and measured time when present; absent
usage remains unknown. A shorter prompt or fewer calls alone is not a quality gain.
Inspect structural flags in context: a missing literal marker can reflect clearer
equivalent guidance. Do not add filler or weaken a check just to make counts green.
Compare the same tasks and model settings, and retain separate held-out cases
before claiming broader improvement. Expert and additional-Host acceptance need
their own evidence. Structural completeness does not justify an automatic rollout.

Direct explanations and paragraph edits follow the main Skill's bounded scope.
The presence of an output path in a reference card does not require an unsolicited
project, saved artifact, review loop or extra model invocation.
