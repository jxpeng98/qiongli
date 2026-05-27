from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.audit_subject_eval_cases import audit_subject_eval_cases, load_subject_eval_cases


REPO_ROOT = Path(__file__).resolve().parents[1]
CASE_DIR = REPO_ROOT / "evals" / "subject-specialization" / "cases"


class SubjectEvalCaseTests(unittest.TestCase):
    def test_eval_cases_load(self) -> None:
        cases = load_subject_eval_cases(CASE_DIR)
        case_ids = {case.id for case in cases}

        self.assertIn("economics-did-identification", case_ids)
        self.assertIn("economics-accounting-disclosure-study", case_ids)

    def test_eval_cases_pass_against_materialized_outputs(self) -> None:
        self.assertEqual(audit_subject_eval_cases(REPO_ROOT), [])

    def test_missing_expected_term_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            case_dir = Path(tmp_dir)
            (case_dir / "missing-term.yaml").write_text(
                "\n".join(
                    [
                        "id: missing-term",
                        "subject: economics",
                        "coverage: focused",
                        'prompt: "Probe expected term reporting."',
                        "expected_skill_refs:",
                        "  - stats-engine",
                        "expected_terms:",
                        "  - term-that-does-not-exist",
                        "expected_domain_profiles:",
                        "  - economics.yaml",
                        "forbidden_domain_profiles: []",
                    ]
                ),
                encoding="utf-8",
            )

            findings = audit_subject_eval_cases(REPO_ROOT, case_dir=case_dir)

        self.assertIn("missing-expected-term", {finding.code for finding in findings})


if __name__ == "__main__":
    unittest.main()
