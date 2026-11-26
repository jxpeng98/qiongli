---
id: statement-generator
stage: D_ethics
description: "Generate standardized ethics, data availability, and conflict-of-interest statements for manuscript inclusion."
inputs:
  - type: EthicsPackage
    description: "Documented ethics decision or current review status, when relevant"
  - type: DataManagementPlan
    description: "Data governance and sharing plan"
outputs:
  - type: Manuscript
    artifact: "manuscript/manuscript.md"
constraints:
  - "Follow the applicable venue's declaration requirements"
  - "Must clearly state IRB approval numbers if applicable"
  - "Describe evidenced data access conditions and include a verified link when applicable"
failure_modes:
  - "Using generic 'data available upon request' statements when prohibited by the journal"
  - "Omitting secondary data source citations in the ethics statement"
tools: [filesystem]
tags: [ethics, statements, compliance, irb, data-availability]
domain_aware: true
---

# Statement Generator Skill

Draft the requested manuscript declaration from the documented status and
access conditions. Use the D2 section of `references/stage-D-ethics.md`.

## Purpose

Generate standardized ethics, data availability, and conflict-of-interest statements for manuscript inclusion.

## Related Task IDs

- `D2` (ethics-statements)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/manuscript.md` (Appended to the statements section)

## When to Use

- When finalizing a manuscript for submission and ensuring compliance with journal statement requirements (e.g., PLOS, Nature, Elsevier).
- When translating an IRB approval into a concise 'Ethics Declarations' paragraph.

## Inputs

- Use the ethics decision, governance plan or author disclosure relevant to
  the requested statement. One availability paragraph can stand alone.
- If a missing input prevents a declaration, identify it and draft
  the supported text. Use `RESEARCH/[topic]/context/gap_notes.md` for formal
  project gaps. Unreported approval, consent, access and conflicts stay unknown.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Input Analysis
- Review the `EthicsPackage` to extract approval statuses, institutional names, and waiver details.
- Review the `DataManagementPlan` to identify where data and code are hosted, or constraints preventing sharing.

### Step 2: Information Synthesis
For each requested or venue-required declaration, check the relevant facts:
1. **Ethical Approval**: Did an IRB approve it? What is the protocol number? Was informed consent obtained?
2. **Data Availability**: Is data public? If so, what is the DOI/link? If not, under what conditions can it be accessed?
3. **Code Availability**: Are analysis scripts published?
4. **Conflicts of Interest**: Are there any financial or non-financial conflicts to declare?

### Step 3: Drafting Statements
Select the applicable paragraphs:
- **Ethics Statement**
- **Data Availability Statement**
- **Code Availability Statement**
- **Competing Interests Statement**

### Step 4: Manuscript Insertion
Format the output as Markdown paragraphs, ready to be appended to the 'Declarations' or 'Methods' section of the manuscript.

## Output Contract

- `Manuscript`: write `RESEARCH/[topic]/manuscript/manuscript.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

- [ ] The requested declarations cover the applicable requirements.
- [ ] Contains specific institutional names and protocol numbers, avoiding placeholders if data exists.
- [ ] Data availability statement clearly delineates public vs. restricted data.
- [ ] The wording conveys the documented status to a manuscript reader. Apply
      ICMJE guidance when the venue requires it.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Missing Protocol Numbers | States "Approved by IRB" without proof | Extract and include the exact numeric ID from the EthicsPackage |
| "Upon reasonable request" | Many journals outright ban this phrase | Specify the exact contact point, conditions, and repository link |
| Conflating Data and Code | Mentioning only data but ignoring scripts | Keep Data Availability and Code Availability explicitly separated |
