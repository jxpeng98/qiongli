# Economics, finance and accounting

Use for economic mechanisms, causal policy evaluation, markets, asset pricing,
corporate finance, reporting, auditing and tax research. Load the matching
economics, finance or accounting profile only for the selected method.

## Frame and assemble the evidence — A/B

- Separate descriptive patterns, causal effects, forecasting, structural estimates
  and theoretical propositions. A replication, measurement correction or new
  dataset can contribute without claiming a new causal mechanism.
- Define the institutional setting, policy announcement/effective dates, agents,
  market and observation unit. Fiscal years, trading days and reporting dates
  are different clocks; record which clock supports each variable.
- Search with mechanism and design terms as well as the topic. Follow working
  papers to their published versions; compare estimates only after checking
  samples, variable definitions and revisions. Record source vintage and citekey.
- For common datasets, assemble a data feasibility note: access/license, identifiers,
  coverage dates, update vintage, merge keys and expected selection losses. A
  database name does not establish access or permission to redistribute it.

## Common methods and decisive checks — C/I

| Work | Settle before estimation | Evidence that changes the interpretation |
|---|---|---|
| DID / policy event study | Treatment timing, comparison cohort, anticipation, target effect, spillovers | Pre-period patterns with uncertainty; cohort support; estimator appropriate to staggered timing and heterogeneous effects. A nonsignificant pretrend test does not establish parallel trends |
| RD | Assignment/running variable, cutoff, sharp/fuzzy design, local population | Sorting and institutional manipulation risks, continuity, bandwidth sensitivity and uncertainty; keep local claims local |
| IV | Instrument mechanism, relevance, exclusion, monotonicity where used | First stage and weak-instrument-robust inference; a first-stage threshold or overidentification test cannot prove exclusion |
| Asset pricing / portfolio backtest | Information available at formation, rebalance rule, investable universe, benchmark | Delistings, survivorship, stale prices, transaction costs, overlapping returns and dependence-aware uncertainty; retain out-of-sample evaluation |
| Financial event study | Event timestamp, estimation/event windows, expected-return model | Confounding events, overlapping windows, dependence across firms/events and specification sensitivity; distinguish announcement response from long-run causal effect |
| Archival accounting | Construct → database item → timing → transformation | Alternative proxies, sample waterfall, reporting-regime comparability, firm/security links and restatements; accrual proxies alone do not establish intent or misconduct |
| Structural / theoretical model | Assumptions, equilibrium/solution concept, identified versus calibrated parameters | Identification argument, fit to untargeted moments where relevant, numerical convergence and sensitivity; model-based counterfactuals retain assumptions |

## Common content to organize

In `design/variable_spec.md`, keep item codes, units/currency, deflators, lags,
release dates, scaling and treatment of missing values. For text-based disclosure
measures, retain document/version IDs, preprocessing, dictionary/model version and
construct validation. Text sentiment is not automatically disclosure quality.

In `analysis_plan.md`, connect each question to its estimand, sample and inference
unit. Choose fixed effects, controls and clustering from the assignment/sampling
process and dependence; do not add post-treatment controls merely to stabilize a
coefficient. Choose robustness checks by threat, without a universal count.

## Interpret, write and deliver — E/F/G/H

- Keep coefficient units, uncertainty, denominator and plausible economic magnitude
  together. A significant alpha is conditional on the benchmark and implementation;
  an accounting association is not evidence of a reporting-channel mechanism.
- Link each table to sample construction and an executed command. Explain why
  sample sizes differ across specifications. Report preregistered and exploratory
  estimates honestly, including null or contradictory results.
- For proprietary data, provide permitted code, data dictionaries and access
  instructions; a synthetic smoke run tests execution, not empirical replication.
- Check the chosen venue's current data/code policy and exemption procedure.
  A local package does not establish repository deposit or editorial approval.

The [AEA data and code policy](https://www.aeaweb.org/journals/data/data-code-policy)
is a source for AEA submissions, not a universal rule for all economics journals.
Use `references/method-diagnostic-contract.md` and the selected economics/finance
profile for their existing method-specific diagnostic artifacts and gates.
