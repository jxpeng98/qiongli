# Stage C — Study Design & Analysis Plan (C1–C5)

This stage converts framing into an executable plan: design choices, measurement, estimands, and pre-specified analysis.

## Canonical outputs (contract paths)

- `C1` → `study_design.md`
- `C1_5` → `design/rival_hypotheses.md`
- `C2` → `instruments/`
- `C3` → `analysis_plan.md`, `design/variable_spec.md`
- `C3_5` → `design/robustness_plan.md`
- `C4` → `data_management_plan.md`, `design/dataset_plan.md`
- `C5` → `preregistration.md`

## Quality gate focus

- `Q1` (question-to-method alignment) is enforced here: every RQ/hypothesis must map to data + model + outcome, or to setting + evidence source + analytic lens for qualitative work.
- `Q4` (reproducibility baseline): document data lineage, missingness, and analysis decisions.
- Semantic gate report: update `quality-gate-report.md` with `q1_rq_method_alignment`; evidence must anchor each RQ/hypothesis to method, data or setting, outcome or evidence form, measurement or evidence source, estimand or analytic target, and analysis strategy.

## Design judgment contract

Start with the requested decision or deliverable and reuse known project context.
A role supplies a perspective, not extra tasks or authority. A narrow design
question can be answered in chat; formal C tasks retain their canonical outputs,
Q1/Q4 evidence and selected method requirements. Ask only about missing facts that
would change the design, claim, access decision or required deliverable.

Choose methods that support the research question under the actual constraints:

- Justify sampling adequacy for the intended inference. Quantitative work may need
  power, precision, a minimum detectable effect, or an explicit account of a fixed
  dataset and its limits. Do not invent effect sizes, variance or achieved power.
  Qualitative work needs a rationale consistent with its analytic tradition and
  available material; saturation and coding agreement are not universal tests.
  In reflexive thematic analysis, discuss scope, richness, analytic development
  and reflexivity rather than imposing saturation or independent coding agreement.
- Select rivals and robustness checks for consequential threats. There is no
  default count of papers, rivals or checks. State what each check could establish,
  its assumptions, what would change the claim, and what remains unresolved.
  A design label, more controls or stable significance does not establish causality.
- Preserve explicit user/protocol requirements, saved minima, required independent
  review, applicable reporting standards and selected method diagnostics. Do not
  waive them because generic defaults were removed. Mark inapplicable reporting
  items with a reason; missing required evidence remains blocked or incomplete.
- Reuse established ethics/access decisions for the exact proposed use. If required
  permission or review is unresolved, block the affected access, collection or
  analysis; authorized planning can continue. Never infer an exemption or approval.
- Preregistration is required when the agreed protocol or applicable requirement
  says so. Record actual collection, access, analysis and registration status.
  A draft is not a registration; do not backdate or describe examined data as unseen.
  Distinguish pre-specified decisions from later amendments and exploratory work.

Plans describe proposed actions. Execution, project writes and external registration
remain subject to the existing tool availability, scope and preview/approval/CAS
owners. Do not change an approved model, protocol or source data to make a check pass.

Methodological basis: [sample-size justification](https://online.ucpress.edu/collabra/article/8/1/33267/120491/Sample-Size-Justification)
and [saturation in thematic analysis](https://uwe-repository.worktribe.com/output/4820803/to-saturate-or-not-to-saturate-questioning-data-saturation-as-a-useful-concept-for-thematic-analysis-and-sample-size-rationales).

---

## C1 — Study Design

**Definition of done**

- Study type justified (experiment / quasi / observational / qualitative / mixed)
- Unit of analysis + sampling frame is clear
- Constructs / sensitizing concepts → measures / evidence sources → data collection procedure is specified
- Case boundaries, setting, and analytic strategy are explicit for qualitative work
- Threats to validity addressed at design time (not only in “limitations”)

**Recommended minimum sections: `study_design.md`**

```markdown
# Study Design

## Research question alignment
| RQ/Hypothesis | Construct(s) | Data source | Outcome/metric or evidence form |
|---|---|---|---|

## Design choice & rationale
- Design type:
- Identification logic (if causal):
- Analytic tradition / qualitative strategy (if qualitative):
- Why alternatives were rejected:

## Sample / data
- Population/frame:
- Inclusion/exclusion:
- Available/target sample, adequacy rationale and limits:

## Measures / operationalization
| Construct / sensitizing concept | Measure / protocol / evidence source | Reliability/validity or trustworthiness notes |
|---|---|---|

## Procedure
- Recruitment / collection:
- Timeline:

## Validity & risk
- Internal:
- Construct:
- External:
- Statistical conclusion:
- Credibility / transferability / dependability / confirmability (if qualitative):
```

For qualitative studies, specify the applicable procedures and their rationale:
- case selection rationale and setting boundaries
- interview / observation / document collection rules
- within-case vs cross-case logic
- reflexivity and memoing plan
- disconfirming-case / rival-interpretation plan

---

## C1_5 — Rival Hypotheses / Alternative Explanations

Goal: identify credible alternative explanations and the limits of distinguishing them.

**Definition of done**
- Consequential rivals grounded in theory or available evidence
- For each rival: how it could produce the pattern, what evidence could distinguish
  it under stated assumptions, and any unresolved limitation

Suggested table: `design/rival_hypotheses.md`

```markdown
| Rival | Mechanism | Observable implication | Design control / test |
|---|---|---|---|
```

---

## C2 — Instruments

Applies to surveys, interviews, coding schemes, rubrics, measurement protocols.

**Definition of done**
- Instrument exists and matches constructs or sensitizing concepts
- Administration protocol (timing, prompts, consent links)
- Versioning plan (if iterative)

Write under: `instruments/` (e.g., `instruments/survey.md`, `instruments/interview_guide.md`).

---

## C3 — Analysis Plan

Specify *before* results stabilize: estimands or analytic targets, models or coding logic, assumptions, missingness/evidence gaps, and inference choices.

**Definition of done**
- Primary estimand(s) or analytic targets clearly defined
- Model family or qualitative analytic procedure specified
- Missing data or evidence-gap strategy specified
- Multiple comparisons / researcher degrees of freedom / rival-interpretation degrees of freedom addressed
- Variable roles, coding, and transformation rules are frozen in `design/variable_spec.md`

Minimum structure: `analysis_plan.md`

```markdown
# Analysis Plan

## Estimands / analytic targets
- Primary:
- Secondary:
- Focal process / meaning / mechanism claims (if qualitative):

## Variables / coding frame
| Role / code family | Variable / code | Measurement / evidence source | Notes |
|---|---|---|---|

## Models / analytic procedures
- Primary model (if quantitative):
- Qualitative analytic procedure (if qualitative):
- Assumptions:
- Diagnostics / trustworthiness checks:

## Missing data / evidence gaps
- Expected missingness or thin spots:
- Handling:

## Inference / interpretation rules
- Effect sizes + uncertainty reporting:
- Quote / episode / case-selection rules for write-up (if qualitative):
- Multiple testing / rival interpretation control:

## Robustness hooks (links to C3_5)
- ...
```

Companion artifact: `design/variable_spec.md`

```markdown
# Variable / Code Specification

| Role | Variable / code | Source | Unit / coding / evidence form | Transformation | Notes |
|---|---|---|---|---|---|
```

---

## C3_5 — Robustness / Sensitivity Plan

Pre-specify checks linked to the study's actual threats and method assumptions.
For qualitative work, choose compatible procedures such as examining rival
interpretations, deviant cases, reflexive memos or sources that are available.

**Definition of done**
- A prioritized list of robustness checks linked to specific threats
- Interpretation rules for changes in magnitude, uncertainty, meaning or scope;
  unresolved threats are not a pass, and results are not a vote by significance

Write into: `design/robustness_plan.md`.

---

## C4 — Data Management Plan (DMP)

Treat as a reproducibility + ethics artifact.

**Definition of done**
- Storage, access control, retention, and sharing plan
- De-identification linkage (D3) if human/sensitive data
- Code/data availability statement draft inputs (for D2/H1)
- Dataset feasibility, provenance, and access constraints are captured in `design/dataset_plan.md`
- Transcript/audio/document handling rules are explicit when qualitative evidence is collected

Write into:
- `data_management_plan.md`
- `design/dataset_plan.md`

Recommended minimal structure for `design/dataset_plan.md`

```markdown
# Dataset Plan

| Dataset | Coverage | Access | Key variables | Risks |
|---|---|---|---|---|
```

---

## C5 — Preregistration Draft

Produce C5 when requested or required by the agreed protocol or applicable rules.
Use a format appropriate to the study and intended registry; record the actual
prior access and analysis history, including work on existing data.

**Definition of done**
- Hypotheses/RQs, design, exclusion rules, and analysis plan are frozen in a prereg doc
- Deviations policy and current draft/registration status described truthfully

Write into: `preregistration.md`.
