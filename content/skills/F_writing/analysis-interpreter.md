---
id: analysis-interpreter
stage: F_writing
description: "Translate quantitative, qualitative, or synthesized findings into analytical narratives that preserve uncertainty, surface mechanisms, and narrow claims to defensible scope conditions."
inputs:
  - type: StatsReport
    description: "Model results, diagnostics, and robustness checks"
    required: false
  - type: EvidenceTable
    description: "Coded qualitative evidence, case summaries, fieldnotes, or synthesis matrices"
    required: false
  - type: AnalysisPlan
    description: "Pre-specified estimands and decision rules"
    required: false
  - type: RobustnessPlan
    description: "Planned robustness checks and threats"
    required: false
outputs:
  - type: ResultInterpretation
    artifact: "manuscript/results_interpretation.md"
constraints:
  - "Must distinguish descriptive findings from causal interpretation"
  - "Must note uncertainty, assumptions, and failed robustness checks"
  - "Must separate observation, interpretation, and implication"
  - "Must surface mechanism candidates, rival explanations, and boundary conditions when evidence permits"
  - "Must keep first-order evidence separate from researcher interpretation"
  - "Must avoid re-litigating the entire methods section"
failure_modes:
  - "Narrative overclaims beyond the estimator and identification strategy"
  - "Null or imprecise findings are reframed as support without justification"
  - "Themes, cases, or quotes are restated without analytic interpretation"
tools: [filesystem]
tags: [writing, results, interpretation, robustness, uncertainty]
domain_aware: true
---

# Analysis Interpreter Skill

## Purpose

Turn quantitative, qualitative or synthesized findings into the requested result
interpretation. Read **Result-to-claim decisions** in
`references/stage-F-writing.md`; it owns the writing contract and three-question
guidance. Choose the depth and structure supported by the actual evidence.

## Related Task IDs

- `F3` — results interpretation component.
- A bounded explanation or sentence correction can stay in chat without claiming
  a completed F3 run.

## Inputs

Read the available result/source, relevant C/E/I decisions, analysis plan and
applicable checks. Reuse the current handoff and existing claim IDs. A previous
summary does not replace the output when a number, method or source has changed.
Missing diagnostics limit the affected inference; they need not block an accurate
explanation of the reported result.

## Process

### Establish what the result says

For quantitative work, verify the estimate, metric, units, direction, denominator,
comparison, time window and available uncertainty against the actual output.
Check relevant assumptions and robustness results, distinguishing performed from
planned analyses. Do not infer a p-value, confidence interval or power calculation
from the label "significant" or from sample size alone.

For qualitative work, inspect the source episodes/quotes, context, case boundaries,
analytic procedure and counterevidence. Describe prevalence or coding agreement
only when recorded and appropriate to that procedure. Evidence breadth is not
established by counting excerpts as independent participants.

For synthesis, retain contributing-study identities, exclusions/dependence,
appraisal and uncertainty. An aggregate conclusion cannot exceed those sources.

### Choose the supported interpretation

| Observed situation | Decision and permitted result |
|---|---|
| Estimate with documented uncertainty | Report magnitude and precision on the actual scale; use `effect-size-interpreter` if a practical translation is supported |
| Estimate without usable uncertainty | Explain the reported value and identify the missing input; do not invent precision or significance |
| Association with an untested causal mechanism | State the association; a proposed mechanism remains an explicit hypothesis, linked to its basis |
| Imprecise or null finding | Describe the compatible range and inferential limit; use a practical/equivalence margin only if justified and actually assessed |
| Theme grounded in contextual evidence | Explain the analytic pattern with source anchors; retain differing cases and the tradition's limits |
| Conflicting or failed sensitivity result | Show how the claim changes and what remains unresolved; a significance vote cannot settle it |
| Changed upstream result or source | Recheck dependent interpretations and name affected prose/tables before reusing them |

Mechanisms, rivals, boundary conditions and implications are useful when they
help answer the requested question and have a basis. They are not a mandatory
ladder for every finding. A supported descriptive interpretation is complete for
that purpose. Do not add a theory, negative case, citation or managerial
recommendation merely to fill an example structure.

### Write and check the requested unit

Keep the observation, supported interpretation and possible implication distinct.
Place consequential limitations beside the claim they limit. Compare with prior
work only when its actual finding and context are available; mark a needed
comparison as unresolved rather than inventing one.

Before returning the text, check its central claims against the current sources,
its consistency with inherited decisions, and any unresolved contradiction. Use
`references/academic-output-rubric.md`. Report the supported result and the exact
claim/action awaiting evidence; continue independent supported drafting.

## Output Contract

For formal saved interpretation, use
`RESEARCH/[topic]/manuscript/results_interpretation.md` through the existing
preview/approval/CAS owner. Preserve user text and claim/decision IDs. Record the
finding, source/output anchor, interpretation, limiting assumption and unresolved
continuation condition at the level needed for reuse; no fixed paragraph format.

### Evidence Ledger and Source Integrity

Follow `references/evidence-ledger-contract.md` for central claims in
`evidence/claim-evidence-ledger.csv`. Keep source locators and citekeys distinct
from the interpretation; an unsupported central claim remains a gap note. Reuse
those IDs and exact claim text in the F4 map when that task is in scope. Use
`references/citation-risk-policy.md` for material citation risks.

For a formal transition, use `references/stage-handoff-contract.md`. Pass current
source/output identity, claim limits, unresolved checks and next supported action.
A complete interpretation draft does not itself establish a passed formal gate,
an independent review, a refreshed graph or submission readiness.
