---
id: discussion-writer
stage: F_writing
description: "Drafts the Discussion section using a structured story spine, separating factual findings from theoretical implications."
inputs:
  - type: ResultsSummary
    description: "Summarized findings and statistical/thematic results"
  - type: ContributionStatement
    description: "Original contribution mapping from the introduction"
outputs:
  - type: DiscussionDraft
    artifact: "manuscript/discussion.md"
  - type: StorySpine
    artifact: "manuscript/discussion_story_spine.md"
constraints:
  - "Must avoid restating results in detail; focus on interpretation"
  - "Must link findings back to the existing literature and theoretical framework"
failure_modes:
  - "Simply repeating the results section without interpretation"
  - "Overclaiming the implications beyond what the evidence supports"
tools: [filesystem]
tags: [writing, discussion, implications, story-spine, drafting]
domain_aware: true
---

# Discussion Writer Skill

Generates the Discussion section using a structured story spine, separating factual findings from theoretical implications.

## Purpose

To draft a compelling and structurally sound Discussion section that interprets the study's findings, situates them within the broader literature, and articulates the theoretical and practical implications without overclaiming.

## When to Use

Use after the Results section has been drafted and the primary findings are finalized. Provide the core contribution statement and literature framing to ensure alignment.

## Expected Inputs

- `RESEARCH/[topic]/manuscript/manuscript.md`
- `RESEARCH/[topic]/framing/research_question.md`
- `RESEARCH/[topic]/framing/contribution_statement.md`
- `RESEARCH/[topic]/literature/literature_map.md`

## Inputs

- `ResultsSummary`: Summarized findings and statistical/thematic results
- `ContributionStatement`: Original contribution mapping from the introduction
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

Use **Result-to-claim decisions** in `references/stage-F-writing.md`. Reuse the
current results and inherited claim limits; a changed upstream source requires
checking dependent interpretations before drafting. The structure below is an
aid for the requested discussion, not a quota of mechanisms or recommendations.

### Step 1: Build the Story Spine
Map the principal findings to the core narrative using a structured spine:
1. **The Core Answer:** A direct, concise answer to the main research question based on the findings.
2. **The Contextualization:** How these findings compare, contrast, or add nuance to the existing literature.
3. **The 'So What' (Theoretical Implications):** How the findings change our understanding of the phenomenon or theoretical model.
4. **Practical implications, when supported:** What the findings imply for a specified use, with its assumptions and limits.

### Step 2: Draft the Opening Paragraph
- Start strong by restating the primary aim of the study.
- Provide the clearest, most direct answer to the research question without repeating statistical details or raw data.
- State the "bottom line" conclusion upfront.

### Step 3: Draft the Interpretation and Comparison
- Dedicate paragraphs to each major finding or theme.
- Interpret the supported pattern and consequential alternatives. An untested explanation remains a hypothesis; a descriptive finding need not establish a mechanism.
- Compare with the actual available literature. Preserve unresolved disagreement and mark missing comparison evidence rather than inventing a source or explanation.

### Step 4: Draft the Implications
- Separate theoretical implications from practical/managerial implications.
- Ensure all claims are strictly tied to the empirical evidence presented in the Results section.

### Step 5: Integration and Tone Check
- Review the draft to ensure a balanced, scholarly tone. Avoid hyperbolic language (e.g., "proves," "solves").
- Output the discussion draft to `manuscript/discussion.md`.
- Output the structural mapping to `manuscript/discussion_story_spine.md`.

## Output Contract

- `DiscussionDraft`: write `RESEARCH/[topic]/manuscript/discussion.md`.
- `StorySpine`: write `RESEARCH/[topic]/manuscript/discussion_story_spine.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

- [ ] Does not unnecessarily repeat raw results or P-values
- [ ] Explicitly answers the primary research question stated in the Introduction
- [ ] Uses relevant inspected literature for comparisons, with missing evidence explicit
- [ ] Theoretical and practical implications are logically derived from the findings

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Restating Results | Reads like a second results section | Focus on *interpretation* and *meaning*, omit raw data |
| Overclaiming | Claiming causality or broad generalizability not supported by design | Narrow or remove the unsupported claim; cautious verbs alone do not supply evidence |
| Ignoring Contradictions | Failing to discuss why results differ from established norms | Directly address unexpected findings and propose potential reasons |
