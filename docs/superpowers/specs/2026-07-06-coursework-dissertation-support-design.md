# Coursework And Dissertation Support Design

## Goal

Make Qiongli support undergraduate-and-above coursework and dissertation work
as first-class academic project types while reusing the existing research
lifecycle, writing harness, citation-risk policy, subject routing, literature
workflows, and quality gates.

The new support should let users bring an assignment brief, marking rubric,
learning outcomes, supervisor feedback, or dissertation handbook and receive a
structured, academically honest workflow for planning, drafting, revising, and
checking the work. Qiongli should stop treating every student-facing request as
a journal manuscript. It should calibrate the output to the assignment,
program level, word count, rubric, and institutional AI policy.

## Current Context

The repository already has strong building blocks:

- `/academic-write` supports title, abstract, introduction, related work,
  literature review, research proposal, methodology, results, discussion,
  limitations, and conclusion drafting.
- `proposal-writer` explicitly covers thesis, dissertation, capstone, and
  course project proposals.
- `/paper-write` can build an outline, manuscript draft, claim-evidence map,
  and figures/tables plan from existing `RESEARCH/[topic]/` artifacts.
- `/lit-review`, `/paper-read`, `/find-gap`, `/study-design`, `/proofread`,
  and `/academic-present` already cover much of a dissertation journey.
- The workflow contract already tracks stages, task IDs, canonical artifacts,
  stage handoff state, evidence ledgers, and claim-evidence integrity.
- The subject runtime now handles undergraduate-and-above business projects,
  but its quality target is still doctoral-level journal contribution.

The gap is product fit. Coursework and dissertations have different success
criteria from manuscripts:

- The rubric matters more than journal novelty.
- Learning outcomes and module requirements can be binding constraints.
- Reflection and positionality may be valid evidence in some assignments.
- Word count, formatting, and submission requirements are strict.
- Students need support that is transparent about AI policy and academic
  integrity.
- Dissertations need chapter and supervisor-feedback state, not only paper
  sections.

## Product Decision

Add two first-class workflows:

```text
/coursework [assignment brief, task, or topic]
/dissertation [topic, program, or level]
```

These workflows should be product wrappers over the existing academic research
engine rather than separate writing systems. They introduce a learning-aware
front end and new artifact contracts, then route into existing research,
literature, design, writing, proofread, and presentation workflows where those
tools are a good fit.

The design uses two new supplemental stages:

- `L`: Coursework and learning-assessment support
- `M`: Dissertation and major-project support

`L` and `M` are not replacements for stages `A-K`. They are project-mode
stages that can orchestrate existing tasks. A dissertation may use `M1` for
planning, then still run `A1`, `B1`, `C1`, `F2`, `J4`, and `K1` as needed.

## Scope

In scope:

- Add `/coursework` and `/dissertation` workflow documents.
- Add `L` and `M` stage definitions to the workflow contract source and
  generated contract documentation.
- Add coursework and dissertation artifacts under the existing
  `RESEARCH/[topic]/` project root.
- Add coursework-specific templates for assignment brief intake, rubric maps,
  learning outcome maps, outlines, claim-evidence plans, draft structure,
  revision plans, and submission checklists.
- Add dissertation-specific templates for dissertation plans, chapter maps,
  chapter status, supervisor feedback logs, milestone plans, and defense prep.
- Add skill specs for assignment analysis, rubric mapping, coursework
  architecture, coursework revision, dissertation planning, chapter
  architecture, supervisor feedback integration, and dissertation readiness.
- Update platform routing so natural language requests for coursework,
  assignment briefs, module rubrics, dissertations, theses, capstones, and
  supervisor feedback route to the new workflows.
- Add academic-integrity and AI-policy checks that are visible in output.
- Add fixtures and tests for routing, artifact contracts, registry validation,
  and policy guardrails.

Out of scope for the first implementation:

- Do not build a grade predictor.
- Do not promise marks, grades, acceptance, or supervisor approval.
- Do not bypass university AI-use policies or produce hidden authorship.
- Do not invent citations, source quotations, page numbers, data, fieldwork,
  interview material, lab results, supervisor comments, or institutional rules.
- Do not make problem-set, exam, quiz, or timed-assessment solving the primary
  product surface. Qiongli can explain concepts and help structure reasoning,
  but the first release focuses on scholarly written coursework.
- Do not replace the existing `/paper` router.
- Do not turn every course case into a business subject activation. Coursework
  classification and subject classification remain separate decisions.

## User-Facing Model

### Coursework

Users can start from any of these:

```text
/coursework Analyze this assignment brief.
/coursework Write a plan for a 2,000-word undergraduate essay.
/coursework Revise this case analysis against the rubric.
/coursework Build a presentation script from this module assignment.
/coursework Check whether this literature review meets the learning outcomes.
```

The workflow classifies the assignment into one of these coursework types:

- `essay`
- `report`
- `case_analysis`
- `reflective_writing`
- `literature_review`
- `research_proposal`
- `presentation`
- `portfolio`
- `lab_or_methods_report`
- `capstone_project`
- `dissertation_component`

The classification controls structure, evidence expectations, tone, and the
kind of upstream research workflows that may be used.

### Dissertation

Users can start from any of these:

```text
/dissertation Plan an undergraduate dissertation on platform governance.
/dissertation Turn this proposal into a chapter plan.
/dissertation Revise Chapter 2 using supervisor feedback.
/dissertation Build a final readiness checklist for this dissertation.
/dissertation Prepare viva questions from my dissertation draft.
```

The workflow records the level:

- `undergraduate`
- `taught_master`
- `professional_master`
- `research_master`
- `doctoral`

The level controls contribution expectations. Undergraduate and taught master's
work should emphasize feasibility, correct method use, literature grounding,
and rubric fit. Doctoral work can reuse the stricter research and journal
quality gates where appropriate.

## New Task IDs

### Stage L: Coursework And Learning Assessment

| Task ID | Purpose | Primary output |
|---|---|---|
| `L1` | Assignment brief intake | `assignment/brief.md` |
| `L2` | Rubric and learning-outcome mapping | `assignment/rubric_map.md`, `assignment/learning_outcomes.md` |
| `L3` | Coursework outline and structure plan | `coursework/outline.md` |
| `L4` | Coursework claim-evidence and citation plan | `coursework/claim_evidence_plan.md` |
| `L5` | Coursework draft or section draft | `coursework/draft.md` |
| `L6` | Coursework revision against rubric | `coursework/revision_plan.md` |
| `L7` | Coursework final readiness check | `assignment/submission_checklist.md` |

### Stage M: Dissertation And Major Project

| Task ID | Purpose | Primary output |
|---|---|---|
| `M1` | Dissertation project planning | `dissertation/dissertation_plan.md` |
| `M2` | Dissertation chapter architecture | `dissertation/chapter_map.md` |
| `M3` | Dissertation chapter drafting | `dissertation/chapters/` |
| `M4` | Supervisor feedback integration | `dissertation/supervisor_feedback_log.md`, `dissertation/revision_plan.md` |
| `M5` | Dissertation milestone and risk planning | `dissertation/milestone_plan.md` |
| `M6` | Dissertation final readiness check | `dissertation/final_readiness.md` |
| `M7` | Viva or defense preparation | `dissertation/defense_prep.md` |

## Artifact Model

Keep using `RESEARCH/[topic]/` so existing context packaging, path conventions,
and handoff tooling continue to work.

```text
RESEARCH/[topic]/
├── assignment/
│   ├── brief.md
│   ├── rubric_map.md
│   ├── learning_outcomes.md
│   ├── academic_integrity_notes.md
│   └── submission_checklist.md
├── coursework/
│   ├── outline.md
│   ├── claim_evidence_plan.md
│   ├── citation_plan.md
│   ├── draft.md
│   ├── revision_plan.md
│   └── final_response.md
├── dissertation/
│   ├── dissertation_plan.md
│   ├── chapter_map.md
│   ├── chapter_status.md
│   ├── milestone_plan.md
│   ├── supervisor_feedback_log.md
│   ├── revision_plan.md
│   ├── final_readiness.md
│   ├── defense_prep.md
│   └── chapters/
└── context/
    ├── research_state.md
    ├── decision_log.md
    ├── boundary_review.md
    └── stage_handoff.md
```

The `assignment/` directory is shared by coursework and dissertation projects
because dissertations often have handbooks, module rules, marking criteria, and
institutional AI-policy language.

## Coursework Workflow Semantics

`/coursework` runs in preview-first mode.

### Step 1: Intake

Parse or request:

- assignment title and module/program context,
- task prompt or brief,
- rubric or marking criteria,
- learning outcomes,
- word count and formatting constraints,
- citation style,
- allowed source types,
- deadline or milestone constraints,
- AI-use policy if available,
- user's current material.

If the brief or rubric is missing, continue with explicit missing-input flags.
Do not invent module rules.

### Step 2: Classify The Coursework

Classify the task type and level. The classifier should separate:

- coursework type,
- subject or method routing,
- project level,
- evidence expectations,
- whether personal reflection is an acceptable evidence source.

Example: a "business case assignment" is coursework type `case_analysis`. It
does not automatically activate the business subject unless the request also
contains scholarly business research signals.

### Step 3: Map Rubric And Learning Outcomes

Create a rubric map that turns each criterion into:

- required capability,
- evidence or content needed,
- quality threshold,
- section where it should appear,
- current status,
- risk if missing.

For learning outcomes, record whether each outcome is addressed directly,
indirectly, or not yet addressed.

### Step 4: Build The Structure

Build an outline appropriate to the task type. Do not default to manuscript
IMRaD structure unless the assignment asks for a research report.

Structure examples:

- Essay: thesis, conceptual setup, argument clusters, counterargument,
  conclusion.
- Report: executive summary, issue, analysis, recommendations, implementation
  or limitations.
- Case analysis: case facts, diagnostic frame, alternatives, recommendation,
  evidence limits.
- Reflection: experience anchor, analytical lens, learning outcome connection,
  implication, boundary.
- Literature review: search scope, themes, synthesis, gap, implication.

### Step 5: Draft Under The Writing Harness

Use the existing writing harness:

- state the section's role,
- state the central claim and evidence threshold,
- draft in paragraph clusters,
- review for drift, support, specificity, and rubric fit,
- ask for missing user material when required.

Personal reflection, placement experience, or fieldwork claims require
user-supplied facts. Qiongli must not invent lived experience.

### Step 6: Revise Against The Rubric

The revision pass compares the draft to:

- rubric map,
- learning outcomes,
- word count,
- citation plan,
- academic integrity notes,
- assignment format.

The output should list concrete revisions and blocked items. It should not
claim the work will receive a grade.

### Step 7: Final Readiness

The final checklist confirms:

- every rubric criterion is addressed or flagged,
- every learning outcome is addressed or flagged,
- citations are present where claims need support,
- personal or empirical claims are user-supplied,
- word count and formatting constraints are visible,
- AI-use and disclosure requirements are recorded,
- missing information remains visible.

## Dissertation Workflow Semantics

`/dissertation` is a wrapper around a long-running project state machine.

### Step 1: Project Profile

Record:

- title or working topic,
- degree level,
- discipline or subject,
- department or program,
- dissertation type,
- word count and chapter expectations,
- handbook or marking criteria,
- supervisor requirements,
- ethics constraints,
- current stage,
- available materials.

Supported dissertation types:

- empirical quantitative,
- empirical qualitative,
- mixed methods,
- systematic or scoping review,
- theoretical or conceptual,
- design science or artifact-based,
- professional or applied project.

### Step 2: Plan The Dissertation

`M1` creates `dissertation/dissertation_plan.md` with:

- working title,
- research problem,
- research questions,
- expected contribution at the correct degree level,
- method or review approach,
- required approvals,
- source and data dependencies,
- timeline,
- risks and fallback options.

This may call existing tasks such as `A1`, `A2`, `A4`, `C1`, `C3`, `C4`, and
`D1` when the project needs stronger research framing or ethics planning.

### Step 3: Build Chapter Architecture

`M2` creates `dissertation/chapter_map.md` with:

- chapter purpose,
- target word count,
- inputs required,
- output expectations,
- dependencies on other chapters,
- evidence threshold,
- status.

Default chapter families:

- introduction,
- literature review,
- methodology,
- findings or results,
- discussion,
- conclusion,
- references and appendices.

The chapter map should adapt for systematic reviews, conceptual dissertations,
and professional projects.

### Step 4: Draft Chapters

`M3` routes chapters through existing writing capabilities:

- literature review chapters can use `/lit-review`, `/paper-read`, and
  `/academic-write`,
- methodology chapters can use `/study-design` and `/academic-write`,
- results chapters can use analysis interpretation and tables/figures plans,
- discussion and conclusion can use the writing harness and claim-evidence map.

Every chapter draft must record unresolved evidence gaps and claim strength.

### Step 5: Integrate Supervisor Feedback

`M4` parses feedback into:

- issue,
- affected chapter or claim,
- required action,
- evidence needed,
- priority,
- status,
- response or revision note.

The feedback workflow must preserve the user's supervisor's meaning. It should
not invent feedback or imply supervisor approval.

### Step 6: Track Milestones And Risks

`M5` tracks:

- proposal approval,
- ethics application,
- data collection,
- analysis,
- chapter draft deadlines,
- supervisor review windows,
- final formatting,
- submission.

Risks should include data access, recruitment, ethics, source availability,
method fit, analysis complexity, and word count pressure.

### Step 7: Final Readiness And Defense

`M6` checks:

- chapter completeness,
- research question alignment,
- method and evidence fit,
- citation coverage,
- claim-evidence traceability,
- formatting and submission requirements,
- AI-use notes,
- unresolved risks.

`M7` prepares viva or defense material when relevant:

- likely questions,
- contribution explanation,
- methods defense,
- limitations defense,
- what changed from proposal to final submission,
- concise oral summary.

## Academic Integrity Contract

Add `assignment/academic_integrity_notes.md` for both workflows.

The contract should record:

- institution or course AI policy when supplied,
- disclosure requirement when supplied,
- what assistance Qiongli provided,
- which facts, experiences, data, or interpretations came from the user,
- missing policy information,
- blocked requests.

Mandatory behavior:

- Do not fabricate citations, quotations, page numbers, datasets, interviews,
  fieldnotes, lab results, placement experiences, supervisor feedback, ethics
  approval, or institutional rules.
- Do not hide uncertainty about sources or evidence.
- Do not promise grades.
- Do not present planned work as completed findings.
- Do not rewrite personal reflection as if Qiongli had the user's lived
  experience.
- When AI-policy information is missing, mark it as missing and advise the user
  to check the relevant course or institution policy before submission.

Allowed behavior:

- Explain the assignment.
- Build an outline.
- Map the rubric.
- Suggest sources to verify.
- Draft or revise sections from user-provided facts and evidence.
- Improve clarity, structure, argument, and citation placement.
- Produce checklists and feedback reports.

## Skill And Template Additions

### New skill specs

Add focused skills instead of expanding one large coursework file:

```text
content/skills/L_coursework/assignment-brief-analyzer.md
content/skills/L_coursework/rubric-mapper.md
content/skills/L_coursework/coursework-architect.md
content/skills/L_coursework/coursework-reviser.md
content/skills/M_dissertation/dissertation-planner.md
content/skills/M_dissertation/chapter-architect.md
content/skills/M_dissertation/supervisor-feedback-integrator.md
content/skills/M_dissertation/dissertation-readiness-checker.md
```

Each skill should define inputs, outputs, constraints, failure modes, and
domain-aware behavior. The skills should be added to `content/skills/registry.yaml`
and generated documentation.

### New templates

```text
content/templates/assignment-brief.md
content/templates/rubric-map.md
content/templates/learning-outcomes.md
content/templates/academic-integrity-notes.md
content/templates/coursework-outline.md
content/templates/coursework-claim-evidence-plan.md
content/templates/coursework-revision-plan.md
content/templates/coursework-submission-checklist.md
content/templates/dissertation-plan.md
content/templates/dissertation-chapter-map.md
content/templates/dissertation-chapter-status.md
content/templates/supervisor-feedback-log.md
content/templates/dissertation-milestone-plan.md
content/templates/dissertation-final-readiness.md
content/templates/dissertation-defense-prep.md
```

Templates should contain explicit missing-information fields rather than blank
prompts that encourage invented content.

## Routing And Trigger Rules

Update platform routing so these requests route to `/coursework`:

- "assignment brief"
- "coursework"
- "module assignment"
- "marking rubric"
- "learning outcomes"
- "write my essay for this course" with policy guardrails
- "case analysis assignment"
- "reflective assignment"
- "portfolio assessment"
- "capstone coursework"

Route to `/dissertation`:

- "dissertation"
- "thesis"
- "undergraduate dissertation"
- "master's dissertation"
- "supervisor feedback"
- "chapter plan"
- "viva prep"
- "defense prep"
- "dissertation handbook"

Routing must preserve subject refinement:

- Coursework type decides the project workflow.
- Subject routing decides whether economics, finance, accounting, business, or
  another subject overlay should load.
- A business case assignment is not enough to activate business unless the
  scholarly business subject signals pass their normal gate.

## Workflow Contract Changes

Update the canonical workflow contract source:

- Add stages `L` and `M`.
- Add task IDs `L1-L7` and `M1-M7`.
- Add expected output artifacts.
- Add refresh points for coursework and dissertation context.
- Add `academic_project_type` or equivalent project metadata.

Suggested project metadata:

```yaml
academic_project_type:
  - journal_manuscript
  - research_paper
  - coursework
  - capstone
  - dissertation
  - thesis
  - presentation
```

Do not replace existing `paper_type`. `paper_type` remains about research
design shape. `academic_project_type` records the educational or publication
context.

## Context Continuity

Extend context state so long coursework and dissertation projects do not lose
constraints:

- current assignment brief,
- rubric criteria,
- learning outcomes,
- AI policy status,
- degree level,
- chapter status,
- supervisor feedback,
- deadline and milestone constraints,
- user-supplied personal or empirical material,
- unresolved evidence gaps.

Stage handoff should include whether the next task is allowed to draft prose or
must first request missing material.

## Evaluation Strategy

Add fixtures that prove both positive routing and guardrails.

### Coursework positives

- Undergraduate essay with rubric and word count routes to `/coursework`.
- Case analysis assignment routes to coursework, not automatically to business
  subject activation.
- Literature review coursework can call literature support without claiming
  systematic-review-grade coverage.
- Reflective assignment requires user-supplied experience before drafting
  personal claims.
- Research proposal coursework routes to coursework and can reuse
  `proposal-writer`.

### Dissertation positives

- Undergraduate dissertation request routes to `/dissertation`.
- Master's dissertation chapter planning creates `M1/M2` artifacts.
- Supervisor feedback request routes to `M4`.
- Dissertation viva prep routes to `M7`.
- Dissertation literature review can reuse `/lit-review` and `/academic-write`.

### Guardrails

- Missing rubric produces gap notes rather than invented criteria.
- Missing AI policy creates an academic-integrity warning.
- Requests for guaranteed grades are blocked.
- Requests to fabricate citations are blocked.
- Personal reflection without user material is blocked or converted into
  questions for the user.
- Timed exam or quiz requests do not route to coursework drafting.

## Expected Code And Content Changes

### Workflow files

Add:

```text
content/workflow/workflows/coursework.md
content/workflow/workflows/dissertation.md
content/workflow/references/stage-L-coursework.md
content/workflow/references/stage-M-dissertation.md
```

Update:

```text
content/workflow/workflows/academic-write.md
content/workflow/workflows/paper.md
content/workflow/references/workflow-contract.md
content/workflow/references/coverage-matrix.md
content/workflow/references/platform-routing.md
content/standards/research-workflow-contract.yaml
```

### Skills and registry

Add `L_coursework` and `M_dissertation` skill directories, then update:

```text
content/skills/registry.yaml
content/skills-summary.md
content/skills-core.md
docs/reference/skills.md
docs/zh/reference/skills.md
```

Update schema enums if the skill schema validates stage names.

### CLI and MCP surfaces

If the CLI exposes workflow names directly, add:

```text
qiongli coursework
qiongli dissertation
```

If MCP exposes workflow routing through `qiongli_task_run`, add the two
workflows to accepted task names and preview output.

The first implementation can keep these as skill/workflow routes without new
provider tools.

### Documentation

Update:

```text
README.md
docs/guide/task-recipes.md
docs/guide/using-agent-skills.md
docs/zh/guide/task-recipes.md
docs/zh/guide/using-agent-skills.md
CLAUDE.md
```

Mention that coursework support is academic-assistance support with visible
policy checks, not a grade guarantee.

## Implementation Slices

### Slice 1: Contracts and docs

- Add stages, task IDs, workflow docs, templates, and routing references.
- Add registry entries for new skills.
- Add generated docs if generation scripts are part of the normal workflow.
- Verify documentation and schema validation.

### Slice 2: Coursework vertical path

- Implement `/coursework` with `L1-L7`.
- Add assignment brief analyzer, rubric mapper, coursework architect, and
  coursework reviser.
- Add coursework routing fixtures and guardrail tests.
- Verify a complete essay/report/case-analysis preview without external
  literature providers.

### Slice 3: Dissertation vertical path

- Implement `/dissertation` with `M1-M7`.
- Add dissertation planner, chapter architect, supervisor feedback integrator,
  and dissertation readiness checker.
- Reuse existing research workflows for literature, study design, chapter
  drafting, proofread, and presentation prep.
- Verify an undergraduate dissertation plan and supervisor-feedback revision
  path.

### Slice 4: Runtime and release hardening

- Add CLI/MCP task exposure where needed.
- Add cross-platform route checks.
- Add Chinese docs.
- Run release-readiness validation before packaging.

## Verification

Required checks after implementation:

```bash
uv run python tooling/scripts/validate_research_standard.py
uv run python -m pytest
```

Focused checks should also cover:

- workflow contract generation,
- skill registry loading,
- platform routing,
- subject routing near-misses,
- coursework guardrails,
- dissertation artifact paths,
- docs links.

If the full test suite is too broad during implementation, each slice must
define a smaller focused command and finish with the full validation before
release.

## Risks And Mitigations

| Risk | Mitigation |
|---|---|
| Coursework support becomes generic ghostwriting | Require rubric, evidence, user material, and academic-integrity notes; block fabrication and grade guarantees |
| Rubric handling invents module rules | Keep missing rubric fields explicit and avoid inferred institutional policy |
| Subject routing over-activates from course wording | Keep coursework type classification separate from subject classification |
| New stages break schema validation | Update schema enums, registry docs, and contract generation in the same slice |
| Dissertation duplicates `/paper` | Make `/dissertation` a wrapper over existing A-K tasks, not a parallel research engine |
| First implementation is too large | Deliver in slices with contract/docs first, then coursework, then dissertation |

## Release Criteria

The feature is release-ready when:

- `/coursework` and `/dissertation` are documented workflow entry points.
- Stage `L` and `M` task IDs appear in the canonical workflow contract.
- Core templates exist and contain missing-information fields.
- Coursework and dissertation skills are in the registry.
- Routing fixtures prove positive and guardrail behavior.
- Guardrail tests block fabrication, grade guarantees, missing reflection
  material, and timed-assessment drafting.
- Existing `/paper`, `/academic-write`, and subject routing behavior still
  passes.
- Documentation explains the academic-integrity boundary clearly.
