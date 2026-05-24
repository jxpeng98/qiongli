# Academic Boundary Interviewer V2 Full Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the grill-me-style `boundary-interviewer` a first-class academic workflow control that can ask boundary questions, persist answers, and force later Qiongli tasks to continue within the answered scholarly boundaries.

**Architecture:** V2 promotes the MVP from workflow documentation into orchestration. A new `bridges/boundary_questions.py` module owns stage/task boundary policy and question selection; `bridges/orchestrator.py` adds boundary metadata to the task packet, loads `context/boundary_review.md`, injects it into draft/review/revision prompts, and reports boundary gate status. `academic-context-maintainer`, the context package builder, validators, and all A-K stage workflows consume the same boundary artifact so later coding, writing, planning, review, compliance, presentation, and submission tasks inherit the user-confirmed limits.

**Tech Stack:** Python `unittest`, `dataclasses`, existing Qiongli orchestrator bridge, Markdown/YAML skill contracts, generated plugin/npm package sync scripts.

---

## File Structure

- Create `bridges/boundary_questions.py`: deterministic academic boundary policy, stage/task question sets, trigger levels, prompt formatting helpers.
- Create `tests/test_boundary_questions.py`: unit tests for policy coverage, question ordering, answered-boundary reuse, and prompt formatting.
- Modify `bridges/orchestrator.py`: task packet fields, boundary artifact loading, draft/review/revision prompt sections, routing notes, validator gate integration.
- Modify `bridges/context_package.py`: include boundary review content in Codex, Claude, and Gemini context packages.
- Modify `tests/test_orchestrator_workflows.py`: orchestration tests for task packet fields, prompt injection, review blocking, and continuation after answers.
- Modify `tests/test_context_package_builder.py`: context package propagation tests.
- Modify `standards/boundary-review-contract.yaml`: promote MVP/future stages into a full A-K stage policy with checkpoint tasks and stage-specific dimensions.
- Modify `standards/research-workflow-contract.yaml`: declare `context/boundary_review.md` as an academic continuity artifact and add boundary refresh focus for A-K.
- Modify `skills/Z_cross_cutting/academic-context-maintainer.md`: consume boundary review and update research state, decision log, and stage handoff.
- Modify `skills/Z_cross_cutting/boundary-interviewer.md`: document V2 runtime behavior, not only manual use.
- Modify all workflow entrypoints under `qiongli-workflow/workflows/*.md`: add stage-specific boundary trigger hooks for B, D, E, G, J, and K while preserving existing MVP hooks.
- Modify `scripts/validate_research_standard.py`: add boundary-review validation for outputs that make claims, choose methods, change evidence, alter code/data, or prepare submission/presentation.
- Modify distribution copies under `plugins/qiongli/skills/qiongli-workflow/` and `packages/npm-qiongli/` through existing sync scripts after root changes pass.

## Academic Policy

V2 must keep the academic framing:

- The interviewer asks one question at a time, but the orchestration policy can schedule a short ordered set.
- The answered artifact is authoritative unless a task declares a `revisit_trigger`.
- A downstream agent may narrow claims, evidence thresholds, or scope, but must not broaden them without a new boundary review entry.
- Boundary review is not a transcript. It records scholarly decisions: construct, claim strength, evidence threshold, method validity, rival explanation, generalizability, ethics/governance, venue/reviewer, code/data, and submission/presentation commitments.
- After the user answers, the next task continues with the locked boundaries by loading `RESEARCH/[topic]/context/boundary_review.md` into the task packet and prompt.

## Stage Coverage Matrix

| Stage | Checkpoint tasks | Default level | Main boundary dimensions |
| --- | --- | --- | --- |
| A framing | A1, A2, A5 | L2 | phenomenon, construct, contribution, claim strength, evidence threshold, venue/reviewer |
| B literature | B1, B4, B6 | L2 | literature corpus, search boundary, inclusion/exclusion, rival explanation, evidence threshold |
| C study design | C1, C3, C5 | L3 | method validity, construct operationalization, identification/trustworthiness, ethics, preregistration |
| D ethics | D1, D2, D3 | L3 | consent, privacy, governance, vulnerable groups, data sharing, dual use |
| E synthesis | E1, E4, E5 | L3 | pooling boundary, heterogeneity, publication bias, certainty, generalizability |
| F writing | F1, F3, F4, F6 | L2 | claim strength, claim-evidence map, rival explanation, limitation, implication |
| G compliance | G1, G2, G3 | L2 | reporting checklist, PRISMA/CONSORT/STROBE fit, claim-evidence verification |
| H submission/rebuttal | H1, H2, H4 | L3 | reviewer promise, fatal flaw, revision scope, disclosure, response credibility |
| I research code | I1, I3, I5, I8 | L3 | data lineage, analysis validity, reproducibility, specification, code review |
| J proofread | J1, J3, J4 | L1 | meaning preservation, similarity/AI-detection risk, final claim wording |
| K presentation | K1, K2, K4 | L1 | audience promise, slide evidence boundary, oral claim strength, disclosure |

---

### Task 1: Full A-K Boundary Contract

**Files:**
- Modify: `tests/test_boundary_interviewer_contract.py`
- Modify: `standards/boundary-review-contract.yaml`

- [ ] **Step 1: Add a failing test for full-stage policy**

Add this test method to `tests/test_boundary_interviewer_contract.py`:

```python
    def test_v2_contract_covers_all_academic_stages(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        stage_policy = contract["stage_boundary_policy"]

        self.assertEqual(set(stage_policy), set("ABCDEFGHIJK"))
        for stage, policy in stage_policy.items():
            self.assertIn("checkpoint_tasks", policy, stage)
            self.assertIn("default_level", policy, stage)
            self.assertIn(policy["default_level"], {"L1", "L2", "L3"}, stage)
            self.assertGreaterEqual(len(policy["dimensions"]), 2, stage)
            self.assertGreaterEqual(len(policy["questions"]), 2, stage)

        self.assertIn("B1", stage_policy["B"]["checkpoint_tasks"])
        self.assertIn("D1", stage_policy["D"]["checkpoint_tasks"])
        self.assertIn("E5", stage_policy["E"]["checkpoint_tasks"])
        self.assertIn("G3", stage_policy["G"]["checkpoint_tasks"])
        self.assertIn("J4", stage_policy["J"]["checkpoint_tasks"])
        self.assertIn("K4", stage_policy["K"]["checkpoint_tasks"])
```

- [ ] **Step 2: Run the failing contract test**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: FAIL because `stage_boundary_policy` is not complete for A-K yet.

- [ ] **Step 3: Add full-stage policy to the contract**

Modify `standards/boundary-review-contract.yaml` by replacing `mvp_trigger_stages` / `future_trigger_stages` stage intent with this V2 policy block while leaving the existing MVP keys for backward compatibility:

```yaml
stage_boundary_policy:
  A:
    default_level: "L2"
    checkpoint_tasks: ["A1", "A2", "A5"]
    dimensions: ["phenomenon_boundary", "construct_boundary", "contribution_boundary", "claim_strength_boundary", "evidence_threshold_boundary", "venue_reviewer_boundary"]
    questions:
      - "What evidence would make this research question answerable in one paper?"
      - "Which population, context, time period, or corpus is explicitly out of scope?"
      - "Which contribution type is being claimed, and which adjacent contribution is excluded?"
  B:
    default_level: "L2"
    checkpoint_tasks: ["B1", "B4", "B6"]
    dimensions: ["phenomenon_boundary", "evidence_threshold_boundary", "rival_explanation_boundary", "generalizability_boundary", "venue_reviewer_boundary"]
    questions:
      - "Which search boundary would a systematic-review reader challenge first?"
      - "Which contrary literature must be included before the gap can be trusted?"
      - "What inclusion or exclusion rule would change the direction of the synthesis?"
  C:
    default_level: "L3"
    checkpoint_tasks: ["C1", "C3", "C5"]
    dimensions: ["construct_boundary", "claim_strength_boundary", "evidence_threshold_boundary", "method_validity_boundary", "rival_explanation_boundary", "ethics_governance_boundary"]
    questions:
      - "What rival explanation would make the preferred design insufficient?"
      - "Which claim type can this design support without overstating causality or generality?"
      - "What evidence would force the design to narrow the research question?"
  D:
    default_level: "L3"
    checkpoint_tasks: ["D1", "D2", "D3"]
    dimensions: ["ethics_governance_boundary", "generalizability_boundary", "method_validity_boundary", "submission_revision_boundary"]
    questions:
      - "What consent, privacy, or governance boundary limits what can be collected or shared?"
      - "Which participant or data-subject risk remains even if identifiers are removed?"
      - "What disclosure must appear in ethics, data management, or submission materials?"
  E:
    default_level: "L3"
    checkpoint_tasks: ["E1", "E4", "E5"]
    dimensions: ["evidence_threshold_boundary", "method_validity_boundary", "rival_explanation_boundary", "generalizability_boundary", "claim_strength_boundary"]
    questions:
      - "What heterogeneity boundary makes pooling or synthesis scientifically invalid?"
      - "Which publication-bias or certainty-grading result would force a weaker conclusion?"
      - "What population, intervention, context, or outcome boundary limits the synthesis?"
  F:
    default_level: "L2"
    checkpoint_tasks: ["F1", "F3", "F4", "F6"]
    dimensions: ["claim_strength_boundary", "evidence_threshold_boundary", "rival_explanation_boundary", "generalizability_boundary", "venue_reviewer_boundary"]
    questions:
      - "Which central claim would a reviewer say exceeds the available evidence?"
      - "Which boundary condition belongs in the discussion rather than being hidden in limitations?"
      - "What finding, interpretation, or implication must be separated to avoid overclaiming?"
  G:
    default_level: "L2"
    checkpoint_tasks: ["G1", "G2", "G3"]
    dimensions: ["evidence_threshold_boundary", "venue_reviewer_boundary", "submission_revision_boundary", "ethics_governance_boundary"]
    questions:
      - "Which reporting requirement changes the evidence or disclosure boundary?"
      - "Which claim-evidence mismatch would block compliance sign-off?"
      - "What checklist item reveals a boundary that the manuscript has not stated?"
  H:
    default_level: "L3"
    checkpoint_tasks: ["H1", "H2", "H4"]
    dimensions: ["submission_revision_boundary", "venue_reviewer_boundary", "claim_strength_boundary", "evidence_threshold_boundary", "ethics_governance_boundary"]
    questions:
      - "What promise in the cover letter or rebuttal cannot be truthfully supported?"
      - "Which reviewer concern requires narrowing a claim instead of adding rhetoric?"
      - "What fatal flaw should be disclosed, corrected, or explicitly bounded?"
  I:
    default_level: "L3"
    checkpoint_tasks: ["I1", "I3", "I5", "I8"]
    dimensions: ["research_code_boundary", "method_validity_boundary", "evidence_threshold_boundary", "ethics_governance_boundary", "generalizability_boundary"]
    questions:
      - "Which code or data decision would change the scientific interpretation of the results?"
      - "What data lineage, split, seed, exclusion, or transformation must be locked before implementation?"
      - "Which reproducibility failure would force a weaker methods or results claim?"
  J:
    default_level: "L1"
    checkpoint_tasks: ["J1", "J3", "J4"]
    dimensions: ["claim_strength_boundary", "submission_revision_boundary", "venue_reviewer_boundary"]
    questions:
      - "Which wording change would alter the scientific meaning rather than merely improve style?"
      - "Which final-proofread change risks broadening a claim beyond the locked boundary?"
  K:
    default_level: "L1"
    checkpoint_tasks: ["K1", "K2", "K4"]
    dimensions: ["claim_strength_boundary", "evidence_threshold_boundary", "venue_reviewer_boundary", "submission_revision_boundary"]
    questions:
      - "What oral or slide claim must be narrower than the manuscript claim?"
      - "Which evidence limitation must be visible to the presentation audience?"
```

- [ ] **Step 4: Re-run the contract test**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: PASS.

- [ ] **Step 5: Commit the contract expansion**

Run:

```bash
git add tests/test_boundary_interviewer_contract.py standards/boundary-review-contract.yaml
git commit -m "feat: expand boundary contract across academic stages"
```

Expected: commit succeeds with only the contract and its test staged.

---

### Task 2: Boundary Question Engine

**Files:**
- Create: `bridges/boundary_questions.py`
- Create: `tests/test_boundary_questions.py`

- [ ] **Step 1: Write failing question-engine tests**

Create `tests/test_boundary_questions.py`:

```python
from __future__ import annotations

import unittest

from bridges.boundary_questions import (
    BoundaryQuestionPlan,
    build_boundary_question_plan,
    format_boundary_prompt_section,
    get_boundary_questions,
)


class BoundaryQuestionsTests(unittest.TestCase):
    def test_every_stage_has_policy(self) -> None:
        for stage in "ABCDEFGHIJK":
            plan = build_boundary_question_plan(task_id=f"{stage}1", required_skills=[])
            self.assertEqual(plan.stage, stage)
            self.assertIn(plan.level, {"L1", "L2", "L3"})
            self.assertGreaterEqual(len(plan.questions), 1)

    def test_checkpoint_task_requires_boundary_artifact(self) -> None:
        plan = build_boundary_question_plan(task_id="C1", required_skills=["study-designer"])
        self.assertTrue(plan.enabled)
        self.assertTrue(plan.required_before_draft)
        self.assertEqual(plan.level, "L3")
        self.assertIn("method_validity_boundary", plan.dimensions)

    def test_non_checkpoint_task_uses_l0_when_boundary_exists(self) -> None:
        existing = "# Boundary Review\n\n- locked_decision: Study is descriptive only.\n"
        plan = build_boundary_question_plan(
            task_id="C2",
            required_skills=["instrument-designer"],
            existing_boundary_review=existing,
        )
        self.assertTrue(plan.enabled)
        self.assertFalse(plan.required_before_draft)
        self.assertEqual(plan.level, "L0")
        self.assertEqual(plan.status, "answered")

    def test_question_order_is_stage_specific(self) -> None:
        questions = get_boundary_questions("B1", max_questions=2)
        self.assertEqual(len(questions), 2)
        self.assertIn("search boundary", questions[0])

    def test_prompt_section_includes_locked_answers(self) -> None:
        plan = BoundaryQuestionPlan(
            enabled=True,
            status="answered",
            task_id="F3",
            stage="F",
            level="L2",
            artifact="context/boundary_review.md",
            required_before_draft=True,
            dimensions=["claim_strength_boundary"],
            questions=["Which central claim would a reviewer say exceeds the available evidence?"],
            reason="checkpoint task",
        )
        rendered = format_boundary_prompt_section(plan, "- locked_decision: Claims are associative.")
        self.assertIn("Academic boundary review", rendered)
        self.assertIn("Claims are associative", rendered)
        self.assertIn("must not broaden", rendered)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing question-engine tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_questions -v
```

Expected: FAIL because `bridges.boundary_questions` does not exist.

- [ ] **Step 3: Implement the question engine**

Create `bridges/boundary_questions.py`:

```python
from __future__ import annotations

from dataclasses import dataclass


BOUNDARY_ARTIFACT = "context/boundary_review.md"


@dataclass(frozen=True)
class BoundaryQuestionPlan:
    enabled: bool
    status: str
    task_id: str
    stage: str
    level: str
    artifact: str
    required_before_draft: bool
    dimensions: list[str]
    questions: list[str]
    reason: str


STAGE_BOUNDARY_POLICY: dict[str, dict[str, object]] = {
    "A": {
        "level": "L2",
        "checkpoint_tasks": ["A1", "A2", "A5"],
        "dimensions": [
            "phenomenon_boundary",
            "construct_boundary",
            "contribution_boundary",
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "venue_reviewer_boundary",
        ],
        "questions": [
            "What evidence would make this research question answerable in one paper?",
            "Which population, context, time period, or corpus is explicitly out of scope?",
            "Which contribution type is being claimed, and which adjacent contribution is excluded?",
        ],
    },
    "B": {
        "level": "L2",
        "checkpoint_tasks": ["B1", "B4", "B6"],
        "dimensions": [
            "phenomenon_boundary",
            "evidence_threshold_boundary",
            "rival_explanation_boundary",
            "generalizability_boundary",
            "venue_reviewer_boundary",
        ],
        "questions": [
            "Which search boundary would a systematic-review reader challenge first?",
            "Which contrary literature must be included before the gap can be trusted?",
            "What inclusion or exclusion rule would change the direction of the synthesis?",
        ],
    },
    "C": {
        "level": "L3",
        "checkpoint_tasks": ["C1", "C3", "C5"],
        "dimensions": [
            "construct_boundary",
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "method_validity_boundary",
            "rival_explanation_boundary",
            "ethics_governance_boundary",
        ],
        "questions": [
            "What rival explanation would make the preferred design insufficient?",
            "Which claim type can this design support without overstating causality or generality?",
            "What evidence would force the design to narrow the research question?",
        ],
    },
    "D": {
        "level": "L3",
        "checkpoint_tasks": ["D1", "D2", "D3"],
        "dimensions": [
            "ethics_governance_boundary",
            "generalizability_boundary",
            "method_validity_boundary",
            "submission_revision_boundary",
        ],
        "questions": [
            "What consent, privacy, or governance boundary limits what can be collected or shared?",
            "Which participant or data-subject risk remains even if identifiers are removed?",
            "What disclosure must appear in ethics, data management, or submission materials?",
        ],
    },
    "E": {
        "level": "L3",
        "checkpoint_tasks": ["E1", "E4", "E5"],
        "dimensions": [
            "evidence_threshold_boundary",
            "method_validity_boundary",
            "rival_explanation_boundary",
            "generalizability_boundary",
            "claim_strength_boundary",
        ],
        "questions": [
            "What heterogeneity boundary makes pooling or synthesis scientifically invalid?",
            "Which publication-bias or certainty-grading result would force a weaker conclusion?",
            "What population, intervention, context, or outcome boundary limits the synthesis?",
        ],
    },
    "F": {
        "level": "L2",
        "checkpoint_tasks": ["F1", "F3", "F4", "F6"],
        "dimensions": [
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "rival_explanation_boundary",
            "generalizability_boundary",
            "venue_reviewer_boundary",
        ],
        "questions": [
            "Which central claim would a reviewer say exceeds the available evidence?",
            "Which boundary condition belongs in the discussion rather than being hidden in limitations?",
            "What finding, interpretation, or implication must be separated to avoid overclaiming?",
        ],
    },
    "G": {
        "level": "L2",
        "checkpoint_tasks": ["G1", "G2", "G3"],
        "dimensions": [
            "evidence_threshold_boundary",
            "venue_reviewer_boundary",
            "submission_revision_boundary",
            "ethics_governance_boundary",
        ],
        "questions": [
            "Which reporting requirement changes the evidence or disclosure boundary?",
            "Which claim-evidence mismatch would block compliance sign-off?",
            "What checklist item reveals a boundary that the manuscript has not stated?",
        ],
    },
    "H": {
        "level": "L3",
        "checkpoint_tasks": ["H1", "H2", "H4"],
        "dimensions": [
            "submission_revision_boundary",
            "venue_reviewer_boundary",
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "ethics_governance_boundary",
        ],
        "questions": [
            "What promise in the cover letter or rebuttal cannot be truthfully supported?",
            "Which reviewer concern requires narrowing a claim instead of adding rhetoric?",
            "What fatal flaw should be disclosed, corrected, or explicitly bounded?",
        ],
    },
    "I": {
        "level": "L3",
        "checkpoint_tasks": ["I1", "I3", "I5", "I8"],
        "dimensions": [
            "research_code_boundary",
            "method_validity_boundary",
            "evidence_threshold_boundary",
            "ethics_governance_boundary",
            "generalizability_boundary",
        ],
        "questions": [
            "Which code or data decision would change the scientific interpretation of the results?",
            "What data lineage, split, seed, exclusion, or transformation must be locked before implementation?",
            "Which reproducibility failure would force a weaker methods or results claim?",
        ],
    },
    "J": {
        "level": "L1",
        "checkpoint_tasks": ["J1", "J3", "J4"],
        "dimensions": [
            "claim_strength_boundary",
            "submission_revision_boundary",
            "venue_reviewer_boundary",
        ],
        "questions": [
            "Which wording change would alter the scientific meaning rather than merely improve style?",
            "Which final-proofread change risks broadening a claim beyond the locked boundary?",
        ],
    },
    "K": {
        "level": "L1",
        "checkpoint_tasks": ["K1", "K2", "K4"],
        "dimensions": [
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "venue_reviewer_boundary",
            "submission_revision_boundary",
        ],
        "questions": [
            "What oral or slide claim must be narrower than the manuscript claim?",
            "Which evidence limitation must be visible to the presentation audience?",
        ],
    },
}


def get_stage_for_task(task_id: str) -> str | None:
    normalized = task_id.strip().upper()
    if not normalized:
        return None
    stage = normalized[:1]
    return stage if stage in STAGE_BOUNDARY_POLICY else None


def get_boundary_questions(task_id: str, max_questions: int | None = None) -> list[str]:
    stage = get_stage_for_task(task_id)
    if not stage:
        return []
    questions = list(STAGE_BOUNDARY_POLICY[stage]["questions"])
    if max_questions is None:
        return questions
    return questions[: max(0, max_questions)]


def build_boundary_question_plan(
    task_id: str,
    required_skills: list[str],
    existing_boundary_review: str = "",
    forced: bool = False,
) -> BoundaryQuestionPlan:
    normalized_task = task_id.strip().upper()
    stage = get_stage_for_task(normalized_task)
    if not stage:
        return BoundaryQuestionPlan(
            enabled=False,
            status="unsupported",
            task_id=normalized_task,
            stage="",
            level="off",
            artifact=BOUNDARY_ARTIFACT,
            required_before_draft=False,
            dimensions=[],
            questions=[],
            reason="task stage has no academic boundary policy",
        )

    policy = STAGE_BOUNDARY_POLICY[stage]
    checkpoint_tasks = {str(item).upper() for item in policy["checkpoint_tasks"]}
    has_answer = bool(existing_boundary_review.strip())
    is_checkpoint = normalized_task in checkpoint_tasks
    required = forced or is_checkpoint
    status = "answered" if has_answer else ("required" if required else "recommended")
    level = "L0" if has_answer and not required else str(policy["level"])
    question_count = 1 if level == "L1" else (5 if level == "L3" else 3)
    reason = (
        "existing boundary artifact loaded"
        if has_answer and not required
        else ("checkpoint task" if is_checkpoint else "stage policy recommendation")
    )

    return BoundaryQuestionPlan(
        enabled=True,
        status=status,
        task_id=normalized_task,
        stage=stage,
        level=level,
        artifact=BOUNDARY_ARTIFACT,
        required_before_draft=required and not has_answer,
        dimensions=list(policy["dimensions"]),
        questions=get_boundary_questions(normalized_task, question_count),
        reason=reason,
    )


def format_boundary_prompt_section(
    plan: BoundaryQuestionPlan,
    existing_boundary_review: str = "",
) -> str:
    if not plan.enabled:
        return ""
    question_lines = "\n".join(f"- {question}" for question in plan.questions)
    existing = existing_boundary_review.strip() or "No answered boundary review artifact was found."
    return f"""Academic boundary review:
- artifact: {plan.artifact}
- status: {plan.status}
- level: {plan.level}
- dimensions: {", ".join(plan.dimensions)}
- required_before_draft: {str(plan.required_before_draft).lower()}
- reason: {plan.reason}

Boundary questions:
{question_lines}

Answered boundary review:
{existing}

Boundary continuation rule:
- Continue the task within the answered boundaries.
- You may narrow a claim, scope, method, evidence threshold, code decision, submission promise, or presentation claim.
- You must not broaden the locked boundary without creating a new boundary review entry and naming the revisit trigger.
"""
```

- [ ] **Step 4: Run the question-engine tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_questions -v
```

Expected: PASS.

- [ ] **Step 5: Commit the question engine**

Run:

```bash
git add bridges/boundary_questions.py tests/test_boundary_questions.py
git commit -m "feat: add academic boundary question engine"
```

Expected: commit succeeds.

---

### Task 3: Orchestrator Task Packet And Prompt Injection

**Files:**
- Modify: `bridges/orchestrator.py`
- Modify: `tests/test_orchestrator_workflows.py`

- [ ] **Step 1: Add failing orchestrator tests**

Add these tests to `tests/test_orchestrator_workflows.py`:

```python
    def test_task_packet_includes_boundary_review_plan(self) -> None:
        orchestrator = MockOrchestrator()
        packet = orchestrator._build_task_packet(
            task_id="B1",
            paper_type="systematic-review",
            topic="ai-literature",
            venue=None,
            artifact_root="RESEARCH/[topic]/",
            required_outputs=["literature/search_strategy.md"],
            contract_required_outputs=["literature/search_strategy.md"],
            deferred_outputs=[],
            required_mcp=["filesystem"],
            required_skills=["literature-search-planner"],
            required_skill_cards=[],
            quality_gates=["Q2"],
            artifact_policy="contract",
            research_depth="standard",
            evidence_expansion_rounds=1,
            boundary_review={
                "enabled": True,
                "status": "required",
                "task_id": "B1",
                "stage": "B",
                "level": "L2",
                "artifact": "context/boundary_review.md",
                "required_before_draft": True,
                "dimensions": ["evidence_threshold_boundary"],
                "questions": ["Which search boundary would a systematic-review reader challenge first?"],
                "existing_review": "",
            },
        )

        self.assertTrue(packet["boundary_review"]["enabled"])
        self.assertEqual(packet["boundary_review"]["stage"], "B")
        self.assertIn("context/boundary_review.md", packet["required_outputs"])

    def test_draft_prompt_injects_answered_boundary_review(self) -> None:
        orchestrator = MockOrchestrator()
        packet = {
            "task_id": "F3",
            "required_outputs": ["manuscript/main.md"],
            "deferred_outputs": [],
            "artifact_policy": "contract",
            "research_depth": "standard",
            "evidence_expansion_rounds": 1,
            "required_skills": ["manuscript-architect"],
            "required_skill_cards": [],
            "quality_gates": ["Q2"],
            "boundary_review": {
                "enabled": True,
                "status": "answered",
                "task_id": "F3",
                "stage": "F",
                "level": "L2",
                "artifact": "context/boundary_review.md",
                "required_before_draft": False,
                "dimensions": ["claim_strength_boundary"],
                "questions": ["Which central claim would a reviewer say exceeds the available evidence?"],
                "existing_review": "- locked_decision: Claims are associative, not causal.",
            },
        }
        prompt = orchestrator._build_task_draft_prompt(
            packet,
            [MCPEvidence(provider="filesystem", status="ok", summary="mock evidence")],
            [],
            None,
        )

        self.assertIn("Academic boundary review", prompt)
        self.assertIn("Claims are associative, not causal", prompt)
        self.assertIn("must not broaden", prompt)

    def test_review_prompt_blocks_boundary_broadening(self) -> None:
        orchestrator = MockOrchestrator()
        packet = {
            "task_id": "F3",
            "required_outputs": ["manuscript/main.md"],
            "deferred_outputs": [],
            "research_depth": "standard",
            "boundary_review": {
                "enabled": True,
                "status": "answered",
                "task_id": "F3",
                "stage": "F",
                "level": "L2",
                "artifact": "context/boundary_review.md",
                "required_before_draft": False,
                "dimensions": ["claim_strength_boundary"],
                "questions": ["Which central claim would a reviewer say exceeds the available evidence?"],
                "existing_review": "- locked_decision: Claims are associative, not causal.",
            },
        }
        prompt = orchestrator._build_task_review_prompt(
            packet,
            [MCPEvidence(provider="filesystem", status="ok", summary="mock evidence")],
            [],
            "Draft claims the intervention caused the outcome.",
        )

        self.assertIn("Block if the draft broadens", prompt)
        self.assertIn("Claims are associative, not causal", prompt)
```

- [ ] **Step 2: Run the failing orchestrator tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_orchestrator_workflows -v
```

Expected: FAIL because `_build_task_packet` has no `boundary_review` argument and prompts do not inject boundary sections.

- [ ] **Step 3: Import boundary helpers**

Modify the imports near the top of `bridges/orchestrator.py`:

```python
from .boundary_questions import (
    BOUNDARY_ARTIFACT,
    build_boundary_question_plan,
    format_boundary_prompt_section,
)
```

- [ ] **Step 4: Add boundary artifact loading**

Add these methods near `_project_root_for_topic` in `bridges/orchestrator.py`:

```python
    def _load_boundary_review_context(
        self,
        cwd: Path,
        artifact_root: str,
        topic: str,
    ) -> str:
        project_root = self._project_root_for_topic(cwd, artifact_root, topic)
        boundary_path = project_root / BOUNDARY_ARTIFACT
        if not boundary_path.is_file():
            return ""
        return boundary_path.read_text(encoding="utf-8")

    def _build_boundary_review_packet(
        self,
        task_id: str,
        required_skills: list[str],
        existing_boundary_review: str,
    ) -> dict[str, Any]:
        plan = build_boundary_question_plan(
            task_id=task_id,
            required_skills=required_skills,
            existing_boundary_review=existing_boundary_review,
        )
        return {
            "enabled": plan.enabled,
            "status": plan.status,
            "task_id": plan.task_id,
            "stage": plan.stage,
            "level": plan.level,
            "artifact": plan.artifact,
            "required_before_draft": plan.required_before_draft,
            "dimensions": list(plan.dimensions),
            "questions": list(plan.questions),
            "reason": plan.reason,
            "existing_review": existing_boundary_review.strip(),
        }

    def _format_boundary_review_context(self, task_packet: dict[str, Any]) -> str:
        boundary_review = task_packet.get("boundary_review", {})
        if not isinstance(boundary_review, dict) or not boundary_review.get("enabled"):
            return ""
        plan = build_boundary_question_plan(
            task_id=str(boundary_review.get("task_id", task_packet.get("task_id", ""))),
            required_skills=[
                str(item) for item in task_packet.get("required_skills", []) if str(item).strip()
            ],
            existing_boundary_review=str(boundary_review.get("existing_review", "")),
            forced=bool(boundary_review.get("required_before_draft")),
        )
        return format_boundary_prompt_section(
            plan,
            str(boundary_review.get("existing_review", "")),
        )
```

- [ ] **Step 5: Extend `_build_task_packet`**

Change the `_build_task_packet` signature and returned dictionary:

```python
        academic_context_update: dict[str, Any] | None = None,
        boundary_review: dict[str, Any] | None = None,
```

Add this field to the returned packet:

```python
            "boundary_review": dict(boundary_review or {}),
```

- [ ] **Step 6: Build boundary packet during `task_run`**

In `task_run`, after `artifact_root, contract_outputs = self._load_task_outputs(normalized_task)`, add:

```python
        existing_boundary_review = self._load_boundary_review_context(
            cwd,
            artifact_root,
            normalized_topic,
        )
        boundary_review = self._build_boundary_review_packet(
            normalized_task,
            agent_plan["required_skills"],
            existing_boundary_review,
        )
```

Before `_build_task_packet`, append the boundary artifact when required:

```python
        if boundary_review.get("required_before_draft"):
            contract_outputs, required_outputs, deferred_outputs = self._append_optional_outputs(
                contract_outputs,
                required_outputs,
                deferred_outputs,
                [str(boundary_review["artifact"])],
            )
```

Pass `boundary_review=boundary_review` into `_build_task_packet`.

- [ ] **Step 7: Inject boundary review into draft prompt**

Inside `_build_task_draft_prompt`, after `targeted_follow_up_section` is calculated, add:

```python
        boundary_section = self._format_boundary_review_context(task_packet)
        boundary_rules = ""
        if boundary_section:
            return_sections.append("- Academic Boundary Review")
            boundary_rules = """
22. Academic boundary review is active.
23. If no answered boundary review exists and required_before_draft is true, ask exactly the first listed boundary question and write the answer to `context/boundary_review.md` before producing broader outputs.
24. If an answered boundary review exists, continue within it and do not broaden claim strength, evidence threshold, population, corpus, method, code/data decision, submission promise, or presentation claim without a new boundary review entry.
"""
```

Add `{boundary_rules}` after `{self_critique_rules}` in the execution rules block, and add `{boundary_section}` before `Additional context:`.

- [ ] **Step 8: Inject boundary review into review and revision prompts**

In `_build_task_review_prompt`, add:

```python
        boundary_section = self._format_boundary_review_context(task_packet)
        boundary_review_rule = ""
        if boundary_section:
            return_sections.append("- Boundary Compliance")
            boundary_review_rule = """
14. Boundary review is active. Block if the draft broadens the locked boundary, upgrades claim strength, lowers evidence threshold, hides a limitation, or makes a code/data/submission/presentation promise beyond the answered boundary.
"""
```

Include `{boundary_section}` in the context area and `{boundary_review_rule}` in the checklist.

In `_build_task_revision_prompt`, include the same boundary section and a rule that revisions must resolve boundary violations by narrowing the output or creating a new boundary review entry.

- [ ] **Step 9: Add routing notes and result data**

In `task_run`, after self-critique routing notes, add:

```python
        if boundary_review.get("enabled"):
            routing_notes.append(
                "Boundary review hook ACTIVE: "
                f"artifact={boundary_review.get('artifact')}, "
                f"status={boundary_review.get('status')}, "
                f"level={boundary_review.get('level')}, "
                f"required_before_draft={boundary_review.get('required_before_draft')}."
            )
            if boundary_review.get("questions"):
                routing_notes.append(
                    "Boundary review first question: "
                    + str(boundary_review["questions"][0])
                )
```

Add `"boundary_review": dict(boundary_review),` to `CollaborationResult.data`.

- [ ] **Step 10: Run orchestrator tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_questions tests.test_orchestrator_workflows -v
```

Expected: PASS.

- [ ] **Step 11: Commit orchestration support**

Run:

```bash
git add bridges/orchestrator.py tests/test_orchestrator_workflows.py
git commit -m "feat: inject academic boundary reviews into task runs"
```

Expected: commit succeeds.

---

### Task 4: Context Package Propagation

**Files:**
- Modify: `bridges/context_package.py`
- Modify: `tests/test_context_package_builder.py`

- [ ] **Step 1: Add failing context package test**

Add this test to `tests/test_context_package_builder.py`:

```python
    def test_context_package_includes_boundary_review_for_all_agents(self) -> None:
        task_packet = {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "ai-writing",
            "boundary_review": {
                "artifact": "context/boundary_review.md",
                "status": "answered",
                "existing_review": "- locked_decision: Claims are associative, not causal.",
            },
        }

        package = build_context_package(task_packet, controller="codex", agents=["claude", "gemini"])

        self.assertIn("Claims are associative, not causal", package["agent_contexts"]["codex"])
        self.assertIn("Claims are associative, not causal", package["agent_contexts"]["claude"])
        self.assertIn("Claims are associative, not causal", package["agent_contexts"]["gemini"])
```

- [ ] **Step 2: Run the failing context package test**

Run:

```bash
.venv/bin/python -m unittest tests.test_context_package_builder -v
```

Expected: FAIL because boundary review content is not rendered into agent contexts.

- [ ] **Step 3: Add a formatter in `bridges/context_package.py`**

Add:

```python
def _boundary_review_text(task_packet: dict[str, object]) -> str:
    boundary = task_packet.get("boundary_review", {})
    if not isinstance(boundary, dict):
        return "Not provided"
    existing = str(boundary.get("existing_review", "")).strip()
    if existing:
        return existing
    status = str(boundary.get("status", "")).strip()
    artifact = str(boundary.get("artifact", "")).strip()
    if status or artifact:
        return f"status: {status or 'unknown'}\nartifact: {artifact or 'context/boundary_review.md'}"
    return "Not provided"
```

- [ ] **Step 4: Render boundary review in each agent context**

Add this section to `_build_codex_context`, `_build_claude_context`, and `_build_gemini_context`:

```python
            f"## Boundary Review\n{_boundary_review_text(task_packet)}",
```

- [ ] **Step 5: Run context package tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_context_package_builder -v
```

Expected: PASS.

- [ ] **Step 6: Commit context package propagation**

Run:

```bash
git add bridges/context_package.py tests/test_context_package_builder.py
git commit -m "feat: propagate boundary reviews in context packages"
```

Expected: commit succeeds.

---

### Task 5: Academic Context Continuity Integration

**Files:**
- Modify: `standards/research-workflow-contract.yaml`
- Modify: `skills/Z_cross_cutting/academic-context-maintainer.md`
- Modify: `tests/test_academic_context_continuity.py`

- [ ] **Step 1: Add failing continuity tests**

Add these checks to `tests/test_academic_context_continuity.py`:

```python
    def test_boundary_review_is_academic_context_artifact(self) -> None:
        content = (REPO_ROOT / "standards" / "research-workflow-contract.yaml").read_text(
            encoding="utf-8"
        )
        self.assertIn('"context/boundary_review.md"', content)
        self.assertIn("boundary_review_required_sections:", content)
        self.assertIn("claim_strength_boundary", content)
        self.assertIn("revisit_trigger", content)

    def test_context_maintainer_consumes_boundary_review(self) -> None:
        content = (
            REPO_ROOT / "skills" / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")
        self.assertIn("context/boundary_review.md", content)
        self.assertIn("Do not broaden locked boundaries", content)
        self.assertIn("research_state.md", content)
        self.assertIn("decision_log.md", content)
        self.assertIn("stage_handoff.md", content)
```

- [ ] **Step 2: Run the failing continuity tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_academic_context_continuity -v
```

Expected: FAIL until the contract and skill mention boundary review continuity.

- [ ] **Step 3: Extend `academic_context_continuity` contract**

In `standards/research-workflow-contract.yaml`, add `context/boundary_review.md` to `academic_context_continuity.artifacts`, and add:

```yaml
  boundary_review_required_sections:
    - "academic boundary map"
    - "claim strength and evidence threshold"
    - "rival explanations and validity risks"
    - "generalizability and venue limits"
    - "research code, submission, or presentation commitments when relevant"
    - "locked decisions and revisit triggers"
  boundary_refresh_points:
    A: "Refresh phenomenon, construct, contribution, claim strength, and evidence threshold boundaries."
    B: "Refresh literature corpus, inclusion/exclusion, and rival-literature boundaries."
    C: "Refresh method-validity, operationalization, and claim-strength boundaries."
    D: "Refresh consent, privacy, governance, and disclosure boundaries."
    E: "Refresh synthesis, pooling, heterogeneity, and certainty boundaries."
    F: "Refresh claim-evidence, interpretation, implication, and limitation boundaries."
    G: "Refresh reporting, checklist, and compliance boundaries."
    H: "Refresh submission, rebuttal, reviewer-promise, and fatal-flaw boundaries."
    I: "Refresh code/data lineage, reproducibility, and analysis-validity boundaries."
    J: "Refresh final wording and meaning-preservation boundaries."
    K: "Refresh presentation claim, audience, and disclosure boundaries."
```

- [ ] **Step 4: Update `academic-context-maintainer`**

Add this section to `skills/Z_cross_cutting/academic-context-maintainer.md`:

```markdown
## Boundary Review Continuity

When `context/boundary_review.md` exists, treat it as a project-level academic constraint, not as optional notes.

- Do not broaden locked boundaries in `research_state.md`, `decision_log.md`, or `stage_handoff.md`.
- If later evidence narrows a boundary, update the boundary review and record the downstream consequence.
- If later work needs to broaden a boundary, create a new boundary review entry with the triggering evidence, affected claim, and required downstream updates.
- Preserve `claim_strength_boundary`, `evidence_threshold_boundary`, `method_validity_boundary`, `generalizability_boundary`, `research_code_boundary`, `submission_revision_boundary`, and `revisit_trigger` fields when refreshing context artifacts.
```

- [ ] **Step 5: Run continuity tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_academic_context_continuity -v
```

Expected: PASS.

- [ ] **Step 6: Commit continuity integration**

Run:

```bash
git add standards/research-workflow-contract.yaml skills/Z_cross_cutting/academic-context-maintainer.md tests/test_academic_context_continuity.py
git commit -m "feat: connect boundary reviews to academic context continuity"
```

Expected: commit succeeds.

---

### Task 6: Boundary Validator Gate

**Files:**
- Modify: `scripts/validate_research_standard.py`
- Modify: `tests/test_research_standard_validator.py`

- [ ] **Step 1: Add failing validator tests**

Add a focused test to `tests/test_research_standard_validator.py` using the existing validator helper style in that file:

```python
    def test_boundary_review_gate_blocks_missing_required_sections(self) -> None:
        project = self._make_project(
            {
                "manuscript/main.md": "The intervention proves a causal effect.",
                "context/boundary_review.md": "# Boundary Review\n\n- locked_decision: Claims are associative only.\n",
            }
        )

        result = self._run_validator(project)

        self.assertIn("boundary_review", result.stdout)
        self.assertIn("claim_strength_boundary", result.stdout)
        self.assertNotEqual(result.returncode, 0)

    def test_boundary_review_gate_passes_when_claims_are_within_boundary(self) -> None:
        project = self._make_project(
            {
                "manuscript/main.md": "The evidence suggests an associative relationship.",
                "context/boundary_review.md": "\n".join(
                    [
                        "# Boundary Review",
                        "## Claim Strength And Evidence Threshold",
                        "- claim_strength_boundary: associative, not causal",
                        "- evidence_threshold_boundary: triangulated observational evidence",
                        "## Locked Decisions And Revisit Triggers",
                        "- locked_decision: Do not use causal language.",
                        "- revisit_trigger: new identification strategy or randomized evidence",
                    ]
                ),
            }
        )

        result = self._run_validator(project)

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
```

- [ ] **Step 2: Run the failing validator tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_research_standard_validator -v
```

Expected: FAIL because the validator does not enforce boundary review sections.

- [ ] **Step 3: Add boundary review validation**

In `scripts/validate_research_standard.py`, add a checker that reads `context/boundary_review.md` when present and validates required academic fields:

```python
BOUNDARY_REVIEW_REQUIRED_MARKERS = (
    "claim_strength_boundary",
    "evidence_threshold_boundary",
    "locked_decision",
    "revisit_trigger",
)


def validate_boundary_review(project_root: Path) -> list[str]:
    boundary_path = project_root / "context" / "boundary_review.md"
    if not boundary_path.is_file():
        return []
    text = boundary_path.read_text(encoding="utf-8").lower()
    issues: list[str] = []
    for marker in BOUNDARY_REVIEW_REQUIRED_MARKERS:
        if marker not in text:
            issues.append(f"boundary_review missing required marker: {marker}")
    manuscript_path = project_root / "manuscript" / "main.md"
    if manuscript_path.is_file() and "associative" in text and "not causal" in text:
        manuscript = manuscript_path.read_text(encoding="utf-8").lower()
        if "proves" in manuscript or "caused" in manuscript or "causal effect" in manuscript:
            issues.append(
                "boundary_review violation: manuscript uses causal wording despite associative boundary"
            )
    return issues
```

Call `validate_boundary_review(project_root)` from the main project validation path and report its issues with the existing issue reporting format.

- [ ] **Step 4: Run validator tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_research_standard_validator -v
```

Expected: PASS.

- [ ] **Step 5: Commit validator gate**

Run:

```bash
git add scripts/validate_research_standard.py tests/test_research_standard_validator.py
git commit -m "feat: validate boundary review claim limits"
```

Expected: commit succeeds.

---

### Task 7: Full Workflow Stage Hooks

**Files:**
- Modify: `qiongli-workflow/workflows/lit-review.md`
- Modify: `qiongli-workflow/workflows/ethics-check.md`
- Modify: `qiongli-workflow/workflows/synthesize.md`
- Modify: `qiongli-workflow/workflows/compliance-check.md`
- Modify: `qiongli-workflow/workflows/proofread.md`
- Modify: `qiongli-workflow/workflows/academic-present.md`
- Modify: `tests/test_boundary_interviewer_contract.py`

- [ ] **Step 1: Add failing workflow coverage test**

Add this test to `tests/test_boundary_interviewer_contract.py`:

```python
    def test_v2_workflows_include_boundary_trigger_for_all_remaining_stages(self) -> None:
        workflow_paths = [
            REPO_ROOT / "qiongli-workflow" / "workflows" / "lit-review.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "ethics-check.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "synthesize.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "compliance-check.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "proofread.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "academic-present.md",
        ]

        for path in workflow_paths:
            content = path.read_text(encoding="utf-8")
            self.assertIn("boundary-interviewer", content, path.as_posix())
            self.assertIn("context/boundary_review.md", content, path.as_posix())
            self.assertIn("locked boundary", content, path.as_posix())
```

- [ ] **Step 2: Run the failing workflow coverage test**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: FAIL until all six remaining workflows contain boundary hooks.

- [ ] **Step 3: Add stage-specific workflow hooks**

Add a short `Academic boundary review` section to each workflow:

```markdown
## Academic Boundary Review

Before drafting this stage's checkpoint outputs, use `boundary-interviewer` when `context/boundary_review.md` is missing, stale, or contradicted by the current task. Continue within the locked boundary when the artifact already answers the stage question. Narrowing is allowed; broadening requires a new boundary review entry with a revisit trigger.
```

Then add one stage-specific sentence:

- `lit-review.md`: "For literature work, lock search boundary, inclusion/exclusion rules, contrary literature, corpus limits, and evidence threshold before synthesis."
- `ethics-check.md`: "For ethics work, lock consent, privacy, governance, vulnerable-group, deidentification, data-sharing, and disclosure boundaries before drafting approvals."
- `synthesize.md`: "For synthesis work, lock pooling, heterogeneity, certainty, publication-bias, and generalizability boundaries before interpreting aggregate evidence."
- `compliance-check.md`: "For compliance work, lock reporting checklist fit, claim-evidence mismatches, and disclosure boundaries before sign-off."
- `proofread.md`: "For proofread work, lock meaning-preservation and final claim wording boundaries before style, similarity, or AI-detection edits."
- `academic-present.md`: "For presentation work, lock audience, slide-evidence, oral-claim, and disclosure boundaries before shortening or dramatizing claims."

- [ ] **Step 4: Run workflow coverage tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: PASS.

- [ ] **Step 5: Commit workflow hooks**

Run:

```bash
git add qiongli-workflow/workflows/lit-review.md qiongli-workflow/workflows/ethics-check.md qiongli-workflow/workflows/synthesize.md qiongli-workflow/workflows/compliance-check.md qiongli-workflow/workflows/proofread.md qiongli-workflow/workflows/academic-present.md tests/test_boundary_interviewer_contract.py
git commit -m "docs: add boundary review hooks to all workflow stages"
```

Expected: commit succeeds.

---

### Task 8: Skill Runtime Documentation

**Files:**
- Modify: `skills/Z_cross_cutting/boundary-interviewer.md`
- Modify: `skills-core.md`
- Modify: `skills-summary.md`
- Modify: `tests/test_boundary_interviewer_contract.py`

- [ ] **Step 1: Add failing documentation behavior test**

Add this test to `tests/test_boundary_interviewer_contract.py`:

```python
    def test_boundary_skill_documents_downstream_continuation(self) -> None:
        content = SKILL.read_text(encoding="utf-8")
        self.assertIn("After the user answers", content)
        self.assertIn("continue within the locked boundary", content)
        self.assertIn("must not broaden", content)
        self.assertIn("revisit_trigger", content)
```

- [ ] **Step 2: Run the failing documentation behavior test**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: FAIL until the skill explains downstream continuation explicitly.

- [ ] **Step 3: Update the boundary-interviewer skill**

Add this section to `skills/Z_cross_cutting/boundary-interviewer.md`:

```markdown
## Downstream Continuation

After the user answers a boundary question, write or update `context/boundary_review.md` and continue within the locked boundary for the current task. Later Qiongli skills must treat that artifact as a constraint:

- They may narrow scope, claim strength, evidence thresholds, method commitments, code/data decisions, or submission/presentation promises.
- They must not broaden a locked boundary without adding a new `revisit_trigger`, the new evidence or user decision that justifies the change, and the affected downstream artifacts.
- When a later task conflicts with the boundary review, ask the smallest necessary follow-up question before continuing.
```

Update `skills-core.md` and `skills-summary.md` so the summary says boundary answers are reused by downstream tasks, not only recorded.

- [ ] **Step 4: Run documentation behavior tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
```

Expected: PASS.

- [ ] **Step 5: Commit skill documentation**

Run:

```bash
git add skills/Z_cross_cutting/boundary-interviewer.md skills-core.md skills-summary.md tests/test_boundary_interviewer_contract.py
git commit -m "docs: document downstream boundary continuation"
```

Expected: commit succeeds.

---

### Task 9: Distribution Sync And Verification

**Files:**
- Modify generated copies under:
  - `plugins/qiongli/skills/qiongli-workflow/`
  - `packages/npm-qiongli/payload/qiongli-workflow/`
  - `packages/npm-qiongli/python-runtime/`
- Test: distribution and contract suites

- [ ] **Step 1: Sync generated package payloads**

Run:

```bash
./scripts/sync_skill_package.sh --target all
uv run python scripts/sync_npm_package_payload.py
```

Expected: root skill, standard, template, workflow, and bridge-adjacent runtime copies are synchronized into plugin and npm payload locations.

- [ ] **Step 2: Run focused verification**

Run:

```bash
.venv/bin/python -m unittest \
  tests.test_boundary_interviewer_contract \
  tests.test_boundary_questions \
  tests.test_orchestrator_workflows \
  tests.test_context_package_builder \
  tests.test_academic_context_continuity \
  tests.test_research_standard_validator \
  tests.test_distribution_payloads \
  tests.test_plugin_distribution_contract \
  tests.test_npm_package_contract \
  tests.test_skill_contract_alignment \
  tests.test_workflow_contract_doc \
  -v
```

Expected: PASS.

- [ ] **Step 3: Inspect generated diff**

Run:

```bash
git status --short
git diff --stat
```

Expected: only V2 boundary files, generated copies of those files, and intended tests are changed.

- [ ] **Step 4: Commit synced distribution**

Run:

```bash
git add \
  bridges/boundary_questions.py \
  bridges/orchestrator.py \
  bridges/context_package.py \
  scripts/validate_research_standard.py \
  standards/boundary-review-contract.yaml \
  standards/research-workflow-contract.yaml \
  skills/Z_cross_cutting/boundary-interviewer.md \
  skills/Z_cross_cutting/academic-context-maintainer.md \
  skills-core.md \
  skills-summary.md \
  qiongli-workflow/workflows/lit-review.md \
  qiongli-workflow/workflows/ethics-check.md \
  qiongli-workflow/workflows/synthesize.md \
  qiongli-workflow/workflows/compliance-check.md \
  qiongli-workflow/workflows/proofread.md \
  qiongli-workflow/workflows/academic-present.md \
  tests/test_boundary_interviewer_contract.py \
  tests/test_boundary_questions.py \
  tests/test_orchestrator_workflows.py \
  tests/test_context_package_builder.py \
  tests/test_academic_context_continuity.py \
  tests/test_research_standard_validator.py \
  plugins/qiongli/skills/qiongli-workflow \
  packages/npm-qiongli/payload/qiongli-workflow \
  packages/npm-qiongli/python-runtime
git commit -m "feat: add full academic boundary review support"
```

Expected: commit succeeds after confirming the staged diff contains no unrelated installer, website, marketplace, or manifest changes.

## Final Verification

Run:

```bash
.venv/bin/python -m unittest tests.test_boundary_interviewer_contract -v
.venv/bin/python -m unittest tests.test_boundary_questions tests.test_orchestrator_workflows tests.test_context_package_builder -v
.venv/bin/python -m unittest tests.test_academic_context_continuity tests.test_research_standard_validator -v
.venv/bin/python -m unittest tests.test_distribution_payloads tests.test_plugin_distribution_contract tests.test_npm_package_contract -v
.venv/bin/python -m unittest tests.test_skill_contract_alignment tests.test_workflow_contract_doc -v
```

Expected: all listed suites pass. The important behavioral result is: once a user answers a grill-me boundary question, `context/boundary_review.md` becomes part of the task packet and context package, and later Qiongli skills continue the task inside that boundary unless they explicitly create a new boundary review entry with a revisit trigger.

## Self-Review

- Spec coverage: The plan covers full A-K stages, answer persistence, downstream continuation, prompt injection, context package propagation, academic context continuity, validator gates, workflow hooks, distribution sync, and tests.
- Placeholder scan: The plan contains concrete file paths, code snippets, commands, expected outcomes, and no open implementation placeholders.
- Type consistency: `BoundaryQuestionPlan`, `boundary_review`, `existing_review`, `required_before_draft`, `context/boundary_review.md`, and `revisit_trigger` are named consistently across tests, implementation snippets, prompts, and contracts.
