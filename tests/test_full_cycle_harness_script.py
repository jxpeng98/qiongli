from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "tooling" / "scripts" / "run_full_cycle_workflow_harness.py"
FIXTURE_ROOT = REPO_ROOT / "tests" / "fixtures" / "full_cycle_harness"


class FullCycleHarnessScriptTests(unittest.TestCase):
    def test_clean_fixture_returns_zero_and_ready_for_h5(self) -> None:
        result, payload = _run_fixture("clean_empirical")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(payload["lifecycle_status"], "ready_for_h5")
        self.assertEqual(payload["blocking_reasons"], [])
        self.assertEqual(payload["topic"], "full-cycle-fixture")
        self.assertEqual(payload["paper_type"], "empirical")
        self.assertEqual(payload["journal_fit"]["status"], "ok")
        self.assertEqual(payload["journal_fit"]["blocking_reasons"], [])
        top_venue = payload["journal_fit"]["ranked_venues"][0]
        self.assertEqual(top_venue["class"], "primary")
        self.assertEqual(top_venue["source"], "venues/journal-of-finance.yaml")
        self.assertFalse(Path(top_venue["source"]).is_absolute())

    def test_clean_fixture_report_is_deterministic_across_runs(self) -> None:
        first_result, first_payload = _run_fixture("clean_empirical")
        second_result, second_payload = _run_fixture("clean_empirical")

        self.assertEqual(first_result.returncode, 0, first_result.stderr)
        self.assertEqual(second_result.returncode, 0, second_result.stderr)
        self.assertEqual(first_payload, second_payload)

    def test_drifted_fixture_returns_one_with_research_question_drift(self) -> None:
        result, payload = _run_fixture("drifted_research_question")

        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("research_question_drift", payload["blocking_reasons"])
        self.assertEqual(payload["lifecycle_status"], "blocked_research_question_drift")

    def test_missing_claim_evidence_fixture_returns_one_with_blocking_report(self) -> None:
        result, payload = _run_fixture("missing_claim_evidence")

        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("F:missing_artifact", payload["blocking_reasons"])
        self.assertIn("missing_claim_evidence", payload["blocking_reasons"])
        self.assertIn(
            "missing evidence/claim-evidence-ledger.csv",
            payload["journal_fit"]["blocking_reasons"],
        )

    def test_journal_overreach_fixture_returns_one_without_primary_recommendation(self) -> None:
        result, payload = _run_fixture("journal_overreach")

        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("unresolved_judge_blocks", payload["blocking_reasons"])
        ranked_venues = payload["journal_fit"]["ranked_venues"]
        self.assertTrue(ranked_venues)
        self.assertEqual(ranked_venues[0]["class"], "stretch")
        self.assertEqual(ranked_venues[0]["desk_reject_risk"], "high")
        self.assertIn("unresolved fatal flaw", ranked_venues[0]["reviewer_risk"])


def _run_fixture(name: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
    with tempfile.TemporaryDirectory() as tmp_dir:
        report_path = Path(tmp_dir) / "report.json"
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT_PATH),
                "--fixture",
                str(FIXTURE_ROOT / name),
                "--json-report",
                str(report_path),
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        if report_path.exists():
            payload = json.loads(report_path.read_text(encoding="utf-8"))
        else:
            payload = {}
    return result, payload


if __name__ == "__main__":
    unittest.main()
