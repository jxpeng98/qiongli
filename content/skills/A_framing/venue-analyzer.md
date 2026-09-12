---
id: venue-analyzer
stage: A_framing
description: "Analyze venue fit, formatting constraints, and reviewer expectations to scope and position a paper for a target publication."
inputs:
  - type: UserQuery
    description: "Candidate venue(s) and paper type"
  - type: RQSet
    description: "Research questions or contribution type"
outputs:
  - type: VenueAnalysis
    artifact: "framing/venue_analysis.md"
constraints:
  - "Must verify applicable venue requirements with sources and uncertainty"
  - "Must identify must-not-fail items for the chosen venue"
  - "Must distinguish official criteria from inferred expectations"
failure_modes:
  - "Venue information not publicly available"
  - "Paper type mismatch with venue scope"
  - "Ignoring recent scope changes or special issue criteria"
tools: [filesystem, scholarly-search]
tags: [framing, venue, journal-selection, formatting]
domain_aware: true
---

# Venue Analyzer Skill

## Purpose

Help the author explore a venue direction or adapt a paper to a chosen journal or
conference, using its actual requirements and the research that can be supported.

## Related Task IDs

- `A5` (venue analysis)

## Output (contract path)

- `RESEARCH/[topic]/framing/venue_analysis.md`

## When to Use

- A target is chosen and the user wants to understand or meet its requirements.
- Early framing needs venue/audience options before a manuscript exists.
- A rejection or change of target requires a fresh fit and adaptation check.

For journal recommendations driven by an existing draft without a chosen target,
use `skills/H_submission/journal-fit-recommender.md` (H5).

## Inputs

Reuse the supplied target, article type, stage, question/contribution and any draft.
Ask for only what changes the requested decision. Research not yet performed is
planned work, not evidence. A narrow answer may stay in chat; formal A5 retains its
artifact. Missing information stays explicit without inventing a project or policy.

## Process

Follow `references/stage-H-submission.md` for shared venue evidence, fit and
modification boundaries before making a recommendation.

1. Establish the direction and constraints. For a chosen venue, assess that venue;
   do not require alternatives. For early exploration, choose a useful candidate
   set from the research question, evidence plan and intended audience. Use known
   budget, OA, timing, language or institutional requirements where relevant.
2. Verify applicable scope, article type, editorial criteria and initial/revision
   requirements. Record official sources, checked dates and applicability using
   the shared requirement table. Local profiles and recent papers can guide
   discovery, but inferred practice must not be labeled an official rule.
3. Explain fit and conflicts. Connect the actual contribution and method to readers
   and published criteria. Separate eligibility, scientific fit and optional
   presentation advice. Leave unknown fees, policies or expectations unresolved.
4. Build a target-specific adaptation map. Link each requirement or supported
   concern to the current manuscript location, proposed change and source. Separate
   wording, structure and reporting from new analysis/data or author decisions.
   Do not strengthen claims simply to match perceived venue prestige.
5. Return the next useful action. If compatible, pass confirmed constraints into
   the relevant writing task under `references/stage-F-writing.md`; package files
   through H1 only when requested. If incompatible, explain why and offer feasible
   alternatives. A plan does not itself authorize file changes or submission.

## Output Contract

Formal A5 writes `RESEARCH/[topic]/framing/venue_analysis.md`. Include:

- Direction, target/candidate set, article type/stage and author constraints.
- Source-backed requirement table, distinguishing facts, inferences and unknowns.
- Fit rationale and unresolved eligibility or evidence conflicts.
- Adaptation map with manuscript anchors, changes, work needed and decision owner.
- Selected direction or provisional options, coverage limits and next action.

Keep findings, interpretation and implications distinct. Do not invent citations,
metrics, policy, results or reviewer comments. Apply
`references/academic-output-rubric.md` to the requested scholarly output.

## Quality Bar

- [ ] The chosen target or useful candidate set is assessed without a quota.
- [ ] Decision-relevant rules identify their source and applicable type/stage.
- [ ] Inferred audience/presentation patterns are distinguished from requirements.
- [ ] Each proposed adaptation preserves research evidence and exposes new work.
- [ ] Missing information limits conclusions; no acceptance outcome is promised.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Rank by prestige or keyword overlap | Compare question, evidence, article type and readers |
| Copy last year's profile as current policy | Verify applicable official instructions or mark unknown |
| Invent implicit rejection rules | Cite the rule or label a supported fit concern as an inference |
| Rewrite the study to satisfy a target | Preserve results; expose the mismatch and author decision |
| Treat a quick question as submission packaging | Return the requested analysis; retain H1 for packaging |
