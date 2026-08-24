# Asset-pricing project research

Date: 2026-08-24

## Selected project

Working title: **CAPM versus Fama-French Three-Factor Pricing Errors in the 25
Size–Book-to-Market Portfolios**

Main question:

> How much does the Fama-French three-factor model attenuate the pricing errors
> produced by CAPM across the 25 U.S. size/book-to-market portfolios?

The claim strength is descriptive and model-comparative, not causal. The
project will report the observed results rather than presuppose that FF3 wins.

## Qiongli Idea Funnel decision

The implementation will record these stable candidates in
`context/idea_funnel.md`:

| Idea ID | Candidate | Decision | Reason |
|---|---|---|---|
| `IF-001` | Compare CAPM and FF3 pricing errors on the 25 size/BM portfolios | keep | One bounded question, public data, standard models, direct Graph-bearing evidence |
| `IF-002` | Test whether the value premium changed after 2000 | reject | Breakpoint choice adds avoidable researcher degrees of freedom |
| `IF-003` | Test momentum profits across size/momentum portfolios | reject | Adds another dataset and benchmark choice without improving PLT-322 coverage |

The user authorized Qiongli to choose a random asset-pricing direction. The
funnel makes that choice auditable; no random-data generator is involved.

## Data authority

Use only these official Kenneth French Data Library inputs:

- [Fama/French 3 Factors CSV archive](https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_CSV.zip)
- [25 Portfolios Formed on Size and Book-to-Market CSV archive](https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/25_Portfolios_5x5_CSV.zip)
- [Data Library index and update notes](https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/data_Library.html)
- [25-portfolio construction details](https://mba.tuck.dartmouth.edu/pages/faculty/Ken.french/Data_Library/tw_5_ports.html)

The detail page defines the portfolios as the 5x5 intersection of NYSE size
and book-to-market quintiles, populated with eligible NYSE, AMEX, and NASDAQ
stocks. The library also warns that historical returns can change when its
source database is revised. Therefore the project must record retrieval time,
archive/member names, byte sizes, sample coverage, and SHA-256 digests.

Raw archives are copyrighted and will not be committed. The analysis downloads
them into an ignored local directory and refuses an unexpected digest. The
committed project contains only source metadata, code, and bounded derived
results.

## Empirical design

- Frequency: monthly.
- Test assets: 25 value-weighted size/book-to-market portfolios.
- Primary sample: July 1963 through the latest common month in the two pinned
  inputs.
- Sensitivity: repeat the same specifications with the equal-weighted section
  already present in the portfolio archive.
- Dependent variable: portfolio return minus the one-month risk-free rate.
- CAPM: intercept plus `Mkt-RF`.
- FF3: intercept plus `Mkt-RF`, `SMB`, and `HML`.
- Inference: OLS with six-lag Newey-West/HAC standard errors.
- Primary comparison: cross-portfolio mean and median absolute monthly alpha,
  adjusted R-squared, and the portfolio-level alpha/HAC t-stat table.
- Diagnostics: factor correlations/condition number, residual autocorrelation,
  heteroskedasticity, missing sentinels, duplicate months, and common-sample
  coverage.
- Interpretation: report findings, interpretations, and implications
  separately; counts of nominally significant alphas are descriptive and carry
  a multiple-testing limitation.

No trading strategy, transaction-cost claim, causal identification, or new
factor-discovery claim is in scope.

## Minimum Qiongli project surface

The source project will use canonical paths that the accepted Graph v1
extractors already understand:

- `context/idea_funnel.md`
- `context/boundary_review.md`
- `context/research_state.md`
- `context/decision_log.md`
- `framing/research_question.md`
- `framing/hypothesis.md`
- `framing/contribution_statement.md`
- `literature/literature_map.md`
- `notes/`, `bibliography.bib`, `retrieval_manifest.csv`, and
  `extraction_table.md`
- `study_design.md`, `analysis_plan.md`, and the minimum `design/` companions
- `data_management_plan.md`
- `code/code_specification.md`, `code/plan.md`,
  `code/performance_profile.md`, and `code/reproducibility_audit.md`
- one runnable analysis entrypoint plus deterministic result tables under
  `analysis/`
- `evidence/claim-evidence-ledger.csv` and `evidence/evidence-ledger.md`
- `quality-gate-report.md`

The project will not hand-author `graph/semantic_links.jsonl`; reviewed
relations must come from canonical artifacts.

## Dependency choice

Use one PEP 723 Python script with a script lock. Parse ZIP/CSV and write JSON,
CSV, and Markdown with the standard library. Use NumPy and statsmodels only for
the matrix/statistical work, because the repository does not already provide a
research-statistics dependency and reimplementing HAC inference would be less
auditable.

Do not add these packages to Qiongli's root runtime dependencies. They belong
only to the research project and remain reproducible through the script lock.

## Graph acceptance implication

The project will provide reviewed, source-bound Graph semantics through:

- idea-to-candidate-gap relations from the Idea Funnel;
- paper-to-literature-cluster and reviewed cluster relations from the
  literature map;
- evidence-to-claim support relations from the post-analysis evidence ledger;
- research-question, contribution, and decision nodes from the context files.

This is enough to test Graph v1 without a generated sidecar or prose inference.
