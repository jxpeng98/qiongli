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


def blocked_gate_yaml(
    gate_id: str,
    check_id: str,
    finding: str,
    issue: str,
    action: str,
) -> str:
    return f"""
    {gate_id}:
      status: BLOCKED
      evidence: []
      semantic_checks:
        - check_id: {check_id}
          status: BLOCKED
          finding: {finding}
          evidence_refs: []
      blocking_issues:
        - issue: {issue}
          required_action: {action}
    """


def report_with_gate_override(gate_id: str, override_yaml: str) -> str:
    gates = {
        "Q1": blocked_gate_yaml(
            "Q1",
            "q1_rq_method_alignment",
            "RQ-method alignment not supplied.",
            "Alignment matrix missing.",
            "Add study_design.md matrix.",
        ),
        "Q2": blocked_gate_yaml(
            "Q2",
            "q2_claim_evidence_traceability",
            "Claim ledger not supplied.",
            "Claim ledger missing.",
            "Add claim-evidence-ledger.csv.",
        ),
        "Q3": blocked_gate_yaml(
            "Q3",
            "q3_reporting_completeness",
            "Reporting checklist not supplied.",
            "Reporting checklist missing.",
            "Add reporting_checklist.md.",
        ),
        "Q4": blocked_gate_yaml(
            "Q4",
            "q4_reproducibility_baseline",
            "Reproducibility audit not supplied.",
            "Reproducibility audit missing.",
            "Add reproducibility_audit.md.",
        ),
    }
    gates[gate_id] = override_yaml
    body = "gates:\n" + "\n".join(
        textwrap.indent(textwrap.dedent(value).strip(), "  ") for value in gates.values()
    )
    return body


def q1_report_with_evidence_refs(evidence_refs_yaml: str) -> str:
    evidence_refs = textwrap.indent(textwrap.dedent(evidence_refs_yaml).strip(), " " * 26)
    return f"""
                gates:
                  Q1:
                    status: PASS
                    evidence:
                      - RESEARCH/topic/study_design.md
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
{evidence_refs}
                    blocking_issues: []
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Claim ledger missing.
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
                        required_action: Add reproducibility_audit.md.
                """


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

    def test_contract_and_template_define_structured_evidence_refs(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        schema = contract.get("evidence_ref_schema", {})

        self.assertEqual(
            ["artifact", "anchor", "supports"],
            schema.get("required_fields"),
        )
        self.assertIn("claim_id", schema.get("optional_fields", []))
        self.assertIn("diagnostic_id", schema.get("optional_fields", []))

        template_path = RepoLayout(REPO_ROOT).templates / "quality-gate-report.md"
        template = template_path.read_text(encoding="utf-8")

        errors: list[str] = []
        payload = self.audit_module._load_report_yaml(template_path, errors)
        self.assertEqual([], errors)
        for gate_id, gate in payload["gates"].items():
            self.assertEqual("BLOCKED", gate["status"], gate_id)
            self.assertEqual([], gate["evidence"], gate_id)
            for check in gate["semantic_checks"]:
                self.assertEqual("BLOCKED", check["status"], gate_id)
                self.assertEqual([], check["evidence_refs"], gate_id)

        examples_heading = "## Structured Evidence Reference Examples"
        self.assertIn(examples_heading, template)
        examples_section = template.split(examples_heading, maxsplit=1)[1]
        examples_yaml = examples_section.split("```yaml\n", maxsplit=1)[1].split(
            "\n```",
            maxsplit=1,
        )[0]
        examples = self.audit_module.yaml.safe_load(examples_yaml)
        for gate_id in ("Q1", "Q2", "Q3", "Q4"):
            refs = examples["evidence_ref_examples"][gate_id]
            self.assertTrue(refs, gate_id)
            for field_name in ("artifact", "anchor", "supports", "claim_id", "diagnostic_id"):
                self.assertTrue(str(refs[0].get(field_name, "")).strip(), gate_id)

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

    def test_gate_report_rejects_malformed_structured_evidence_refs(self) -> None:
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
                      - RESEARCH/topic/study_design.md
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
                          - artifact: RESEARCH/topic/study_design.md
                            anchor: ""
                            supports: ""
                    blocking_issues: []
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Claim ledger missing.
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
                        required_action: Add reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] missing field: anchor",
            result.errors,
        )
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] missing field: supports",
            result.errors,
        )

    def test_gate_report_can_require_evidence_artifact_paths_to_exist(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir)
            report_path = project_root / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: PASS
                    evidence:
                      - RESEARCH/topic/study_design.md
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: PASS
                        finding: RQ-method alignment verified.
                        evidence_refs:
                          - artifact: RESEARCH/topic/study_design.md
                            anchor: rq-method-outcome-matrix
                            supports: Shows RQ-method alignment.
                    blocking_issues: []
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Claim ledger missing.
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
                        required_action: Add reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(
                report_path,
                contract,
                project_root=project_root,
            )

        self.assertFalse(result.passed)
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] artifact path does not exist: "
            "RESEARCH/topic/study_design.md",
            result.errors,
        )

    def test_gate_report_uses_contract_required_fields_for_structured_evidence_refs(
        self,
    ) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        contract["evidence_ref_schema"] = {
            "required_fields": ["artifact", "anchor", "supports", "claim_id"],
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                q1_report_with_evidence_refs(
                    """
                    - artifact: RESEARCH/topic/study_design.md
                      anchor: rq-method-outcome-matrix
                      supports: Shows RQ-method alignment.
                    """,
                ),
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] missing field: claim_id",
            result.errors,
        )

    def test_gate_report_accepts_existing_in_root_structured_evidence_artifact(
        self,
    ) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir)
            artifact_path = project_root / "RESEARCH" / "topic" / "study_design.md"
            artifact_path.parent.mkdir(parents=True)
            artifact_path.write_text("# Study Design\n", encoding="utf-8")
            report_path = project_root / "quality-gate-report.md"
            write_report(
                report_path,
                q1_report_with_evidence_refs(
                    """
                    - artifact: RESEARCH/topic/study_design.md
                      anchor: rq-method-outcome-matrix
                      supports: Shows RQ-method alignment.
                    """,
                ),
            )

            result = self.audit_module.audit_gate_report(
                report_path,
                contract,
                project_root=project_root,
            )

        self.assertTrue(result.passed)
        self.assertEqual([], result.errors)

    def test_gate_report_rejects_absolute_structured_evidence_artifact_paths(
        self,
    ) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            project_root = tmp_path / "project"
            project_root.mkdir()
            outside_path = tmp_path / "outside.md"
            outside_path.write_text("# Outside\n", encoding="utf-8")
            report_path = project_root / "quality-gate-report.md"
            write_report(
                report_path,
                q1_report_with_evidence_refs(
                    f"""
                    - artifact: {outside_path}
                      anchor: outside
                      supports: Points outside the project.
                    """,
                ),
            )

            result = self.audit_module.audit_gate_report(
                report_path,
                contract,
                project_root=project_root,
            )

        self.assertFalse(result.passed)
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] artifact path must be "
            f"project-relative: {outside_path}",
            result.errors,
        )

    def test_gate_report_rejects_structured_evidence_artifact_traversal(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            project_root = tmp_path / "project"
            project_root.mkdir()
            outside_path = tmp_path / "outside.md"
            outside_path.write_text("# Outside\n", encoding="utf-8")
            report_path = project_root / "quality-gate-report.md"
            write_report(
                report_path,
                q1_report_with_evidence_refs(
                    """
                    - artifact: ../outside.md
                      anchor: outside
                      supports: Traverses outside the project.
                    """,
                ),
            )

            result = self.audit_module.audit_gate_report(
                report_path,
                contract,
                project_root=project_root,
            )

        self.assertFalse(result.passed)
        self.assertIn(
            "Q1 semantic_checks[1] evidence_refs[1] artifact path escapes project root: "
            "../outside.md",
            result.errors,
        )

    def test_gate_report_keeps_string_evidence_refs_backward_compatible(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        contract["evidence_ref_schema"] = {
            "required_fields": ["artifact", "anchor", "supports", "claim_id"],
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir)
            report_path = project_root / "quality-gate-report.md"
            write_report(
                report_path,
                q1_report_with_evidence_refs(
                    """
                    - RESEARCH/topic/missing-study-design.md
                    """,
                ),
            )

            result = self.audit_module.audit_gate_report(
                report_path,
                contract,
                project_root=project_root,
            )

        self.assertTrue(result.passed)
        self.assertEqual([], result.errors)

    def test_gate_status_cannot_understate_semantic_check_status(self) -> None:
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
                      - RESEARCH/topic/study_design.md
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: FAIL
                        finding: Method cannot answer the stated causal question.
                        evidence_refs:
                          - artifact: RESEARCH/topic/study_design.md
                            anchor: rq-method-outcome-matrix
                            supports: Shows a causal RQ paired with descriptive evidence only.
                    blocking_issues:
                      - issue: RQ requires a stronger identification strategy.
                        required_action: Narrow the RQ or change the design.
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Claim ledger missing.
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
                        required_action: Add reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 status PASS understates semantic check status FAIL", result.errors)
        self.assertNotIn(
            "Q1 semantic_checks[1] status FAIL requires non-empty evidence_refs",
            result.errors,
        )

    def test_pass_or_warn_gate_cannot_have_blocking_issues(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        for status in ("PASS", "WARN"):
            with self.subTest(status=status):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    report_path = Path(tmp_dir) / "quality-gate-report.md"
                    write_report(
                        report_path,
                        f"""
                        gates:
                          Q1:
                            status: {status}
                            evidence:
                              - RESEARCH/topic/study_design.md
                            semantic_checks:
                              - check_id: q1_rq_method_alignment
                                status: {status}
                                finding: Alignment mostly holds but one measure needs review.
                                evidence_refs:
                                  - artifact: RESEARCH/topic/study_design.md
                                    anchor: rq-method-outcome-matrix
                                    supports: Shows the measure that needs review.
                            blocking_issues:
                              - issue: A required outcome is still missing.
                                required_action: Add the missing outcome row.
                          Q2:
                            status: BLOCKED
                            evidence: []
                            semantic_checks:
                              - check_id: q2_claim_evidence_traceability
                                status: BLOCKED
                                finding: Claim ledger missing.
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
                                required_action: Add reproducibility_audit.md.
                        """,
                    )

                    result = self.audit_module.audit_gate_report(report_path, contract)

                self.assertFalse(result.passed)
                self.assertIn(
                    f"Q1 status {status} cannot have blocking_issues",
                    result.errors,
                )

    def test_pass_warn_or_fail_semantic_check_requires_evidence_refs(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        for status in ("PASS", "WARN", "FAIL"):
            blocking_issues_yaml = (
                """
                            blocking_issues:
                              - issue: RQ-method alignment failed.
                                required_action: Update the study design.
                """
                if status == "FAIL"
                else """
                            blocking_issues: []
                """
            )
            with self.subTest(status=status):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    report_path = Path(tmp_dir) / "quality-gate-report.md"
                    write_report(
                        report_path,
                        f"""
                        gates:
                          Q1:
                            status: {status}
                            evidence:
                              - RESEARCH/topic/study_design.md
                            semantic_checks:
                              - check_id: q1_rq_method_alignment
                                status: {status}
                                finding: RQ-method alignment failed.
                                evidence_refs: []
                        {blocking_issues_yaml}
                          Q2:
                            status: BLOCKED
                            evidence: []
                            semantic_checks:
                              - check_id: q2_claim_evidence_traceability
                                status: BLOCKED
                                finding: Claim ledger missing.
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
                                required_action: Add reproducibility_audit.md.
                        """,
                    )

                    result = self.audit_module.audit_gate_report(report_path, contract)

                self.assertFalse(result.passed)
                self.assertIn(
                    f"Q1 semantic_checks[1] status {status} requires non-empty evidence_refs",
                    result.errors,
                )

    def test_q1_to_q4_realistic_failure_fixtures_are_rejected(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        cases = {
            "Q1": (
                """
                Q1:
                  status: PASS
                  evidence:
                    - RESEARCH/topic/study_design.md
                  semantic_checks:
                    - check_id: q1_rq_method_alignment
                      status: FAIL
                      finding: A causal RQ is paired with descriptive evidence only.
                      evidence_refs:
                        - artifact: RESEARCH/topic/study_design.md
                          anchor: rq-method-outcome-matrix
                          supports: Shows the mismatch between causal question and descriptive design.
                  blocking_issues:
                    - issue: Causal claim requires a stronger design or narrowed question.
                      required_action: Revise the RQ or change the identification strategy.
                """,
                "Q1 status PASS understates semantic check status FAIL",
            ),
            "Q2": (
                """
                Q2:
                  status: PASS
                  evidence:
                    - RESEARCH/topic/manuscript/manuscript.md
                  semantic_checks:
                    - check_id: q2_claim_evidence_traceability
                      status: FAIL
                      finding: A strong causal claim is supported only by descriptive evidence.
                      evidence_refs:
                        - artifact: RESEARCH/topic/manuscript/claims_evidence_map.md
                          anchor: central-claim-1
                          supports: Shows the overclaim and weak evidence class.
                  blocking_issues:
                    - issue: Central claim overstates the available evidence.
                      required_action: Narrow the claim or add stronger evidence.
                """,
                "Q2 status PASS understates semantic check status FAIL",
            ),
            "Q3": (
                """
                Q3:
                  status: WARN
                  evidence:
                    - RESEARCH/topic/reporting_checklist.md
                  semantic_checks:
                    - check_id: q3_reporting_completeness
                      status: BLOCKED
                      finding: Required reporting items are missing without waivers.
                      evidence_refs: []
                  blocking_issues:
                    - issue: Missing required checklist item and no waiver exists.
                      required_action: Complete the reporting checklist or record a waiver.
                """,
                "Q3 status WARN understates semantic check status BLOCKED",
            ),
            "Q4": (
                """
                Q4:
                  status: PASS
                  evidence:
                    - RESEARCH/topic/code/reproducibility_audit.md
                  semantic_checks:
                    - check_id: q4_reproducibility_baseline
                      status: FAIL
                      finding: A reported result cannot be traced to an input, script, command, or output artifact.
                      evidence_refs:
                        - artifact: RESEARCH/topic/code/reproducibility_audit.md
                          anchor: missing-result-trace
                          supports: Shows the untraceable result.
                  blocking_issues:
                    - issue: Result table lacks input, command, and output provenance.
                      required_action: Add the missing analysis trace or remove the result.
                """,
                "Q4 status PASS understates semantic check status FAIL",
            ),
        }

        for gate_id, (gate_yaml, expected_error) in cases.items():
            with self.subTest(gate_id=gate_id):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    report_path = Path(tmp_dir) / "quality-gate-report.md"
                    write_report(report_path, report_with_gate_override(gate_id, gate_yaml))

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
