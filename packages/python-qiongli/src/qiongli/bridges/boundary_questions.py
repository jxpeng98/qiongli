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
    del required_skills

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
