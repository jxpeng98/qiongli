# Analysis Results

## Finding

The common sample contains 756 monthly observations from 196307 through 202606. Across the 25 value-weighted portfolios, mean absolute monthly alpha is 0.1979 percentage points under CAPM and 0.0871 under FF3. The corresponding mean attenuation is 55.98%. Median attenuation is 74.18%.

Under equal weighting, mean absolute-alpha attenuation is 54.57% and median attenuation is 77.73%. FF3 leaves 6 value-weighted and 6 equal-weighted portfolio intercepts with descriptive absolute HAC t-statistics above 1.96.

## Interpretation

Within this pinned sample, the result supports H1 and H3 only when their status is recorded as `supported` in `analysis_summary.json`. Lower alpha and higher fit are consistent with SMB and HML absorbing return variation omitted by a market-only benchmark. They do not establish that FF3 is a true structural model or that the factor construction is independent of the size/book-to-market test assets.

## Diagnostic And Rival Boundary

Portfolio-level estimates and paired changes are in `model_results.csv` and `model_comparison.csv`; factor design checks are in `factor_diagnostics.json`; residual dependence and heteroskedasticity screens are in `residual_diagnostics.csv`. The threshold counts are descriptive and are not a family-wise or joint pricing-model test.

## Limitations

- The factors and test assets share size/book-to-market construction.
- HAC with six lags addresses within-series covariance but is not a cross-portfolio joint test.
- Results apply to the selected U.S. portfolio grid, monthly frequency, time window, and exact archived vintage.
- Historical source revisions can change results; digest drift therefore fails closed.
