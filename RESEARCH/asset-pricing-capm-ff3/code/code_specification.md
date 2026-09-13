---
task_id: I5
template_type: code_specification
topic: asset-pricing-capm-ff3
primary_artifact: code/code_specification.md
---

# Code Specification

## Spec Contract Block

```json
{
  "task_id": "I5",
  "topic": "asset-pricing-capm-ff3",
  "method_or_pipeline": "pinned CAPM and FF3 portfolio time-series regressions",
  "primary_artifact": "code/code_specification.md",
  "estimand": "relative reduction in cross-portfolio mean absolute monthly alpha",
  "analysis_plan_source": "analysis_plan.md",
  "inputs": [
    {"path": "analysis/data/raw/F-F_Research_Data_Factors_CSV.zip", "schema": "one-member official ZIP with monthly Mkt-RF, SMB, HML, RF CSV section"},
    {"path": "analysis/data/raw/25_Portfolios_5x5_CSV.zip", "schema": "one-member official ZIP with monthly 25-column value/equal-weighted sections"}
  ],
  "outputs": [
    {"path": "analysis/results/model_results.csv", "format": "deterministic CSV"},
    {"path": "analysis/results/model_comparison.csv", "format": "deterministic CSV"},
    {"path": "analysis/results/model_summary.csv", "format": "deterministic CSV"},
    {"path": "analysis/results/residual_diagnostics.csv", "format": "deterministic CSV"},
    {"path": "analysis/results/factor_diagnostics.json", "format": "canonical pretty JSON"},
    {"path": "analysis/results/analysis_summary.json", "format": "canonical pretty JSON"},
    {"path": "analysis/results/results.md", "format": "deterministic Markdown"},
    {"path": "analysis/results/output_digests.json", "format": "SHA-256 manifest over other deterministic outputs"}
  ],
  "dataset_lineage": {
    "raw_inputs": ["two official digest-pinned Kenneth French ZIP archives"],
    "cleaning_rules": ["parse named monthly sections", "convert percentages to decimals", "subtract RF once", "inner-join dates"],
    "exclusions": ["months before 1963-07", "dates outside common coverage"],
    "derived_variables": ["portfolio excess return", "absolute alpha", "paired alpha change", "attenuation ratio"],
    "sample_construction": "all 25 portfolios over 1963-07 through the latest common pinned month"
  },
  "manuscript_outputs": [],
  "diagnostics": ["factor correlations", "condition numbers", "Durbin-Watson", "Ljung-Box(6)", "Breusch-Pagan"],
  "robustness_checks": ["complete equal-weighted rerun", "portfolio-level paired comparison"],
  "dependencies": {
    "python": ["numpy==2.3.2", "pandas==2.3.2", "statsmodels==0.14.5"]
  },
  "seeds_policy": {
    "global_seed": "not applicable",
    "nondeterminism_notes": "No random operation is permitted; deterministic order and formatting are required."
  },
  "acceptance_tests": [
    {"name": "input integrity", "metric": "archive/member/date/sentinel checks", "pass_condition": "all checks pass before estimation"},
    {"name": "model grid", "metric": "row count", "pass_condition": "100 model rows, 50 paired comparisons, 4 summary rows"},
    {"name": "sample consistency", "metric": "date and nobs fields", "pass_condition": "all rows share the locked common sample"},
    {"name": "determinism", "metric": "byte comparison", "pass_condition": "--check recomputation equals every committed deterministic output"}
  ],
  "blocked_decisions": []
}
```

## Goal

Build the smallest auditable pipeline that downloads/verifies the two official inputs, estimates the locked model grid, writes diagnostics, and proves deterministic reruns. Pandas is pinned only because it is a statsmodels runtime dependency whose newer API is incompatible with the selected statsmodels release.

## Non-Goals

- No framework, package, notebook, database, service layer, plotting stack, or product dependency change.
- No raw-data redistribution, constituent reconstruction, new factor, joint GRS test, or trading output.

## Inputs And Outputs

The JSON contract block and `design/dataset_plan.md` are authoritative. Paths are resolved relative to `run_analysis.py`; absolute paths never enter committed outputs.

## Functional Requirements

1. GIVEN a missing local archive, WHEN the normal run starts, THEN download only its fixed official HTTPS URL.
2. GIVEN any archive/member digest or path mismatch, WHEN validation runs, THEN exit nonzero before parsing or output mutation.
3. GIVEN malformed/duplicate dates, missing portfolios, sentinels, or nonfinite values, WHEN constructing the sample, THEN exit nonzero.
4. GIVEN valid inputs, WHEN fitting, THEN emit exactly 25 portfolios × 2 models × 2 weighting schemes.
5. GIVEN `--check`, WHEN recomputing, THEN compare every expected deterministic output byte-for-byte and write nothing.
6. GIVEN `--profile`, WHEN executing, THEN print a bounded cProfile summary while preserving the same output contract.

## Non-Functional Requirements

- Performance: complete on a laptop in under 30 seconds after dependencies and inputs are cached.
- Determinism: sorted rows, fixed decimal formatting, canonical JSON key ordering, no timestamps in result outputs.
- Logging: concise stdout status only; retrieval time belongs solely to `analysis/provenance.json`.
- Safety: no ZIP extraction to disk; read the validated member in memory.

## Edge Cases And Failure Modes

- HTTP error or partial archive; digest drift; extra ZIP member; traversal-like member; invalid UTF-8.
- Missing section/header, repeated portfolio name, invalid `YYYYMM`, duplicate month, sentinel, or nonfinite number.
- Missing fixed start month, insufficient observations, rank-deficient design, zero CAPM mean absolute alpha.
- Missing/stale deterministic output under `--check`.

## Validation Matrix

| Check | Metric / Observable | Pass Condition | Artifact |
|---|---|---|---|
| Parser/input | explicit assertions | all trusted-boundary checks pass | stdout and provenance |
| Model grid | CSV counts | 100 / 50 / 4 / 100 rows | result CSV files |
| Diagnostic coverage | output fields | every model has DW, LB(6), BP | `residual_diagnostics.csv` |
| Determinism | bytes and SHA-256 | second run is identical | `output_digests.json`; `--check` |

## Disallowed Shortcuts

- Do not skip digest validation, ignore missing data, use conventional OLS standard errors, silently change the sample, or label descriptive threshold counts as family-wise inference.

## Blocked Decisions / Escalations

- None. A digest/schema change must stop and become a separate explicit review.
