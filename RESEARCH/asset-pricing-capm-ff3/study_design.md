# Study Design: CAPM Versus FF3 Pricing Errors

## 1. Overview

- Research goal: descriptive and model-comparative.
- Primary RQ: How much does FF3 attenuate CAPM pricing errors across the 25 U.S. size/book-to-market portfolios?
- Hypotheses: H1–H3 in `framing/hypothesis.md`.
- Contribution: a digest-pinned current-vintage replication and reproducibility artifact.

## 2. Background

CAPM supplies a market-only benchmark, while the Fama-French literature motivates size and book-to-market factors. Historical summaries cannot establish the result for the exact current archive; this study computes it directly and records inference limits.

## 3. RQ-Method-Outcome Matrix

| RQ / Hypothesis | Data Source | Measurement | Estimand / Target | Method | Outcome / Evidence |
|---|---|---|---|---|---|
| Main RQ / H1 | Official factors plus value-weighted 25 portfolios | Portfolio return minus RF; Mkt-RF, SMB, HML | Cross-portfolio relative reduction in mean absolute alpha | 25 CAPM and 25 FF3 time-series OLS regressions with HAC(6) covariance | `analysis/results/model_summary.csv` |
| SRQ2 / H2 | Same common sample | Portfolio alpha and HAC standard error | Count and identity of FF3 intercepts with absolute HAC t-statistic above 1.96 | Portfolio-level regression output; descriptive threshold | `analysis/results/model_results.csv` |
| SRQ3 / H3 | Equal-weighted section of the same archive | Same variables under alternative portfolio weighting | Sign and magnitude of equal-weighted attenuation | Repeat all 50 model fits | `analysis/results/model_summary.csv` |
| SRQ4 | Factors and fitted residuals | Factor correlations, condition number, DW, Ljung-Box(6), Breusch-Pagan | Diagnostic prevalence and range | Deterministic diagnostic functions | `analysis/results/factor_diagnostics.json`; `residual_diagnostics.csv` |

## 4. Design Choice

- Study type: observational secondary-data model comparison.
- Identification logic: none; no causal effect is estimated.
- Unit of analysis: portfolio-month for estimation and portfolio-model for comparisons.
- Setting: U.S. monthly returns in the Kenneth French Data Library.

| Decision | Chosen Option | Rejected Alternatives | Reason |
|---|---|---|---|
| Models | CAPM and FF3 | Additional factors | Preserves one answerable comparison. |
| Start date | 1963-07 | 1926 start or result-selected subsamples | Fixed conventional FF3-era sample. |
| Covariance | HAC with six lags | Conventional OLS SE | Monthly residual dependence is plausible. |
| Weighting | Value-weighted primary; equal-weighted sensitivity | One weighting only | Tests dependence on portfolio weighting without new data. |

## 5. Population And Sample

- Population/frame: the 25 intersections of five size and five book-to-market portfolios defined by the official source.
- Inclusion: months from 1963-07 present in factors and the selected portfolio section.
- Exclusion: dates outside common coverage; any month containing a missing sentinel causes failure rather than deletion.
- Target N: all common months and all 25 portfolios; no sampling-based power calculation is applicable.

## 6. Measures And Procedures

| Construct | Role | Operationalization / Evidence Source | Notes |
|---|---|---|---|
| Portfolio excess return | Outcome | portfolio percent return minus RF, divided by 100 | Monthly decimal return. |
| Market excess return | CAPM/FF3 factor | Mkt-RF divided by 100 | Official factor file. |
| Size factor | FF3 factor | SMB divided by 100 | Official factor file. |
| Value factor | FF3 factor | HML divided by 100 | Official factor file. |
| Pricing error | Primary fitted quantity | intercept alpha | Benchmark-relative proxy only. |
| Attenuation | Primary comparison | 1 − mean(abs FF3 alpha) / mean(abs CAPM alpha) | Reported by weighting scheme. |

Procedure: verify archive names and digests, parse monthly sections, reject duplicates/sentinels, inner-join dates, enforce the fixed start, estimate models, compute diagnostics, and serialize deterministic outputs.

## 7. Analysis Summary

Full specification: `analysis_plan.md`. No exploratory model selection is allowed inside the primary run.

## 8. Validity And Risk

- Construct: alpha depends on the chosen benchmark and is not directly observed mispricing.
- Internal: the design does not identify causality or risk-factor mechanisms.
- External: the result may not generalize beyond these portfolios, the U.S., monthly data, or the pinned vintage.
- Statistical conclusion: HAC addresses within-series dependence, but no cross-portfolio joint test is claimed.
- Measurement: official aggregate portfolios avoid constituent cleaning choices but inherit upstream revisions.

## 9. Ethics And Governance

There are no human participants, PII, credentials, or restricted microdata. Raw publisher files are downloaded locally, digest-verified, ignored by Git, and not redistributed.

## 10. Reproducibility

One PEP 723 script, a script lock, fixed archive digests, deterministic serialization, and a `--check` mode own the computational baseline.

## 11. Handoff Artifacts

| Artifact | Purpose | Owner |
|---|---|---|
| `analysis_plan.md` | Estimands, models, inference rules | C3 |
| `design/variable_spec.md` | Exact variables and transformations | C3 |
| `design/dataset_plan.md` | Sources, coverage, digests, constraints | C4 |
| `data_management_plan.md` | Storage, retention, sharing | C4 |
| `code/code_specification.md` | Executable I/O and failure contract | I5 |

## 12. Preregistration

No public preregistration is claimed. The committed design and analysis plan act as a local decision record for this reproducibility project.
