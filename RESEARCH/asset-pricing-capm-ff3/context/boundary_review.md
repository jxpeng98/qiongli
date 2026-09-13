# Boundary Review

## Scholarly Decision Context

- task_id: A1
- stage: A
- paper_type: empirical
- target_venue: none
- current_research_question_or_claim: How much does FF3 attenuate CAPM pricing errors across the 25 U.S. size/book-to-market portfolios?
- trigger_level: high
- trigger_reason: The user requested a random topic; model-comparison language can easily become a causal or universal pricing claim.

## Artifact Evidence Checked First

- `context/idea_funnel.md`: IF-001 selected; IF-002 and IF-003 rejected.
- `context/research_state.md`: absent at first review.
- `context/decision_log.md`: absent at first review.
- `context/stage_handoff.md`: not required because the project stops before submission-facing work.
- stage_specific_artifacts: approved PLT-322 PRD and design.

## One-Question Academic Loop

| Question ID | Boundary Dimension | Question | Recommended Answer | User Or Artifact Answer | Status | Why This Matters |
|---|---|---|---|---|---|---|
| BQ-001 | Contribution and scope | Should the random topic be treated as a new asset-pricing discovery or as a reproducible comparison? | Use a reproducible descriptive comparison; require evidence before any result claim. | The approved task limits the project to CAPM versus FF3, real public data, and analysis-stage evidence. | locked | Prevents novelty and causality overclaims. |
| BQ-002 | Evidence threshold | What minimum evidence permits the attenuation claim? | All 25 portfolios under both models, HAC inference, diagnostics, and an equal-weighted sensitivity. | Approved implementation plan. | locked | A favorable subset cannot stand in for the portfolio grid. |

## Academic Boundary Map

| Dimension | Included / Claimed | Excluded / Not Claimed | Evidence Basis |
|---|---|---|---|
| Phenomenon / population / context | Monthly U.S. 25 size/BM portfolios in the pinned official vintage | Individual securities, non-U.S. markets, live trading | Official data construction pages and archives |
| Construct definition | Pricing error is the time-series regression intercept for portfolio excess return | Structural mispricing or arbitrage profit | `analysis_plan.md` |
| Contribution type | Replication and transparent model comparison | New theory, new factor, causal explanation | `context/idea_funnel.md` |
| Method / design | OLS time-series regressions with six-lag HAC covariance | Fama-MacBeth, GRS joint test, portfolio optimization | `study_design.md` |
| Generalizability | The selected portfolios and data vintage | Universal superiority of FF3 | `design/validity-threat-matrix.md` |
| Venue / reviewer expectation | Reproducibility and calibrated interpretation | Submission-readiness claim | project scope |

## Claim Strength And Evidence Threshold

- claim_strength: descriptive, model-comparative.
- strongest_defensible_wording: “In this pinned sample, FF3 reduced the cross-portfolio mean absolute monthly intercept by X relative to CAPM.”
- evidence_required_to_support: deterministic model results, a cross-model comparison table, HAC uncertainty, residual and factor diagnostics, and equal-weighted sensitivity.
- evidence_that_would_weaken_or_falsify: non-positive attenuation, concentrated reversals, invalid inputs, or diagnostics that make the regression comparison unreliable.
- finding_interpretation_implication_boundary: findings report fitted quantities; interpretation may mention factor exposure; implications stop at reproducibility and benchmark choice.

## Rival Explanations And Counterevidence

| Rival / counterevidence | Why It Matters | How This Workflow Will Address It |
|---|---|---|
| Shared size/BM construction between factors and test assets | Better fit may be partly mechanical. | State as a residual limitation; do not claim an independent test of factor truth. |
| Historical-vintage revision | Results can move without code changes. | Pin both ZIP SHA-256 digests and fail closed on drift. |
| Serial correlation and heteroskedasticity | Conventional OLS standard errors can misstate uncertainty. | Use six-lag HAC covariance and residual diagnostics. |
| Weighting choice | Value-weighted results may not describe smaller constituents. | Repeat the full comparison on equal-weighted portfolio returns. |

## Validity Or Trustworthiness Risk

- internal_validity_or_identification: no causal identification is attempted.
- construct_validity_or_operationalization: intercept magnitude is a benchmark-relative pricing-error proxy, not observed mispricing.
- external_validity_or_transferability: limited to selected U.S. portfolios and the pinned date range.
- statistical_conclusion_or_inference: HAC lag choice is fixed but not uniquely correct; cross-portfolio dependence is not used for joint inference.
- credibility_dependability_confirmability_if_qualitative: not applicable.

## Generalizability Limit

- population_limit: 25 U.S. size/book-to-market portfolios.
- setting_limit: Kenneth French Data Library definitions and CRSP-based universe.
- time_period_limit: July 1963 through the latest common month in the pinned archives.
- data_or_measurement_limit: monthly portfolio-level returns; no constituent-level reconstruction.
- model_or_assumption_limit: linear time-series factor models with a fixed six-lag HAC covariance.

## Venue Or Reviewer Risk

- likely_reviewer_objection: the same characteristics help construct factors and test assets.
- desk_reject_risk: high for a novelty-led empirical finance journal without a new identification or theory contribution.
- claim_or_scope_adjustment: present as an empirical/reproducibility project and record the overlap as a limitation.

## Locked Decision

| Decision ID | Decision | Rationale | Confidence | Evidence Basis | Downstream Impact |
|---|---|---|---|---|---|
| BD-001 | Use IF-001 and reject post-2000 and momentum branches. | It is the smallest question answerable with a clean public pipeline. | high | `context/idea_funnel.md` | Fixes A/C/I scope. |
| BD-002 | Interpret alpha only as benchmark-relative pricing error. | The design has no causal or structural identification. | high | Sharpe (1964); Fama and French (1993); design review | Controls all result wording. |
| BD-003 | Pin official archives and stop on digest drift. | The Data Library can revise historical series. | high | official archive behavior | Controls analysis execution. |
| BD-004 | Treat value-weighted results as primary and equal-weighted as sensitivity. | The primary comparison matches common portfolio-return reporting while exposing weighting dependence. | medium | official archive supplies both sections | Controls output hierarchy. |

## Open Questions

| Question | Why It Remains Open | Next Task Or Artifact | Revisit Trigger |
|---|---|---|---|
| Does FF3 materially attenuate alpha in the pinned vintage? | It is an empirical result, not a planning assumption. | `analysis/results/model_summary.csv` | completed analysis |

## Revisit Trigger

- Upstream digest/schema change, material failed diagnostic, or a request to claim beyond the selected portfolio grid.

## Downstream Sync

- `context/research_state.md`: scope, current evidence, and risks.
- `context/decision_log.md`: BD-001 through BD-004 as stable decisions.
- `context/stage_handoff.md`: not required at this analysis-stage stopping point.
- `manuscript/claims_evidence_map.md`: not produced because no manuscript is in scope.
- `design/validity-threat-matrix.md`: construction overlap, dependence, revisions.
- `code/code_specification.md`: digest stop rule and deterministic output checks.
