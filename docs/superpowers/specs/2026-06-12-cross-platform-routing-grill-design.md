# Cross-Platform Qiongli Routing And Stage Grill Design

## Goal

Make Qiongli trigger reliably across Codex, Claude / Claude Code, Gemini, and CLI
surfaces whenever the user is doing academic research work, even when the user
does not explicitly invoke `$qiongli`, `/paper`, `/lit-review`, or another command.

The change also makes `grillme` a stage-aware research judgment mechanism rather
than a standalone prompt. The workflow should use light automatic grilling at
high-risk or ambiguous moments and deep grill loops when the user explicitly asks
to be challenged.

## Non-Goals

- Do not make Qiongli trigger for ordinary software engineering, generic file
  cleanup, unrelated business writing, or non-academic document edits.
- Do not replace platform-specific skill/plugin systems with a new runtime.
- Do not force every small research edit into a long interview.
- Do not duplicate `boundary-interviewer` and `self-critique`; extend and route
  through those existing cross-cutting skills.

## Problem

Qiongli currently relies too heavily on explicit workflow words and command-style
invocation. Users often ask naturally:

- "Read this file and improve the literature review."
- "I don't know how to start this topic."
- "Help me revise this methods section."
- "Modify this analysis script."
- "Check whether this result supports my claim."
- "Prepare a response to these reviewer comments."

Those requests may be handled as generic file reading, generic editing, or generic
coding. The result loses Qiongli's academic contracts: stage routing, evidence
thresholds, claim calibration, method validity, reproducibility, reviewer risk,
and artifact continuity.

For code tasks, Stage I already says the workflow is academic research code, not
product engineering. The current guidance is still too abstract, so generated code
can look like a computer engineering project instead of an academic analysis
pipeline centered on estimands, data lineage, model diagnostics, tables, figures,
and reproducible manuscript evidence.

## Design

### 1. Cross-Platform Trigger Contract

Add a canonical trigger contract to `content/workflow/SKILL.md` and
`content/workflow/references/platform-routing.md`.

The contract should state that Qiongli should be considered whenever a request
belongs to the academic research lifecycle, including:

- framing a topic, question, hypothesis, contribution, or venue fit
- reading papers, PDFs, notes, citations, bibliographies, or literature folders
- searching, screening, mapping, extracting, or synthesizing literature
- designing a study, variables, instruments, robustness checks, data plans, or
  preregistration
- writing or revising academic sections, proposals, manuscripts, abstracts,
  tables, figures, or claims maps
- interpreting statistical results, effect sizes, models, diagnostics, or
  robustness checks
- reading or editing academic analysis code, notebooks, Stata scripts, R scripts,
  Python scripts, Quarto files, or replication packages
- checking reporting compliance, PRISMA, tone, citation support, originality,
  submission packages, rebuttals, peer review responses, or presentations

The contract should also define non-trigger cases:

- generic software feature work unrelated to research analysis
- generic prose editing without scholarly claim, evidence, venue, or citation risk
- file organization or format conversion without academic interpretation
- project maintenance tasks for the Qiongli repository itself, unless the user is
  changing research workflow behavior

### 2. Ambiguity Trigger

Add an ambiguity trigger. When the user appears unsure, underspecified, or asks
for judgment in an academic context, Qiongli should run a light boundary/grill
step before producing a final artifact.

Trigger phrases include, in English or Chinese:

- "I don't know how to start"
- "not sure"
- "help me decide"
- "which direction"
- "is this reasonable"
- "what should I do next"
- "帮我判断"
- "不知道怎么做"
- "不确定"
- "方向不清楚"
- "帮我想想"
- "这样是否合理"

The automatic response should not ask a broad questionnaire. It should inspect
available artifacts first, then ask one blocking academic question with a
recommended answer and rationale.

### 3. Platform Coverage

The same canonical contract must be distributed to all supported surfaces:

- Codex: skill description, plugin manifest description, keywords, and default
  prompts.
- Claude / Claude Code: skill package text, Claude plugin metadata, workflow
  wrappers, and MCP-adjacent guidance.
- Gemini: Gemini extension metadata and workflow descriptions.
- CLI / npm / Python package: canonical workflow package and generated payloads.
- Portable `qiongli-workflow`: synced skill package used by non-plugin installs.

The platform-specific metadata should point to the same conceptual routing rules,
but the canonical wording should live in `content/workflow/` to avoid divergent
behavior across packages.

### 4. Stage-Aware Grill Contract

Extend Qiongli's existing `boundary-interviewer` and `self-critique` model into a
formal stage-aware grill contract.

Light automatic grill should run when:

- a stage starts with vague or underdefined academic scope
- a stage handoff contains open risks or stale decisions
- a central claim, method, evidence threshold, analysis decision, submission
  promise, or presentation claim changes
- the user expresses uncertainty or asks for judgment

Deep grill should run when the user explicitly asks:

- "grill me"
- "stress-test this"
- "challenge this"
- "act like Reviewer 2"
- "find fatal flaws"
- "严格质询"
- "像审稿人一样挑战"
- "找致命问题"

The grill loop must ask one question at a time, include the recommended answer,
and record the decision or unresolved issue.

### 5. Stage-Specific Grill Lenses

Each stage should have a distinct grill lens:

- Stage A: scope, contribution, claim strength, audience, venue fit
- Stage B: search bias, classic-paper deference, synthesis logic, missing rival
  literatures, inclusion/exclusion risk
- Stage C: identification, measurement validity, rival hypotheses, data
  feasibility, power or sensitivity logic
- Stage D: participant risk, privacy, consent clarity, governance, dual-use risk
- Stage E: heterogeneity, publication bias, pooling defensibility, null or
  contradictory evidence
- Stage F: claim-evidence alignment, causal language, interpretation depth,
  alternative explanations, limitation specificity
- Stage G: reporting completeness, logical transitions, tone calibration,
  checklist compliance
- Stage J: AI-trace risk, human scholarly voice, citation originality, final
  proofread integrity
- Stage H: reviewer empathy, response tone, impossible requests, contradiction
  introduced by revisions, fatal flaw exposure
- Stage I: estimand-to-code traceability, data lineage, diagnostic coverage,
  reproducibility, computational assumptions, analysis artifact outputs
- Stage K: audience fit, claim compression, visual evidence integrity, slide
  narrative, unsupported simplification

### 6. Cross-Stage Grill Memory

Grill outputs must be reusable across stages rather than isolated to one exchange.
Use existing artifacts as the memory spine:

- `RESEARCH/[topic]/context/boundary_review.md` for locked scholarly boundaries
- `RESEARCH/[topic]/context/decision_log.md` for resolved decisions
- `RESEARCH/[topic]/context/stage_handoff.md` for unresolved risks passed forward
- `RESEARCH/[topic]/review/self_critique_log.md` for issue lineage across review
  rounds

Downstream stages must inspect these artifacts before asking new questions. If a
previous grill issue is still open, the downstream stage must either resolve it,
carry it forward, or explicitly mark why it is no longer relevant.

Examples:

- A Stage B search-bias issue should constrain Stage C design and Stage F related
  work claims.
- A Stage C identification risk should constrain Stage I analysis code and Stage F
  causal language.
- A Stage I reproducibility risk should constrain Stage H data availability and
  Stage G reporting checks.
- A Stage H reviewer concern can reopen Stage F writing or Stage C design only if
  the revisit trigger is recorded.

### 7. Academic Analysis Code Routing

When a user reads or edits `.py`, `.R`, `.Rmd`, `.qmd`, `.do`, `.m`, `.jl`,
`.ipynb`, or replication files in an academic project, Qiongli should route to
Stage I if the task affects data, models, tables, figures, results, or
reproducibility.

Stage I code guidance should be strengthened:

- Start from research estimand, hypothesis, analysis plan, or manuscript output,
  not application architecture.
- Preserve dataset lineage: raw input, cleaning, exclusion, derived variables,
  missingness, joins, and sample construction.
- Make model diagnostics and robustness checks first-class outputs.
- Write tables and figures to predictable manuscript-facing locations.
- Prefer scripts/notebooks that are readable to researchers over service layers,
  controllers, unnecessary classes, or backend-style abstractions.
- Keep reproducibility visible: seeds, dependency lock notes, command logs, and
  rerun instructions.
- Separate findings, interpretation, and implications in reports.

## Implementation Scope

1. Update canonical routing text in `content/workflow/SKILL.md`.
2. Update `content/workflow/references/platform-routing.md` with the
   cross-platform trigger contract.
3. Update `content/skills/Z_cross_cutting/boundary-interviewer.md` and
   `content/skills/Z_cross_cutting/self-critique.md` to formalize stage-aware
   grill behavior and cross-stage memory.
4. Update Stage I files:
   - `content/workflow/references/stage-I-code.md`
   - `content/workflow/workflows/code-build.md`
   - `content/skills/I_code/code-builder.md`
   - `content/skills/I_code/code-specification.md`
   - `content/skills/I_code/code-planning.md`
5. Update plugin and extension metadata for Codex, Claude, Gemini, and Next plugin
   packages so discovery terms include research analysis, manuscripts, papers,
   review, rebuttal, statistics, reproducibility, and academic code.
6. Materialize distribution payloads in place so packaged copies match the
   canonical source.
7. Add regression tests that assert:
   - canonical skill text contains cross-platform trigger and ambiguity routing
   - stage-aware grill is no longer limited to future stages
   - Stage I text includes academic analysis code constraints
   - Codex / Claude / Gemini metadata includes academic workflow discovery terms
   - materialized packages contain the same routing contract

## Risks

- Over-triggering could interrupt simple edits. Mitigation: define clear
  non-trigger cases and use light grill only for academic ambiguity or risk.
- Platform metadata may drift from canonical source. Mitigation: tests should
  check materialized payloads and plugin manifests.
- Grill loops could become performative. Mitigation: require one blocking question,
  recommended answer, artifact inspection first, and issue carry-forward.
- Stage I could become too rigid for exploratory analysis. Mitigation: distinguish
  exploratory notebooks from claim-supporting analysis; exploratory work records
  assumptions and does not claim final evidence status.

## Acceptance Criteria

- A user request about academic work can be routed to Qiongli without explicit
  `$qiongli` or slash-command invocation.
- Ambiguous academic requests trigger a light boundary/grill step with one
  recommended question.
- Explicit grill requests trigger a deeper stage-aware grill loop.
- Grill decisions and open issues can pass between stages through existing
  context artifacts.
- Academic code requests route to Stage I and prefer analysis-pipeline structure
  over generic software-engineering scaffolds.
- Distribution payload tests verify the contract is present across supported
  platform packages.
