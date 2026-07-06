from __future__ import annotations

import unittest

from bridges.project_inference import infer_project_manifest_suggestion


class ProjectInferenceTests(unittest.TestCase):
    def test_method_only_finance_event_study_borrows_lens_without_manifest_switch(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "policy announcement effects",
                "context": "Use an event study design.",
            },
            draft_content="Estimate the announcement window.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["subject_mode"], "auto")
        self.assertIn("event-study", suggestion["method_lenses"])
        borrowed_lenses = suggestion["subject_refinement"]["borrowed_lenses"]
        self.assertIsInstance(borrowed_lenses[0], dict)
        self.assertIn("event-study", [lens["lens"] for lens in borrowed_lenses])

    def test_strong_finance_event_study_with_returns_data_suggests_finance(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "earnings announcement returns",
                "context": "event study abnormal returns from CRSP for Journal of Finance",
            },
            draft_content="Use an event window and check leakage.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "finance")
        self.assertEqual(suggestion["subject_mode"], "suggested")
        self.assertIn("event-study", suggestion["method_lenses"])
        self.assertEqual(suggestion["subject_refinement"]["decision"], "suggest_subject")
        self.assertGreaterEqual(suggestion["confidence"], 0.6)

    def test_detects_economics_did(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "minimum wage DID", "context": "parallel trends causal identification"},
            draft_content="Difference-in-differences design needs pre-trends.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "economics")
        self.assertEqual(suggestion["subject_mode"], "suggested")
        self.assertIn("did", suggestion["method_lenses"])

    def test_returns_auto_when_evidence_is_weak(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "writing introduction", "context": "revise paragraph"},
            draft_content="",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["subject_mode"], "auto")
        self.assertEqual(suggestion["confidence"], 0.0)

    def test_ordinary_past_tense_did_does_not_trigger_economics(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "revise literature review",
                "context": "The previous draft did not connect the paragraphs clearly.",
            },
            draft_content="We did the introduction first and then tightened the framing.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["subject_mode"], "auto")
        self.assertEqual(suggestion["confidence"], 0.0)

    def test_ordinary_return_does_not_trigger_finance(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "revise conclusion",
                "context": "Return to the main claim in the final paragraph.",
            },
            draft_content="The author should return later to the motivation.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["subject_mode"], "auto")
        self.assertEqual(suggestion["confidence"], 0.0)


if __name__ == "__main__":
    unittest.main()
