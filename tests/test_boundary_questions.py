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
