from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.validate_research_standard import (
    ValidationReport,
    validate_controller_mode_contracts,
    validate_literature_first_contracts,
)


class ResearchStandardValidatorTests(unittest.TestCase):
    def test_strict_controller_mode_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("standards/agent-run-contract.yaml", joined)
        self.assertIn("scripts/audit_solo_role_gates.py", joined)

    def test_non_strict_controller_mode_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_non_strict_literature_first_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_literature_first_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_strict_literature_first_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_literature_first_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("scripts/audit_literature_search_quality.py", joined)
        self.assertIn("scripts/materialize_literature_search_bundle.py", joined)
        self.assertIn("templates/search-diagnostics.md", joined)
        self.assertIn("qiongli-workflow/references/literature-search-quality-contract.md", joined)

    def test_strict_controller_mode_contract_runs_solo_gate_audit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            for relative_path in (
                "standards/agent-run-contract.yaml",
                "standards/solo-role-policy.yaml",
                "templates/agent-run-packet.json",
                "templates/agent-review-packet.md",
                "templates/agent-handoff.md",
                "templates/solo-task-packet.md",
                "templates/solo-self-review.md",
                "templates/implementation-intent.md",
                "templates/writing-claim-map.md",
                "templates/quality-gate-report.md",
                "scripts/audit_solo_role_gates.py",
                "scripts/audit_agent_handoffs.py",
            ):
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("placeholder\n", encoding="utf-8")
            run_path = root / "runs" / "bad-run.json"
            run_path.parent.mkdir(parents=True, exist_ok=True)
            run_path.write_text(
                '{"run_id": "bad-run", "execution_mode": "solo_codex"}',
                encoding="utf-8",
            )
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("bad-run", joined)
        self.assertIn("verification_status", joined)


if __name__ == "__main__":
    unittest.main()
