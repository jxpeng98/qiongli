---
id: effect-size-interpreter
stage: F_writing
description: "Translate raw effect sizes into meaningful magnitude language using benchmarks, contextual comparisons, and practical significance framing."
inputs:
  - type: StatsReport
    description: "Model output with effect size estimates"
  - type: AnalysisPlan
    description: "Planned effect size metrics and benchmarks"
    required: false
outputs:
  - type: EffectInterpretation
    artifact: "manuscript/effect_interpretation.md"
constraints:
  - "Must justify any benchmark or conversion with applicable evidence and assumptions"
  - "Must distinguish statistical from practical significance"
  - "Must preserve the source metric, units, uncertainty and supported claim type"
failure_modes:
  - "Using Cohen's 'small/medium/large' without field context"
  - "Ignoring confidence interval width when interpreting magnitude"
  - "Confusing standardized and unstandardized effect sizes"
tools: [filesystem]
tags: [writing, effect-size, interpretation, practical-significance, magnitude]
domain_aware: true
---

# Effect Size Interpreter Skill

## Purpose

Explain the substantive magnitude of an observed estimate in supported units.
Read **Result-to-claim decisions** in `references/stage-F-writing.md`. Decide what
can be interpreted from the actual estimate before choosing a conversion or
benchmark; keep uncertainty and the design's claim limit attached.

## Related Task IDs

- `F3` — effect interpretation component.
- A bounded magnitude explanation can stay in chat.

## Inputs

Use the actual model/synthesis output and relevant analysis-plan decisions:
metric and parameterization, units/scaling, population/comparison, time horizon,
uncertainty and any prespecified practical threshold. Inspect the source when a
label or unit is ambiguous. Missing benchmark information does not prevent an
accurate explanation of the original estimate.

## Process

### Decide which translation is justified

| What must be decided? | Evidence needed | What can proceed? |
|---|---|---|
| What does one unit mean? | Outcome/exposure units, scaling, transformations, contrast and model link | Explain the estimate on its stated scale. Resolve ambiguous standardization or log/link interpretation before converting |
| Can we express it in original units? | Required SDs, reference values or model predictions, with their source and applicable population | Calculate and check the conversion, retaining inputs and assumptions. Otherwise report the original scale and name the missing input |
| Can relative effects become absolute effects? | Compatible baseline risk/rate, time horizon, effect definition and required model assumptions | Show an explicitly conditional translation. OR, RR and HR are distinct; do not substitute one for another or invent a baseline |
| Is the magnitude practically meaningful? | A justified threshold or genuinely comparable prior estimates, with source/context | Compare with that basis and uncertainty. Without it, describe magnitude and leave practical importance unresolved; no universal small/medium/large cutoff |
| What does an imprecise/null estimate allow? | Available interval and its method, plus a justified decision margin if used | Explain the range compatible with the analysis. Do not declare equivalence, absence of effect or achieved power from a nonsignificant result |

For example, a standardized coefficient alone does not establish a percentage
change in the outcome. Record missing scaling information rather than borrowing
numbers from a writing example. A unit conversion can clarify size without
establishing a causal effect or policy recommendation.

### Calculate only supported quantities

Use an available, appropriate tool for a requested calculation and retain its
inputs, formula/implementation, result and assumptions. Check units, direction,
rounding and uncertainty propagation before using it in prose. Preserve the
reported interval type and level; do not relabel a credible interval or create a
95% confidence interval when its necessary inputs are absent.

For subgroup or interaction results, use the relevant conditional contrast and
its uncertainty. A difference in significance between groups does not itself
establish a difference between their effects. For null findings, any precision
or sensitivity analysis needs its actual design and assumptions; do not infer
power from the observed effect.

The [Cochrane effect-measure guidance](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-06)
is a primary reference when those intervention measures apply. The selected
model's parameterization and source output govern other translations.

## Output Contract

For the formal F3 component, use `EffectInterpretation` at
`RESEARCH/[topic]/manuscript/effect_interpretation.md`, matching Stage F and the
workflow contract. If an older project contains `effect_size_interpretation.md`,
read it as prior material and propose reconciliation through the existing owner;
do not silently rename or remove it.

Keep the original estimate/uncertainty, source anchor, supported translation,
assumptions and unresolved condition together. Use a table only when it helps
compare results. Do not require a conversion or a quota of benchmark papers when
the evidence does not support them.

### Evidence Ledger and Source Integrity

For central claims, follow `references/evidence-ledger-contract.md` and reuse
claim IDs in `evidence/claim-evidence-ledger.csv` and the F4 map when in scope.
Citations must point to actual sources; unverified benchmarks remain gaps. Apply
`references/academic-output-rubric.md` and, for material citation risks,
`references/citation-risk-policy.md`.

Persist through the existing preview/approval/CAS owner. In a formal handoff,
retain the claim, output/source version and limitations using
`references/stage-handoff-contract.md`. A completed bounded interpretation may
still leave a practical recommendation or stronger claim unsupported.
