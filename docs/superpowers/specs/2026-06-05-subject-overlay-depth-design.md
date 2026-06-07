# Subject Overlay Depth Design

Date: 2026-06-05
Branch base: `dev`

## Purpose

The current Qiongli subject packages already select domain profiles, venue profiles, subject-specific skills, and skill overlays. The weak point is that most subject overlays are short descriptive notes. They name the discipline but do not consistently tell the runtime agent what to check, what evidence is required, when to block, or how to adapt a generic skill to a subject-specific scholarly standard.

This change deepens the existing overlay layer without changing the materializer architecture. The goal is to make active subjects behave like discipline-aware research assistants rather than generic workflow maps with a short subject label.

## Scope

Deepen overlays for the existing non-core subjects:

- `economics`
- `accounting`
- `business`
- `finance`
- `political-economy`
- `geoeconomics`
- `economics-accounting`

The primary edit surface is:

- `content/subjects/*/overlays/skills/*.md`

Tests and audits may be updated to lock in minimum overlay depth, but the subject catalog and materializer should remain structurally unchanged unless a failing test reveals a real gap.

## Out Of Scope

- No new subject-packaging schema fields.
- No new subject playbook loader.
- No duplicated copies of generic skills under subject directories.
- No materializer redesign.
- No attempt to create a full literature review for each subject.
- No hidden expansion of the skill registry beyond what the overlays need.

## Literature-Grounded Design Basis

The overlays should be informed by canonical, top-journal, or field-defining works. These sources are design anchors, not content that should be quoted or summarized at length inside every overlay.

### Economics

Representative anchors:

- Imbens and Angrist, "Identification and Estimation of Local Average Treatment Effects," `Econometrica`, 1994.
- Bertrand, Duflo, and Mullainathan, "How Much Should We Trust Differences-in-Differences Estimates?", `Quarterly Journal of Economics`, 2004.
- Callaway and Sant'Anna, "Difference-in-Differences with Multiple Time Periods," `Journal of Econometrics`, 2021.
- Sun and Abraham, "Estimating Dynamic Treatment Effects in Event Studies with Heterogeneous Treatment Effects," `Journal of Econometrics`, 2021.

Overlay implication: economics overlays must force agents to name the estimand, identifying variation, comparison group, treatment timing, identifying assumptions, standard-error plan, and method-specific diagnostics before causal interpretation.

### Accounting

Representative anchors:

- Ball and Brown, "An Empirical Evaluation of Accounting Income Numbers," `Journal of Accounting Research`, 1968.
- Dechow, Sloan, and Sweeney, "Detecting Earnings Management," `The Accounting Review`, 1995.
- Healy and Palepu, "Information Asymmetry, Corporate Disclosure, and the Capital Markets," `Journal of Accounting and Economics`, 2001.
- Kothari, Leone, and Wasley, "Performance Matched Discretionary Accrual Measures," `Journal of Accounting and Economics`, 2005.

Overlay implication: accounting overlays must distinguish theoretical construct, accounting institution, empirical proxy, source item, fiscal timing, sample filter, and construct-validity risk before interpreting coefficients or disclosure effects.

### Business And Management

Representative anchors:

- Eisenhardt, "Building Theories from Case Study Research," `Academy of Management Review`, 1989.
- Barney, "Firm Resources and Sustained Competitive Advantage," `Journal of Management`, 1991.
- Teece, Pisano, and Shuen, "Dynamic Capabilities and Strategic Management," `Strategic Management Journal`, 1997.
- Gioia, Corley, and Hamilton, "Seeking Qualitative Rigor in Inductive Research," `Organizational Research Methods`, 2013.

Overlay implication: business overlays must push beyond practitioner description. They should require a target theory conversation, construct definitions, level of analysis, empirical setting justification, method transparency, rival framing, and doctoral-level journal contribution.

### Finance

Representative anchors:

- Fama and MacBeth, "Risk, Return, and Equilibrium: Empirical Tests," `Journal of Political Economy`, 1973.
- Fama and French, "Common Risk Factors in the Returns on Stocks and Bonds," `Journal of Financial Economics`, 1993.
- Carhart, "On Persistence in Mutual Fund Performance," `Journal of Finance`, 1997.
- Kothari and Warner, "Econometrics of Event Studies," `Handbook of Empirical Corporate Finance`, 2007.

Overlay implication: finance overlays must require return construction, asset universe, benchmark model, factor exposure, event-date and event-window logic, delisting and survivorship handling, look-ahead leakage checks, and inference choices matched to asset-pricing or corporate-finance claims.

### Political Economy

Representative anchors:

- North and Weingast, "Constitutions and Commitment," `Journal of Economic History`, 1989.
- Acemoglu, Johnson, and Robinson, "The Colonial Origins of Comparative Development," `American Economic Review`, 2001.
- Rodrik, Subramanian, and Trebbi, "Institutions Rule," `Journal of Economic Growth`, 2004.
- Besley and Persson, "The Origins of State Capacity," `American Economic Review`, 2009.

Overlay implication: political-economy overlays must require actors, institutions, incentives, distributional conflict, state capacity or policy mechanism, economic outcome, rival political mechanisms, and claim calibration when assignment or timing is weak.

### Geoeconomics

Representative anchors:

- Hirschman, `National Power and the Structure of Foreign Trade`, 1945.
- Baldwin, `Economic Statecraft`, 1985.
- Drezner, `The Sanctions Paradox`, 1999.
- Farrell and Newman, "Weaponized Interdependence," `International Security`, 2019.

Overlay implication: geoeconomics overlays must require sender, target, instrument, intermediary, network or supply-chain position, exposure construction, response channel, coercion or resilience logic, timing, evasion/substitution, and separation of strategic evidence from policy rhetoric.

### Reference Links Used For This Design

- Economics: Imbens and Angrist 1994 at Stanford GSB; Bertrand, Duflo, and Mullainathan 2004 at Oxford Academic; Callaway and Sant'Anna 2021 via Journal of Econometrics/RePEc; Sun and Abraham 2021 via RePEc.
- Accounting: Ball and Brown 1968 via RePEc; Healy and Palepu 2001 at Harvard Business School; Kothari, Leone, and Wasley 2005 at Kellogg/University of Miami; Dechow, Sloan, and Sweeney 1995 via The Accounting Review citation records.
- Business: Eisenhardt 1989 at Academy of Management Review; Barney 1991 at Journal of Management; Teece, Pisano, and Shuen 1997 at Strategic Management Journal/Harvard Business School; Gioia, Corley, and Hamilton 2013 at Organizational Research Methods.
- Finance: Fama and MacBeth 1973 via Journal of Political Economy/RePEc; Fama and French 1993 at ScienceDirect; Carhart 1997 via Journal of Finance DOI records; Kothari and Warner 2007 via Handbook of Empirical Corporate Finance records.
- Political economy: Acemoglu, Johnson, and Robinson 2001 at the American Economic Association; Rodrik, Subramanian, and Trebbi 2004 at Harvard Kennedy School/RePEc; Besley and Persson 2009 at the American Economic Association; North and Weingast 1989 via Cambridge DOI records.
- Geoeconomics: Hirschman 1945 via Open Library; Baldwin 1985 via Princeton/Google Books catalog records; Drezner 1999 via Cambridge University Press; Farrell and Newman 2019 via Belfer Center/International Security DOI records.

## Overlay Structure

Each overlay should be expanded into a consistent instruction shape:

```markdown
## <Subject> Overlay

### Activation
When this overlay applies and what generic skill behavior it modifies.

### Required Context
Inputs the agent must look for before performing the generic skill in this subject.

### Subject-Specific Procedure
Concrete steps that adapt the generic skill to the subject.

### Reviewer-Risk Checks
Discipline-specific objections a strong reviewer would raise.

### Output Requirements
What the produced artifact must explicitly contain.

### Blocked Conditions
What missing evidence should stop or narrow the claim instead of being invented.
```

For `replace_sections` overlays such as `stats-engine`, keep the required section names (`Quality Bar`, `Common Pitfalls`) so the materializer can continue replacing exact sections. Within those headings, add equivalent depth through checklists and pitfall tables rather than changing the section contract.

## Subject-Specific Overlay Targets

### Economics

Expand the existing overlays for:

- `study-designer`
- `robustness-planner`
- `analysis-interpreter`
- `manuscript-architect`
- `stats-engine`

The expanded economics layer should make causal language conditional on design evidence. It should require event-study or pretrend diagnostics for DID, weak-instrument checks for IV, bandwidth/manipulation checks for RD, clustering at the assignment or shock level, and a clear distinction between estimand and estimating equation.

### Accounting

Expand the existing overlays for:

- `manuscript-architect`
- `stats-engine`
- `variable-constructor`

The expanded accounting layer should require construct-proxy mapping, fiscal timing, source database/item definitions, missingness and winsorization rules, sample filter accounting, disclosure/reporting institutional logic, and separation between measurement validity and causal identification.

### Business

Expand the existing overlays for:

- `study-designer`
- `stats-engine`
- `manuscript-architect`

The expanded business layer should require target literature stream, theory contribution, construct definitions, level-of-analysis fit, setting justification, qualitative transparency or quantitative construct validity, and a rival framing that could lead to desk rejection.

### Finance

Expand the existing overlays for:

- `study-designer`
- `stats-engine`
- `manuscript-architect`

The expanded finance layer should require claim classification across asset pricing, corporate finance, market microstructure, risk, event study, theory, or methods. It should force return construction, benchmark/factor model, event window, estimation window, delisting treatment, survivorship, look-ahead, stale-price, overlapping-observation, and standard-error checks.

### Political Economy

Expand the existing overlays for:

- `literature-mapper`
- `study-designer`
- `stats-engine`
- `manuscript-architect`

The expanded political-economy layer should require actor-institution-outcome maps, incentives, distributional conflict, rival political/economic mechanisms, policy timing, institutional assignment, and direct mechanism evidence before strong causal or policy interpretation.

### Geoeconomics

Expand the existing overlays for:

- `literature-mapper`
- `venue-analyzer`
- `study-designer`
- `manuscript-architect`

The expanded geoeconomics layer should require instrument-sender-target maps, strategic objective classification, network/supply-chain exposure, target response, timing, substitution/evasion, countermeasure risk, and policy-claim calibration.

### Economics-Accounting

Expand the existing overlays for:

- `manuscript-architect`
- `stats-engine`

The expanded composite layer should require both economics identification discipline and accounting measurement discipline. It should not blindly union the two subjects. It should instruct agents to resolve tension between causal estimands, disclosure institutions, archival proxies, fiscal timing, capital-market outcomes, and reporting-setting mechanisms.

## Testing And Audit Design

Add or update tests before implementation:

- A subject overlay depth test should scan every overlay file and require at least four instructional sections for append overlays: `Activation`, `Required Context`, `Subject-Specific Procedure`, `Reviewer-Risk Checks`, `Output Requirements`, or `Blocked Conditions`.
- `replace_sections` overlays should be exempt from the exact section-name requirement but must still include richer checklist/table content under the required replacement headings.
- Existing materializer tests should continue to pass without changes to the package layout.
- Subject eval cases should continue to find the expected subject terms in materialized outputs.

The expected red test is that current short overlays fail the new depth rule.

## Verification

Run focused verification:

```bash
python3 -m unittest tests.test_subject_specialization_audit -v
python3 -m unittest tests.test_subject_materializer -v
python3 -m unittest tests.test_subject_eval_cases -v
python3 scripts/audit_subject_specialization.py
python3 scripts/audit_subject_eval_cases.py
```

If the repository's standard validation is required before merge, run the broader release or validation script separately.

## Risks

- Overlays could become too long and reduce token efficiency. Mitigation: make each overlay procedural, not encyclopedic.
- Literature anchors could be overfit into citations rather than behavior. Mitigation: translate papers into audit checks and reviewer-risk conditions.
- Replacing section headings in `stats-engine` overlays could break materialization. Mitigation: preserve exact `Quality Bar` and `Common Pitfalls` headings.
- Subject overlap could blur `political-economy`, `geoeconomics`, and `economics-accounting`. Mitigation: give each subject a distinct mechanism map and blocked-claim rule.

## Acceptance Criteria

- All existing non-core subject overlays are meaningfully deeper than the current descriptive bullets.
- Append overlays use explicit instruction sections.
- Replacement overlays preserve materializer-compatible headings and add discipline-specific quality bars and pitfalls.
- Literature anchors are reflected in concrete checks, not pasted summaries.
- Subject materialization still works for `complete` and `focused` coverage.
- Focused subject audits and eval-case tests pass.
