---
id: boundary-interviewer
stage: Z_cross_cutting
description: "Clarify scholarly boundaries one question at a time before high-risk Qiongli tasks proceed."
inputs:
  - type: UserQuery
    description: "User request or task packet with unresolved academic scope, claim, evidence, method, venue, or handoff boundaries"
  - type: AnyArtifact
    description: "Existing research artifacts that may already settle the academic boundary"
outputs:
  - type: BoundaryReview
    artifact: "context/boundary_review.md"
constraints:
  - "Must inspect available academic artifacts before asking the user"
  - "Must ask only one scholarly boundary question at a time"
  - "Must provide a recommended answer with academic rationale and downstream impact"
failure_modes:
  - "Treating the process as generic software requirements gathering"
  - "Asking broad questionnaires instead of the next blocking scholarly question"
  - "Allowing overclaiming by failing to set evidence thresholds"
  - "Flattening uncertainty instead of preserving rival explanations and reviewer risks"
tools: [filesystem]
tags: [cross-cutting, boundary, grill-me, academic-judgment, claim-strength, evidence-threshold, validity, generalizability, venue-fit, handoff]
domain_aware: true
---

# Boundary Interviewer Skill

Clarify scholarly boundaries before high-risk Qiongli work proceeds.

## Purpose

Use this skill when a task could drift in research scope, claim strength, evidence threshold, method validity, writing promise, or submission commitment. It adapts the grill-me pattern into an academic workflow: inspect existing artifacts first, ask one blocking question at a time, recommend the most defensible answer, and preserve the decision for downstream stages.

The skill is not a generic requirements interview. It should help answer: what can this project honestly claim, for whom, with what evidence, against which rivals, and under what limits?

## Academic Grill Loop

The Academic Grill Loop is Qiongli's academic idea discovery and boundary-critique adaptation of the grill-me interaction pattern. It is not a generic grill-me clone: the loop must convert a broad topic or early idea into a defensible scholarly choice by testing claim strength, evidence threshold, rival explanations, feasibility, venue expectations, and generalizability.

Credit: the interaction pattern is inspired by Matt Pocock's `grill-me` skill at `https://github.com/mattpocock/skills/blob/main/skills/productivity/grill-me/SKILL.md`. Qiongli changes the substance of the loop for academic work: questions are paper-type aware, evidence-bound, reviewer-sensitive, and recorded as reusable research constraints.

Use the Academic Grill Loop when the user is brainstorming, selecting among research ideas, starting Stage A, running `/find-gap`, or entering a later stage with an idea whose academic boundary is still unclear. The loop is:

1. Inspect artifacts first: `research_state`, `decision_log`, RQ, contribution, gap analysis, design, claim map, and handoff notes.
2. Choose the smallest scholarly uncertainty that would most change the idea, claim, method, or venue position.
3. Ask one scholarly question at a time.
4. Provide a recommended answer with academic rationale, expected evidence basis, claim-strength implication, reviewer or venue consequence, and confidence.
5. Record the answer as a boundary decision, including what would force the idea to narrow or be revisited.

For Stage A idea-discovery, prefer questions that move from "interesting topic" to "publishable research idea":

- What evidence would make this idea answerable in one paper?
- Which population, setting, time period, or corpus is out of scope?
- Which construct definition makes the idea testable or interpretable?
- What contribution type is primary, and which adjacent contribution is not being claimed?
- Who would cite this work if it succeeds, and why would they care?

## When to Use

- Before high-risk framing, study design, writing, submission, research code, or handoff work when scope, claim strength, evidence threshold, validity risk, generalizability, or reviewer expectations remain unclear.
- When `context/boundary_review.md` is missing, stale, or contradicted by the current task.
- When broadening a previous boundary would change downstream artifacts, claims, evidence thresholds, code decisions, or submission promises.
- When a vague topic or brainstorm needs to become a defensible academic idea before `A1`, `A2`, `A4`, or `A5` artifacts are drafted.

## Related Task IDs

- MVP trigger stages: `A`, `C`, `F`, `H`, `I`
- Future trigger stages: `B`, `D`, `E`, `G`, `J`, `K`

## Output (contract path)

- `RESEARCH/[topic]/context/boundary_review.md`

## Inputs

- `UserQuery`: Current request, workflow command, or task packet.
- `AnyArtifact`: Existing artifacts that may already define the boundary:
  - `context/research_state.md`
  - `context/decision_log.md`
  - `context/stage_handoff.md`
  - `framing/research_question.md`
  - `framing/contribution_statement.md`
  - `theoretical_framework.md`
  - `gap_analysis.md`
  - `study_design.md`
  - `analysis_plan.md`
  - `design/validity-threat-matrix.md`
  - `manuscript/claims_evidence_map.md`
  - `revision/response_matrix.md`
  - `code/code_specification.md`
- If a required input is missing, record the missing artifact and ask only the next blocking academic question. Do not invent citations, findings, sample sizes, reviewer expectations, or institutional requirements.

## Process

1. Map the task to an academic boundary dimension from `standards/boundary-review-contract.yaml`.
2. Inspect existing artifacts before asking the user. If the boundary is already settled, cite the artifact and record the decision.
3. Choose the next blocking question. Prefer the question whose answer would most change the research question, claim strength, evidence threshold, method validity, or submission position.
4. Ask exactly one user-facing question.
5. Include a recommended answer with:
   - academic rationale
   - expected evidence basis
   - claim-strength implication
   - likely reviewer or venue consequence
   - confidence
6. Record the answer in `context/boundary_review.md`.
7. Preserve the boundary between finding, interpretation, and implication when the question affects writing claims.
8. Preserve rival explanations, null cases, contradictory evidence, or trustworthiness risks when the question affects design or synthesis.
9. Sync locked decisions into `context/decision_log.md`; sync unresolved risks into `context/stage_handoff.md`.

## Downstream Continuation

After the user answers a boundary question, write or update `context/boundary_review.md` and continue within the locked boundary for the current task. Later Qiongli skills must treat that artifact as a constraint:

- They may narrow scope, claim strength, evidence thresholds, method commitments, code/data decisions, or submission/presentation promises.
- They must not broaden a locked boundary without adding a new `revisit_trigger`, the new evidence or user decision that justifies the change, and the affected downstream artifacts.
- When a later task conflicts with the boundary review, ask the smallest necessary follow-up question before continuing.

## Output Contract

Write `RESEARCH/[topic]/context/boundary_review.md` using `templates/boundary-review.md`.

The artifact must include:

- scholarly decision context
- artifacts checked before asking the user
- one-question academic loop
- academic boundary map
- claim strength and evidence threshold
- rival explanations and counterevidence
- validity or trustworthiness risk
- generalizability limit
- venue or reviewer risk
- locked decision
- revisit trigger
- downstream sync targets

## Quality Bar

- [ ] The question targets a named academic boundary dimension.
- [ ] Existing artifacts were inspected before asking the user.
- [ ] No more than one user-facing question was asked at a time.
- [ ] Every question included a recommended answer and academic rationale.
- [ ] Claim strength is explicit: descriptive, interpretive, associative, causal, predictive, methodological, normative, or exploratory.
- [ ] Evidence threshold states what would support, weaken, or falsify the claim.
- [ ] Rival explanations or counterevidence are named when material.
- [ ] Validity, trustworthiness, or reproducibility risks are preserved instead of smoothed over.
- [ ] Generalizability limits are concrete enough for a reviewer to evaluate.
- [ ] Downstream updates identify `decision_log`, `stage_handoff`, claim map, validity matrix, or code specification impacts.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Treating the skill like software requirements gathering | The output locks implementation details but misses scholarly risk | Ask about claim, evidence, method validity, and reviewer consequence |
| Asking a long questionnaire | The user answers mechanically and the model avoids judgment | Ask the next blocking academic question only |
| Asking what artifacts already answer | The workflow ignores its own evidence chain | Inspect research state, decision log, handoff, claim map, and design artifacts first |
| Overclaiming by default | The manuscript later exceeds evidence | Force claim-strength and evidence-threshold fields |
| Hiding rival explanations | Reviewers surface them later as fatal flaws | Record rival explanations, null cases, and contradictory evidence early |
| Treating limitations as cosmetic | Boundaries become boilerplate | State exact population, setting, data, measure, model, or venue limits |
