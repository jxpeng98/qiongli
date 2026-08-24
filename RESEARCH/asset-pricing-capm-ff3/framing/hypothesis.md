# Hypotheses (A1_5)

## Mapping To RQs

| RQ | Hypothesis IDs |
|---|---|
| Main RQ / SRQ1 | H1 |
| SRQ2 | H2 |
| SRQ3 | H3 |
| SRQ4 | diagnostic conditions, not a directional hypothesis |

## H1 — Primary Directional Comparison

- Statement: Across the 25 value-weighted portfolios, FF3 has a lower mean absolute monthly intercept than CAPM.
- Mechanism intuition: SMB and HML absorb common return variation associated with size and book-to-market exposure that a market-only benchmark omits.
- Boundary conditions: the pinned U.S. monthly sample, these test assets, and this factor construction.
- Operationalization: `mean(abs(alpha_ff3)) < mean(abs(alpha_capm))`.

## H2 — Residual Pricing Errors

- Statement: FF3 does not eliminate every portfolio-level pricing error; at least one portfolio retains an absolute HAC t-statistic above 1.96.
- Mechanism intuition: a three-factor linear model need not span every return pattern or sample-specific residual.
- Boundary conditions: this is a descriptive threshold count, not a family-wise error-controlled joint test.
- Operationalization: count of `abs(alpha_hac_t) > 1.96` in FF3 results.

## H3 — Weighting Sensitivity

- Statement: The sign of aggregate attenuation is the same for equal-weighted and value-weighted portfolios.
- Mechanism intuition: if size and value exposures drive the comparison broadly, the direction should not depend entirely on value weighting.
- Boundary conditions: magnitude may differ because equal weighting changes constituent influence.
- Operationalization: attenuation ratio is positive under both weighting schemes.

## Rival Explanations

1. Shared size/book-to-market construction between factors and test assets can mechanically favor FF3.
2. Historical sample composition and upstream revisions can change attenuation without changing the model.
3. Serial dependence, heteroskedasticity, or factor correlation can weaken portfolio-level inference.
4. The fixed HAC lag may not capture every dependence pattern.
