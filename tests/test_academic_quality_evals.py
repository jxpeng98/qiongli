from __future__ import annotations

import unittest
from pathlib import Path

from scripts.run_academic_quality_evals import run_evals


REPO_ROOT = Path(__file__).resolve().parents[1]


class AcademicQualityEvalTests(unittest.TestCase):
    def test_offline_eval_cases_exist_for_required_scenarios(self) -> None:
        case_dir = REPO_ROOT / "evals" / "academic_quality" / "cases"
        expected = {
            "empirical-causal-design.yaml",
            "systematic-review.yaml",
            "qualitative-coding.yaml",
            "theory-contribution.yaml",
            "code-first-methods.yaml",
            "reviewer-rebuttal.yaml",
            "q1-rq-method-mismatch.yaml",
            "q2-unsupported-claim.yaml",
            "q3-reporting-gap.yaml",
            "q4-reproducibility-gap.yaml",
            "economics-did-invalid-parallel-trends.yaml",
            "finance-event-study-leakage.yaml",
        }
        found = {path.name for path in case_dir.glob("*.yaml")}
        self.assertEqual(expected, found)

    def test_eval_runner_scores_cases_without_network(self) -> None:
        result = run_evals(REPO_ROOT / "evals" / "academic_quality" / "cases")

        self.assertEqual(12, result.case_count)
        self.assertEqual([], result.errors)
        expected_dimensions = {
            "artifact_completeness",
            "evidence_traceability",
            "no_fabricated_sources",
            "claim_calibration",
            "venue_fit",
            "method_validity",
            "scholarly_voice",
            "quality_gate_compliance",
            "domain_method_fit",
        }
        self.assertEqual(expected_dimensions, set(result.dimension_scores))


if __name__ == "__main__":
    unittest.main()
