---
id: prereg-writer
stage: C_design
description: "Generate preregistration documents for OSF/AsPredicted/ClinicalTrials.gov from study design and analysis plan."
inputs:
  - type: DesignSpec
    description: "Complete study design"
  - type: AnalysisPlan
    description: "Pre-specified analysis plan"
  - type: HypothesisSet
    description: "Testable hypotheses"
outputs:
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must distinguish hypotheses or analytic aims, planned decisions and prior knowledge"
  - "Must accurately disclose collection, access, analysis and registration status"
  - "Must specify methods and amendment rules appropriate to the study and agreed protocol"
failure_modes:
  - "Exploratory research doesn't fit prereg template"
  - "Analysis plan too flexible to pre-specify fully"
tools: [filesystem]
tags: [design, preregistration, OSF, AsPredicted, transparency]
domain_aware: true
---

# Pre-registration Writer Skill

## Purpose

Draft a preregistration that states what is planned, what is already known and
how later changes will be reported. Follow the **Design judgment contract** in
`references/stage-C-design.md` and the selected registry's actual requirements.
Preparing a draft does not register it or establish prospective status.

## Related Task IDs

- `C5` (preregistration)

## When to Use

- Preregistration is requested or required by the agreed protocol or applicable rules.
- A study or secondary analysis needs a transparent record of planned decisions.
- An existing registration needs a clearly dated amendment or deviation record.

## Inputs

- `DesignSpec`: study design, procedures and sampling rationale.
- `AnalysisPlan`: specified methods and interpretation rules.
- `HypothesisSet`: hypotheses, questions or interpretive aims appropriate to the study.
- Actual data collection, access and analysis history; ethics/access status;
  intended registry/template and any existing registration record.

Reuse supplied facts. Missing information remains an explicit gap, including
whether data were examined or approval covers the proposed use. Formal gap notes
use `RESEARCH/[topic]/context/gap_notes.md` through the existing write owner;
otherwise provide proposed text without claiming a file was saved.

## Process

### Establish status before drafting

Record collection dates, what each analyst has accessed or examined, analyses
already performed and the work still planned. Existing data do not imply they
were unseen; a model not yet run does not erase prior inspection of its outcomes.
Use the actual registry timestamp/ID only when supported by a registration record.
Never backdate a draft or assert that relevant variables were unexamined unless
that is known to be true. Explain limits on confirmatory claims caused by prior
knowledge and distinguish future pre-specified work from exploratory decisions.

Check the intended registry's current template and timing requirements when
available. If unavailable, prepare a clearly labeled generic draft and leave
registry compliance unverified. Do not invent registry fields, IDs or approval.

### Specify the study

Use `templates/preregistration-template.md` for the canonical draft structure,
and add details required by the chosen registry and study:

- Questions/hypotheses and primary/secondary status. Include directional predictions
  only when justified and intended; do not manufacture them for qualitative or
  non-directional questions.
- Design, units, setting, collection/reuse procedures and sampling adequacy under
  Stage C. Preserve required power assumptions and stopping rules; do not insert
  default effect sizes, alpha, sample counts or saturation claims.
- Outcomes/measures or qualitative analytic targets and evidence sources. Verify
  any instrument properties or reliability estimates before using them.
- Exact planned models, estimators, covariates and inference rules when applicable;
  for qualitative work, analytic approach, researcher role and evidence/decision
  handling. Identify assumptions and diagnostic/reporting commitments.
- Justified exclusion, transformation, missingness and multiplicity decisions.
  Do not choose deletion or imputation from a generic missing-percentage recipe.
- Planned robustness/sensitivity checks and their interpretation; exploratory
  analyses separately labeled. Preserve protocol requirements and independent review.

### Check readiness and deviations

Check that each question maps to its proposed evidence and method, that the plan
is feasible, and that required inputs/reviews are complete. Missing consent,
ethics or access decisions block the affected activity; permitted planning can
continue. Never equate a completed draft with permission to begin analysis.

Retain the original specification and date later amendments. Use the deviation
log without rewriting history:

| Date | Section | Original Plan | Change Made | Justification |
|------|---------|--------------|-------------|---------------|

Describe whether each change preceded or followed relevant data inspection or
analysis. A change of plan does not retrospectively make prior work confirmatory.
External registration is a separate authorized action whose result must be observed.

## Output Contract

- `Preregistration`: `RESEARCH/[topic]/preregistration.md` for a formal C5 draft;
  keep the existing project path and preview/approval/CAS owner.
- A chat status review can give draft text and gaps without claiming C5 completion.
- Separate findings, interpretations and implications; do not invent citations,
  data, sample sizes, statistical results, reviewer comments or registration status.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose.

## Quality Bar

A draft is ready for the requested review when its question-method mapping,
applicable specification and sampling rationale are complete, prior knowledge is
accurately disclosed, and outstanding requirements are visible. Registry readiness
also depends on its actual template and timing rules. Report draft, submitted and
registered states separately; missing required review or registration evidence
cannot be called complete.

## Common Pitfalls

- Treating a local date or a draft filename as a registry timestamp.
- Copying an example's coefficients, exclusions or missing-data rules into a protocol.
- Claiming unseen data after examining outcomes, or hiding later amendments.
- Dropping an agreed protocol requirement because it is optional for other studies.
