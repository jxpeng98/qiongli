---
description: 从已有研究资料撰写完整论文，保留正式产物、证据映射和适用的报告检查
---

# Paper Writing

Draft the requested manuscript from existing research artifacts. Canonical Task
IDs: `F1` outline, `F3` full draft, `F4` claim-evidence map, `F5` figures/tables.
An outline-only request stops at F1; use `/academic-write` for a single section.

## Target

$ARGUMENTS

## Inputs and scope

Reuse the selected `RESEARCH/[topic]/` folder, paper type, venue constraints and
citation style. Ask only for a consequential missing decision. Do not assume an
empirical design when the supplied materials do not establish the paper type.

Read the artifacts needed for the manuscript: study design and analysis/results
for empirical work; synthesis and available review records for a systematic
review; the relevant argument and source materials for other paper types.
Keep missing results, citations and procedures as gaps. Continue supported
sections; ask before an unresolved choice changes the scope or central claim.
Starting an upstream study or review needs its own task scope.

## Draft and check

Use `manuscript-architect` and the shared Writing Harness Contract in
`references/stage-F-writing.md`. Reuse or refine the Story Spine and outline;
choose drafting order and chunk size from the task's length, dependencies and
evidence risks. An authorized full-draft request does not require another user
confirmation after the outline. Explicit review protocols and project write
approvals still apply.

For the requested formal tasks, use the canonical output paths:

| Task | Output / template |
|---|---|
| F1 | `manuscript/outline.md`; `templates/manuscript-outline.md` |
| F3 | `manuscript/manuscript.md`, `manuscript/results_interpretation.md`, `manuscript/effect_interpretation.md`; `templates/manuscript-skeleton.md` |
| F4 | `manuscript/claims_evidence_map.md`; `templates/claim-evidence-map.md` |
| F5 | `manuscript/figures_tables_plan.md`, `manuscript/tables/`, `manuscript/figures/`; `templates/figures-tables-plan.md` |

Preserve existing prose, stable claim IDs and citation keys. A full paper's
central claims need the F4 integrity check; use F5 when figures/tables belong
to the requested deliverable. Mark non-applicable interpretation outputs with
their basis rather than inventing analyses to populate them.

For formal readiness, run the applicable `reporting-checker` or `prisma-checker`
and record unresolved requirements. The shared Stage F contract owns Q1/Q2
checks. Return the draft and its material gaps; claim readiness only with the
required evidence. Submission packaging remains a separate requested outcome.

Proposed artifacts are not saved files. Apply registered project changes only
through preview/approval/CAS and verify the write result.
