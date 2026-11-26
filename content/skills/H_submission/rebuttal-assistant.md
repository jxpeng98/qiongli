---
id: rebuttal-assistant
stage: H_submission
description: "Convert reviewer comments into an actionable revision plan, point-by-point response matrix, and formal response letter."
inputs:
  - type: ReviewerComments
    description: "Supplied reviewer points and the editor decision when applicable"
  - type: Manuscript
    description: "Relevant manuscript version or passages for verifying changes"
outputs:
  - type: ResponseToReviewers
    artifact: "revision/response_matrix.md"
  - type: ResponseLetter
    artifact: "revision/response_letter.md"
constraints:
  - "Must address every single reviewer point — no silent ignoring"
  - "Must distinguish criticism from suggestion from optional comment"
  - "Must maintain respectful, constructive tone even for hostile reviews"
failure_modes:
  - "Dismissive responses that antagonize reviewers"
  - "Over-promising changes that cannot be delivered"
  - "Missing implicit concerns behind explicit questions"
tools: [filesystem]
tags: [submission, rebuttal, revision, reviewer-response, R&R]
domain_aware: false
---

# Rebuttal Assistant Skill

Use the H2 contract in `references/stage-H-submission.md` for revision status, manuscript versions and submission boundaries.

## Purpose

Convert reviewer comments into an actionable revision plan, point-by-point response matrix, and formal response letter.

## Related Task IDs

- `H2` (rebuttal / response)

## Output (contract path)

- `RESEARCH/[topic]/revision/response_matrix.md`
- `RESEARCH/[topic]/revision/response_letter.md`

## When to Use

- After receiving a Revise & Resubmit (R&R) decision
- After a conditional accept with minor/major revisions
- When preparing the response matrix before making changes

## Inputs

Use the actual comments, any editor priorities, author decisions and the
manuscript passages needed to check a response. A single supplied comment can
receive a standalone reply. If a claimed edit or analysis cannot be verified,
draft the supported answer and identify the gap for the author. Ask for a
missing comment before attributing a concern to a reviewer.

## Process

1. For the requested scope, split compound comments into addressable points.
   Keep their wording, reviewer, round and stable IDs. Classify requests in
   light of the editor's instructions. Distinguish the reviewer's stated
   concern from your interpretation of why it matters.
2. Choose how to respond to each point. An evidenced correction, clarification
   of the current text, reasoned disagreement or explanation of a constraint
   can each answer a concern. Confirm any new commitment with the author.
3. In the letter, identify the concern and give the answer in terms a reviewer
   can understand without the chat or internal planning notes. Cite verified
   manuscript locations for completed changes. Keep proposed analyses,
   completed analyses and their results distinct.
4. For a package, reconcile the response matrix and letter against the
   supplied comments and the same manuscript version. Record any missing input
   or unverified change for follow-up. Apply the venue's markup instructions
   when preparing the revised manuscript.

## Output Contract

- The formal `ResponseToReviewers` matrix uses `templates/rebuttal-response-matrix.md`
  at `RESEARCH/[topic]/revision/response_matrix.md`.
- The formal `ResponseLetter` uses `templates/rebuttal-letter.md` at
  `RESEARCH/[topic]/revision/response_letter.md`. A separate revision plan
  belongs at `RESEARCH/[topic]/revision/revision_plan.md` when needed.
- Separate finding, interpretation, and implication when answering a scholarly
  challenge. Do not invent citations, data, sample sizes, statistical results,
  or reviewer comments. Use `references/academic-output-rubric.md`.
- For a project response that changes central claims, update the existing
  evidence ledger under `references/evidence-ledger-contract.md`. Follow
  `references/citation-risk-policy.md` when a response introduces citation risk.

## Quality Bar

Each supplied point has an answer or an identified author decision. For a
complete package, the matrix and letter agree about what changed, why and
where. Every claim of completed work can be verified against the revision or
analysis record. Identify unresolved items before describing a letter as ready.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Thanking the reviewer without answering the question | State the substantive answer and its basis. |
| Promising a new analysis on the author's behalf | Explain the current evidence and keep the possible analysis pending author agreement. |
| Reporting an edit based on a planned change or sample letter | Check the current text and location before claiming completion. |
