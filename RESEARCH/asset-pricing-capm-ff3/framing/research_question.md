# Research Question (A1)

## Topic

Comparative time-series pricing errors from CAPM and FF3 for publicly available U.S. size/book-to-market portfolios.

## Main RQ

How much does the Fama-French three-factor model attenuate CAPM pricing errors across the 25 U.S. size/book-to-market portfolios from July 1963 through the latest common month in the pinned official data vintage?

## Sub-RQs

1. How do the cross-portfolio mean and median absolute monthly intercepts differ between CAPM and FF3 for value-weighted portfolios?
2. Which portfolio intercepts remain large relative to their six-lag HAC uncertainty after adding SMB and HML?
3. Does the direction and magnitude of attenuation persist for equal-weighted portfolio returns?
4. Are factor collinearity, residual autocorrelation, or heteroskedasticity material limitations for interpretation?

## Framing

- PEO: population = 25 U.S. size/BM portfolios; exposure = benchmark factor set; outcome = portfolio-level intercept magnitude and adjusted fit.
- This is a descriptive observational model comparison, not a causal design.

## Core Constructs & Definitions

| Construct | Working definition | Observable proxy |
|---|---|---|
| Portfolio excess return | Monthly portfolio return above the one-month Treasury bill rate | Portfolio return minus RF |
| CAPM pricing error | Intercept from excess return on Mkt-RF | Monthly alpha |
| FF3 pricing error | Intercept from excess return on Mkt-RF, SMB, and HML | Monthly alpha |
| Attenuation | Relative reduction in cross-portfolio absolute alpha | 1 − mean absolute FF3 alpha / mean absolute CAPM alpha |
| Model fit | In-sample explained variation adjusted for regressor count | Adjusted R-squared |

## Scope Boundaries

- Included: monthly U.S. portfolios, value-weighted primary analysis, equal-weighted sensitivity, HAC(6) inference.
- Excluded: causal interpretation, security selection, non-U.S. markets, additional factors, joint GRS testing, and investment recommendations.

## Evidence That Would Answer The RQ

- Primary outcomes: 100 portfolio-model estimates, four model summaries, 50 paired comparisons, and model diagnostics.
- Minimum viable dataset: complete common monthly coverage for four factors and 25 portfolios in both weighting sections.

## Risks & Feasibility

- Feasible: two small public ZIP archives and one deterministic Python entrypoint.
- Novel: low as a theory contribution; useful as an auditable current-vintage replication.
- Ethical: no participants or private data; respect publisher control of raw files.
- Relevant: tests the research workflow on real evidence and supplies a representative scholarly project.

## Keywords

- models: CAPM; Fama-French three-factor model; factor regressions.
- assets: size portfolios; book-to-market portfolios; test assets.
- outcomes: alpha; pricing error; adjusted R-squared; HAC covariance.
