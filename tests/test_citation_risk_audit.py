from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

from scripts.audit_citation_risk import audit_citation_integrity


REPO_ROOT = Path(__file__).resolve().parents[1]


class CitationRiskAuditTests(unittest.TestCase):
    def test_bundled_citation_risk_assets_exist(self) -> None:
        for path in (
            RepoLayout(REPO_ROOT).workflow / "references" / "citation-risk-policy.md",
            RepoLayout(REPO_ROOT).templates / "citation-risk-report.md",
        ):
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"Missing {path}")

    def test_source_ids_present_in_bibliography_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            ledger = root / "ledger.csv"
            bibliography = root / "bibliography.bib"
            ledger.write_text(
                "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n"
                "C1,Claim,finding,paper,Smith2024,p. 4,notes/smith.md,high,,supported\n",
                encoding="utf-8",
            )
            bibliography.write_text("@article{Smith2024,\n  title={Example}\n}\n", encoding="utf-8")

            result = audit_citation_integrity(ledger, bibliography)

        self.assertEqual([], result.errors)

    def test_fabricated_source_id_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            ledger = root / "ledger.csv"
            bibliography = root / "bibliography.bib"
            ledger.write_text(
                "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n"
                "C1,Claim,finding,paper,Missing2026,p. 4,notes/missing.md,high,,supported\n",
                encoding="utf-8",
            )
            bibliography.write_text("@article{Smith2024,\n  title={Example}\n}\n", encoding="utf-8")

            result = audit_citation_integrity(ledger, bibliography)

        self.assertIn("source_id not found in bibliography: Missing2026", result.errors)


if __name__ == "__main__":
    unittest.main()
