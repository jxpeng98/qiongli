from __future__ import annotations

import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
TOOLING_SCRIPTS = REPO_ROOT / "tooling" / "scripts"
if str(TOOLING_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(TOOLING_SCRIPTS))

import release_acceptance_evidence  # noqa: E402


class ReleaseAcceptanceEvidenceTests(unittest.TestCase):
    def test_renders_subject_eval_and_preview_smoke_summary(self) -> None:
        eval_report = {
            "case_count": 3,
            "metrics": {
                "decision_accuracy": 1.0,
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.875,
            },
            "threshold_failures": [],
        }
        smoke_report = {
            "mode": "preview",
            "summary": {"total": 2, "passed": 2, "failed": 0},
        }

        rendered = release_acceptance_evidence.render_acceptance_evidence(
            eval_report,
            smoke_report,
        )

        self.assertIn("## Subject Runtime Evidence", rendered)
        self.assertIn(
            "- Subject router eval: passed (cases: 3, threshold_failures: 0)",
            rendered,
        )
        self.assertIn("decision_accuracy=1.000", rendered)
        self.assertIn("suggest_subject_precision=0.875", rendered)
        self.assertIn(
            "- Subject runtime smoke: passed (mode: preview, passed: 2/2, failed: 0)",
            rendered,
        )

    def test_renders_failure_counts_without_hiding_evidence(self) -> None:
        rendered = release_acceptance_evidence.render_acceptance_evidence(
            {
                "case_count": 1,
                "metrics": {},
                "threshold_failures": ["decision_accuracy below threshold"],
            },
            {
                "mode": "preview",
                "summary": {"total": 2, "passed": 1, "failed": 1},
            },
        )

        self.assertIn(
            "- Subject router eval: failed (cases: 1, threshold_failures: 1)",
            rendered,
        )
        self.assertIn(
            "- Subject runtime smoke: failed (mode: preview, passed: 1/2, failed: 1)",
            rendered,
        )


if __name__ == "__main__":
    unittest.main()
