# Variable Specification

| Role | Variable | Source | Unit / Coding | Transformation | Notes |
|---|---|---|---|---|---|
| outcome | portfolio_return | 25-portfolios monthly section | percent per month | divide by 100 | 25 named columns under each weighting. |
| outcome | excess_return | derived | decimal per month | portfolio_return/100 − RF/100 | Fitted dependent variable. |
| factor | mkt_rf | factor archive `Mkt-RF` | percent per month | divide by 100 | CAPM and FF3. |
| factor | smb | factor archive `SMB` | percent per month | divide by 100 | FF3 only. |
| factor | hml | factor archive `HML` | percent per month | divide by 100 | FF3 only. |
| risk-free | rf | factor archive `RF` | percent per month | divide by 100 | Subtracted once from portfolio returns. |
| estimate | alpha_monthly | fitted intercept | decimal per month | none | Benchmark-relative pricing-error proxy. |
| estimate | alpha_hac_se | robust covariance | decimal per month | HAC(6), correction enabled | Used for descriptive t-statistic. |
| estimate | adjusted_r_squared | fitted model | unit interval not guaranteed | statsmodels definition | Comparison accounts for factor count. |
| comparison | attenuation | derived across portfolios | ratio | 1 − mean absolute FF3 alpha / mean absolute CAPM alpha | Primary value-weighted target. |

Missing codes `-99.99` and `-999` are invalid in the analysis window; no imputation or winsorization is permitted.
