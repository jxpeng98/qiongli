# Meta-analysis / Evidence Synthesis Plan Template

<!--
Usage:
- Use this document to pre-specify how you will synthesize outcomes (meta-analysis, narrative, qualitative).
- Use the decision guidance in references/stage-E-synthesis.md; complete applicable sections only.
- Aligns with PRISMA 2020: items 12–15 and 13d–13f.
Save to: RESEARCH/[topic]/meta_analysis_plan.md
-->

# Evidence Synthesis Plan

## Review: [Your Review Title]
## Topic Folder: `RESEARCH/[topic]/`
## Date: [YYYY-MM-DD]

---

## 1) Outcomes and Synthesis Type

| Outcome ID | Outcome Definition | Primary Timepoint | Eligible Designs | Planned Synthesis | Rationale |
|-----------:|--------------------|-------------------|------------------|------------------|-----------|
| O1 | | | | Meta-analysis / Narrative / Qualitative | |
| O2 | | | | Meta-analysis / Narrative / Qualitative | |

**Direction Convention:** Define what “positive” means for each outcome (e.g., lower score = better).

For each consequential choice, record its source basis, missing prerequisite and
condition to proceed or revisit. Reuse decision IDs from `context/decision_log.md`.

---

## 2) Study Grouping Rules (Per Outcome)

Define how studies will be grouped for each synthesis:
- Population subgroups:
- Setting/context:
- Intervention/exposure types:
- Comparator types:
- Measurement instruments:
- Follow-up windows:

---

## 3) Data Preparation Rules

### Effect size selection (within a study)
- Primary endpoint selection rule:
- Multiple timepoints rule:
- Multiple eligible measures rule:
- Multiple reports of same study (dedup/merge rule):

### Multi-arm / Cluster / Repeated Measures
- Multi-arm trials handling:
- Cluster designs handling (ICC / effective sample size):
- Repeated measures handling (change scores, correlation assumptions):

### Handling missing or incomplete reporting
- Missing information and source/recovery action (contact requires actual authorization):
- Derive SE from CI/p-values? (rules below)
- Imputation rules (if any):

**Derivation conditions:** For a symmetric normal-based 95% interval on the
analysis scale, `SE ≈ (upper - lower) / (2 * 1.96)`. Ratio measures generally need
the log scale. Check the interval method/level; t-based intervals need the
appropriate critical value/degrees of freedom, and other interval constructions
may not permit this recovery. Record the source and assumptions; otherwise leave
the variance unavailable. See the applicable [effect-measure guidance](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-06).

---

## 4) Meta-analysis Specification (If Applicable)

### Effect measures (PRISMA item 12)
For each outcome, specify the *analysis scale*:
| Outcome ID | Effect Measure | Analysis Scale | Notes / Conversions |
|-----------:|----------------|----------------|---------------------|
| O1 | OR / RR / SMD / MD / Fisher’s z / etc. | log-scale / raw / z | |

### Model choice and estimators
- Effect model and target quantity (justify its assumptions):
- Between-study variance (τ²) estimator: DL / REML / Paule-Mandel / other
- CI method: normal / Hartung-Knapp / other

### Heterogeneity (PRISMA item 13e)
- Statistics to report: I², τ², Q
- Evidence that would change grouping, model or claim scope:
- Planned subgroup analyses:
- Planned meta-regression (only if enough studies; specify covariates):

### Sensitivity analyses (PRISMA item 13f)
Pre-specify:
- Leave-one-out:
- Exclude high risk-of-bias:
- Alternative τ² estimator:
- Alternative effect-size choice (adjusted vs unadjusted):

### Small-study effects / missing results (PRISMA item 14)
Planned assessments (choose as appropriate):
- Funnel plot:
- Small-study-effects test (justify applicability and information available):
- Trim-and-fill (exploratory):

---

## 5) Certainty / Confidence in Evidence (Optional)

Use the framework required by the protocol or justified for the review question;
if using GRADE:
- Outcomes assessed:
- Downgrade/upgrade rules:
- Planned Summary of Findings table: `RESEARCH/[topic]/grade_sof.md`

---

## 6) Software / Reproducibility

- Data source: `RESEARCH/[topic]/effect_size_table.md` (and optional CSV export)
- Code location: `RESEARCH/[topic]/analysis/`
- Tools:
  - Available implementation and version appropriate to the chosen method:
  - Input/output identity and checks needed before results can feed writing:

---

## 7) Deviations / Amendments Log

| Date | Change | Rationale |
|------|--------|-----------|
| | | |
