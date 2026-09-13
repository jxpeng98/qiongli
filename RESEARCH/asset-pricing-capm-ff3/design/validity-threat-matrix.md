# Validity Threat Matrix

| Threat Category | Specific Threat | Risk Level | Evidence | Mitigation | Residual Limitation |
|---|---|---|---|---|---|
| construct validity | Regression alpha may not equal economic mispricing. | high | Model definition in `analysis_plan.md` | Use “benchmark-relative pricing error” consistently. | Economic interpretation remains model-dependent. |
| internal validity | No causal identification of why size/value factors improve fit. | high | Observational design | Make no causal claim. | Mechanisms remain unresolved. |
| external validity | 25 U.S. portfolios may not represent other assets or markets. | high | Official portfolio scope | State population and time limits. | No international or security-level generalization. |
| statistical conclusion validity | Serial/cross-portfolio dependence and multiple portfolio screens. | medium | Monthly regressions and 25 tests | HAC(6); label threshold counts descriptive. | No joint model-rejection inference. |
| measurement validity | Upstream definitions and historical series can change. | medium | Official documentation and archives | Pin SHA-256 and fail on drift. | A future reviewed vintage can differ. |
| data leakage | Result-selected windows or factors could favor a model. | low | Locked 1963-07 start and two models | No post-hoc model/window selection. | Historical model development is inherently in-sample to the literature. |
| missingness | Source sentinel or unmatched date could change samples. | low | File documents `-99.99` and `-999` sentinels | Fail rather than impute; require common sample. | None for accepted run; upstream gaps would block. |
| confounding | Shared size/BM construction aligns factors and test assets. | high | Official factor/portfolio definitions | Disclose overlap and narrow interpretation. | The design is not an independent out-of-sample model test. |
| selection bias | Canonical test assets were selected rather than a random asset universe. | high | Approved project scope | Treat portfolio grid as the target population. | Results do not estimate universe-wide performance. |
