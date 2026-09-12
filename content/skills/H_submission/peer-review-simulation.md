---
id: peer-review-simulation
stage: H_submission
description: "Review a manuscript through relevant referee lenses, grounding findings in evidence and distinguishing simulation from actual independent review."
inputs:
  - type: Manuscript
    description: "Draft manuscript for simulated review"
outputs:
  - type: PeerReviewSimulation
    artifact: "revision/peer_review_simulation.md"
constraints:
  - "Must disclose actual review participants; simulated lenses are not independent reviewers"
  - "Must aggregate and reconcile conflicting feedback"
  - "Must produce actionable items, not vague criticism"
failure_modes:
  - "Personas converge to similar critique (lack of diversity)"
  - "Missing domain expertise for specialized methods"
  - "Reviews are too positive (false reassurance)"
tools: [filesystem]
tags: [submission, peer-review, simulation, multi-persona, red-team]
domain_aware: false
---

# Peer Review Simulation Skill

## Purpose

Assess a manuscript through relevant referee lenses. Ground findings in evidence
and distinguish a simulated critique from actual independent review or an editor's
decision. A focused check need not become a full review panel.

## Related Task IDs

- `H3` (peer review simulation)

## Output (contract path)

- `RESEARCH/[topic]/revision/peer_review_simulation.md`

## When to Use

- The author requests pre-submission critique or a formal H3 review.
- A specific method, claim or section needs referee-style scrutiny.
- For an actual confidential journal review, first establish the applicable
  confidentiality and AI-use permission under the shared contract.

Explaining a paper belongs to reading; responding to actual received comments
belongs to revision/rebuttal work. Reviewing does not automatically authorize either
manuscript edits or submitting the review.

## Inputs

Use the supplied manuscript/version, review scope, target/type/stage and prior
findings. State which text, supplementary material and underlying evidence were
actually inspected. Missing raw data or an unstated procedure is an evidence gap,
not proof of an invalid study. A narrow answer may stay in chat.

## Process

Follow `references/stage-H-submission.md` for applicable criteria, issue severity,
confidentiality, source handling and formal write boundaries.

### 1. Choose the review scope

Start with the active model and the lenses relevant to the claim and design. A
full H3 report covers methods/evidence, contribution/positioning and internal
consistency; a focused request uses only its relevant lens. One model's personas
remain simulated self-review, not independent reviewers.

When independent review is requested or required, follow
`skills/Z_cross_cutting/model-collaborator.md` with available authorized reviewers.
Record actual participants and source versions. If unavailable, leave that
requirement unresolved. Existing BLOCK findings, protocols and formal minimum
independent-review counts remain binding.

### 2. Examine the evidence using applicable standards

Choose questions that could change the assessment; these are examples, not a
mandatory checklist for every discipline or article type:

| Lens | Relevant questions |
|---|---|
| Quantitative / computational | Does the design support the stated estimand or prediction? Are assumptions, measurement, uncertainty, data handling and needed diagnostics justified? For prediction, check leakage and evaluation design where applicable. |
| Qualitative / mixed methods | Do sampling, interpretation and reflexivity fit the approach? Are claims traceable to material and contrary cases? Does integration support mixed-method conclusions? |
| Theory / humanities / conceptual | Are premises, sources, interpretation and argumentative steps adequate? Are rival readings and the scope of the claim addressed? Do not require empirical hypotheses for a nonempirical argument. |
| Evidence synthesis | Does the stated review type justify selection and synthesis? Apply relevant search/reporting standards without imposing systematic-review requirements on every review. |
| Protocol / registered report | Assess the proposed question/design at this stage; do not demand completed results or retroactive preregistration. |
| Contribution / readers | Does the actual contribution address the venue's published criteria? Distinguish novelty, replication, null findings, synthesis and practical value rather than assuming a novelty hierarchy. |
| Consistency / reporting | Do abstract, claims, methods, results, figures and limitations agree? Are required statements and relevant evidence accessible or truthfully restricted? |

Use applicable domain/method guidance when needed. Name expertise or access limits.
Do not apply generic citation-age, sample-size, missing-data percentage or extra
robustness quotas. A formatting preference is not a scientific flaw.

### 3. Ground and reconcile findings

For every material issue, identify the manuscript location, supporting passage or
result, applicable criterion, consequence and proportionate remedy. Check whether
another section, supplement or defensible methodological choice answers the concern.
Label uncertain questions; do not inflate them to findings. Keep strengths where
they inform the assessment, without inventing praise or issue counts.

| Issue ID | Source / reviewer / lens | Location and evidence | Criterion / consequence | Severity | Remedy / uncertainty |
|---|---|---|---|---|---|---|

Deduplicate by the underlying issue, preserving its source and prior ID across
revisions. Resolve disagreement against the same manuscript and criteria, not vote
counts. A substantiated blocker remains blocking even if only one reviewer raises
it. If the evidence cannot decide a disagreement, record what would resolve it.

### 4. Return the requested assessment

Prioritize actual findings and state readiness only within completed checks.
A clean review may have zero findings. List unavailable checks and any independent
review requirement separately. A suggested new experiment is new work, not an
assumed prerequisite merely to strengthen a paper. Stop at the review unless the
user also requested revisions; an ordinary review needs only targeted verification
of subsequent fixes, not repeated unchanged panels.

## Output Contract

Formal H3 writes `RESEARCH/[topic]/revision/peer_review_simulation.md` with:

- Review basis: manuscript version/anchors, target/type/stage and applicable sources.
- Actual participants, selected lenses, self-review status and access limits.
- Source-bound findings, strengths and reconciliation table.
- Prioritized actions, unresolved checks and scoped readiness.

Follow the shared contract for evidence-ledger and material citation-risk updates.
Apply `references/academic-output-rubric.md`. Never invent reviewer comments,
independent participants, data, citations or journal decisions.

## Quality Bar

- [ ] Requested scope completed; applicable method and venue criteria are explicit.
- [ ] Findings have evidence, locations, consequences and proportional remedies.
- [ ] Missing reporting, scientific flaws, venue mismatch and preferences are distinct.
- [ ] Actual independence requirements are met or visibly unresolved.
- [ ] Contradictions and counterevidence were considered; repeated votes are not proof.
- [ ] Readiness claims match the evidence and do not predict acceptance.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Make every manuscript look like a quantitative study | Review the actual design and claim |
| Perform a hostile persona | Test plausible objections fairly against the full supplied source |
| Assume an omitted detail means an omitted procedure | Ask for the evidence and distinguish reporting from validity |
| Count simulated reviewers as independent | Disclose actual participants and unmet requirements |
| Recommend cosmetic fixes for a validity problem | Explain the needed reanalysis, new evidence or claim reduction |
