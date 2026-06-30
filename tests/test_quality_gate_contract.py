from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "audit_quality_gates.py"
CONTRACT_PATH = RepoLayout(REPO_ROOT).standards / "quality-gate-contract.yaml"


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

    def test_contract_defines_semantic_check_ids_for_each_gate(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)

        gates = contract.get("gates", {})
        expected_checks = {
            "Q1": {"q1_rq_method_alignment"},
            "Q2": {"q2_claim_evidence_traceability"},
            "Q3": {"q3_reporting_completeness"},
            "Q4": {"q4_reproducibility_baseline"},
        }
        for gate_id, check_ids in expected_checks.items():
            gate = gates[gate_id]
            self.assertIn("semantic_checks", gate)
            found = {item["check_id"] for item in gate["semantic_checks"]}
            self.assertTrue(check_ids.issubset(found), gate_id)
            self.assertIn("semantic_checks", gate["report_fields"])

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
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
                          - reports/q1-verification.md
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence:
                      - reports/q2-review.md
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: WARN
                        finding: Claim-evidence traceability needs review.
                        evidence_refs:
                          - reports/q2-review.md
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: PASS
                        finding: Reporting completeness verified.
                        evidence_refs:
                          - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: WARN
                        finding: Reproducibility baseline needs review.
                        evidence_refs:
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
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
                          - reports/q1-verification.md
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence:
                      - reports/q2-review.md
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: WARN
                        finding: Claim-evidence traceability needs review.
                        evidence_refs:
                          - reports/q2-review.md
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: PASS
                        finding: Reporting completeness verified.
                        evidence_refs:
                          - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: WARN
                        finding: Reproducibility baseline needs review.
                        evidence_refs:
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
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
                          - reports/q1-verification.md
                    blocking_issues: []
                  Q2:
                    status: WARN
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: WARN
                        finding: Claim-evidence traceability needs review.
                        evidence_refs:
                          - reports/q2-review.md
                    blocking_issues: []
                  Q3:
                    status: PASS
                    evidence:
                      - reports/q3-implementation.md
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: PASS
                        finding: Reporting completeness verified.
                        evidence_refs:
                          - reports/q3-implementation.md
                    blocking_issues: []
                  Q4:
                    status: WARN
                    evidence:
                      - reports/q4-validation.md
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: WARN
                        finding: Reproducibility baseline needs review.
                        evidence_refs:
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

    def test_gate_report_rejects_malformed_semantic_check(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: BLOCKED
                        evidence_refs: []
                    blocking_issues:
                      - issue: Alignment matrix missing.
                        required_action: Add study_design.md matrix.
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 semantic_checks[1] missing required field: finding", result.errors)

    def test_gate_report_rejects_missing_expected_semantic_check_id(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q1_wrong_check
                        status: BLOCKED
                        finding: Wrong semantic check id supplied.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Alignment matrix missing.
                        required_action: Add study_design.md matrix.
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 missing semantic check id: q1_rq_method_alignment", result.errors)

    def test_gate_report_rejects_malformed_structured_evidence_and_blocking_issue(
        self,
    ) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: FAIL
                    evidence:
                      - artifact: RESEARCH/topic/study_design.md
                        anchor: ""
                        supports: RQ mapping.
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: FAIL
                        finding: RQ mapping is incomplete.
                        evidence_refs:
                          - RESEARCH/topic/study_design.md#rq-method-matrix
                    blocking_issues:
                      - issue: ""
                        required_action: Add the missing RQ-method row.
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 evidence[1] missing field: anchor", result.errors)
        self.assertIn("Q1 blocking_issues[1] missing field: issue", result.errors)

    def test_gate_report_rejects_non_list_evidence_and_blocking_issues(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: FAIL
                    evidence:
                      artifact: RESEARCH/topic/study_design.md
                      anchor: matrix
                      supports: mapping
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: FAIL
                        finding: RQ mapping is incomplete.
                        evidence_refs:
                          - RESEARCH/topic/study_design.md#rq-method-matrix
                    blocking_issues:
                      issue: Missing row
                      required_action: Add row
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 evidence must be a list", result.errors)
        self.assertIn("Q1 blocking_issues must be a list", result.errors)

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
