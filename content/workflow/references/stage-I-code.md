# Stage I — Research Code Support (I1–I9)

This stage makes the computational parts reproducible: implementation, pipelines, audits, and cross-model review.

## Canonical outputs (contract paths)

- `I1` → `analysis/` (method implementation)
- `I2` → `analysis/` (reproduction)
- `I3` → `analysis/` (data pipeline)
- `I4` → `code/reproducibility_audit.md`
- `I5` → `code/code_specification.md`
- `I6` → `code/plan.md`
- `I7` → `code/performance_profile.md`, `code/container_config/`, `code/documentation/`, `analysis/`
- `I8` → `code/code_review.md`
- `I9` → `release/`

## Quality gate focus

- `Q4` (reproducibility baseline) is the primary gate in this stage.
- Semantic gate report: update `quality-gate-report.md` with `q4_reproducibility_baseline`; evidence must anchor data lineage, code entrypoints, commands, environment, outputs, seeds, and rerun limits.

---

## Recommended workflow pattern (CCG-style)

For a formal implementation package, reuse this sequence and its required
artifacts. A bounded debugging or analysis question uses only the relevant task:

1. **Specification (`I5`)**: extract constraints (I/O schema, invariants, edge cases, metrics, tooling)
2. **Planning (`I6`)**: produce a zero-decision plan (tasks, dependencies, checkpoints)
3. **Execution (`I7`)**: implement + profile + document
4. **Review (`I8`)**: check logic, statistical validity and failure cases; use actual independent review when required
5. **Audit (`I4`)**: seeds, versions, determinism, data provenance, rerun instructions

## Academic Analysis Code

Stage I code is academic analysis code, not application architecture. Start from
the estimand, hypothesis, analysis plan, or manuscript-facing table/figure that
the code must support. Only add abstractions that make the research pipeline more
auditable.

Required analysis-code constraints:

- Preserve dataset lineage: raw input, cleaning rules, exclusions, missingness,
  joins, derived variables, and sample construction.
- Treat model diagnostics and robustness checks as first-class outputs, not
  optional plots added after the fact.
- Write manuscript-facing tables, figures, and machine-readable result files to
  predictable paths under `RESEARCH/[topic]/analysis/` or
  `RESEARCH/[topic]/manuscript/`.
- Record seeds, dependency notes, command logs, and rerun instructions.
- Separate finding, interpretation, and implication in analysis reports.
- Prefer scripts, notebooks, Quarto files, or small modules readable by
  researchers over service layers, controllers, framework scaffolding, or
  unnecessary classes.

When exploratory analysis is requested, label outputs as exploratory and record
assumptions. Do not let exploratory code silently become claim-supporting
evidence without a Stage I specification, plan, execution record, and review.

## I8 — Academic Code Review

**Definition of done**
- Review covers method fidelity, inferential validity, leakage risks, and reproducibility evidence
- Findings are severity-ranked and tied to concrete code or artifact evidence
- Blocking academic risks are separated from non-blocking cleanup
- Prefer a dedicated academic code reviewer role when available, rather than folding I8 into implementation ownership

## Data and execution checks

- Preserve immutable raw inputs. Validate schema, units, ranges, missing-value
  codes, identifier uniqueness and join cardinality before transformation. Report
  unmatched rows, observation loss and derived-variable timing; do not silently
  coerce ambiguous values or overwrite source data.
- Keep training/tuning/test partitions and group/time boundaries through cleaning,
  imputation and feature selection. A seed does not repair leakage or establish
  deterministic hardware/library behavior.
- Compare a small known calculation, synthetic invariant or trusted reference
  implementation where it can catch a consequential error. Check model convergence,
  dependence and uncertainty using the selected method's diagnostics.
- Bind results to inputs, code/config/environment, command and exit status.
  Distinguish code written, execution attempted, successful smoke check and empirical
  replication. Failed runs and unavailable real data remain explicit limits.
- Export tables/figures from recorded results rather than manually changing
  numbers. Carry the same sample, units and claim IDs into Stage F.

## I9 — Reproducible release package

Use `skills/I_code/release-packager.md` for the requested `release/` artifact.
Include authorized files, environment/entrypoint, data-access instructions and
the verified scope of reruns. Inspect for credentials, restricted data and private
identifiers before sharing. Local assembly does not authorize publication or
redistribution; do not claim a repository deposit without its actual result.

---

## What “done” looks like for code artifacts

### `analysis/`
- Contains runnable scripts/notebooks with clear entrypoints
- Includes a minimal dataset stub or synthetic data generator for verification
- Writes outputs to a predictable location (avoid hidden state)
- Records dataset lineage, model diagnostics, robustness checks, and
  manuscript-facing output paths

### `code/container_config/`
- Optional but recommended when dependencies are fragile
- Minimal `Dockerfile` or environment instructions

### `code/documentation/`
- `README.md` for how to run, reproduce, and interpret outputs

---

## Review ownership

Keep the user's configured Host and model. Use
`skills/Z_cross_cutting/model-collaborator.md` for requested/required independent
review when authorized tools are available. Assign roles by the bounded task,
not vendor stereotypes. Same-agent review is self-review; if independence is
required and unavailable, report that remaining gap instead of simulating it.
