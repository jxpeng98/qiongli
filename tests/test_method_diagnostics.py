from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.audit_method_diagnostics import audit_method_diagnostic_report


REPO_ROOT = Path(__file__).resolve().parents[1]


class MethodDiagnosticsTests(unittest.TestCase):
    def test_stage_c_skills_reference_method_diagnostics(self) -> None:
        expected_tokens = (
            "method-diagnostic-report.md",
            "validity-threat-matrix.md",
            "construct validity",
            "data leakage",
        )
        for relative in (
            "skills/C_design/study-designer.md",
            "skills/C_design/robustness-planner.md",
            "skills/C_design/variable-operationalizer.md",
        ):
            content = (REPO_ROOT / relative).read_text(encoding="utf-8")
            for token in expected_tokens:
                with self.subTest(relative=relative, token=token):
                    self.assertIn(token, content)

    def test_complete_method_diagnostic_report_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "method-diagnostic-report.md"
            report_path.write_text(
                textwrap.dedent(
                    """\
                    # Method Diagnostic Report

                    ## Design Summary
                    Summary.

                    ## Validity Threat Matrix
                    | Threat | Risk | Mitigation |
                    |---|---|---|
                    | construct validity | medium | triangulate measures |

                    ## Method-Specific Checks
                    Checks.

                    ## Insufficient Input Notes
                    None.
                    """
                ),
                encoding="utf-8",
            )

            result = audit_method_diagnostic_report(report_path)

        self.assertEqual([], result.errors)


if __name__ == "__main__":
    unittest.main()
