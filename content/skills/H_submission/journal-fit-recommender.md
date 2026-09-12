---
id: journal-fit-recommender
stage: H_submission
description: "Recommend journals from an existing manuscript using venue profile, contribution, methods, evidence, and reviewer-risk fit."
inputs:
  - type: Manuscript
    description: "Current manuscript draft or structured manuscript sections"
  - type: ClaimGraph
    description: "Claim-evidence map for manuscript claims"
  - type: VenueAnalysis
    description: "Venue profile evidence, venue assumptions, or prior venue analysis"
outputs:
  - type: JournalFitRecommendation
    artifact: "submission/journal_fit_recommendation.md"
constraints:
  - "Must be manuscript-first, not target-first"
  - "Must block best-journal claims when manuscript evidence is missing"
  - "Must classify venues as primary, stretch, safe, fallback, or do_not_submit"
tools: [filesystem]
tags: [submission, journal-selection, venue-fit, manuscript-review]
domain_aware: true
---

# Journal Fit Recommender

## Purpose

Recommend journals or conferences from the manuscript that exists now. This
manuscript-first H5 task explains fit, evidence gaps and revision costs without
promising acceptance or letting prestige determine the result.

## Related Task IDs

- `H5` (reverse journal-fit recommendation)

## When to Use

- An existing manuscript needs suitable submission targets.
- A rejected draft needs a fresh comparison against its current evidence.
- A formal lifecycle reaches H5.

For early exploration without a draft, or adaptation to an already chosen target,
use `skills/A_framing/venue-analyzer.md` (A5). A5 assumptions alone do not establish
H5 fit; read the manuscript before making a definitive recommendation.

## Inputs

Use the current draft or supplied sections, question/contribution, methods or
argument/evidence design, limitations and claim support. Reuse the claim map,
evidence ledger, previous reviews and venue analysis when available. Their
information may be supplied in chat; a particular filename is not a prerequisite
for a narrow answer. Local venue profiles are discovery aids, not verified policy.

If only an abstract or partial draft is available, name provisional candidates
where support permits, state what was inspected and request the missing material
needed to rank them. Do not claim to have read an unseen full manuscript. A gap
blocks the affected conclusion, not every useful part of the answer.

## Process

Follow `references/stage-H-submission.md` for source verification, fit, adaptation,
review boundaries and formal write requirements.

1. Read the manuscript. Summarize its actual question, contribution, design,
   evidence strength, limitations and article type with source locations. Identify
   central issues that changing the journal would not solve.
2. Establish the author's relevant constraints, reusing supplied preferences.
   Separate hard limits (for example, budget or a required publication route)
   from preferences. Ask only when the distinction changes the recommendation.
3. Find plausible venues. Use the local/subject catalog and official discovery
   outside it as needed. Verify each decision-relevant rule for the exact venue,
   article type, track/year and submission stage. Do not fill a candidate quota.
4. Check eligibility and hard constraints before comparing scope, contribution,
   evidence/method and audience fit. Record unknowns and disqualifying conflicts.
   Cite both venue sources and manuscript locations; explain uncertainty rather
   than fabricating scores, acceptance probabilities or reviewer preferences.
5. Classify assessed venues using the stable classes below. With missing decisive
   evidence, keep a lead explicitly provisional and unranked until checked; do
   not convert unknown eligibility to either a confirmed fit or a rejection.
6. Explain the smallest required revisions per viable candidate, distinguishing
   presentation/reporting from new research and author commitments. State what
   would change the ranking. Transfer the selected target and requirements to A5
   adaptation or H1 packaging only within the user's requested scope.

## Output Contract

Formal H5 writes `RESEARCH/[topic]/submission/journal_fit_recommendation.md` and
also `RESEARCH/[topic]/submission/journal_fit_recommendation.json` when requested
by the caller or formal contract. Preserve the existing structured-output schema.
A focused answer may stay in chat. Include review scope, manuscript version or
anchors, author constraints, candidate coverage and source/checked-date table.

| Venue | Class | Scope fit | Contribution fit | Method/evidence fit | Reviewer risk | Desk-reject risk | Required revision |
|---|---|---|---|---|---|---|---|

For each assessed venue, link to applicable sources and manuscript evidence.
Use these classes exactly; no need to populate all five:

- `primary`: strongest supported fit among the evaluated, eligible options.
- `stretch`: potentially eligible but a documented fit/evidence gap needs work;
  not a label for an unmet hard requirement.
- `safe`: comparatively conservative fit; never guaranteed acceptance.
- `fallback`: viable alternative with an explained tradeoff.
- `do_not_submit`: demonstrated scope/eligibility/constraint conflict for this
  manuscript or unresolved substantiated blocker, with its reason.

Keep provisional leads and open checks separate from a definitive ranking. If
no candidate can be assessed, return the missing evidence and next checks rather
than a best-journal claim. Preserve facts, inferences and uncertainties; apply
`references/academic-output-rubric.md`. Never invent policies, citations, data,
metrics, acceptance probabilities or reviewer comments.

## Quality Bar

- [ ] The actual manuscript scope and claim support were inspected and identified.
- [ ] Candidate coverage and source currency are explicit; the catalog is not a whitelist.
- [ ] Hard constraints precede fit; unknowns remain open checks.
- [ ] Each assessed venue has a justified class, evidence and specific revision needs.
- [ ] Provisional leads cannot be mistaken for submission-ready recommendations.
- [ ] Missing evidence blocks definitive ranking; `safe` promises no outcome.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Rank a draft by title or abstract alone | Bound the answer to what was supplied |
| Reuse an early target preference as a result | Compare the manuscript's actual contribution and evidence |
| Treat all reviews or methods papers alike | Verify the exact article type and any exceptions |
| Exclude a field because no profile exists | Discover official criteria and expose coverage limits |
| Make every fit issue a scientific flaw | Separate venue mismatch from validity and reporting |
