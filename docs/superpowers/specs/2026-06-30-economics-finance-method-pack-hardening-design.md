# Economics And Finance Method Pack Hardening Design

## Goal

Strengthen the economics and finance domain method packs so Qiongli agents can
select, justify, audit, and block domain-specific methods with clearer evidence
contracts. The updated packs should expose method-level canonical references,
gate relevance, diagnostic artifacts, and failure triggers without turning the
domain profiles into long literature-review chapters.

The immediate focus is the canonical source under
`content/skills/domain-profiles/`. Generated package mirrors and installed
payloads remain out of scope for direct editing.

## Current Context

Qiongli already has the right distribution and routing shape:

- `content/skills/domain-profiles/economics.yaml` and
  `content/skills/domain-profiles/finance.yaml` define subject method guidance.
- `packages/python-qiongli/src/qiongli/subject_materializer.py` materializes
  domain profiles into installed subject payloads.
- `packages/python-qiongli/src/qiongli/local_plugin_installer.py` calls subject
  materialization while building local plugin installs.
- `tests/test_distribution_payloads.py` verifies specialized subject payload
  availability for subjects such as finance and economics-accounting.
- `tests/test_universal_installer.py` verifies subject installation behavior and
  `SUBJECT_MANIFEST.json` output.

The remaining gap is method-pack depth. The current profiles are useful routing
guidance, but they do not yet provide a consistently audited method contract
that tells an agent which diagnostics are mandatory, which quality gates a
method affects, which literature anchors are canonical, and which conditions
should block strong claims.

## User-Facing Install Answer

Yes. When a user installs Qiongli through the full CLI path, optimized subject
content can be called by agents as long as the relevant subject payload is
materialized during install.

The important distinction is:

- `--profile full` controls the runtime surface: CLI, MCP, plugin integration,
  doctor checks, and local execution capabilities.
- `--subject` and coverage controls select which subject payload and domain
  profiles are materialized.

For economics or finance usage, the installer should copy the updated domain
profiles into the installed Qiongli workflow payload. Agents then see the
optimized discipline guidance through the Qiongli skill resources available in
the target client. The implementation should preserve and test this behavior
rather than assuming source-only profiles are visible everywhere.

## Non-Goals

- Do not implement the real local-agent smoke test in this change. Add it to the
  roadmap as a later installation/runtime confidence check.
- Do not edit generated payload mirrors, release ZIP contents, plugin cache
  copies, or installed artifacts directly.
- Do not import `docs/superpowers` files from `.worktrees/`.
- Do not create a full bibliography manager inside YAML profiles.
- Do not replace Q1-Q4 semantic gates. Domain packs should feed those gates with
  method-specific checks, evidence anchors, diagnostics, and blockers.
- Do not claim a method is valid solely because it appears in a canonical paper.
  Diagnostics, identification assumptions, and data conditions still decide
  whether a method is appropriate.

## Repository Documentation Policy

Track source development documentation under `docs/superpowers/` so design
history and implementation plans survive branch merges and releases.

Keep `.superpowers/` ignored because it is a local runtime and tool artifact
directory. Do not track worktree-local copies under `.worktrees/`.

## Recommended Method-Pack Schema

Keep the schema compact enough for agents to use at runtime. Add or normalize
the following fields for each high-value method entry.

```yaml
method_packs:
  - method_id: did_staggered_adoption
    name: "Difference-in-differences with staggered adoption"
    gate_relevance:
      - Q1
      - Q2
      - Q4
    canonical_references:
      - citation_key: "callaway_santanna_2021_did"
        role: "modern estimator and group-time treatment effects"
      - citation_key: "sun_abraham_2021_eventstudy"
        role: "heterogeneous treatment effect diagnostic risk"
    diagnostic_artifacts:
      - artifact: "RESEARCH/[topic]/analysis/parallel_trends_diagnostics.md"
        required_for: "causal DID claims"
      - artifact: "RESEARCH/[topic]/analysis/event_study_plot.md"
        required_for: "dynamic treatment-effect claims"
    failure_triggers:
      - "No credible comparison group or untreated/not-yet-treated contrast."
      - "Pre-treatment trends are materially divergent without a justified design response."
      - "Strong causal claim is made from a two-way fixed effects estimate under heterogeneous treatment effects without robustness evidence."
```

Field intent:

- `canonical_references`: short, method-level anchors that an agent can cite as
  why a method belongs in the pack. These are not a claim that the user's study
  satisfies the method.
- `gate_relevance`: which Q1-Q4 gates the method can affect.
- `diagnostic_artifacts`: concrete local artifacts that must exist before
  strong claims pass.
- `failure_triggers`: explicit conditions that should downgrade, fail, or block
  method claims.

Optional fields may be added if existing profiles already have compatible
patterns, but the first pass should avoid schema sprawl.

## Economics Coverage

Prioritize method packs that are common, high-risk, and easy for agents to
overclaim:

- Causal panel and policy evaluation: difference-in-differences, event-study
  DID, synthetic control, regression discontinuity, instrumental variables,
  fixed effects panel models, clustered inference, and multiple-testing risks.
- Structural and applied microeconometrics: discrete choice, demand estimation,
  selection models, matching and weighting, mediation/decomposition only when
  the assumptions are explicit.
- Macroeconomics and forecasting: VAR/SVAR, local projections, DSGE calibration
  or estimation, nowcasting, unit roots/cointegration, forecast evaluation, and
  real-time data revision risks.
- Accounting-adjacent empirical work: earnings management proxies, accruals,
  audit/financial reporting event studies, restatement or disclosure measures,
  and archival data construction.

Each economics method should distinguish descriptive, predictive, causal, and
structural claims because the failure modes differ.

## Finance Coverage

Prioritize method packs that are frequently used and frequently misapplied:

- Asset pricing and factor models: CAPM, Fama-French style factors,
  Fama-MacBeth regressions, portfolio sorts, multiple testing and data snooping,
  factor construction, and out-of-sample validation.
- Corporate finance and market microstructure: event studies, abnormal returns,
  announcement-window design, liquidity/spread measures, endogeneity and
  selection in corporate decisions.
- Risk and volatility: GARCH-family models, realized volatility, tail risk,
  value-at-risk, expected shortfall, stress testing, and backtesting.
- Portfolio and derivatives: mean-variance optimization, shrinkage covariance,
  robust allocation, Black-Scholes/Merton option pricing assumptions, hedging
  error diagnostics, and transaction cost sensitivity.

Each finance method should separate pricing, prediction, risk measurement,
causal inference, and normative allocation claims.

## Literature Evidence Protocol

Use classic and recent literature as method anchors, but keep each profile entry
operational. For every method family:

- Prefer primary papers, official working papers, publisher pages, NBER pages,
  arXiv, SSRN, or author-hosted PDFs over secondary summaries.
- Include classic references for baseline method identity.
- Include recent references when practice has changed, especially for DID,
  synthetic control, RD inference, weak instruments, machine learning in causal
  inference, asset-pricing factor proliferation, volatility and tail-risk
  backtesting, and portfolio robustness.
- Record only compact citation keys and method roles in YAML. Do not paste long
  summaries or unverifiable claims into the runtime profile.
- Use diagnostic obligations and failure triggers to translate literature into
  agent behavior.

Suggested seed families:

- Difference-in-differences: modern staggered-adoption estimators,
  heterogeneous treatment-effect warnings, pre-trend and sensitivity checks.
- Regression discontinuity: bandwidth selection, robust bias-corrected
  inference, manipulation tests, and design-validity diagnostics.
- Synthetic control: donor-pool construction, placebo/permutation inference,
  augmented or generalized extensions where appropriate.
- Instrumental variables: relevance, exclusion, monotonicity, weak-instrument
  diagnostics, over-identification, and local average treatment effect limits.
- Event studies: expected-return model choice, event-window contamination,
  cross-sectional dependence, abnormal-return aggregation, and multiple events.
- Asset pricing: factor model specification, test assets, factor construction,
  cross-sectional pricing tests, data snooping, and out-of-sample decay.
- Risk models: volatility clustering, model diagnostics, VaR/ES backtesting,
  tail dependence, and stress-scenario coverage.

## Audit Design

Extend `tooling/scripts/audit_domain_method_packs.py` rather than adding an
unrelated validator, unless the existing script cannot cleanly represent these
checks.

Required audit behavior:

- Validate only canonical domain profiles under `content/skills/domain-profiles/`.
- For `economics.yaml` and `finance.yaml`, require each method pack to include
  `canonical_references`, `gate_relevance`, `diagnostic_artifacts`, and
  `failure_triggers`.
- Validate `gate_relevance` values against known quality gates.
- Validate that diagnostic artifact paths are concrete project artifacts rather
  than vague instructions.
- Validate that failure triggers are non-empty, specific, and blocking-oriented.
- Validate that canonical references have stable keys and short roles.
- Keep error output actionable: profile, method id, field, and reason.

## Test Plan

Use test-first implementation:

1. Add failing tests in `tests/test_domain_method_packs.py` for missing
   canonical references, gate relevance, diagnostics, and failure triggers.
2. Add or update audit CLI tests so malformed profile snippets fail with clear
   errors.
3. Update economics and finance profiles until the audit passes.
4. If payload materialization behavior changes or field preservation is not
   already covered, add a focused test around installed subject payload output.
5. Run the domain method-pack tests, distribution payload tests, and universal
   installer subject tests.

Target commands:

```bash
uv run python -m unittest tests.test_domain_method_packs -v
uv run python -m unittest tests.test_distribution_payloads -v
uv run python -m unittest tests.test_universal_installer -v
```

## Acceptance Criteria

- `docs/superpowers/` is tracked while `.superpowers/` remains ignored.
- Economics and finance domain profiles expose method-level references, gate
  relevance, diagnostics, and failure triggers for the selected method packs.
- The audit rejects incomplete or vague economics/finance method entries.
- The optimized fields remain available in installed subject payloads for full
  CLI installations when the matching subjects are selected.
- Tests demonstrate both validation behavior and payload visibility where
  needed.
- The real local-agent smoke test is recorded as roadmap work, not implemented
  prematurely in this change.
