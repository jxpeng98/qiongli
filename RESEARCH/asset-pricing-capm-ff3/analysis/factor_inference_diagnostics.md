# Factor And Inference Diagnostics

## Factor Design

- FF3 design condition number: 36.2578; CAPM design condition number: 22.4183.
- Factor correlations: Mkt-RF/SMB 0.2905, Mkt-RF/HML -0.2098, and SMB/HML -0.1385.
- The input checks found no duplicate months or missing-value sentinels in the analysis sample.

These values do not indicate an extreme factor-collinearity failure in this design.

## Residual Screens

Across 100 portfolio-model fits:

- 48 have Ljung-Box(6) p-values below 0.05.
- 45 have Breusch-Pagan p-values below 0.05.
- Durbin-Watson statistics range from 1.7342 to 2.4675.

HAC covariance with six lags is used for every alpha standard error, but the prevalence of residual flags is evidence that model adequacy remains limited. HAC does not repair an incomplete conditional-mean model or account for joint dependence across portfolios.

## Threshold Counts

Six value-weighted and six equal-weighted FF3 intercepts have absolute HAC t-statistics above 1.96. These are descriptive screens only: there is no multiplicity adjustment, family-wise error claim, or joint asset-pricing test.
