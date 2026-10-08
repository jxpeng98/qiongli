---
id: discussion-writer
stage: F_writing
description: "Draft or revise a discussion that explains findings in relation to the research question and inspected literature."
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
  - "Must relate findings to the research question and relevant inspected literature or framework"
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

Use for a requested discussion draft or revision from available findings and
literature. Provisional findings permit a bounded draft with their status intact;
formal readiness requires the current finalized results.

## Expected Inputs

- `RESEARCH/[topic]/manuscript/manuscript.md`
- `RESEARCH/[topic]/framing/research_question.md`
- `RESEARCH/[topic]/framing/contribution_statement.md`
- `RESEARCH/[topic]/literature/literature_map.md`

## Inputs

- `ResultsSummary`: Summarized findings and statistical/thematic results
- `ContributionStatement`: Original contribution mapping from the introduction
- Reuse supplied findings, research question and literature; equivalent material
  need not arrive in these named files. Ask only for a missing input that changes
  the interpretation. Continue supported work with explicit unresolved gaps;
  formal project gap notes use the existing approved write owner.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

Read the shared Writing Harness Contract in `references/stage-F-writing.md`,
including **Result-to-claim decisions**, prose review and completion. Reuse the
current results and inherited claim limits; a changed upstream source requires
checking dependent interpretations before drafting. The structure below is an
aid for the requested discussion, not a quota of mechanisms or recommendations.

### Connect the question, findings and interpretation
Keep the Story Spine inside the requested writing task unless a separate mapping
is requested. Useful questions for organizing the discussion are:
1. **The Core Answer:** A direct, concise answer to the main research question based on the findings.
2. **The Contextualization:** How these findings compare, contrast, or add nuance to the existing literature.
3. **The 'So What' (Theoretical Implications):** How the findings change our understanding of the phenomenon or theoretical model.
4. **Practical implications, when supported:** What the findings imply for a specified use, with its assumptions and limits.

### Draft the discussion
Give a clear answer to the research question and enough result detail to assess
the interpretation. Choose the opening and paragraph order for the actual
argument; neither restating the aim nor one paragraph per finding is mandatory.

- Interpret the supported pattern and consequential alternatives. An untested explanation remains a hypothesis; a descriptive finding need not establish a mechanism.
- Compare with the actual available literature. Preserve unresolved disagreement and mark missing comparison evidence rather than inventing a source or explanation.

- Develop theoretical or practical implications where supported and relevant to
  this paper. Keep their assumptions and inferential limits explicit.
- Anchor this study's findings to its results, literature comparisons to inspected
  external sources, and author synthesis to its premises and reasoning. External
  findings need not appear as this study's results. Do not require a new theory or
  a practical recommendation when the evidence supports a narrower contribution.

### Review the complete requested unit
Check that the discussion answers the introduction's question with the same
constructs and scope. Explain consequential disagreement without manufacturing a
resolution. Apply the shared source-support and prose reviews separately; remove
unsupported bridges and repair missing explanation before polishing wording.

## Output Contract

- Return the requested prose for a direct draft/edit; no project files or separate
  Story Spine are required. Preserve the requested edit depth and author meaning.
- When selected as formal outputs, `DiscussionDraft` uses
  `RESEARCH/[topic]/manuscript/discussion.md` and `StorySpine` uses
  `RESEARCH/[topic]/manuscript/discussion_story_spine.md`.
- An existing discussion in `manuscript/manuscript.md` remains the selected edit
  target unless the user requests a separate draft. A saved separate draft does
  not silently replace the manuscript; integration uses preview/approval/CAS and
  preserves claim IDs and source bindings.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- For formal project claims, update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` through the existing write owner; direct prose needs no new ledger.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- Apply `references/citation-risk-policy.md`. For formal project work with material
  citation risk, use `RESEARCH/[topic]/proofread/citation-risk-report.md` through
  the write owner. A direct draft/edit returns consequential unresolved gaps with
  the requested prose and needs no report file.

## Quality Bar

- [ ] Does not unnecessarily repeat raw results or P-values
- [ ] Explicitly answers the primary research question stated in the Introduction
- [ ] Uses relevant inspected literature for comparisons, with missing evidence explicit
- [ ] Any theoretical or practical implications follow from inspected evidence
      and explicit premises; a supported narrower contribution is sufficient.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Restating Results | Reads like a second results section | Explain the meaning and retain result detail needed to assess it |
| Overclaiming | Claiming causality or broad generalizability not supported by design | Narrow or remove the unsupported claim; cautious verbs alone do not supply evidence |
| Ignoring Contradictions | Consequential disagreement remains unexplained | Inspect comparability and supported alternatives; unresolved reasons stay unresolved |
