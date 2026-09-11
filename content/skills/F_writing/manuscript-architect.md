---
id: manuscript-architect
stage: F_writing
description: "Build coherent paper structure from outline to section-level drafts with claim-evidence integrity and analysis-depth checking."
inputs:
  - type: RQSet
    description: "Research questions"
  - type: EvidenceTable
    description: "Synthesized evidence"
  - type: DesignSpec
    description: "Study design (for empirical or qualitative papers)"
    required: false
outputs:
  - type: ManuscriptOutline
    artifact: "manuscript/outline.md"
  - type: Manuscript
    artifact: "manuscript/manuscript.md"
  - type: ClaimGraph
    artifact: "manuscript/claims_evidence_map.md"
  - type: FiguresTablesPlan
    artifact: "manuscript/figures_tables_plan.md"
constraints:
  - "Must maintain claim-evidence alignment across sections"
  - "Must follow venue-specific sectioning requirements"
  - "Every section must have an explicit analytical job"
failure_modes:
  - "Evidence gaps discovered during writing"
  - "Scope creep from unfocused claims"
  - "Discussion that merely restates results without interpretation"
tools: [filesystem]
tags: [writing, manuscript, outline, drafting, claim-evidence]
domain_aware: true
---

# Manuscript Architect Skill

Build the requested outline, section or manuscript from available research
materials, with a coherent argument and traceable claims.

## Purpose

Use this card for manuscript structure and claim-evidence integrity. Keep story
spine construction, paragraph ordering and coherence checks inside this writing
task; they do not need separate agents or generated artifacts.

## When to Use

- Draft a paper from an existing research project or supplied notes and results.
- Restructure a section whose argument or evidence needs work.
- Prepare a formal claim map or assess manuscript readiness.

## Inputs

Reuse the request, current draft, research questions (`RQSet`), evidence
(`EvidenceTable`) and relevant design (`DesignSpec`). Infer the paper type only
from supplied material; do not default an unspecified paper to empirical research.
Use known venue, citation and word-limit requirements. Ask only for a missing
input that changes the result; supported portions can proceed with visible gaps.

## Writing Harness Contract

Read `references/stage-F-writing.md` for the shared contract: Story Spine,
section purpose, evidence boundaries, drafting freedom, review and completion.
It applies to this card, `/academic-write`, `/paper-write` and writing roles.
A section edit does not require full-paper outputs. Templates are optional
structural aids except for machine-readable contract fields.

## Process

1. Reuse or refine the outline around the question, contribution and available
   evidence. Select a drafting order and granularity suited to the requested
   deliverable; do not add an outline approval to an authorized full draft.
2. Draft with source anchors and the paper type's structure. Methods and results
   may be descriptive. Discussion interprets supported findings, distinguishes
   hypotheses and explains consequential alternatives or limits.
3. Check claims against the actual sources and analysis. Fix concrete defects,
   verify the affected text and preserve unresolved gaps. A clean review needs
   no invented criticism; explicit run/protocol requirements still apply.
4. For formal F4, reconcile the claim map as described below. Use F5 figure/table
   support only when those outputs belong to the requested task.
5. For formal readiness, apply the relevant reporting and semantic gates.
   Return the draft and material gaps; submission packaging is a separate task.

### Claim-evidence integrity

Build `manuscript/claims_evidence_map.md` from `templates/claim-evidence-map.md`.
Preserve its exact table headers and reuse evidence-ledger claim IDs and atomic
claim text. Assign stable `CLM-###` claim IDs only to new claims, and never
renumber or reuse a recorded ID. Keep citation keys distinct from evidence
pointers: citation edges record attribution and do not by themselves prove a
claim. Preserve user-written prose and source anchors when merging.

Check that evidence supports both the claim's content and its strength. An
association is not a causal result; a quotation is not proof of prevalence.
Missing measurement, recruitment, uncertainty or robustness details remain
unknown. Narrow unsupported claims or record gaps rather than adding plausible
but unreported procedures, results, negative cases or citations.

### Formal readiness

Before marking the selected formal manuscript task ready, create or update
`RESEARCH/[topic]/quality-gate-report.md` with a Q2 `semantic_checks` entry using
`q2_claim_evidence_traceability`. Use structured evidence refs: each
`evidence_refs` item includes `artifact`, `anchor` and `supports`, with optional
`claim_id` or `diagnostic_id`. Evidence may reference the claim-evidence ledger,
claim map, source note, analysis output, citation anchor or explicit gap note.
Unsupported central claims remain `BLOCKED` until resolved or appropriately
narrowed. A direct paragraph revision needs no gate report.

Use `reporting-checker` for the applicable empirical or qualitative standard,
and `prisma-checker` for a systematic review. Apply the actual venue or study
requirements; do not force statistical reporting onto qualitative/theory work
or claim an unperformed check passed.

## Output Contract

Produce only the selected task's artifacts from `references/stage-F-writing.md`:

- `ManuscriptOutline`: `RESEARCH/[topic]/manuscript/outline.md`.
- `Manuscript`: `RESEARCH/[topic]/manuscript/manuscript.md`; formal F3 also
  retains `manuscript/results_interpretation.md` and
  `manuscript/effect_interpretation.md`, with non-applicability explained.
- `ClaimGraph`: `RESEARCH/[topic]/manuscript/claims_evidence_map.md`.
- `FiguresTablesPlan`: `RESEARCH/[topic]/manuscript/figures_tables_plan.md`;
  formal F5 retains `manuscript/tables/` and `manuscript/figures/`.

Templates: `templates/manuscript-outline.md`, `templates/manuscript-skeleton.md`,
`templates/claim-evidence-map.md`, `templates/figures-tables-plan.md`.

Separate finding, interpretation and implication. Apply
`references/academic-output-rubric.md`; preserve claim strength and author voice
when humanizing prose. Never invent citations, data, sample sizes, statistical
results or reviewer comments. Registered project writes use preview/approval/CAS;
proposed text is not a saved artifact.

### Evidence Ledger and Source Integrity

For project claims, reconcile `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv`
through `references/evidence-ledger-contract.md`. Unsupported central claims
become `gap_note` rows and `context/gap_notes.md` entries. Apply
`references/citation-risk-policy.md` and update
`proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

- [ ] The requested section or manuscript fulfills its purpose and known constraints.
- [ ] Central claims preserve the source meaning, evidence strength and inferential limits.
- [ ] Required formal outputs, claim IDs and source anchors are retained.
- [ ] Applicable checks have evidence; unperformed checks and unresolved gaps remain visible.
- [ ] The response distinguishes proposed drafts, verified writes and formal readiness.

## Common Pitfalls

| Pitfall | Consequence | Correction |
|---|---|---|
| A template drives the argument | Empty sections or invented novelty | Organize around the evidence and requested contribution |
| Every paragraph needs a mechanism | Description becomes unsupported interpretation | Match depth to the section's job |
| A small edit starts a full workflow | Unrequested files and delays | Stop at the requested unit |
| Review repeats unchanged work | Criticism becomes a quota | Fix actual defects; retain explicit review requirements |
| Polishing hides an evidence gap | The prose overclaims | Narrow the claim or disclose the missing evidence |
