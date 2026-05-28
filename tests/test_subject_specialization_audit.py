from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import scripts.audit_subject_specialization as subject_audit
from scripts.audit_subject_specialization import audit_subject_specialization


REPO_ROOT = Path(__file__).resolve().parents[1]


class SubjectSpecializationAuditTests(unittest.TestCase):
    def test_current_subjects_pass_depth_audit(self) -> None:
        self.assertEqual([], audit_subject_specialization(REPO_ROOT))

    def test_focused_output_excludes_unselected_profiles(self) -> None:
        self.assertEqual([], audit_subject_specialization(REPO_ROOT, subjects=["economics"]))

    def test_rogue_focused_profile_is_reported_even_when_not_in_forbidden_set(self) -> None:
        real_materialize = subject_audit.materialize_subject_package

        def materialize_with_rogue_profile(options: subject_audit.MaterializeOptions) -> None:
            real_materialize(options)
            if options.subject == "economics" and options.coverage == "focused":
                profile_root = options.out / "skills" / "domain-profiles"
                profile_root.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(
                    REPO_ROOT / "skills" / "domain-profiles" / "finance.yaml",
                    profile_root / "finance.yaml",
                )

        with patch.object(subject_audit, "materialize_subject_package", side_effect=materialize_with_rogue_profile):
            findings = audit_subject_specialization(REPO_ROOT, subjects=["economics"])

        self.assertIn("unrelated-focused-profile", {finding.code for finding in findings})
        self.assertTrue(
            any("finance.yaml" in finding.message for finding in findings),
            [f"{finding.subject}: {finding.code}: {finding.message}" for finding in findings],
        )

    def test_unknown_subject_reports_clear_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "unknown subject\\(s\\): does-not-exist"):
            audit_subject_specialization(REPO_ROOT, subjects=["does-not-exist"])

    def test_missing_overlay_term_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            self._copy_minimal_repo(temp_root)
            overlay = temp_root / "subjects" / "economics" / "overlays" / "skills" / "manuscript-architect.md"
            overlay.write_text(
                "## Generic Overlay\n\nUse a clear structure and explain the contribution.\n",
                encoding="utf-8",
            )

            findings = audit_subject_specialization(temp_root, subjects=["economics"])

        self.assertTrue(
            any(finding.code == "missing-subject-term" for finding in findings),
            [f"{finding.subject}: {finding.code}: {finding.message}" for finding in findings],
        )

    def _copy_minimal_repo(self, temp_root: Path) -> None:
        for name in ("qiongli-workflow", "skills", "subjects"):
            shutil.copytree(REPO_ROOT / name, temp_root / name)


if __name__ == "__main__":
    unittest.main()
