# Literature Map

## Clustering Basis

- primary_basis: model role and inferential role.
- secondary_basis: empirical test-asset context.
- scope_limit: targeted anchors for this analysis; no exhaustive field map.

## Included Studies

| Citekey | Primary Cluster ID | Secondary Cluster IDs | Evidence Limit | Source Anchor |
|---|---|---|---|---|
| sharpe1964capm | LC-001 |  | Full text | notes/sharpe1964capm.md#extracted-point |
| famaFrench1992crosssection | LC-002 | LC-001 | Abstract only | notes/famaFrench1992crosssection.md#extracted-point |
| famaFrench1993commonrisk | LC-002 | LC-001 | Abstract only | notes/famaFrench1993commonrisk.md#extracted-point |
| famaFrench1996multifactor | LC-002 | LC-001 | Abstract only | notes/famaFrench1996multifactor.md#extracted-point |
| neweyWest1987hac | LC-003 |  | Abstract only | notes/neweyWest1987hac.md#extracted-point |

## Concept Streams

| Cluster ID | Cluster Label | Basis | Core Argument | Representative Papers | Evidence Limits |
|---|---|---|---|---|---|
| LC-001 | Market-only equilibrium benchmark | theory | Expected return is tied to market-related nondiversifiable risk under equilibrium assumptions. | sharpe1964capm | Full text for Sharpe; later comparisons are abstract-limited. |
| LC-002 | Size and value multifactor benchmark | model and evidence | Size and book-to-market organize return variation that a market-only benchmark can leave unexplained. | famaFrench1992crosssection; famaFrench1993commonrisk; famaFrench1996multifactor | Abstract-only extraction; no field-consensus claim. |
| LC-003 | Dependence-robust regression inference | method | Portfolio time-series inference should allow heteroskedasticity and autocorrelation in regression disturbances. | neweyWest1987hac | Abstract-level method anchor; lag choice remains project-specific. |

## Evidence Gaps

| Gap ID | Open Problem | Cluster IDs | Source Anchors | Project Relevance | Status |
|---|---|---|---|---|---|
| GAP-001 | Current-vintage attenuation magnitudes must be computed rather than inferred from historical summaries. | LC-001; LC-002; LC-003 | notes/famaFrench1996multifactor.md#limitation-for-this-project | Directly motivates the pinned analysis and its inference contract. | open |
| GAP-002 | Shared size/BM construction limits independence between factors and test assets. | LC-001; LC-002 | notes/kennethFrenchDataLibrary.md#extracted-points | Restricts interpretation of the CAPM-to-FF3 fit gain. | open |
| GAP-003 | The fixed HAC lag is transparent but not uniquely selected by the methodological anchor. | LC-001; LC-002; LC-003 | notes/neweyWest1987hac.md#limitation-for-this-project | Applies to inference for both benchmark models. | open |
| GAP-004 | Widespread residual diagnostic flags leave conditional-mean adequacy unresolved. | LC-001; LC-002; LC-003 | analysis/factor_inference_diagnostics.md#residual-screens | Separates covariance correction from model adequacy. | open |
| GAP-005 | Portfolio-by-portfolio screens do not provide a cross-portfolio joint pricing-model test. | LC-001; LC-002; LC-003 | analysis/factor_inference_diagnostics.md#threshold-counts | Bounds the inferential claim for both models. | open |
| GAP-006 | A small set of portfolios has larger absolute alpha under FF3 despite aggregate attenuation. | LC-001; LC-002; LC-003 | analysis/factor_model_sensitivity.md#portfolio-exceptions | Prevents a universal model-dominance claim. | open |
| GAP-007 | Stability across subperiods and later publisher vintages is not established by one pinned full-sample run. | LC-001; LC-002; LC-003 | analysis/factor_model_sensitivity.md#interpretation-boundary | Limits temporal generalization and future refresh claims. | open |

## Inter-Cluster Relationships

| Source Cluster ID | Relation | Target Cluster ID | Source Anchor | Evidence Limit | Status |
|---|---|---|---|---|---|
| LC-002 | competing | LC-001 | notes/famaFrench1996multifactor.md#extracted-point | Abstract-only benchmark comparison | reviewed |
| LC-002 | nested | LC-001 | analysis_plan.md#models | FF3 retains the market factor and adds SMB and HML in this project | reviewed |
| LC-003 | complementary | LC-002 | analysis_plan.md#models | Methodological support for planned inference, not model truth | reviewed |
| LC-003 | complementary | LC-001 | analysis_plan.md#models | The same dependence-robust covariance rule is used for CAPM inference | reviewed |

## Project Positioning

- addressed_cluster_ids: LC-001; LC-002; LC-003
- addressed_gap_ids: GAP-001; GAP-004; GAP-006
- contribution_boundary: compute and audit current-vintage comparative quantities; do not claim to resolve the economic origin of size or value effects.
- unsupported_novelty_claims: universal FF3 superiority; a new anomaly; causal explanation of value or size premia.
