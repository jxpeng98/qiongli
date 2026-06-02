from __future__ import annotations

import tempfile
import textwrap
import unittest
import shutil
from pathlib import Path

from qiongli.source_layout import RepoLayout

from scripts.audit_skill_resource_links import audit_package_resource_links


REPO_ROOT = Path(__file__).resolve().parents[1]


class SkillResourceLinkTests(unittest.TestCase):
    def test_repo_packages_have_no_missing_internal_resource_links(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            package_dir = Path(tmp_dir) / "qiongli-workflow"
            layout = RepoLayout(REPO_ROOT)
            shutil.copytree(layout.workflow, package_dir)
            for rel in ("skills", "templates", "standards", "roles", "venue-profiles"):
                shutil.copytree(
                    layout.resolve_source_path(rel),
                    package_dir / rel,
                    ignore=shutil.ignore_patterns("CLAUDE.project.md"),
                )
            shutil.copy2(layout.skills_core, package_dir / "skills-core.md")
            shutil.copy2(layout.skills_summary, package_dir / "skills-summary.md")

            missing = audit_package_resource_links(package_dir)

        self.assertEqual([], missing)

    def test_audit_reports_missing_template_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            package_dir = Path(tmp_dir) / "qiongli-workflow"
            package_dir.mkdir()
            (package_dir / "SKILL.md").write_text(
                textwrap.dedent(
                    """\
                    # Demo

                    Load `templates/missing-template.md` before writing output.
                    """
                ),
                encoding="utf-8",
            )

            missing = audit_package_resource_links(package_dir)

        self.assertEqual(1, len(missing))
        self.assertEqual("templates/missing-template.md", missing[0].target)


if __name__ == "__main__":
    unittest.main()
