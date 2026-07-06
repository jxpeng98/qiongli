from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

from scripts.audit_domain_method_packs import audit_domain_profile


REPO_ROOT = Path(__file__).resolve().parents[1]


class DomainMethodPackAuditTests(unittest.TestCase):
    def test_economics_and_finance_profiles_have_executable_method_pack_fields(self) -> None:
        for name in ("economics", "finance"):
            with self.subTest(name=name):
                result = audit_domain_profile(
                    RepoLayout(REPO_ROOT).skills / "domain-profiles" / f"{name}.yaml"
                )

                self.assertEqual([], result.errors)

    def test_economics_and_finance_method_packs_have_enhanced_contract_fields(self) -> None:
        for name in ("economics", "finance"):
            with self.subTest(name=name):
                result = audit_domain_profile(
                    RepoLayout(REPO_ROOT).skills / "domain-profiles" / f"{name}.yaml"
                )

                self.assertEqual([], result.errors)

    def test_invalid_method_pack_reports_missing_enhanced_contract_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "minimal.yaml"
            profile.write_text(
                "\n".join(
                    [
                        "domain: test",
                        "display_name: Test",
                        "libraries: {}",
                        "method_templates:",
                        "  - name: Bare Method",
                        "    tier: standard",
                        "    assumptions: [A]",
                        "    required_diagnostics: [D]",
                        "    required_artifacts: [R]",
                        "    failure_modes: [F]",
                        "    minimum_report_fields: [M]",
                    ]
                ),
                encoding="utf-8",
            )

            result = audit_domain_profile(profile)

        joined = "\n".join(result.errors)
        self.assertIn("canonical_references", joined)
        self.assertIn("gate_relevance", joined)
        self.assertIn("diagnostic_artifacts", joined)
        self.assertIn("failure_triggers", joined)

    def test_invalid_gate_relevance_reports_allowed_quality_gates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "bad-gate.yaml"
            profile.write_text(
                "\n".join(
                    [
                        "domain: test",
                        "display_name: Test",
                        "libraries: {}",
                        "method_templates:",
                        "  - name: Bad Gate Method",
                        "    tier: standard",
                        "    assumptions: [A]",
                        "    required_diagnostics: [D]",
                        "    required_artifacts: [R]",
                        "    failure_modes: [F]",
                        "    minimum_report_fields: [M]",
                        "    canonical_references:",
                        "      - citation_key: example_2024_method",
                        "        role: baseline method anchor",
                        "    gate_relevance: [Q1, Q9]",
                        "    diagnostic_artifacts:",
                        "      - artifact: RESEARCH/[topic]/analysis/example.md",
                        "        required_for: example claims",
                        "    failure_triggers:",
                        "      - missing comparison group blocks causal claims",
                    ]
                ),
                encoding="utf-8",
            )

            result = audit_domain_profile(profile)

        self.assertIn("gate_relevance contains unsupported gate: Q9", "\n".join(result.errors))

    def test_invalid_method_pack_reports_missing_required_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "minimal.yaml"
            profile.write_text(
                "\n".join(
                    [
                        "domain: test",
                        "display_name: Test",
                        "libraries: {}",
                        "method_templates:",
                        "  - name: Bare Method",
                        "    tier: standard",
                    ]
                ),
                encoding="utf-8",
            )

            result = audit_domain_profile(profile)

        joined = "\n".join(result.errors)
        self.assertIn("Bare Method", joined)
        self.assertIn("required_diagnostics", joined)
        self.assertIn("missing or empty", joined)

    def test_missing_profile_reports_error_without_traceback(self) -> None:
        result = audit_domain_profile(Path("/definitely/missing/domain-profile.yaml"))

        self.assertTrue(result.errors)
        self.assertIn("Failed to read profile", result.errors[0])

    def test_malformed_yaml_reports_error_without_traceback(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "malformed.yaml"
            profile.write_text("domain: [unterminated\n", encoding="utf-8")

            result = audit_domain_profile(profile)

        self.assertTrue(result.errors)
        self.assertIn("Malformed YAML", result.errors[0])


if __name__ == "__main__":
    unittest.main()
