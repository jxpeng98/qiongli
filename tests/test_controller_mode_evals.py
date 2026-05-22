from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.run_controller_mode_evals import REQUIRED_DIMENSIONS, run_evals


REPO_ROOT = Path(__file__).resolve().parents[1]
CASE_DIR = REPO_ROOT / "evals" / "controller_modes"


def write_case(path: Path, overrides: dict[str, object] | None = None) -> None:
    payload: dict[str, object] = {
        "id": path.stem,
        "execution_mode": "solo",
        "controller": "codex",
        "primary_agent": "codex",
        "reviewer_agent": "claude",
        "verifier_agent": "codex",
        "expected_artifacts": ["writing/writing-claim-map.md"],
        "artifacts_written": ["writing/writing-claim-map.md"],
        "verification": {
            "status": "passed",
            "expected_blocked": False,
            "evidence": [{"command": "python -m pytest", "status": "passed"}],
        },
        "scores": {dimension: 1.0 for dimension in REQUIRED_DIMENSIONS},
    }
    if overrides:
        payload.update(overrides)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


class ControllerModeEvalTests(unittest.TestCase):
    def test_offline_eval_fixtures_exist_for_required_controller_modes(self) -> None:
        expected = {
            "solo_codex_writing.json",
            "solo_claude_code.json",
            "claude_primary_codex_review.json",
            "codex_primary_claude_review.json",
            "duo_disagreement.json",
            "verification_blocked.json",
        }
        found = {path.name for path in CASE_DIR.glob("*.json")}

        self.assertTrue(expected.issubset(found))

    def test_legal_fixtures_pass_and_count_expected_blocked_verification(self) -> None:
        summary = run_evals(CASE_DIR)

        self.assertEqual("passed", summary["status"])
        self.assertEqual(6, summary["case_count"])
        self.assertEqual([], summary["failures"])
        self.assertEqual(1, summary["blocked_verification_count"])
        self.assertEqual(set(REQUIRED_DIMENSIONS), set(summary["scores"]))
        blocked_cases = [
            case
            for case in summary["cases"]
            if case["verification_outcome"] == "expected_blocked"
        ]
        self.assertEqual(["verification_blocked"], [case["id"] for case in blocked_cases])

    def test_missing_required_dimension_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            case_path = Path(tmp_dir) / "missing_dimension.json"
            scores = {dimension: 1.0 for dimension in REQUIRED_DIMENSIONS}
            scores.pop("handoff_quality")
            write_case(case_path, {"scores": scores})

            summary = run_evals(Path(tmp_dir))

        self.assertEqual("failed", summary["status"])
        joined = "\n".join(summary["failures"])
        self.assertIn("missing_dimension", joined)
        self.assertIn("handoff_quality", joined)

    def test_boolean_score_dimension_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            case_path = Path(tmp_dir) / "boolean_score.json"
            scores: dict[str, object] = {
                dimension: 1.0 for dimension in REQUIRED_DIMENSIONS
            }
            scores["artifact_completeness"] = True
            write_case(case_path, {"scores": scores})

            summary = run_evals(Path(tmp_dir))

        self.assertEqual("failed", summary["status"])
        joined = "\n".join(summary["failures"])
        self.assertIn("boolean_score", joined)
        self.assertIn("artifact_completeness", joined)

    def test_invalid_execution_mode_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            case_path = Path(tmp_dir) / "invalid_mode.json"
            write_case(case_path, {"execution_mode": "pair-programming"})

            summary = run_evals(Path(tmp_dir))

        self.assertEqual("failed", summary["status"])
        self.assertIn("execution_mode", "\n".join(summary["failures"]))

    def test_passed_verification_requires_passed_command_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            case_path = Path(tmp_dir) / "dishonest_verification.json"
            write_case(
                case_path,
                {
                    "verification": {
                        "status": "passed",
                        "expected_blocked": False,
                        "evidence": [
                            {
                                "command": "python -m unittest tests.test_controller_mode_evals",
                                "status": "failed",
                            }
                        ],
                    },
                },
            )

            summary = run_evals(Path(tmp_dir))

        self.assertEqual("failed", summary["status"])
        joined = "\n".join(summary["failures"])
        self.assertIn("dishonest_verification", joined)
        self.assertIn("passed verification needs passed command evidence", joined)

    def test_cli_outputs_json_summary(self) -> None:
        completed = subprocess.run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts" / "run_controller_mode_evals.py"),
                str(CASE_DIR),
            ],
            check=True,
            capture_output=True,
            text=True,
        )

        summary = json.loads(completed.stdout)
        self.assertEqual("passed", summary["status"])
        self.assertEqual(6, summary["case_count"])
        self.assertEqual(1, summary["blocked_verification_count"])


if __name__ == "__main__":
    unittest.main()
