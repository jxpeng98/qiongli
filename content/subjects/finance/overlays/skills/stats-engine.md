## Quality Bar

- For asset pricing claims, define the test assets, factor model, return frequency, sample window, and risk-adjusted benchmark.
- Event studies must define estimation windows, event windows, expected-return model, confounding-event filters, and CAR tests.
- Corporate finance panels must document fixed effects, clustering, timing, treatment definition, and identification threats.
- Return construction must address delisting returns, survivorship bias, stale prices, and look-ahead bias.
- Inference must match the data structure, including Newey-West, clustering, Fama-MacBeth, or portfolio-sort requirements.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Look-ahead bias | Creates false predictability or overstated alpha | Construct signals using only information available at portfolio formation |
| Survivorship bias | Overstates returns and weakens external validity | Include delisted, failed, or missing firms when the research question requires them |
| Weak benchmark model | Mislabels risk compensation as abnormal performance | Test alternative factor models and report risk-adjusted sensitivity |
| Overlapping returns | Understates standard errors | Use Newey-West or design-appropriate corrections |
| Event-date leakage | Confounds CAR interpretation | Audit announcement timing, anticipation, and nearby events |
