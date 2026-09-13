# Academic Idea Funnel

## Source Prompt And Existing Artifacts

- source_prompt: Build a random, genuine asset-pricing empirical project with Qiongli.
- paper_type: empirical
- target_venue: none; this is a reproducible research project, not a submission package.
- starting_stage_or_entrypoint: Academic Idea Funnel before A1.
- artifacts_checked: no prior research artifacts existed; the approved PLT-322 task fixed public data, repository ownership, and analysis-stage scope.

## Candidate Idea Triage

| Idea ID | One-Sentence Idea | Paper Type Fit | Candidate Gap | Contribution Type | Evidence Needed | Feasibility | Novelty Risk | Reviewer Risk | Triage Decision |
|---|---|---|---|---|---|---|---|---|---|
| IF-001 | Compare CAPM and Fama-French three-factor pricing errors for the 25 U.S. size/book-to-market portfolios. | Direct empirical benchmark comparison. | A current, reproducible descriptive comparison using the official public series is useful as a bounded teaching and workflow artifact. | Empirical replication and measurement. | Official factors and portfolio returns; time-series regressions; HAC inference. | High: two public monthly files. | High if presented as a novel factor result. | Circular test assets and factors; no causal identification. | keep |
| IF-002 | Re-estimate whether the U.S. value premium disappeared after 2000. | Empirical subsample analysis. | Regime dependence could be examined. | Empirical update. | Multiple breakpoint choices and strong data-snooping controls. | Medium. | High because the breakpoint is selected after observing history. | Post-hoc window selection. | reject |
| IF-003 | Test whether momentum subsumes size and value. | Empirical factor comparison. | Competing factor span. | Empirical benchmark expansion. | Momentum files, more models, multiple-testing policy. | Medium. | Medium. | Scope expansion beyond one clean comparison. | reject |

## Recommended Research Idea

- recommended_idea_id: IF-001
- recommended_idea: Quantify how much FF3 attenuates CAPM pricing errors across the 25 size/book-to-market portfolios.
- why_this_can_be_one_paper: One test-asset set, two nested benchmark models, one fixed monthly sample, and one equal-weighted sensitivity answer a single descriptive question.
- why_not_the_other_options: IF-002 embeds an arbitrary regime choice; IF-003 adds another factor family and multiple-testing burden.
- confidence: high for feasibility; medium for scholarly contribution.
- assumptions_to_verify_next: common monthly coverage, stable official file structure, factor multicollinearity, and residual dependence.

## Claim, Gap, And Contribution

- core_claim: On the fixed official sample, FF3 either does or does not reduce the absolute intercepts left by the CAPM for these 25 portfolios.
- research_question: How much does FF3 attenuate CAPM pricing errors for the 25 U.S. size/book-to-market portfolios?
- candidate_gap: A compact, fully pinned, current-sample reproduction that makes model comparison and diagnostics auditable.
- contribution_type: empirical replication and reproducibility artifact.
- primary_non_claim: No new factor, causal mechanism, investment strategy, or universal model-validity claim.
- key_constructs_or_phenomenon: monthly portfolio excess return, factor exposure, regression intercept, adjusted R-squared.
- population_context_time_or_corpus_boundary: U.S. portfolios in the Kenneth French Data Library, July 1963 through the latest common pinned month.

## Evidence Plan

| Evidence Need | Minimum Viable Source | What Would Support The Claim | What Would Weaken Or Falsify It | Next Task |
|---|---|---|---|---|
| Model definition | Sharpe (1964); Fama and French (1993); official factor description | Source-anchored CAPM and FF3 specifications | Incompatible factor definitions | A1/A1_5 |
| Test assets | Official 25-portfolio file and construction page | 25 complete monthly series | Missing or duplicate months; sentinel values | C1/C4 |
| Comparative result | 100 HAC regressions and deterministic summaries | Lower FF3 absolute alpha with higher fit | No attenuation or unstable diagnostics | I7/F4 |
| Inference quality | Newey and West (1987); residual diagnostics | HAC results plus disclosed residual dependence | Severe unexplained misspecification | C3_5/I7 |

## Weakest Assumption And Rival Risk

- weakest_assumption: Smaller in-sample intercepts for FF3 are interpreted only as attenuation on these test assets, not proof that FF3 is the true pricing model.
- rival_explanation: The portfolios and factors share size/book-to-market construction, so better fit may partly reflect mechanical alignment.
- contradictory_or_null_evidence_to_search: portfolios where FF3 absolute alpha increases, weak HML/SMB loadings, and equal-weighted reversals.
- validity_or_trustworthiness_risk: serial correlation, heteroskedasticity, factor collinearity, and upstream historical revisions.
- feasibility_risk: the official archive can change while retaining its URL.
- ethical_or_governance_risk: raw files remain governed by their publisher and are not redistributed.

## Reviewer And Venue Fit

- likely_reviewer_question: Is the result anything more than an in-sample comparison on portfolios related to factor construction?
- venue_fit_signal: Suitable as a transparent replication or teaching artifact, not a novelty-led journal submission.
- desk_reject_or_scope_risk: High if framed as discovering or validating a universal asset-pricing law.
- how_to_make_the_idea_more_defensible: Pin inputs, report both weighting schemes, disclose construction overlap, and keep claims descriptive.

## Next Stage Recommendation

- next_stage_recommendation: Lock the descriptive boundary, then run A1, A1_5, A2, targeted B2/B6, C1/C3/C3_5/C4, and I5-I7/I4.
- recommended_task_id: A1
- rationale: The data and comparison are feasible; the claim boundary must precede analysis.
- prerequisite_artifacts: `context/boundary_review.md`
- stop_or_continue: continue under the user's approved implementation plan.

## Boundary Review Handoff

- boundary_review_handoff: Preserve the fixed sample, descriptive claim strength, construction-overlap caveat, and digest-refresh stop rule.
- decisions_to_lock_in `context/boundary_review.md`:
  - research_question_or_claim: comparative attenuation of pricing errors.
  - contribution_type: reproducible empirical replication.
  - claim_strength: descriptive and model-comparative.
  - evidence_threshold: all 25 portfolios, both models, HAC inference, diagnostics, and equal-weighted sensitivity.
  - rival_explanations: shared sort construction, changing samples, and model misspecification.
  - generalizability_limit: selected U.S. portfolios and pinned historical vintage only.
  - venue_or_reviewer_risk: overclaiming novelty or model truth.
- unresolved_boundary_questions: none blocking implementation.
- revisit_trigger: digest change, file-schema change, material diagnostic failure, or a request for causal/general factor claims.
