from __future__ import annotations

import unittest

from bridges.project_inference import infer_project_manifest_suggestion


class ProjectInferenceTests(unittest.TestCase):
    def test_detects_finance_event_study(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "earnings announcement returns",
                "context": "event study abnormal returns factor exposure",
            },
            draft_content="Use an event window and check leakage.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "finance")
        self.assertIn("event-study", suggestion["method_lenses"])
        self.assertGreaterEqual(suggestion["confidence"], 0.6)

    def test_detects_economics_did(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "minimum wage DID", "context": "parallel trends causal identification"},
            draft_content="Difference-in-differences design needs pre-trends.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "economics")
        self.assertIn("did", suggestion["method_lenses"])

    def test_returns_auto_when_evidence_is_weak(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "writing introduction", "context": "revise paragraph"},
            draft_content="",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["confidence"], 0.0)


if __name__ == "__main__":
    unittest.main()
