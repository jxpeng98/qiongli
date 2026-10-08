---
id: meta-optimizer
stage: F_writing
description: "Optimize title, abstract, keywords, and metadata for discoverability, impact, and venue compliance."
inputs:
  - type: Manuscript
    description: "Current manuscript or supplied passage"
  - type: VenueAnalysis
    description: "Venue requirements"
outputs:
  - type: MetadataOptimization
    artifact: "manuscript/meta_optimization.md"
constraints:
  - "Must preserve accuracy — do not overclaim in abstract"
  - "Must comply with venue-specific abstract format and keyword policies"
  - "Must optimize for search discoverability (database indexing, Google Scholar)"
failure_modes:
  - "Title is catchy but does not reflect actual scope"
  - "Abstract omits key results to save words"
  - "Generic or misleading keywords obscure the actual contribution"
tools: [filesystem]
tags: [writing, title, abstract, keywords, SEO, metadata, discoverability]
domain_aware: true
---

# Meta Optimizer Skill

Optimize the requested title, abstract, keywords or metadata while preserving the
paper's meaning, evidence strength and actual scope. This card serves F6; formal
submission packaging remains H1.

## Inputs and scope

Reuse the current manuscript, supplied findings and known venue requirements.
Read `references/stage-F-writing.md` for the shared Writing Harness Contract,
including prose, source support, review and completion. A title-only or abstract
edit needs only the material needed to do that edit; missing author, funding or
submission fields do not block it. Ask only for information that changes the
requested result. A provisional abstract must retain provisional study status.

For venue-specific work, use `references/stage-H-submission.md` to distinguish
verified requirements from local profile advice. Apply the actual article type,
word limit and keyword rules. Unverified conventions are suggestions, not gates.

## Title

Represent the question or supported contribution with recognizable field terms.
Include context or method when it changes what the reader should expect. Choose
length, punctuation, question form and abbreviations for clarity and the actual
venue; there is no universal fifteen-word ceiling or preferred title formula.
Do not imply causality, novelty or broader applicability than the paper supports.
A request for one title does not require alternatives or a visible checklist.

## Abstract

Make the paper's question, approach, supported answer and significance clear to
its intended reader. An empirical abstract usually needs the study design and
specific findings; a theoretical paper needs its argument and contribution, and
a proposal must distinguish plans from completed work. Select the details that
let the reader understand and assess this paper. Do not force empirical methods
or results into other article types.

Use structured headings only where requested or required. Choose an opening that
communicates something specific; an accurate “This paper examines…” is acceptable.
Avoid generic importance claims used only to make the opening sound impressive.
Keep technical terms consistent and explain those the intended reader needs.
Follow the venue's citation policy rather than assuming abstracts never cite.

Compare the finished abstract with the current manuscript, not an earlier draft:
question, constructs, sample, findings, numbers, uncertainty and inference strength
must agree. Recheck affected claims when results change. An earlier abstract can
be a working draft; its drafting order is not a completion criterion.

## Keywords and optional metadata

Choose accurate search terms, including title terms when useful. Broader, narrower,
method and domain terms are options, not a required mixture. Use controlled
vocabularies and counts when the target venue requires them or they aid discovery;
do not invent a controlled term or exclude a key construct just to avoid overlap.

Only include submission metadata when requested. Reuse verified author details,
funding, data availability, conflicts, acknowledgments and AI-use information;
unknown fields remain unknown. Do not assign authorship or infer disclosures.
Completion of a title/abstract task is separate from submission readiness.

## Output contract

Return only the selected title, abstract, keywords or requested explanation for a
direct edit. For formal F6, `MetadataOptimization` uses
`RESEARCH/[topic]/manuscript/meta_optimization.md`, containing the selected
components. Registered writes use preview/approval/CAS; proposed text is not a
saved artifact. Do not create a second manuscript or silently alter its claims.

Separate finding, interpretation and implication. Apply
`references/academic-output-rubric.md` and `references/citation-risk-policy.md`.
For formal project claims, reconcile stable claim IDs and inspected support under
`references/evidence-ledger-contract.md`. A direct edit needs no new ledger or
citation-risk report. Never invent data, citations, results or venue rules.

## Quality bar

- The selected components accurately represent the current manuscript and paper type.
- Necessary reasoning and qualifications survive concise, readable expression.
- Actual venue requirements and requested length/format are satisfied; unknown
  requirements and evidence gaps remain explicit.
- The response stays within the requested components and edit depth; unrelated
  submission fields, backup titles and internal checklists are not default outputs.
