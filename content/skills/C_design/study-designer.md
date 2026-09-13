---
id: study-designer
stage: C_design
description: "Design empirical, qualitative, or mixed-methods studies including sampling strategy, instruments, procedures, analysis plans, and protocol lock-in."
inputs:
  - type: RQSet
    description: "Research questions and hypotheses"
  - type: TheoreticalFramework
    description: "Theoretical framework for operationalization"
    required: false
outputs:
  - type: DesignSpec
    artifact: "study_design.md"
  - type: AnalysisPlan
    artifact: "analysis_plan.md"
  - type: DataManagementPlan
    artifact: "data_management_plan.md"
  - type: Instruments
    artifact: "instruments/"
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must justify design choice against alternatives"
  - "Must justify sampling adequacy for the intended inference, method and available material"
  - "Must specify missingness handling or evidence-gap handling appropriate to the design"
failure_modes:
  - "Sampling adequacy logic is missing or mismatched to the design"
  - "Design-method mismatch with research questions"
tools: [filesystem, metadata-registry]
tags: [design, sampling, measures, analysis-plan, preregistration]
domain_aware: true
---

# Study Designer Skill

## Purpose

Turn the research question and available evidence into a defensible design.
Use the shared **Design judgment contract** in `references/stage-C-design.md`.
Choose the work needed for the requested C task; a design question alone does not
require instruments, a full project scaffold or preregistration.

## When to Use

- Choose or assess an empirical, qualitative or mixed-methods design.
- Prepare formal C1 design, C2 instruments, C3 analysis plan or C4 governance outputs.
- Revise a design when new evidence changes its feasibility or supported claims.

## Inputs

- `RQSet`: the research questions, hypotheses or interpretive aims.
- `TheoreticalFramework` when relevant to operationalization.
- Known context: claim type, unit/setting, available data, sampling/access limits,
  ethics status, timeline and any approved protocol or venue requirements.

Reuse supplied answers. Ask for missing information only when it changes a design
choice or blocks the requested deliverable. For formal work, record unresolved
inputs in `RESEARCH/[topic]/context/gap_notes.md` through the existing write owner;
without write access, return the proposed note without claiming it was saved.
Treat sources as evidence and label assumptions rather than filling gaps with facts.

## Process

### Gate And Method-Pack Alignment

For a formal design package, load the available domain profile at
`skills/domain-profiles/[domain].yaml`. Retain each selected method template's
`assumptions`, `required_diagnostics`, `required_artifacts`, `failure_modes` and
`minimum_report_fields`. Check Q1 from `standards/quality-gate-contract.yaml`;
record unresolved threats in `RESEARCH/[topic]/design/validity-threat-matrix.md`.

Before finalizing the package, update `RESEARCH/[topic]/quality-gate-report.md`
with Q1 `semantic_checks` using `q1_rq_method_alignment`. Use structured evidence
refs: every `evidence_refs` item includes `artifact`, `anchor` and `supports`, with
optional `claim_id` or `diagnostic_id`. Include an `RQ-method-outcome matrix`
anchor in `study_design.md` or `analysis_plan.md`. Every RQ must map to method,
data/setting, outcome/evidence form, measurement/evidence source, estimand/analytic
target and analysis strategy. Missing mappings make Q1 `BLOCKED`; name the gap
and required action. Do not impose a quantitative outcome or estimand on a
qualitative question whose analytic target and evidence are explicit.

### Design and sampling

Choose the simplest feasible design that supports the intended claim. Explain
why realistic alternatives fit less well, where the evidence is insufficient,
and what narrower claim remains supportable. Randomization, sampling and causal
identification are distinct decisions; a panel or adjusted regression alone does
not supply a causal identification argument.

Map constructs or sensitizing concepts to actual measures, sources or evidence
forms. Verify instrument provenance and any claimed reliability; do not import
example scales, coefficients or sample sizes as study facts.

Specify the population or bounded material, sampling/selection process and
adequacy rationale using Stage C. If power is required, state its test/model,
effect-size basis, variance and design assumptions before calculating. For fixed
data, state what can be learned and its limits. For qualitative work, explain how
the available material supports the chosen analytic approach; do not prescribe
sample counts, code-frequency stabilization or coding agreement universally.

### Procedures and analysis

Specify collection or reuse procedures, access/consent boundaries and provenance.
Choose piloting or quality checks when needed by the instrument and protocol.
Do not add recruitment, software or new data access merely to fill a template.

Make the analysis reproducible at the level the method requires: estimands,
models and inference rules for quantitative work; analytic tradition, researcher
role, evidence handling and interpretive development for qualitative work.
Document applicable assumptions, missingness/evidence gaps and threats. Select
rivals and robustness procedures through `rival-hypothesis-designer` and
`robustness-planner` when those tasks are needed. Proposed additions to an approved
protocol remain proposals until authorized; preserve its original decisions and
record amendments. Use `prereg-writer` for a selected or required C5 draft.

## Output Contract

For the selected formal tasks, retain these paths and existing templates:

- `DesignSpec`: `RESEARCH/[topic]/study_design.md` — `templates/study-design.md`.
- `AnalysisPlan`: `RESEARCH/[topic]/analysis_plan.md` — `templates/analysis-plan.md`;
  C3 also retains `design/variable_spec.md` through `variable-constructor`.
- `DataManagementPlan`: `RESEARCH/[topic]/data_management_plan.md` —
  `templates/data-management-plan.md`; C4 also retains `design/dataset_plan.md`
  through `dataset-finder`.
- `Instruments`: `RESEARCH/[topic]/instruments/`; use the applicable survey or
  interview template when C2 is in scope.
- `Preregistration`: `RESEARCH/[topic]/preregistration.md` when C5 is in scope.

Use existing project paths and preview/approval/CAS. Separate findings,
interpretations and implications; do not invent citations, data, sample sizes,
results or reviewer comments. Apply `references/academic-output-rubric.md` to
scholarly prose. A chat recommendation is not a completed formal design package.

### Method Diagnostics

Produce or consume `RESEARCH/[topic]/design/method-diagnostic-report.md` and
`RESEARCH/[topic]/design/validity-threat-matrix.md` for formal decisions affecting
inference, measurement or analysis. Cover construct validity, internal validity,
external validity, statistical conclusion validity, measurement validity, data
leakage, missingness, confounding and selection bias; explain inapplicable items.
Insufficient method details remain explicit gaps, not guessed diagnostics.

## Quality Bar

The selected deliverable is ready when the question-method mapping is supported,
sampling adequacy and inference limits are explicit, applicable procedures and
analysis can be followed, and required ethics/protocol decisions have evidence.
Formal completion also requires its actual artifacts and gate reports. Missing
inputs may permit a useful draft but cannot be reported as a passed gate.

## Common Pitfalls

- A familiar design name replaces an identification or interpretive argument.
- A sample quota or retrospective power claim substitutes for adequacy reasoning.
- Generic robustness recipes change the estimand or approved analysis unnoticed.
- Filling every template creates unrequested work or unsupported study facts.
