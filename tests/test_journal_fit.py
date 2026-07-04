from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.bridges.journal_fit import recommend_journals


class JournalFitTests(unittest.TestCase):
    def test_blocks_when_manuscript_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write_venue(root / "venues" / "journal-a.yaml", "journal-a", ["finance"])

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "blocked")
        self.assertEqual(report["ranked_venues"], [])
        self.assertIn("missing manuscript/manuscript.md", report["blocking_reasons"])

    def test_allows_report_with_blocking_reasons_when_non_manuscript_inputs_are_missing(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns.",
            )
            _write_venue(
                root / "venues" / "journal-of-finance.yaml",
                "journal-of-finance",
                ["finance", "event study", "abnormal returns"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertNotEqual(report["ranked_venues"], [])
        self.assertIn("missing framing/contribution_statement.md", report["blocking_reasons"])
        self.assertIn("missing study_design.md", report["blocking_reasons"])
        self.assertIn("missing evidence/claim-evidence-ledger.csv", report["blocking_reasons"])

    def test_ranks_primary_and_do_not_submit_venues_from_profile_fit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns and CRSP data.",
            )
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            _write(
                root / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,status\nc1,abnormal returns,supported\n",
            )
            finance_profile = root / "venues" / "journal-of-finance.yaml"
            management_profile = root / "venues" / "general-management.yaml"
            _write_venue(
                finance_profile,
                "journal-of-finance",
                ["finance", "event study", "abnormal returns", "CRSP"],
            )
            _write_venue(
                management_profile,
                "general-management",
                ["management", "qualitative", "organization theory"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"], limit=5)

        ranked = report["ranked_venues"]
        self.assertEqual(report["status"], "ok")
        self.assertEqual(ranked[0]["venue_id"], "journal-of-finance")
        self.assertEqual(ranked[0]["class"], "primary")
        self.assertEqual(ranked[-1]["venue_id"], "general-management")
        self.assertEqual(ranked[-1]["class"], "do_not_submit")
        self.assertEqual(ranked[0]["source"], str(finance_profile))
        self.assertIn("abnormal returns", ranked[0]["matched_terms"])
        for key in (
            "venue_id",
            "class",
            "score",
            "scope_fit",
            "contribution_fit",
            "method_evidence_fit",
            "reviewer_risk",
            "desk_reject_risk",
            "matched_terms",
            "required_revision",
            "source",
        ):
            self.assertIn(key, ranked[0])

    def test_marks_stretch_when_fit_is_high_but_fatal_flaw_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns and CRSP data.",
            )
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            _write(
                root / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,status\nc1,abnormal returns,partial\n",
            )
            _write(root / "revision" / "fatal_flaw_analysis.md", "Decision: block_submission")
            _write_venue(
                root / "venues" / "journal-of-finance.yaml",
                "journal-of-finance",
                ["finance", "event study", "abnormal returns"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertEqual(report["ranked_venues"][0]["class"], "stretch")
        self.assertIn("unresolved fatal flaw", report["ranked_venues"][0]["reviewer_risk"])

    def test_passed_fatal_flaw_report_does_not_downgrade_high_fit_venue(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns and CRSP data.",
            )
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            _write(
                root / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,status\nc1,abnormal returns,supported\n",
            )
            _write(root / "revision" / "fatal_flaw_analysis.md", "Decision: pass\nNo fatal flaws.")
            _write_venue(
                root / "venues" / "journal-of-finance.yaml",
                "journal-of-finance",
                ["finance", "event study", "abnormal returns"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["ranked_venues"][0]["class"], "primary")
        self.assertNotIn("unresolved fatal flaw", report["ranked_venues"][0]["reviewer_risk"])

    def test_skips_malformed_venue_yaml_and_keeps_valid_profiles(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns and CRSP data.",
            )
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            _write(
                root / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,status\nc1,abnormal returns,supported\n",
            )
            _write(root / "venues" / "broken.yaml", "venue_id: [broken\n")
            _write_venue(
                root / "venues" / "journal-of-finance.yaml",
                "journal-of-finance",
                ["finance", "event study", "abnormal returns"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertEqual(["journal-of-finance"], [item["venue_id"] for item in report["ranked_venues"]])

    def test_repeated_single_broad_term_cannot_be_primary(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(root / "manuscript" / "manuscript.md", "Finance paper about finance.")
            _write(root / "framing" / "contribution_statement.md", "Finance contribution.")
            _write(root / "study_design.md", "Finance study design.")
            _write(
                root / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,status\nc1,finance finding,supported\n",
            )
            _write_repeated_term_venue(
                root / "venues" / "broad-finance.yaml",
                "broad-finance",
                "finance",
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertGreaterEqual(report["ranked_venues"][0]["score"], 0.75)
        self.assertNotEqual(report["ranked_venues"][0]["class"], "primary")

    def test_invalid_bytes_claim_evidence_ledger_does_not_crash(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(
                root / "manuscript" / "manuscript.md",
                "Finance event study with abnormal returns and CRSP data.",
            )
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            ledger = root / "evidence" / "claim-evidence-ledger.csv"
            ledger.parent.mkdir(parents=True, exist_ok=True)
            ledger.write_bytes(b"\xff")
            _write_venue(
                root / "venues" / "journal-of-finance.yaml",
                "journal-of-finance",
                ["finance", "event study", "abnormal returns"],
            )

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertIn("incomplete claim evidence", report["blocking_reasons"])


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_venue(path: Path, venue_id: str, keywords: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(
            {
                "venue_id": venue_id,
                "community": keywords[0],
                "article_types": ["research article"],
                "contribution_expectations": keywords,
                "methods_expectations": keywords,
                "evidence_standards": keywords,
                "writing_style": ["direct"],
                "common_reviewer_objections": ["weak fit"],
                "formatting_constraints": {"word_limit": 12000},
                "required_reporting_standards": [],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )


def _write_repeated_term_venue(path: Path, venue_id: str, term: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(
            {
                "venue_id": venue_id,
                "community": term,
                "article_types": [term],
                "contribution_expectations": [term],
                "methods_expectations": [term],
                "evidence_standards": [term],
                "writing_style": [term],
                "common_reviewer_objections": ["weak fit"],
                "formatting_constraints": {"word_limit": 12000},
                "required_reporting_standards": [],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
