---
id: evidence-synthesizer
stage: E_synthesis
description: "Synthesize evidence narratively, qualitatively, or quantitatively (meta-analysis) with PRISMA-aligned reporting."
inputs:
  - type: ExtractionTable
    description: "Extracted study data"
  - type: QualityTable
    description: "Quality assessment results"
outputs:
  - type: EvidenceTable
    artifact: "synthesis.md"
  - type: SynthesisMatrix
    artifact: "synthesis_matrix.md"
constraints:
  - "Must justify synthesis by question, compatibility, dependence and available evidence"
  - "Must produce structured evidence table with claim-evidence-strength mapping"
failure_modes:
  - "Excessive heterogeneity prevents pooling"
  - "Insufficient studies for quantitative synthesis"
tools: [filesystem, stats-engine]
tags: [synthesis, meta-analysis, narrative, evidence-table, PRISMA]
domain_aware: false
---

# Evidence Synthesizer Skill

## Purpose

Produce a source-traceable narrative, qualitative or quantitative synthesis.
Read **Synthesis decisions** in `references/stage-E-synthesis.md` for the three
questions, selected task outputs and Q2/Q4. The model chooses the analysis path
from the actual review question and evidence.

## When to Use

Use for integrating included evidence, choosing a synthesis method, or revising
an existing synthesis after an extraction, appraisal or source correction.

## Inputs

For a formal synthesis, inspect the relevant `extraction_table.md`,
`quality_table.md`, source notes and eligibility decisions. Reuse the selected
outcomes/themes, protocol and existing `synthesis_matrix.md`. A bounded task may
use supplied material directly; do not claim that it covers a complete review.

When an input is missing, identify the affected decision and record the gap in
`context/gap_notes.md` through the existing write owner. Continue supported work;
never create an extraction or appraisal finding to satisfy a prerequisite.

## Process

### Choose a method per outcome or theme

| Candidate method | Evidence needed before choosing it | If the condition is unmet |
|---|---|---|
| Quantitative pooling | Compatible question/estimands, population/comparison/time, usable estimates and uncertainty, and an explicit dependence strategy | Resolve the missing input, separate compatible subsets, or synthesize reported findings without a pooled estimate |
| Qualitative synthesis | Extracted findings with context/source anchors, a suitable analytic tradition and appraisal | Keep uncertain interpretations provisional; seek the specific contextual or source evidence needed |
| Narrative synthesis | Comparable groupings, reported findings and their magnitude/context/uncertainty and bias limits | Narrow the synthesis to what was inspected; report the uncovered scope rather than inventing consensus |

Multiple methods may serve different outcomes. Record the rationale and
continuation conditions in the existing E1 plan when that task is in scope.
Study count or I² alone does not determine the method. Apply the review's actual
reporting standard; PRISMA is not a substitute for these method decisions.

### Quantitative execution

1. Preserve study, report, cohort, comparison and time-point identities. Resolve
   duplicate reports and overlapping samples before assigning independent weights.
2. Build E2 from source-bound estimates: metric, direction, scale, uncertainty,
   sample/analysis population, adjustment set and page/table/row locator. Record
   conversions with their inputs and assumptions. An unreported variance is not
   zero; study size alone does not supply it.
3. Choose the effect model, estimator and interval method for the target and
   assumptions. Document why they fit. Random effects is an option with its own
   assumptions, not an automatic remedy for incompatible studies.
4. Execute the specified analysis with available tools; retain code/config,
   input/output identity and observed diagnostics. Check the output against the
   contributing rows before drafting E3. If tools or inputs are unavailable,
   provide the plan or supported partial result and identify the unexecuted work.
5. Select sensitivity, heterogeneity and missing-results checks for the actual
   threats and available information. Carry failed or uninformative checks into
   the conclusion; do not switch models merely to obtain significance. Retain
   required protocol checks and label later amendments/exploration.

Useful resources: `templates/effect-size-extraction-table.md`,
`templates/meta-analysis-plan.md`, `templates/meta-analysis-report.md`. Choose an
existing analysis template only after its method fits the selected plan.
[Stage E's Cochrane source](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-10)
provides the quantitative method basis for compatible intervention syntheses.

### Qualitative and narrative execution

For qualitative synthesis, select the approach and coding/interpretive procedure
from the review question and source material. Inductive, deductive and combined
approaches are choices; retain source context, analytic memos, counterevidence and
uncertainty. Produce the applicable E5 dictionary/codebook and evidence matrix.
Use a relevant confidence framework when required or justified; an informal
assessment must not be presented as a completed formal framework.

For narrative synthesis, organize evidence by the justified outcome/theme and
comparison. Keep magnitude, uncertainty, design, context and risk of bias
visible. Explain contradictions only as far as sources support; count neither
significant results nor papers mentioning a theme as proof of effect or certainty.
Appraisal grades alone do not establish outcome-level certainty.

## Output Contract

- `EvidenceTable`: `RESEARCH/[topic]/synthesis.md`.
- `SynthesisMatrix`: `RESEARCH/[topic]/synthesis_matrix.md`.
- Selected E1–E4 and qualitative E5 tasks retain the outputs in Stage E.
- Keep findings, interpretation and implication distinct. Use
  `references/academic-output-rubric.md` before finalizing.
- Persist through the existing preview/approval/CAS owner; preserve prior work.

### Evidence Ledger and Source Integrity

For central scholarly claims, follow `references/evidence-ledger-contract.md` in
`evidence/claim-evidence-ledger.csv`. Reuse claim IDs and citekeys; each support
row points to the inspected source or executed output. Unsupported claims remain
`gap_note` / `needs_evidence`, with the missing basis in `context/gap_notes.md`.
For manuscript-facing citation risks use `references/citation-risk-policy.md`
and the existing `proofread/citation-risk-report.md` when material.

## Completion and handoff

Check that each conclusion has the actual contributing evidence, a justified
method, reproducible transformations and visible appraisal/uncertainty limits.
A file's existence or an attractive plot is not that check. Report the requested
unit's status and specific unmet requirements without declaring the whole stage
ready from a partial result.

Use `references/stage-handoff-contract.md` for formal transitions: pass the
current decision/source basis, supported claims, unresolved action and condition
for resumption to F. A changed extraction or source reopens only dependent
analyses and claims; preserve history and identify their stale downstream uses.
