from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "audit_quality_gates.py"
CONTRACT_PATH = REPO_ROOT / "standards" / "quality-gate-contract.yaml"


def load_audit_module():
    if not MODULE_PATH.exists():
        raise AssertionError(f"Missing audit script: {MODULE_PATH}")
    spec = importlib.util.spec_from_file_location("audit_quality_gates", MODULE_PATH)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def write_report(path: Path, yaml_block: str) -> None:
    path.write_text(
        "# Quality Gate Report\n\n```yaml\n"
        + textwrap.dedent(yaml_block).strip()
        + "\n```\n",
        encoding="utf-8",
    )


class QualityGateContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.audit_module = load_audit_module()

    def test_contract_defines_q1_to_q4_with_required_fields(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)

        gates = contract.get("gates", {})
        self.assertEqual({"Q1", "Q2", "Q3", "Q4"}, set(gates))
        for gate in gates.values():
            for field in (
                "name",
                "required_evidence",
                "pass_criteria",
                "fail_conditions",
                "report_fields",
            ):
                self.assertIn(field, gate)
                self.assertTrue(gate[field])

    def test_gate_report_fails_for_missing_gate_status(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: PASS
                    evidence: []
                    blocking_issues: []
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q2", "\n".join(result.errors))

    def test_gate_report_accepts_complete_report(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: PASS
                    evidence:
                      - reports/q1-verification.md
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence:
                      - reports/q2-review.md
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    blocking_issues: []
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertTrue(result.passed)
        self.assertEqual([], result.errors)

    def test_gate_report_rejects_pass_or_warn_without_evidence(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        cases = (
            (
                "Q1",
                "PASS",
                "Q1 status PASS requires non-empty evidence",
                """
                gates:
                  Q1:
                    status: PASS
                    evidence: []
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence:
                      - reports/q2-review.md
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    blocking_issues: []
                """,
            ),
            (
                "Q2",
                "WARN",
                "Q2 status WARN requires non-empty evidence",
                """
                gates:
                  Q1:
                    status: PASS
                    evidence:
                      - reports/q1-verification.md
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence: []
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    blocking_issues: []
                """,
            ),
        )
        for gate_id, status, expected_error, yaml_block in cases:
            with self.subTest(gate_id=gate_id, status=status):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    report_path = Path(tmp_dir) / "quality-gate-report.md"
                    write_report(report_path, yaml_block)

                    result = self.audit_module.audit_gate_report(report_path, contract)

                self.assertFalse(result.passed)
                self.assertIn(expected_error, result.errors)

    def test_cli_reports_missing_contract_without_traceback(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            missing_contract = Path(tmp_dir) / "missing.yaml"
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(report_path, "gates: {}\n")

            result = subprocess.run(
                [
                    sys.executable,
                    str(MODULE_PATH),
                    "--strict",
                    "--contract",
                    str(missing_contract),
                    "--report",
                    str(report_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

        self.assertNotEqual(0, result.returncode)
        self.assertIn("[FAIL]", result.stdout)
        self.assertIn("Failed to load quality gate contract", result.stdout)
        self.assertNotIn("Traceback", result.stderr)

    def test_cli_reports_malformed_contract_without_traceback(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            contract_path = Path(tmp_dir) / "quality-gate-contract.yaml"
            contract_path.write_text("gates:\n  Q1: [unterminated\n", encoding="utf-8")
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(report_path, "gates: {}\n")

            result = subprocess.run(
                [
                    sys.executable,
                    str(MODULE_PATH),
                    "--strict",
                    "--contract",
                    str(contract_path),
                    "--report",
                    str(report_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

        self.assertNotEqual(0, result.returncode)
        self.assertIn("[FAIL]", result.stdout)
        self.assertIn("Failed to load quality gate contract", result.stdout)
        self.assertNotIn("Traceback", result.stderr)


if __name__ == "__main__":
    unittest.main()
