# Manuscript Claim Map

| Claim ID | Claim | Claim Type | Citation Keys | Evidence Pointer | Confidence | Action |
|---|---|---|---|---|---|---|
| CLM-001 | FF3 reduces mean absolute monthly alpha by 55.98 percent across the 25 value-weighted portfolios in the pinned sample. | result | famaFrench1993commonrisk; famaFrench1996multifactor | analysis/results/analysis_summary.json#primary_result | high | keep |
| CLM-002 | FF3 reduces mean absolute monthly alpha by 54.57 percent across the 25 equal-weighted portfolios in the pinned sample. | robustness | famaFrench1993commonrisk; famaFrench1996multifactor | analysis/results/analysis_summary.json#sensitivity_result | high | keep |
| CLM-003 | Six value-weighted FF3 intercepts have absolute HAC t-statistics above 1.96. | result | neweyWest1987hac | analysis/results/model_summary.csv#value_weighted-ff3 | high | hedge |
| CLM-004 | Mean adjusted R-squared rises from 0.7338 under CAPM to 0.9096 under FF3 for value-weighted portfolios. | result | famaFrench1993commonrisk | analysis/results/model_summary.csv#value_weighted | high | hedge |
| CLM-005 | Residual screens flag serial dependence in 48 and heteroskedasticity in 45 of the 100 fitted models. | limitation | neweyWest1987hac | analysis/results/residual_diagnostics.csv | high | keep |
| CLM-006 | The factors and test assets share size and book-to-market construction. | limitation | famaFrench1993commonrisk | notes/kennethFrenchDataLibrary.md#extracted-points | high | keep |
| CLM-007 | HAC covariance with six lags is used to allow within-series heteroskedasticity and autocorrelation. | method | neweyWest1987hac | analysis_plan.md#models | medium | hedge |
