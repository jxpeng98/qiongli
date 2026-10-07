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
- `Q2` (claim-evidence traceability): check during drafting/revision; `F4` and `G3` record formal coverage.
- Semantic gate report: update `quality-gate-report.md` with `q2_claim_evidence_traceability`; evidence must anchor central claims to the claim-evidence ledger, source notes, analysis outputs, citations, or explicit gap notes.

## Writing Harness Contract

This shared contract applies to Stage F workflows, skill cards and role prompts.
It specifies the result and boundaries; the active model chooses how to reach it.
Formal task outputs and Q1/Q2 gates above remain required for the selected task.
A direct paragraph answer does not require the full artifact set or a gate report.

### Trace the text being written

Use `references/evidence-verification.md` for manuscript-first claim detection
and source traversal during drafting and revision. Start from the actual text,
not only a completed reading note or a pre-existing claim map. Cover substantive
assertions throughout the requested unit, including background, methods, results,
discussion, captions and explanatory notes. Track independently supported clauses;
new explanations and connective inferences can introduce claims too.

Draft from inspected evidence with its scope and locator available. If a useful
explanation needs an uninspected premise, inspect the authorized source or keep
that premise as a named gap; do not draft it as fact and attach a nearby citation
afterward. Reconcile changed wording, attribution and manuscript location with
the final claim/source records. This is part of writing the selected unit, not a
requirement to start F4 or a separate reviewer for every chat paragraph.

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
Use **Re-review after a source change** in `references/evidence-verification.md`
to inspect indirect inferences and unmapped uses as well as recorded links.

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

### Readable, specific prose

Apply readability while composing every requested section, including methods,
results and discussion, not only the literature review or a later J2 polish.
Use `references/scholarly-voice.md` for the shared clarity decisions and English
or Chinese phrasing. Keep a recognizable subject, clear referents and an information
order that lets each sentence develop the preceding point. Explain the actual
relation between ideas before adding a transition. Abstract nouns, stacked
modifiers and translated clause order must not obscure who does what or what a
finding means. Retain necessary technical terms and qualifications; neither
ornate phrasing nor uniformly short sentences establishes scholarly quality.

Make detail serve the argument: unpack a consequential concept, comparison or
inferential step with inspected evidence. Removing repeated generalities is
compatible with explaining the important point more fully. During revision,
preserve quotations, numbers, citation attribution and the author's intended
meaning; flag unresolved ambiguity rather than silently choosing a new claim.
When shortening, preserve the evidence-to-conclusion step. A compact sequence of
claims without their reasons remains shallow; replacing those reasons with an
abstract label does not repair it. Explain the relevant relation directly, with
the amount of detail the reader needs to assess it.

When a paragraph is shallow, locate the missing step: what the evidence actually
shows, how it bears on the paragraph's point, or what follows for this paper.
Explain that step with available source detail. Define a consequential term at
first use, unpack a comparison's basis, and explain how an inference follows
from its premises. A list of authors or a paraphrase of results cannot substitute
for the needed explanation. Unsupported mechanisms, extra citations and longer
sentences do not add depth.

Across paragraphs, carry forward a specific established point or unresolved
question. Check whether the next paragraph answers, qualifies, contrasts with or
builds on it; supply the missing reasoning or reorder within the authorized edit.
Do not manufacture continuity with “therefore”, “however” or a repeated topic label.
Keep consistent terms and explicit referents so that the reader can follow the
argument without consulting internal notes.

### Literature review tied to the paper

For a manuscript's literature review/related work, reuse its research question,
central argument and scope before choosing themes. The section should explain
what existing evidence lets this paper assume, question or investigate. A missing
research question may permit a provisional source synthesis, but not invented
project positioning; ask for the missing focus when it determines the argument.
A section request does not by itself commission a systematic review or new search.

Develop the consequential comparisons beyond an author/topic inventory. Select
the details that explain the inference: what a source argues or finds, how its
evidence supports that statement, and what context or limitation changes its
relevance here. Compare studies or interpretations on a shared question using
their actual constructs, designs, texts, populations or time horizons as relevant.
Explain why convergence or disagreement matters; a difference in topic alone
does not establish a contradiction. Do not fill unreported methods or mechanisms
from expectations, treat overlapping samples as independent confirmation, or turn
abstract-only access into a claim of full-paper appraisal.

Connect the resulting synthesis to a concrete choice in this paper: a construct
definition, theoretical expectation, comparison, design decision or unresolved
question. "This is relevant to our topic" is insufficient without that connection.
Explain what remains unknown and what answering it would change. A list of
estimators, datasets or robustness checks does not by itself explain a substantive
contribution. Likewise, saying an estimate is not new or a dataset cannot identify
a mechanism does not replace explaining what the cited evidence establishes.
Keep competing explanations and limits that constrain the proposed contribution.
An unmeasured outcome is a gap in the inspected corpus, not proof that no prior
research exists. The evidence may support adopting or refining an existing account
rather than claiming novelty. These are analytical decisions, not a fixed
paragraph template, a study-count quota or a demand for a mechanism everywhere.

Use boundary reviews, feasibility decisions and analysis plans as drafting inputs,
not ready-made literature-review prose. Preserve consequential inferential limits
and methodological debates, but keep detailed implementation/reporting rules in
the section that needs them. The reader should understand the literature's actual
findings and unresolved question without reconstructing them from statements about
what this paper will not claim. A review may motivate a design; that motivation
must not crowd out the evidence it is supposed to synthesize.

When comparing estimates, explain whether their outcomes, populations, periods
and estimands permit the proposed comparison. A numerical ordering alone cannot
establish why studies differ. If the available sources do not support that
comparison, distinguish their questions and preserve the uncertainty rather than
inventing a reconciliation. For a supplied section, check available surrounding
definitions before treating an unexplained term as absent from the whole paper.

### Citation coverage and explanatory notes

Apply `references/citation-risk-policy.md` throughout the requested text,
including background, definitions and footnotes, not just the central claims.
Add inspected, claim-matching support where it is needed; citation coverage is
not a target number of references. Do not fabricate a source or a detail to make
a paragraph appear more complete. Keep unsupported assertions visibly unresolved
or narrow them to the available evidence.

Use an explanatory footnote/endnote when a useful qualification, term distinction,
historical aside or secondary detail would interrupt the main argument. Keep the
main claim, the evidence needed to assess it and consequential inferential limits
in the body. A note must not conceal a contradiction or missing support. Distinguish
explanatory notes from a venue's bibliographic note system: both may need source
citations, and neither exempts its factual claims from checking.

Make the body understandable on its own. A concept needed to follow the argument
belongs in the body; a secondary naming distinction or source-specific detail may
belong in a note. When such a detail matters for the intended reader, supply its
supported explanation instead of merely leaving a technical label unexplained.
Track a note's substantive assertions to their own passages, not automatically
to the nearest body citation. Do not invent extra content to force a footnote.

Follow the requested format and venue's note rules; use Markdown footnotes only
when the deliverable supports them. Preserve existing note identifiers where
possible, resolve every marker to its definition, avoid duplicate definitions
and check citations inside notes. If notes are forbidden, incorporate the useful
explanation concisely in the permitted form. Do not add notes without a reader
need, invent page locators, or use notes to evade the requested length limit.

### Review and completion

Read the complete requested unit for flow and depth: can the reader follow the
argument without reconstructing missing steps? For related work, inspect the
source comparisons and their specific connection to the paper. Check citation
coverage in background as well as analysis, and note placement and references
when notes are used. Revise the demonstrated defect; do not append these internal
checks as boilerplate to a requested passage.

Check the delivered unit against its opening question and carry the same
constructs, populations, outcomes and time frames through its later conclusions.
When the scope changes, explain that change. Check how each paragraph advances
the preceding point; a set of individually fluent paragraphs may still fail as a
section. Within the requested scope, repair a missing link or expose an unresolved
premise before spending effort on stylistic polish. Do not infer whole-manuscript
coherence from an isolated paragraph or silently reorganize an unrequested section.

Keep two review conclusions distinct: whether the statements are supported, and
whether the complete unit explains and connects them clearly. A source-accurate
author list can still fail depth/coherence; fluent prose can still overclaim.
After a meaning-bearing edit, recheck the affected claim and source location as
well as the revised flow. Do not mark the unit complete with a known missing
explanation, unsupported bridge or unresolved substantive note presented as fact.

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
- Source-dependent claims, including background and notes, have matching support
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

Build this map from the actual manuscript, not only the claims already in a ledger.
Every substantive source-dependent or inferential assertion in the selected scope
must trace to inspected evidence or an explicit unresolved gap.

**Definition of done**
- Coverage includes all selected body sections, captions and footnotes/endnotes
- Compound assertions are separated when their evidence or qualifiers differ
- Each claim has an inspected evidence pointer or a named gap, with manuscript location
- Claim types retain the evidence-ledger vocabulary; source results, synthesis and inference remain distinct

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
