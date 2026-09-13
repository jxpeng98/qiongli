# Robustness And Sensitivity Plan

| Check ID | Threat | Prespecified Check | Interpretation Rule |
|---|---|---|---|
| ROB-001 | Portfolio weighting | Repeat the complete CAPM/FF3 grid on equal-weighted returns. | H3 is supported only if attenuation remains positive; magnitude differences are reported, not hidden. |
| ROB-002 | Portfolio heterogeneity | Report every paired alpha change, not only aggregate summaries. | Portfolios with increased absolute alpha weaken broad attenuation language. |
| ROB-003 | Serial dependence | HAC(6) covariance plus Ljung-Box(6) residual diagnostics. | Residual serial correlation is a disclosed limitation; no conventional-SE fallback. |
| ROB-004 | Heteroskedasticity | HAC covariance plus Breusch-Pagan diagnostics. | Diagnostic rejections qualify uncertainty claims but do not alter the locked estimator. |
| ROB-005 | Factor collinearity | Correlation matrix and condition number for FF3 design. | Extreme condition numbers block coefficient-level interpretation. |
| ROB-006 | Data-vintage drift | Exact SHA-256 validation before parsing. | Any mismatch stops the run until explicit review; results are never silently refreshed. |

Not included: arbitrary subperiods, alternate HAC lags, five-factor/momentum models, and GRS testing. Add them only under a new approved research scope.

## Execution Status

| Check ID | Status | Observed Result | Evidence |
|---|---|---|---|
| ROB-001 | PASS | Equal-weighted mean attenuation is 54.57%. | `analysis/factor_model_sensitivity.md` |
| ROB-002 | PASS | Five portfolio-weighting cases have increased absolute alpha. | `analysis/results/model_comparison.csv` |
| ROB-003 | WARN | 48 of 100 Ljung-Box(6) screens have p < 0.05. | `analysis/factor_inference_diagnostics.md` |
| ROB-004 | WARN | 45 of 100 Breusch-Pagan screens have p < 0.05. | `analysis/factor_inference_diagnostics.md` |
| ROB-005 | PASS | FF3 condition number is 36.2578; absolute pairwise correlations are below 0.30. | `analysis/results/factor_diagnostics.json` |
| ROB-006 | PASS | Both archive bytes match the reviewed SHA-256 values. | `analysis/provenance.json` |
