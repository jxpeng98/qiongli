---
id: robustness-planner
stage: C_design
description: "Pre-specify robustness checks, sensitivity analysis, and bounds scaling linked to specific identification threats."
inputs:
  - type: DesignSpec
    description: "Study design with identification strategy"
  - type: AnalysisPlan
    description: "Primary model and assumptions"
outputs:
  - type: RobustnessPlan
    artifact: "design/robustness_plan.md"
constraints:
  - "Must link each check to a specific threat or assumption"
  - "Must pre-specify interpretation, including what changes the claim or remains unresolved"
  - "Must address applicable model, data or interpretive threats and retain required diagnostics"
failure_modes:
  - "Robustness table is a checklist without threat-specific motivation"
  - "Too many checks without prioritization inflates researcher degrees of freedom"
  - "Qualitative robustness dismissed as 'not applicable'"
tools: [filesystem]
tags: [design, robustness, sensitivity-analysis, endogeneity, trustworthiness]
domain_aware: true
---

# Robustness Planner Skill

## Purpose

Plan checks that test consequential threats to the study's claims. Follow the
**Design judgment contract** in `references/stage-C-design.md`; the number of
checks is not a quality measure. Retain explicit protocol requirements.

## Related Task IDs

- `C3_5` (robustness/sensitivity plan)

## When to Use

- Design or revise threat-specific checks for a planned analysis.
- Assess sensitivity of quantitative findings or qualitative interpretations.
- Distinguish planned checks from checks proposed after results were examined.

## Inputs

- `DesignSpec`: question, design, intended claims and identification/analytic logic.
- `AnalysisPlan`: planned methods, assumptions, available evidence and prior access.
- Any approved checks, decision criteria and reporting obligations.

Reuse known inputs. Explain a consequential gap rather than inventing a test,
parameter or result. Formal gap notes use `context/gap_notes.md` through the
existing write owner; chat-only advice need not create project files.

## Process

### Gate And Method-Pack Robustness

For formal C3_5, use Q1 and Q4 from `standards/quality-gate-contract.yaml` to
connect checks to validity and reproducibility. Retain the active domain
profile's selected `method_templates[*].required_diagnostics`. If a selected
method has no matching template, record that gap in
`RESEARCH/[topic]/design/validity-threat-matrix.md` instead of inventing required
diagnostics or claiming the method gate is complete.

### Link threats, checks and interpretation

For each consequential threat, specify why it matters, what available evidence
could examine it, the check's assumptions and whether it changes the estimand,
sample, measurement, uncertainty or interpretation. A check that needs unavailable
data remains unavailable. More controls, a different estimator or an instrument
cannot repair identification without a defensible argument for its assumptions.

For qualitative work, choose procedures compatible with the analytic tradition
and source access: for example, inspect disconfirming material, compare available
accounts, or trace how reflexive memos informed interpretation. Do not make
independent double-coding, agreement coefficients, member checking or saturation
universal requirements. Preserve them when the selected method/protocol requires
them, and explain any conflict rather than silently discarding it.

Prioritize checks by the consequence of the threat, then feasibility. Retain all
explicitly required checks; do not add or drop checks to reach a generic quota.
Pre-specify what patterns would narrow, challenge or leave a claim unresolved.
Interpret magnitude, uncertainty and meaning; stable signs or p-values across
many variants do not establish robustness by themselves. Do not hide a failed
check, treat an unrun check as passed or upgrade a claim because a variant looks
more favorable. Post-result changes need a dated rationale and an accurate label.

## Output Contract

For formal `RobustnessPlan`, write
`RESEARCH/[topic]/design/robustness_plan.md` through preview/approval/CAS. Preserve
the table below; the criterion may describe uncertainty or unresolved outcomes,
rather than forcing an unsupported binary judgment. A narrow question can return
advice in chat without claiming C3_5 completion.

```markdown
# Robustness / Sensitivity Plan

## Threat Inventory
| Threat | Severity | Source | Addressed by |
|--------|---------|--------|--------------|

## Robustness Checks
| # | Threat | Check | Changes | Pass/Fail Criterion | Priority |
|---|--------|-------|---------|---------------------|----------|

## Sensitivity Analyses
| Method | Parameter | Threshold |
|--------|-----------|-----------|

## Interpretation Rules
Describe the claim consequences, assumptions and unresolved outcomes for each check.

## Reporting Commitment
Identify where every planned result, failure, unrun check and deviation will be reported.
```

Separate findings, interpretations and implications. Do not invent citations,
data, sample sizes, results or reviewer comments. Apply
`references/academic-output-rubric.md` to scholarly prose.

### Method Diagnostics

Produce or consume `RESEARCH/[topic]/design/method-diagnostic-report.md` and
`RESEARCH/[topic]/design/validity-threat-matrix.md` for formal choices affecting
inference, measurement or analysis. Cover construct validity, internal validity,
external validity, statistical conclusion validity, measurement validity, data
leakage, missingness, confounding and selection bias; explain inapplicable items.
Insufficient method details remain explicit gaps.

## Quality Bar

Each check has a threat, assumptions, feasible evidence or a stated access gap,
interpretation criteria and reporting location. Required checks remain visible,
including those not yet feasible. A complete plan does not mean checks have run
or their gates have passed; an unresolved design threat remains a limitation or
block on the affected claim.

## Common Pitfalls

- Counting favorable results instead of examining what each check establishes.
- Trying many variants without a threat-specific reason or deviation record.
- Claiming robustness while omitting failures, unavailable data or unrun checks.
