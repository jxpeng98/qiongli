# Stage F — Manuscript Writing (F1–F6)

This stage turns artifacts into a publishable narrative: outline → draft → claims/evidence → figures/tables → abstract/title.

## Canonical outputs (contract paths)

- `F1` → `manuscript/outline.md`
- `F2` → `manuscript/manuscript.md` (section-level drafting)
- `F3` → `manuscript/manuscript.md`, `manuscript/results_interpretation.md`, `manuscript/effect_interpretation.md`
- `F4` → `manuscript/claims_evidence_map.md`
- `F5` → `manuscript/figures_tables_plan.md`, `manuscript/tables/`, `manuscript/figures/`
- `F6` → `manuscript/meta_optimization.md`
- Proposal / opening report → `proposal/research_proposal.md` (skill: `proposal-writer`; no separate task ID)

## Quality gate focus

- `Q1` (question-to-method alignment): intro/method/results must answer the same RQ(s).
- `Q2` (claim-evidence traceability): enforced via `F4` and `G3`.
- Semantic gate report: update `quality-gate-report.md` with `q2_claim_evidence_traceability`; evidence must anchor central claims to the claim-evidence ledger, source notes, analysis outputs, citations, or explicit gap notes.

## Writing Harness Contract

This shared contract applies to Stage F workflows, skill cards and role prompts.
It specifies the result and boundaries; the active model chooses how to reach it.
Formal task outputs and Q1/Q2 gates above remain required for the selected task.
A direct paragraph answer does not require the full artifact set or a gate report.

### Result-to-claim decisions

Apply the three questions in `references/academic-output-rubric.md` to the claims
that determine the requested text. Inspect the current output/source and inherited
limits before reusing a draft or interpretation.

| What must be decided? | Evidence needed | What can proceed? |
|---|---|---|
| Is this a result we can report? | Executed output or inspected source excerpt, version, sample/denominator, units and uncertainty or qualitative context | Draft verified findings. An unexecuted analysis remains planned; inconsistent numbers block that numerical claim until reconciled, while unaffected text can proceed |
| Which wording does the evidence support? | C's claim type and assumptions, E/I's findings, source anchors and failed checks | State the strongest supported claim. Remove or narrow an unsupported causal/mechanistic assertion; adding "may" alone does not supply evidence |
| Can we interpret magnitude or practical importance? | Metric, scaling, time horizon, population and a justified benchmark or conversion inputs | Explain the available estimate and uncertainty. Missing context limits the practical translation; do not manufacture a benchmark or substitute an example number |
| How do contrary findings affect the argument? | Comparable results, negative cases, prespecified/exploratory status and source-specific limitations | Integrate supported convergence/divergence. An unexplained contradiction stays visible; choose narrower prose rather than inventing a mechanism |
| Is the requested text complete? | Its source-to-claim checks, applicable formal outputs and unresolved dependencies | Return the checked unit, or label the specific incomplete part. Separate an evidence-bounded draft from a gate pass or submission-readiness claim |

Carry the claim ID, source locator, supported wording and limiting assumption in
the existing interpretation artifact and claim-evidence records when those are in
scope. On a changed upstream result, identify dependent manuscript passages,
tables, abstracts and recommendations; revise them together within authorized
scope or name the remaining affected work in the handoff. Do not silently reuse
the old estimate or promote a previous review into evidence for the new claim.

### Direction and freedom

- Reuse relevant source material and settled decisions, including
  `context/boundary_review.md`, `context/decision_log.md`,
  `context/stage_handoff.md` and `review/self_critique_log.md` when present.
- Keep a coherent **Story Spine**: what the text argues, the section's job,
  evidence supporting its central claim and where that claim stops. Reuse an
  existing outline; a short edit needs no separate Story Spine presentation.
- Choose structure, drafting order, tools and chunk size from the requested
  length, evidence dependencies and risk. A bounded section may be drafted and
  checked as a whole. Split longer work when checkpoints help catch a specific
  risk, a dependency remains unsettled or the user requests staged delivery.
- Follow the user, venue or agreed protocol when it specifies a structure or
  review process. Do not infer extra approval steps from a template or role name.

### Evidence and quality

- Check for mainline drift, unsupported claims, logic jumps, contradiction with
  settled decisions, and generic or vague claims. Preserve source meaning,
  numbers, citations, uncertainty and the distinction between findings,
  interpretation and implication. Never infer measurement from a study label.
- Match analytical depth to the section's job and evidence. Discussion should
  explain what the findings mean; methods and results may be descriptive.
  Mechanisms, tensions, alternatives and implications are useful lenses, not a
  per-paragraph quota. Narrow claims when evidence cannot support more depth.
- Interpret qualitative themes when supported, keeping quotations and episodes
  as evidence anchors. Do not invent mechanisms, negative cases or theoretical
  contributions to satisfy a writing template.
- Preserve the design's claim type from Stage A/C and the selected discipline
  guide. A predictor is not necessarily a cause, an indirect path is not a
  demonstrated mechanism, and a source interpretation is not a measured effect.
- Check numerical statements against the actual result: population, denominator,
  units, direction, time window and uncertainty. Distinguish statistical from
  substantive importance. A nonsignificant result does not establish equivalence;
  equivalence/noninferiority claims need their own design, margin and analysis.
- Preserve the source's level of methodological detail. A named analysis or
  transformation does not establish its unreported grouping, parameters or
  implementation. Check those qualifiers against the source before compressing
  prose; omit unsupported detail or label a necessary inference explicitly,
  rather than presenting a customary procedure as a reported fact.
- Separate planned from performed methods and confirmatory from exploratory
  analyses. Account for deviations, missing observations and null/contradictory
  findings before compressing results into an abstract or recommendation.
- For theory/humanities, make premises, source interpretation, objections and
  scope inspectable without forcing empirical section names. For mixed methods,
  explain what integration adds and where the strands disagree.

### Review and completion

Check the delivered text against the requested length and format, as well as its
claims. Use the user's counting rule. If none is specified, count words for an
English word limit; for a Chinese character limit, count non-whitespace Unicode
characters in the prose, including punctuation and digits, excluding standalone
source-anchor citations. Keep that convention consistent during drafting and
review. Citation strings, headings and editorial notes must not pad a short
paragraph. Check the actual final text rather than estimating from its appearance;
after editing, recount the changed unit. Return only the paragraph when that is
the requested format; an internal length check needs no extra checklist or count.
If evidence cannot support the requested length, identify the gap rather than
adding unsupported claims or silently returning an undersized completed draft.

Check the requested unit before returning it. Revise concrete defects and verify
what changed; further passes need new evidence, a remaining defect or an explicit
review requirement. Honor configured minimum passes and required independent
review, including settings carried by an existing run. A clean check is allowed
and does not require invented revisions. Missing evidence or an exhausted budget
does not establish readiness.

Continue within the agreed scope when evidence settles the decision. Ask only
for a consequential unresolved choice, scope change or required write approval;
continue independent supported work while that branch is blocked. Registered
project writes still use preview/approval/CAS and preserve user material.

Finish when the requested deliverable meets its applicable checks, or report the
specific unmet requirement. Record claim-support evidence and unresolved issues
at the artifact or claim level needed for traceability; routine paragraphs need
no checkpoint transcript. Preserve stable claim IDs and review issue lineage.

---

## Proposal / Opening Report — Research Proposal Writing

Use `proposal-writer` when the requested output is a research proposal, prospectus, opening report, 开题报告, or study plan. This is a pre-results approval artifact: it synthesizes the RQ, literature gap, theory, contribution, study design, analysis plan, ethics/data management, feasibility, timeline, and risks into `proposal/research_proposal.md`.

Definition of done:
- The proposal states what will be studied, why it matters, how it will be executed, and what would count as success.
- Missing citations, data access, ethics status, sample-size rationale, or institutional requirements are visible as gap notes.
- Planned findings, interpretations, and implications are separated from completed results.
- The proposal is not treated as a preregistration, systematic review protocol, or manuscript draft.

Write into: `proposal/research_proposal.md`.

---

## F1 — Manuscript Outline

**Definition of done**
- Section headings match venue norms
- Each section has bullet “promises” (what it will deliver)
- Results section mirrors analysis plan outputs or the planned qualitative findings architecture

Write into: `manuscript/outline.md`.

---

## F2 — Single Section / Component Drafting

Use when you want to draft one component precisely (e.g., “intro gap paragraph”).

**Definition of done**
- The component has a clear rhetorical role (setup / gap / contribution / method / evidence / limitation)
- The component fulfills its section purpose at the depth supported by the evidence
- Citations are present where claims of prior work are made
- No new claims that contradict earlier artifacts

Write into: `manuscript/manuscript.md` (or a section placeholder within it).

---

## F3 — Full Draft

**Definition of done (minimum)**
- All sections required by the paper type and venue exist; do not force an empirical structure onto other paper types
- Methods contain enough detail for replication or audit (given the artifact set), including sampling, access, data sources, analytic procedure, and reflexivity for qualitative work
- Results are consistent with analysis plan and reported with uncertainty or transparent evidence structure
- Findings in qualitative papers are analytic claims; quotes, vignettes, and episodes are evidence anchors rather than the finding itself
- Related work and discussion interpret tensions, mechanisms, and alternative explanations instead of paraphrasing sources or results
- Limitations discuss validity threats (not only “small sample”)
- The narrative states where claims stop: boundary conditions, contradictory cases, and inferential limits are explicit
- Discussion distinguishes participant attributions, author interpretation, and speculative implication
- Result narration is externalized enough to support reuse in `manuscript/results_interpretation.md`
- Effect magnitude is translated into practical terms in `manuscript/effect_interpretation.md` when applicable

Write into:
- `manuscript/manuscript.md`
- `manuscript/results_interpretation.md`
- `manuscript/effect_interpretation.md`

---

## F4 — Claim–Evidence Map

This is the anti-overclaim tool: every major claim must trace to evidence (data, analysis, or citations).

**Definition of done**
- All major claims in abstract/introduction/discussion appear in the map
- Each claim has at least one evidence pointer
- Claims are typed (novelty / mechanism / empirical effect / robustness / synthesis)

Use `templates/claim-evidence-map.md` for `manuscript/claims_evidence_map.md`.
Preserve its exact headers, stable claim IDs and distinction between evidence
pointers and citation keys. Do not substitute a differently shaped summary table.

---

## F5 — Figures & Tables Plan

Plan visuals early so the narrative has “anchors”.

**Definition of done**
- List of planned figures/tables with:
  - purpose (what question it answers)
  - data source
  - caption claim (what it will show)
- Mapping from results sections → figures/tables

Write into: `manuscript/figures_tables_plan.md`.

Recommended pairing:
- `table-generator` for `manuscript/tables/`
- `figure-specifier` for `manuscript/figures/`

For qualitative papers, common `F5` outputs include:
- data structure diagrams
- process models
- case comparison matrices
- evidence tables linking cases/quotes to claims

---

## F6 — Abstract & Title Optimization (Indexing / SEO / Reviewer Scanning)

**Definition of done**
- Title reflects: construct + setting + method + contribution (as appropriate)
- Abstract includes: problem, gap, method, main result(s), implication
- Keywords reflect both author terms and common index terms (without stuffing)

Write into: `manuscript/meta_optimization.md`.
