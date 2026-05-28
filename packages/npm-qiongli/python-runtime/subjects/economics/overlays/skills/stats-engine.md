## Quality Bar

- [ ] The estimand, identifying variation, and comparison group are explicit.
- [ ] Standard errors use the economics-appropriate clustering level; clustered standard errors are reported when treatment varies by group or panel unit.
- [ ] DID designs include pre-trend/event-study diagnostics and avoid naive TWFE when staggered adoption with heterogeneous effects is plausible.
- [ ] IV designs report first-stage strength, reduced form, exclusion-rationale, and weak-instrument robust checks where applicable.
- [ ] RD designs report bandwidth choice, manipulation checks, covariate balance, and bandwidth sensitivity.
- [ ] Regression tables separate baseline, fixed-effects, controls, and robustness specifications.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Naive TWFE under staggered adoption | Biased treatment effects under heterogeneity | Use Callaway-Sant'Anna, Sun-Abraham, imputation, or clearly justify TWFE |
| Wrong clustering level | Over-rejection and misleading precision | Cluster at the treatment assignment or shock level |
| Weak instruments | Biased and unstable IV estimates | Report first-stage diagnostics and weak-IV robust inference |
| Bad controls | Post-treatment adjustment bias | Use only pre-treatment controls for causal specifications |
| Selective robustness | Specification search hidden as validation | Predeclare primary specification and show bounded sensitivity |
