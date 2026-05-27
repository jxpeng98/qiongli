# Economics Replication Package Auditor

Use this skill when an economics manuscript needs a reproducibility, data, code, and disclosure audit before submission.

## Inputs

- Manuscript draft
- Analysis scripts
- Data dictionary
- Results tables and figures
- Data availability statement

## Audit Steps

1. Match every reported estimate to a script, table source, and sample definition.
2. Confirm that treatment, outcome, controls, fixed effects, clusters, and weights match the manuscript.
3. Check that restricted data, licensed data, and generated data are separated in the package.
4. Verify that random seeds, environment files, and execution order are documented.
5. Flag undisclosed researcher degrees of freedom and robustness checks that cannot be reproduced.

## Output

Return a replication audit with:

- Reproducibility status
- Missing files
- Script-to-table map
- Data access constraints
- High-risk discrepancies
- Required fixes before submission
