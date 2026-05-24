# Academic Boundary Interviewer MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a grill-me-style academic boundary interviewer that helps Qiongli users lock scholarly scope, claim strength, evidence thresholds, method validity, and venue-facing commitments before high-risk research work proceeds.

**Architecture:** Treat `boundary-interviewer` as an academic decision-support skill, not a generic requirements interview. The canonical source of truth stays in root `skills/`, `standards/`, `templates/`, `schemas/`, and workflow files. The MVP adds one cross-cutting skill, one academic boundary contract, one `BoundaryReview` artifact template, and workflow hooks only for the highest-risk stages: A framing, C study design, F writing, H submission/rebuttal, and I research code.

**Tech Stack:** Markdown skill specs, YAML academic contracts, Qiongli research-state and decision-log artifacts, Python `unittest` with `PyYAML`, existing Qiongli package sync scripts.

---

## Academic Design Commitments

The MVP must keep the grill-me idea, but translate it into academic work:

- Boundary questions are about scholarly judgment, not software requirements.
- The primary object is the research claim: what can be said, for whom, under what evidence, and with what caveats.
- The skill must ask one question at a time, but each question must belong to a named academic boundary dimension.
- The artifact must preserve decisions that reviewers, supervisors, coauthors, or future workflow stages would care about.
- The first release is not a global gate. It validates the mechanism in stages where boundary drift is most damaging.

### Academic Boundary Dimensions

Use these dimensions throughout the contract, skill, template, and workflow hooks:

1. **Phenomenon boundary**: what exact phenomenon, process, population, context, time period, or corpus is in scope.
2. **Construct boundary**: how key terms are defined, what rival definitions are excluded, and what contested terms remain unstable.
3. **Contribution boundary**: whether the work claims theoretical, empirical, methodological, dataset, systems, or synthesis contribution.
4. **Claim-strength boundary**: whether claims are descriptive, interpretive, associative, causal, predictive, methodological, or normative.
5. **Evidence-threshold boundary**: what evidence would support, weaken, or falsify the claim.
6. **Method-validity boundary**: what design, identification, operationalization, robustness, or trustworthiness assumptions must hold.
7. **Rival-explanation boundary**: what alternative mechanisms, null cases, contradictory literatures, or deviant cases must be confronted.
8. **Generalizability boundary**: where findings stop: population, setting, measurement, model assumption, data source, language, time, or venue.
9. **Ethics/governance boundary**: participant risk, consent scope, privacy, deidentification, data access, and sharing constraints.
10. **Venue/reviewer boundary**: what the target community expects and what would trigger desk rejection or reviewer pushback.
11. **Research-code boundary**: how code choices affect analysis validity, reproducibility, data lineage, and claim-evidence traceability.
12. **Submission/revision boundary**: what the author can truthfully promise to submit, revise, disclose, or not do.

### MVP Trigger Policy

- **MVP stages:** `A`, `C`, `F`, `H`, `I`
- **Future stages:** `B`, `D`, `E`, `G`, `J`, `K`
- **L0:** inspect existing artifacts only; no user question.
- **L1:** ask one blocking academic boundary question.
- **L2:** ask up to five ordered academic boundary questions.
- **L3:** use a decision tree for irreversible, publication-critical, or ethics-sensitive decisions.

---

### Task 1: Academic Boundary Contract

**Files:**
- Create: `tests/test_boundary_interviewer_contract.py`
- Create: `standards/boundary-review-contract.yaml`

- [ ] **Step 1: Write the failing academic contract test**

Create `tests/test_boundary_interviewer_contract.py` with:

```python
from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = REPO_ROOT / "standards" / "boundary-review-contract.yaml"


class BoundaryInterviewerContractTests(unittest.TestCase):
    def test_contract_declares_academic_boundary_model(self) -> None:
        self.assertTrue(CONTRACT.is_file())
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))

        self.assertEqual(contract["name"], "boundary-review-contract")
        self.assertEqual(contract["artifact"], "context/boundary_review.md")
        self.assertEqual(contract["purpose"], "academic-boundary-clarification")
        self.assertEqual(contract["mvp_trigger_stages"], ["A", "C", "F", "H", "I"])
        self.assertEqual(contract["future_trigger_stages"], ["B", "D", "E", "G", "J", "K"])

        dimensions = set(contract["academic_boundary_dimensions"])
        for expected in (
            "phenomenon_boundary",
            "construct_boundary",
            "contribution_boundary",
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "method_validity_boundary",
            "rival_explanation_boundary",
            "generalizability_boundary",
            "ethics_governance_boundary",
            "venue_reviewer_boundary",
            "research_code_boundary",
            "submission_revision_boundary",
        ):
            self.assertIn(expected, dimensions)

    def test_contract_requires_scholarly_decision_fields(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        required = set(contract["required_fields"])

        for expected in (
            "research_question_or_claim",
            "boundary_dimension",
            "question",
            "recommended_answer",
            "claim_strength",
            "evidence_threshold",
            "rival_explanations",
            "validity_or_trustworthiness_risk",
            "generalizability_limit",
            "venue_or_reviewer_risk",
            "decision_log_update",
            "revisit_trigger",
        ):
            self.assertIn(expected, required)

    def test_contract_has_stage_specific_academic_questions(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        stage_questions = contract["stage_question_sets"]

        self.assertIn("A", stage_questions)
        self.assertIn("C", stage_questions)
        self.assertIn("F", stage_questions)
        self.assertIn("H", stage_questions)
        self.assertIn("I", stage_questions)

        self.assertIn("What evidence would make this research question answerable in one paper?", stage_questions["A"])
        self.assertIn("What rival explanation would make the preferred design insufficient?", stage_questions["C"])
        self.assertIn("Which central claim would a reviewer say exceeds the available evidence?", stage_questions["F"])
        self.assertIn("What promise in the cover letter or rebuttal cannot be truthfully supported?", stage_questions["H"])
        self.assertIn("Which code or data decision would change the scientific interpretation of the results?", stage_questions["I"])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: FAIL because the contract file does not exist yet.

- [ ] **Step 3: Add the academic boundary contract**

Create `standards/boundary-review-contract.yaml`:

```yaml
contract_version: "1.0.0"
name: "boundary-review-contract"
purpose: "academic-boundary-clarification"
artifact: "context/boundary_review.md"

principles:
  inspect_artifacts_before_asking: true
  one_question_at_a_time: true
  recommended_answer_required: true
  preserve_academic_uncertainty: true
  never_invent_evidence_or_citations: true
  sync_downstream_decisions: true

levels:
  L0:
    name: "silent-artifact-check"
    behavior: "Inspect existing research artifacts and record the boundary if already settled."
  L1:
    name: "single-blocking-academic-question"
    behavior: "Ask one boundary question when a scholarly decision blocks safe progress."
  L2:
    name: "short-academic-boundary-loop"
    behavior: "Ask up to five ordered questions across the most relevant academic dimensions."
  L3:
    name: "publication-critical-decision-tree"
    behavior: "Walk a decision tree for irreversible, ethics-sensitive, or submission-critical choices."

mvp_trigger_stages:
  - "A"
  - "C"
  - "F"
  - "H"
  - "I"

future_trigger_stages:
  - "B"
  - "D"
  - "E"
  - "G"
  - "J"
  - "K"

academic_boundary_dimensions:
  - "phenomenon_boundary"
  - "construct_boundary"
  - "contribution_boundary"
  - "claim_strength_boundary"
  - "evidence_threshold_boundary"
  - "method_validity_boundary"
  - "rival_explanation_boundary"
  - "generalizability_boundary"
  - "ethics_governance_boundary"
  - "venue_reviewer_boundary"
  - "research_code_boundary"
  - "submission_revision_boundary"

required_fields:
  - "task_id"
  - "stage"
  - "trigger_level"
  - "trigger_reason"
  - "research_question_or_claim"
  - "boundary_dimension"
  - "question"
  - "recommended_answer"
  - "user_answer_or_artifact_answer"
  - "claim_strength"
  - "evidence_threshold"
  - "rival_explanations"
  - "validity_or_trustworthiness_risk"
  - "generalizability_limit"
  - "venue_or_reviewer_risk"
  - "locked_decision"
  - "open_question"
  - "evidence_basis"
  - "decision_log_update"
  - "stage_handoff_update"
  - "revisit_trigger"

stage_question_sets:
  A:
    - "What evidence would make this research question answerable in one paper?"
    - "Which population, context, time period, or corpus is explicitly out of scope?"
    - "Which key construct has the most contested definition, and which definition will this project use?"
    - "What contribution type is primary, and which contribution types are not being claimed?"
    - "Who would cite this work if it succeeds, and why would they care?"
  B:
    - "Which literature stream must be included to avoid confirmation bias?"
    - "Which search boundary would a systematic-review reader challenge first?"
    - "What counterevidence or null-result vocabulary should be searched explicitly?"
  C:
    - "What rival explanation would make the preferred design insufficient?"
    - "Is the strongest defensible claim causal, associative, descriptive, interpretive, or exploratory?"
    - "Which validity threat would most likely invalidate the study?"
    - "What measurement or operationalization choice could flip the interpretation?"
    - "What evidence would force the design to narrow its claim?"
  D:
    - "What consent, privacy, or governance boundary limits what can be collected or shared?"
    - "Could linkage, reidentification, or secondary-use risk change the design?"
  E:
    - "What heterogeneity condition would make pooling inappropriate?"
    - "Which evidence source most threatens the certainty rating?"
  F:
    - "Which central claim would a reviewer say exceeds the available evidence?"
    - "Where must the manuscript distinguish finding, interpretation, and implication?"
    - "Which boundary condition belongs in the discussion rather than being hidden in limitations?"
    - "What alternative explanation must be acknowledged before the claim is credible?"
  G:
    - "Which cross-section inconsistency would damage reviewer trust most?"
    - "Which reporting item is missing enough to block submission?"
  H:
    - "What promise in the cover letter or rebuttal cannot be truthfully supported?"
    - "Which submission claim is most likely to trigger desk rejection or reviewer objection?"
    - "What reviewer request is impossible, and what narrower alternative can be offered?"
  I:
    - "Which code or data decision would change the scientific interpretation of the results?"
    - "Which analysis-plan assumption must the implementation preserve exactly?"
    - "Which data lineage, seed, dependency, or preprocessing choice affects reproducibility?"
  J:
    - "Which citation, similarity, or tone risk could still undermine final acceptance?"
  K:
    - "Which spoken claim needs a stronger caveat for this audience?"

sync_targets:
  - "context/research_state.md"
  - "context/decision_log.md"
  - "context/stage_handoff.md"
  - "manuscript/claims_evidence_map.md"
  - "design/validity-threat-matrix.md"
  - "code/code_specification.md"
```

- [ ] **Step 4: Run the contract test again**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS for the academic boundary model checks.

### Task 2: Boundary Review Academic Template And Artifact Type

**Files:**
- Modify: `tests/test_boundary_interviewer_contract.py`
- Create: `templates/boundary-review.md`
- Modify: `schemas/artifact-types.yaml`

- [ ] **Step 1: Add template and artifact-type tests**

Add these constants to `tests/test_boundary_interviewer_contract.py`:

```python
TEMPLATE = REPO_ROOT / "templates" / "boundary-review.md"
ARTIFACT_TYPES = REPO_ROOT / "schemas" / "artifact-types.yaml"
```

Add these test methods:

```python
    def test_template_preserves_academic_decision_record(self) -> None:
        self.assertTrue(TEMPLATE.is_file())
        template = TEMPLATE.read_text(encoding="utf-8")

        for heading in (
            "# Boundary Review",
            "## Scholarly Decision Context",
            "## Artifact Evidence Checked First",
            "## One-Question Academic Loop",
            "## Academic Boundary Map",
            "## Claim Strength And Evidence Threshold",
            "## Rival Explanations And Counterevidence",
            "## Validity Or Trustworthiness Risk",
            "## Generalizability Limit",
            "## Venue Or Reviewer Risk",
            "## Locked Decision",
            "## Revisit Trigger",
            "## Downstream Sync",
        ):
            self.assertIn(heading, template)

    def test_boundary_review_artifact_type_is_registered_as_academic_output(self) -> None:
        self.assertTrue(ARTIFACT_TYPES.is_file())
        payload = yaml.safe_load(ARTIFACT_TYPES.read_text(encoding="utf-8"))
        artifact_types = {item["name"]: item for item in payload["artifact_types"]}

        self.assertIn("BoundaryReview", artifact_types)
        boundary_review = artifact_types["BoundaryReview"]
        self.assertEqual(boundary_review["format"], "markdown")
        self.assertIn("boundary-interviewer", boundary_review["produced_by"])
        self.assertIn("academic-context-maintainer", boundary_review["consumed_by"])
        self.assertIn("manuscript-architect", boundary_review["consumed_by"])
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: FAIL because the template and artifact-type registration are missing.

- [ ] **Step 3: Add the academic boundary review template**

Create `templates/boundary-review.md`:

```markdown
# Boundary Review

Save to: `RESEARCH/[topic]/context/boundary_review.md`

Use this artifact to record academic boundary decisions before high-risk Qiongli tasks proceed. This is not a chat transcript or generic requirements note. It records the minimum scholarly decisions needed to prevent scope drift, overclaiming, weak evidence linkage, method mismatch, or unsafe submission promises.

## Scholarly Decision Context

- task_id:
- stage:
- paper_type:
- target_venue:
- current_research_question_or_claim:
- trigger_level:
- trigger_reason:

## Artifact Evidence Checked First

List the artifacts inspected before asking the user.

- `context/research_state.md`:
- `context/decision_log.md`:
- `context/stage_handoff.md`:
- stage_specific_artifacts:

## One-Question Academic Loop

| Question ID | Boundary Dimension | Question | Recommended Answer | User Or Artifact Answer | Status | Why This Matters |
|---|---|---|---|---|---|---|
| BQ-001 |  |  |  |  | open |  |

## Academic Boundary Map

| Dimension | Included / Claimed | Excluded / Not Claimed | Evidence Basis |
|---|---|---|---|
| Phenomenon / population / context |  |  |  |
| Construct definition |  |  |  |
| Contribution type |  |  |  |
| Method / design |  |  |  |
| Generalizability |  |  |  |
| Venue / reviewer expectation |  |  |  |

## Claim Strength And Evidence Threshold

- claim_strength:
- strongest_defensible_wording:
- evidence_required_to_support:
- evidence_that_would_weaken_or_falsify:
- finding_interpretation_implication_boundary:

## Rival Explanations And Counterevidence

| Rival / counterevidence | Why It Matters | How This Workflow Will Address It |
|---|---|---|
|  |  |  |

## Validity Or Trustworthiness Risk

- internal_validity_or_identification:
- construct_validity_or_operationalization:
- external_validity_or_transferability:
- statistical_conclusion_or_inference:
- credibility_dependability_confirmability_if_qualitative:

## Generalizability Limit

- population_limit:
- setting_limit:
- time_period_limit:
- data_or_measurement_limit:
- model_or_assumption_limit:

## Venue Or Reviewer Risk

- likely_reviewer_objection:
- desk_reject_risk:
- claim_or_scope_adjustment:

## Locked Decision

| Decision | Rationale | Confidence | Evidence Basis | Downstream Impact |
|---|---|---|---|---|
|  |  |  |  |  |

## Open Questions

| Question | Why It Remains Open | Next Task Or Artifact | Revisit Trigger |
|---|---|---|---|
|  |  |  |  |

## Revisit Trigger

-

## Downstream Sync

- `context/research_state.md`:
- `context/decision_log.md`:
- `context/stage_handoff.md`:
- `manuscript/claims_evidence_map.md`:
- `design/validity-threat-matrix.md`:
- `code/code_specification.md`:
```

- [ ] **Step 4: Register `BoundaryReview` as an academic artifact**

Add this item under the cross-cutting section in `schemas/artifact-types.yaml`:

```yaml
  - name: BoundaryReview
    description: "Academic boundary decision record covering scope, constructs, contribution, claim strength, evidence threshold, rival explanations, validity risks, generalizability limits, venue expectations, and downstream handoff"
    format: markdown
    produced_by: [boundary-interviewer]
    consumed_by: [academic-context-maintainer, manuscript-architect, study-designer, submission-packager, code-specification, self-critique, model-collaborator]
```

- [ ] **Step 5: Run the focused tests again**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS for contract, template, and artifact type.

### Task 3: Boundary Interviewer Skill As Academic Method

**Files:**
- Modify: `tests/test_boundary_interviewer_contract.py`
- Create: `skills/Z_cross_cutting/boundary-interviewer.md`
- Modify: `skills/registry.yaml`
- Modify: `skills-core.md`
- Modify: `skills-summary.md`

- [ ] **Step 1: Add skill registration tests**

Add these constants:

```python
REGISTRY = REPO_ROOT / "skills" / "registry.yaml"
SKILL = REPO_ROOT / "skills" / "Z_cross_cutting" / "boundary-interviewer.md"
SKILLS_CORE = REPO_ROOT / "skills-core.md"
SKILLS_SUMMARY = REPO_ROOT / "skills-summary.md"
```

Add this test:

```python
    def test_boundary_interviewer_skill_is_academic_not_generic_requirements(self) -> None:
        self.assertTrue(SKILL.is_file())
        registry = yaml.safe_load(REGISTRY.read_text(encoding="utf-8"))
        entries = {item["id"]: item for item in registry["skills"]}

        self.assertIn("boundary-interviewer", entries)
        entry = entries["boundary-interviewer"]
        self.assertEqual(entry["stage"], "Z_cross_cutting")
        self.assertEqual(entry["file"], "skills/Z_cross_cutting/boundary-interviewer.md")
        self.assertIn("BoundaryReview", entry["outputs"])
        self.assertIn("claim-strength", entry["tags"])
        self.assertIn("evidence-threshold", entry["tags"])

        skill_text = SKILL.read_text(encoding="utf-8")
        for phrase in (
            "finding, interpretation, and implication",
            "rival explanations",
            "validity or trustworthiness",
            "generalizability",
            "venue or reviewer risk",
        ):
            self.assertIn(phrase, skill_text)

        self.assertIn("## boundary-interviewer", SKILLS_CORE.read_text(encoding="utf-8"))
        self.assertIn("| boundary-interviewer |", SKILLS_SUMMARY.read_text(encoding="utf-8"))
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: FAIL because the skill and registry entries are missing.

- [ ] **Step 3: Add the academic skill file**

Create `skills/Z_cross_cutting/boundary-interviewer.md`:

```markdown
---
id: boundary-interviewer
stage: Z_cross_cutting
description: "Clarify scholarly boundaries one question at a time before high-risk Qiongli tasks proceed."
inputs:
  - type: UserQuery
    description: "User request or task packet with unresolved academic scope, claim, evidence, method, venue, or handoff boundaries"
  - type: AnyArtifact
    description: "Existing research artifacts that may already settle the academic boundary"
outputs:
  - type: BoundaryReview
    artifact: "context/boundary_review.md"
constraints:
  - "Must inspect available academic artifacts before asking the user"
  - "Must ask only one scholarly boundary question at a time"
  - "Must provide a recommended answer with academic rationale and downstream impact"
failure_modes:
  - "Treating the process as generic software requirements gathering"
  - "Asking broad questionnaires instead of the next blocking scholarly question"
  - "Allowing overclaiming by failing to set evidence thresholds"
  - "Flattening uncertainty instead of preserving rival explanations and reviewer risks"
tools: [filesystem]
tags: [cross-cutting, boundary, grill-me, academic-judgment, claim-strength, evidence-threshold, validity, generalizability, venue-fit, handoff]
domain_aware: true
---

# Boundary Interviewer Skill

Clarify scholarly boundaries before high-risk Qiongli work proceeds.

## Purpose

Use this skill when a task could drift in research scope, claim strength, evidence threshold, method validity, writing promise, or submission commitment. It adapts the grill-me pattern into an academic workflow: inspect existing artifacts first, ask one blocking question at a time, recommend the most defensible answer, and preserve the decision for downstream stages.

The skill is not a generic requirements interview. It should help answer: what can this project honestly claim, for whom, with what evidence, against which rivals, and under what limits?

## Related Task IDs

- MVP trigger stages: `A`, `C`, `F`, `H`, `I`
- Future trigger stages: `B`, `D`, `E`, `G`, `J`, `K`

## Output (contract path)

- `RESEARCH/[topic]/context/boundary_review.md`

## Inputs

- `UserQuery`: Current request, workflow command, or task packet.
- `AnyArtifact`: Existing artifacts that may already define the boundary:
  - `context/research_state.md`
  - `context/decision_log.md`
  - `context/stage_handoff.md`
  - `framing/research_question.md`
  - `framing/contribution_statement.md`
  - `theoretical_framework.md`
  - `gap_analysis.md`
  - `study_design.md`
  - `analysis_plan.md`
  - `design/validity-threat-matrix.md`
  - `manuscript/claims_evidence_map.md`
  - `revision/response_matrix.md`
  - `code/code_specification.md`

If a required input is missing, record the missing artifact and ask only the next blocking academic question. Do not invent citations, findings, sample sizes, reviewer expectations, or institutional requirements.

## Process

1. Map the task to an academic boundary dimension from `standards/boundary-review-contract.yaml`.
2. Inspect existing artifacts before asking the user. If the boundary is already settled, cite the artifact and record the decision.
3. Choose the next blocking question. Prefer the question whose answer would most change the research question, claim strength, evidence threshold, method validity, or submission position.
4. Ask exactly one user-facing question.
5. Include a recommended answer with:
   - academic rationale
   - expected evidence basis
   - claim-strength implication
   - likely reviewer or venue consequence
   - confidence
6. Record the answer in `context/boundary_review.md`.
7. Preserve the boundary between finding, interpretation, and implication when the question affects writing claims.
8. Preserve rival explanations, null cases, contradictory evidence, or trustworthiness risks when the question affects design or synthesis.
9. Sync locked decisions into `context/decision_log.md`; sync unresolved risks into `context/stage_handoff.md`.

## Output Contract

Write `RESEARCH/[topic]/context/boundary_review.md` using `templates/boundary-review.md`.

The artifact must include:

- scholarly decision context
- artifacts checked before asking the user
- one-question academic loop
- academic boundary map
- claim strength and evidence threshold
- rival explanations and counterevidence
- validity or trustworthiness risk
- generalizability limit
- venue or reviewer risk
- locked decision
- revisit trigger
- downstream sync targets

## Quality Bar

- [ ] The question targets a named academic boundary dimension.
- [ ] Existing artifacts were inspected before asking the user.
- [ ] No more than one user-facing question was asked at a time.
- [ ] Every question included a recommended answer and academic rationale.
- [ ] Claim strength is explicit: descriptive, interpretive, associative, causal, predictive, methodological, normative, or exploratory.
- [ ] Evidence threshold states what would support, weaken, or falsify the claim.
- [ ] Rival explanations or counterevidence are named when material.
- [ ] Validity, trustworthiness, or reproducibility risks are preserved instead of smoothed over.
- [ ] Generalizability limits are concrete enough for a reviewer to evaluate.
- [ ] Downstream updates identify `decision_log`, `stage_handoff`, claim map, validity matrix, or code specification impacts.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Treating the skill like software requirements gathering | The output locks implementation details but misses scholarly risk | Ask about claim, evidence, method validity, and reviewer consequence |
| Asking a long questionnaire | The user answers mechanically and the model avoids judgment | Ask the next blocking academic question only |
| Asking what artifacts already answer | The workflow ignores its own evidence chain | Inspect research state, decision log, handoff, claim map, and design artifacts first |
| Overclaiming by default | The manuscript later exceeds evidence | Force claim-strength and evidence-threshold fields |
| Hiding rival explanations | Reviewers surface them later as fatal flaws | Record rival explanations, null cases, and contradictory evidence early |
| Treating limitations as cosmetic | Boundaries become boilerplate | State exact population, setting, data, measure, model, or venue limits |
```

- [ ] **Step 4: Register the skill**

Add this entry to the `Z_cross_cutting` section of `skills/registry.yaml`:

```yaml
  - id: boundary-interviewer
    stage: Z_cross_cutting
    version: "0.10.1"
    file: skills/Z_cross_cutting/boundary-interviewer.md
    canonical: true
    summary: "Clarify scholarly boundaries one question at a time before high-risk Qiongli work proceeds."
    display_name: "Boundary Interviewer"
    when_to_use: "Use before high-risk framing, study design, writing, submission, research code, or handoff work when research scope, claim strength, evidence threshold, validity risk, generalizability, or reviewer expectations remain unclear."
    summary_zh: "在高风险学术任务前用一次一个问题的方式确认研究范围、主张强度、证据阈值、有效性风险与审稿边界。"
    display_name_zh: "学术边界追问器"
    when_to_use_zh: "当研究问题、claim 强度、证据阈值、方法有效性、外推范围、投稿承诺或阶段交接仍不清楚时使用。"
    inputs: [UserQuery, AnyArtifact]
    outputs: [BoundaryReview]
    tags: [cross-cutting, boundary, grill-me, academic-judgment, claim-strength, evidence-threshold, validity, generalizability, venue-fit, handoff]
    depends_on: []
    compatible_models: [codex, claude, gemini, gpt]
    tooling_requirements: [filesystem]
    domain_aware: true
```

- [ ] **Step 5: Add core and summary references**

Add this section to `skills-core.md` near the other cross-cutting skills:

```markdown
## boundary-interviewer

**Purpose:** Clarify scholarly boundaries one question at a time before high-risk Qiongli work proceeds.

**Process:**
1. Inspect existing research artifacts before asking the user.
2. Map the task to an academic boundary dimension: phenomenon, construct, contribution, claim strength, evidence threshold, method validity, rival explanation, generalizability, ethics/governance, venue/reviewer, research code, or submission/revision.
3. Ask exactly one blocking academic question when artifacts cannot answer it.
4. Provide a recommended answer with rationale, evidence threshold, reviewer consequence, and confidence.
5. Record claim strength, evidence threshold, rival explanations, validity or trustworthiness risk, generalizability limit, and downstream sync targets.
6. Sync downstream-relevant decisions into `context/decision_log.md` or `context/stage_handoff.md`.

**Output:** `BoundaryReview` -> `context/boundary_review.md`
```

Add this row to the `Z - Cross-Cutting` table in `skills-summary.md`:

```markdown
| boundary-interviewer | Clarify scholarly boundaries one question at a time before high-risk work proceeds |
```

- [ ] **Step 6: Run the focused tests again**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS for the academic skill registration and references.

### Task 4: Academic Workflow Hooks

**Files:**
- Modify: `tests/test_boundary_interviewer_contract.py`
- Modify: `qiongli-workflow/workflows/paper.md`
- Modify: `qiongli-workflow/workflows/study-design.md`
- Modify: `qiongli-workflow/workflows/code-build.md`
- Modify: `qiongli-workflow/workflows/academic-write.md`
- Modify: `qiongli-workflow/workflows/submission-prep.md`

- [ ] **Step 1: Add workflow hook tests**

Add this test method:

```python
    def test_mvp_workflows_include_academic_boundary_trigger(self) -> None:
        workflow_paths = [
            REPO_ROOT / "qiongli-workflow" / "workflows" / "paper.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "study-design.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "code-build.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "academic-write.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "submission-prep.md",
        ]

        for path in workflow_paths:
            content = path.read_text(encoding="utf-8")
            self.assertIn("Academic Boundary Review Trigger", content, path.as_posix())
            self.assertIn("boundary-interviewer", content, path.as_posix())
            self.assertIn("claim strength", content, path.as_posix())
            self.assertIn("evidence threshold", content, path.as_posix())
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: FAIL because the MVP workflow hooks do not exist yet.

- [ ] **Step 3: Add the router hook to `paper.md`**

In `qiongli-workflow/workflows/paper.md`, after the canonical Task ID selection section, add:

```markdown
## Step 1.6: Academic Boundary Review Trigger (MVP)

For MVP, use `boundary-interviewer` before proceeding when the selected task belongs to `A`, `C`, `F`, `H`, or `I` and any of these academic boundaries remain unclear:

- research question answerability
- construct definition or contested terminology
- contribution type and non-claims
- claim strength and evidence threshold
- rival explanations or contradictory evidence
- validity, trustworthiness, or reproducibility risk
- generalizability limit
- venue or reviewer expectation
- submission or revision promise

The boundary pass must inspect project artifacts first. If artifacts do not settle the issue, ask one blocking academic question with a recommended answer, evidence threshold, and downstream impact.
```

- [ ] **Step 4: Add the study design hook**

In `qiongli-workflow/workflows/study-design.md`, after "Step 1: Clarify Research Question & Constraints", add:

```markdown
### Academic Boundary Review Trigger (MVP)

Before drafting `C1` through `C5`, use `boundary-interviewer` when the design still has unresolved scholarly boundaries around claim strength, unit of analysis, population or case boundary, construct operationalization, identification logic, sampling, saturation, data access, analysis strategy, rival hypotheses, validity, trustworthiness, ethics constraints, or preregistration commitments.

The boundary question must identify what kind of claim the design can support: descriptive, interpretive, associative, causal, predictive, methodological, or exploratory. It must also state what evidence would weaken the design enough to narrow the claim.
```

- [ ] **Step 5: Add the research-code hook**

In `qiongli-workflow/workflows/code-build.md`, after "Workflow Steps", add:

```markdown
## Academic Boundary Review Trigger (MVP)

For `I5` and `I6`, use `boundary-interviewer` before writing the specification or plan when a code decision could change scientific interpretation, reproducibility, or claim-evidence traceability.

Use the boundary pass for:

- analysis-plan assumptions that implementation must preserve
- data provenance, preprocessing, missingness, split, or leakage decisions
- random seed, stochastic operation, dependency, or environment choices that affect reproducibility
- synthetic or fixture data limits that affect validation
- output artifacts that feed manuscript claims, figures, tables, or evidence ledgers
- non-goals that prevent the code from implying a broader method claim than the paper supports

The boundary pass must ask one academic question at a time and must record which code decision could change the research claim or evidence threshold.
```

- [ ] **Step 6: Add the academic writing hook**

In `qiongli-workflow/workflows/academic-write.md`, after "Step 1: Understand the Writing Task", add:

```markdown
### Academic Boundary Review Trigger (MVP)

Use `boundary-interviewer` before drafting when the writing task has unresolved boundaries around section purpose, target audience, venue expectations, claim strength, available evidence, citation gaps, contribution type, finding-vs-interpretation-vs-implication, boundary conditions, or whether planned work is being written as completed work.

The boundary pass must identify the strongest defensible wording for the central claim and the evidence threshold required to use that wording. If the answer changes a central claim, synchronize the decision into `context/decision_log.md` or flag it for `academic-context-maintainer`.
```

- [ ] **Step 7: Add the submission/rebuttal hook**

In `qiongli-workflow/workflows/submission-prep.md`, before generating submission artifacts, add:

```markdown
## Academic Boundary Review Trigger (MVP)

Use `boundary-interviewer` before submission or rebuttal work when unresolved boundaries remain around journal fit, submission readiness, missing statements, data/code availability, authorship declarations, reviewer-sensitive weaknesses, impossible reviewer requests, response commitments, or claims that exceed the evidence.

The boundary pass must ask one academic question at a time. The answer must state which submission or revision artifact changes, what promise can truthfully be made, and whether the decision belongs in `context/decision_log.md` or `context/stage_handoff.md`.
```

- [ ] **Step 8: Run the focused tests again**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS for workflow hook coverage.

### Task 5: Distribution Sync

**Files:**
- Modify generated package contents under `qiongli-workflow/`
- Modify generated plugin copy under `plugins/qiongli/skills/qiongli-workflow/`
- Modify generated npm payload/runtime under `packages/npm-qiongli/`

- [ ] **Step 1: Sync the portable skill package and plugin package**

Run:

```bash
./scripts/sync_skill_package.sh --target all
```

Expected: the academic boundary contract, template, skill, registry entry, `skills-core.md`, and `skills-summary.md` are copied into `qiongli-workflow/`, then mirrored into `plugins/qiongli/skills/qiongli-workflow/`.

- [ ] **Step 2: Sync the npm payload and Python runtime copy**

Run:

```bash
uv run python scripts/sync_npm_package_payload.py
```

Expected: `packages/npm-qiongli/payload/qiongli-workflow/` and `packages/npm-qiongli/python-runtime/` contain the academic boundary interviewer assets.

- [ ] **Step 3: Run distribution checks**

Run:

```bash
uv run pytest tests/test_distribution_payloads.py tests/test_plugin_distribution_contract.py tests/test_npm_package_contract.py -q
```

Expected: PASS. If generated copies are stale, rerun the sync scripts and inspect only boundary-interviewer-related generated diffs.

### Task 6: Academic Verification

**Files:**
- No source edits expected beyond earlier tasks.

- [ ] **Step 1: Run focused boundary tests**

Run:

```bash
uv run pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS.

- [ ] **Step 2: Run skill and contract alignment tests**

Run:

```bash
uv run pytest tests/test_skill_contract_alignment.py tests/test_skill_structure_lint.py tests/test_skill_resource_links.py -q
```

Expected: PASS.

- [ ] **Step 3: Run workflow and orchestrator prompt tests**

Run:

```bash
uv run pytest tests/test_workflow_contract_doc.py tests/test_orchestrator_workflows.py -q
```

Expected: PASS. This MVP adds workflow-level academic boundary hooks; it does not require automatic orchestrator prompt injection yet.

- [ ] **Step 4: Inspect the implementation diff for academic focus**

Run:

```bash
git diff -- tests/test_boundary_interviewer_contract.py standards/boundary-review-contract.yaml templates/boundary-review.md schemas/artifact-types.yaml skills/Z_cross_cutting/boundary-interviewer.md skills/registry.yaml skills-core.md skills-summary.md qiongli-workflow/workflows/paper.md qiongli-workflow/workflows/study-design.md qiongli-workflow/workflows/code-build.md qiongli-workflow/workflows/academic-write.md qiongli-workflow/workflows/submission-prep.md
```

Expected: the diff emphasizes academic boundary dimensions, claim discipline, evidence thresholds, validity risks, generalizability limits, and venue/reviewer consequences rather than generic software planning.

### Post-MVP Academic Coverage Roadmap

This MVP is intentionally scoped to A/C/F/H/I. Expand only after the boundary artifact proves useful in actual research workflows.

1. Add optional orchestrator task-packet support: `boundary_review.enabled`, `boundary_review.level`, `boundary_review.dimension`, and `boundary_review.artifact`.
2. Add `bridges/boundary_questions.py` as a preflight question source, separate from `bridges/critique_questions.py`, so preflight boundary clarification and post-draft critique do not collapse into the same behavior.
3. Add future-stage question sets:
   - `B`: search boundaries, inclusion criteria, counterevidence, corpus drift, citation snowballing limits.
   - `D`: consent, privacy, participant risk, deidentification, secondary-use limits.
   - `E`: pooling decision, heterogeneity, certainty grading, nulls, publication bias.
   - `G`: reporting completeness, cross-section consistency, claim-evidence contradictions.
   - `J`: citation risk, similarity risk, final proofread boundaries.
   - `K`: audience, time budget, spoken claim strength, appendix defense.
4. Add a stage risk matrix to `standards/research-workflow-contract.yaml` only after MVP behavior is stable.
5. Add tests proving every necessary stage has one of: MVP trigger, future trigger, explicit L0 artifact-check behavior, or documented reason for no boundary review.
