# neweyWest1987hac

- evidence_limit: abstract_only
- source_anchor: primary-paper abstract/metadata
- inference_strength: direct_evidence within abstract limits
- project_role: covariance-estimation anchor

## Extracted Point

Newey and West describe a positive semi-definite covariance estimator consistent under heteroskedasticity and autocorrelation. The analysis uses the installed statsmodels HAC implementation with a prespecified maximum lag of six.

## Limitation For This Project

Six lags are a transparent design choice, not a uniquely optimal lag-selection result.
