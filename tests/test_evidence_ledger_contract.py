from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.audit_evidence_contract import audit_evidence_ledger


REPO_ROOT = Path(__file__).resolve().parents[1]


class EvidenceLedgerContractTests(unittest.TestCase):
    def test_bundled_evidence_contract_assets_exist(self) -> None:
        expected = [
            REPO_ROOT / "templates" / "evidence-ledger.md",
            REPO_ROOT / "templates" / "claim-evidence-ledger.csv",
            REPO_ROOT / "qiongli-workflow" / "references" / "evidence-ledger-contract.md",
        ]
        for path in expected:
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"Missing {path}")

    def test_valid_ledger_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            ledger = Path(tmp_dir) / "ledger.csv"
            ledger.write_text(
                "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n"
                "C1,Main supported claim,finding,paper,Smith2024,p. 4,notes/smith.md,high,Single study,supported\n",
                encoding="utf-8",
            )

            result = audit_evidence_ledger(ledger)

        self.assertEqual([], result.errors)

    def test_invalid_enum_and_unsupported_claim_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            ledger = Path(tmp_dir) / "ledger.csv"
            ledger.write_text(
                "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n"
                "C1,Unsupported central claim,unsupported,paper,,,,medium,,unsupported\n",
                encoding="utf-8",
            )

            result = audit_evidence_ledger(ledger)

        error_blob = "\n".join(result.errors)
        self.assertIn("invalid claim_type", error_blob)
        self.assertIn("unsupported claims must use evidence_type=gap_note", error_blob)


if __name__ == "__main__":
    unittest.main()
