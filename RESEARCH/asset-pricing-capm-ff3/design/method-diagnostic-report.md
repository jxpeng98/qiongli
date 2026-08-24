# Method Diagnostic Report

## Design Summary

- Paper type: empirical.
- Design: observational secondary-data time-series benchmark comparison.
- Data: official monthly U.S. factors and 25 size/book-to-market portfolios.
- Target population: the published portfolio grid, not individual firms or investors.
- Interpretation logic: compare benchmark-relative intercept magnitudes and fitted variation; do not infer causality or universal model truth.

## Validity Threat Matrix

See `design/validity-threat-matrix.md`.

## Method-Specific Checks

- Factor correlation and multicollinearity checks.
- Alpha, factor loadings, HAC t-statistics, and adjusted R-squared.
- Residual autocorrelation and heteroskedasticity diagnostics.
- Full value-weighted and equal-weighted model grids.
- Source digest, schema, duplicate, missingness, and common-sample checks.

## Observed Diagnostic Result

- FF3 condition number is 36.2578; pairwise factor correlations have absolute values below 0.30.
- 48 of 100 residual series have Ljung-Box(6) p-values below 0.05.
- 45 of 100 residual series have Breusch-Pagan p-values below 0.05.
- Durbin-Watson ranges from 1.7342 to 2.4675.
- All 100 models use HAC covariance with six lags; threshold counts remain descriptive rather than joint inference.

Verdict: the model grid is estimable and the factor design does not trigger an extreme-collinearity failure, but the residual-screen prevalence requires qualified model-adequacy language.

## Failure Triggers

- Digest or expected ZIP-member mismatch.
- Any duplicate/malformed month, missing sentinel, nonfinite value, missing portfolio, or inconsistent sample.
- Fewer than 25 portfolios or 100 total model estimates.
- Non-positive primary attenuation does not fail computation, but it falsifies H1 and must be reported truthfully.
- Extreme collinearity or widespread residual failures require interpretation to be downgraded, not suppressed.

## Insufficient Input Notes

- No constituent-level holdings are available, so portfolio construction is accepted from official documentation rather than independently reconstructed.
- No cross-portfolio joint asset-pricing test is planned; portfolio-level t-statistics are descriptive.
- No venue-specific reporting checklist applies because submission is outside scope.
