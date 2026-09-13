# Research State

## Paper Identity

- topic: CAPM versus Fama-French three-factor pricing errors in 25 size/book-to-market portfolios
- paper_type: empirical
- target_venue: none
- current_stage: I/F — analysis executed and bounded interpretation complete
- last_updated_by_task: I4/I7/F-results
- last_updated_at: 2026-08-24

## Current Research Question / Thesis

- main_question_or_thesis: How much does FF3 attenuate CAPM pricing errors across the 25 U.S. size/book-to-market portfolios?
- contribution_claim: A digest-pinned, diagnostics-aware reproduction of the model comparison on a current official data vintage.
- why_this_question_now: It provides a genuine research project for Qiongli workflow and Graph v1 migration acceptance without inventing private evidence.

## Scope Boundaries and Working Definitions

- included_scope: monthly official factors and 25 portfolio returns, CAPM/FF3, value/equal weighting, HAC inference.
- excluded_scope: causality, individual stocks, international assets, new factors, trading advice, joint model rejection, and manuscript submission.
- unit_of_analysis: portfolio-month for estimation; portfolio-model for comparison.
- contested_terms_and_working_definitions: “pricing error” means a time-series intercept conditional on the specified benchmark.

## Locked Decisions

| Decision area | Current position | Confidence | Source artifacts |
|---|---|---|---|
| Framing | IF-001; descriptive comparison | high | `context/idea_funnel.md`; `context/boundary_review.md` |
| Literature boundary | targeted primary-source reading; native-only search | high | `context/qiongli_search_plan.json`; `search_log.md` |
| Design / identification | observational benchmark comparison; no causal identification | high | `study_design.md`; `analysis_plan.md` |
| Ethics / governance | public aggregate data; raw archives not redistributed | high | `data_management_plan.md` |
| Synthesis / interpretation | FF3 materially attenuates aggregate alpha but residual flags and shared construction constrain model-adequacy claims | high | `analysis/results/`; `manuscript/results_interpretation.md` |
| Submission positioning | out of scope | high | approved task PRD |

## Current Evidence Position

- strongest_supported_claims: value-weighted mean absolute alpha attenuation is 55.98%; equal-weighted attenuation is 54.57%; all results are traceable to pinned inputs.
- claims_still_provisional: economic mechanism and any conclusion beyond the selected portfolio grid.
- contradictory_or_null_evidence: five portfolio-weighting cases have increased absolute alpha; 48 serial-dependence and 45 heteroskedasticity screens are flagged.
- what_would_change_the_current_position: archive digest drift, a failed deterministic check, or approved joint/subperiod evidence.

## Active Risks and Fragility Points

- conceptual_risks: shared sort construction can overstate independence of the comparison.
- methodological_risks: fixed HAC lag choice and unmodeled cross-portfolio dependence.
- evidentiary_risks: targeted rather than systematic literature coverage; upstream data revisions.
- writing_or_submission_risks: model-fit language could be mistaken for causal or universal validity.

## Next-Stage Priorities

1. Preserve the pinned analysis and claim boundaries.
2. Migrate this project under PLT-322 and verify Graph v1 projection and Desktop consumption.
3. Add new empirical scope only through a separately approved design change.

## Source Artifact Anchors

- task_ids: A1, A1_5, A2, B2, B6, C1, C3, C3_5, C4, I5, I6
- authoritative_artifacts: `context/boundary_review.md`, `analysis_plan.md`, `code/code_specification.md`
- state_changes_since_last_update: analysis, deterministic check, diagnostics, evidence ledger, bounded interpretation, and reproducibility audit completed.
