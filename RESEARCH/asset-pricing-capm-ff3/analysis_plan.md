# Analysis Plan

## 1. Study ID

- Title: CAPM Versus Fama-French Three-Factor Pricing Errors in 25 Size–Book-to-Market Portfolios
- RQs: `framing/research_question.md`
- Design type: observational time-series model comparison.

## 2. Data Overview

- Unit: portfolio-month.
- Sources: official Fama/French Research Factors and 25 Portfolios Formed on Size and Book-to-Market CSV archives.
- Time: 1963-07 through the latest common month in the pinned inputs, expected 2026-06 for the current digests.
- Exclusions: pre-1963-07 rows and any noncommon dates. Duplicate dates, malformed dates, missing sentinels, or a missing portfolio fail the run.

## 3. Outcomes / Targets

| Outcome ID | Definition | Measure | Timepoint | Primary? |
|---|---|---|---|---|
| O1 | Portfolio-model pricing error | Monthly regression intercept alpha | Full common sample | Y |
| O2 | Cross-portfolio absolute pricing error | Mean and median of absolute alpha | Full common sample | Y |
| O3 | Model fit | Adjusted R-squared | Full common sample | Y |
| O4 | Portfolio-level alpha uncertainty | HAC(6) SE, t-statistic, p-value | Full common sample | Y |
| O5 | Inference diagnostics | factor condition/correlation; residual DW, LB(6), BP | Full common sample | N |

## 4. Estimands / Claims

- CAPM for portfolio p: `R_p,t − RF_t = alpha_p + beta_p (Mkt_t − RF_t) + error_p,t`.
- FF3: add `s_p SMB_t + h_p HML_t`.
- Primary aggregate target: `1 − mean_p |alpha_p,FF3| / mean_p |alpha_p,CAPM|` for value-weighted portfolios.
- Secondary targets: median absolute-alpha reduction, adjusted-R-squared change, portfolio-level alpha changes, and equal-weighted counterparts.
- Practical-significance rule: report magnitudes in monthly percentage points; do not impose an investment threshold.

## 5. Quantitative Analysis

| Outcome ID | Model / Estimator | Covariates | Assumption Checks | Robustness |
|---|---|---|---|---|
| O1–O4 | statsmodels OLS with intercept and HAC covariance | CAPM: Mkt-RF; FF3: Mkt-RF, SMB, HML | finite values, full rank, condition number, residual diagnostics | equal-weighted portfolios |

- HAC configuration: `cov_type=HAC`, `maxlags=6`, small-sample correction enabled.
- Multiple comparisons: no family-wise confirmatory claim. `abs(t)>1.96` counts are descriptive screens and labeled accordingly.
- Missing data: no imputation. Source sentinel `-99.99` or `-999` in the analysis window fails closed.
- Subgroups: none. No post-hoc period split is part of the primary analysis.

## 6. Data Quality Checks

- ZIP contains exactly one expected, traversal-safe member.
- Archive SHA-256 equals the reviewed pin.
- Monthly headers and all 25 unique portfolio names are present.
- Dates are valid `YYYYMM`, unique, sorted after parsing, and common coverage is contiguous enough to exceed 600 months.
- All analysis values are finite and free of source sentinels.
- Every fitted model has the same start, end, and observation count within a weighting scheme.

## 7. Diagnostics

- Factor correlation matrix and design-matrix condition numbers.
- Durbin-Watson statistic.
- Ljung-Box Q statistic and p-value at lag six.
- Breusch-Pagan LM statistic and p-value.
- Portfolio-level CAPM-to-FF3 alpha direction and absolute change.

## 8. Reporting Plan

- `analysis/results/model_results.csv`: 100 model rows.
- `analysis/results/model_comparison.csv`: 50 paired portfolio rows.
- `analysis/results/model_summary.csv`: CAPM and FF3 summaries by weighting.
- `analysis/results/residual_diagnostics.csv`: 100 diagnostic rows.
- `analysis/results/factor_diagnostics.json`: sample and factor design checks.
- `analysis/results/analysis_summary.json`: bounded headline quantities and model configuration.
- `analysis/results/results.md`: human-readable finding/interpretation/limitation separation.
- `analysis/results/output_digests.json`: SHA-256 over deterministic result outputs.

## 9. Deviations Log

| Date | Deviation | Rationale | Impact |
|---|---|---|---|
| 2026-08-24 | None before execution. | Not applicable. | None. |
