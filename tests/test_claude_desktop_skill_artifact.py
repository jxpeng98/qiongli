from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
BUILD_ARTIFACTS_PATH = RepoLayout(REPO_ROOT).scripts / "build_plugin_artifacts.py"


def _load_build_artifacts_module():
    spec = importlib.util.spec_from_file_location("build_plugin_artifacts", BUILD_ARTIFACTS_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {BUILD_ARTIFACTS_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ClaudeDesktopSkillArtifactTests(unittest.TestCase):
    DESKTOP_FILE_BUDGET = 180

    @classmethod
    def setUpClass(cls) -> None:
        cls.build_module = _load_build_artifacts_module()

    def test_build_artifacts_creates_claude_desktop_skill_zip(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()

        with tempfile.TemporaryDirectory() as tmp:
            artifacts = self.build_module.build_artifacts(REPO_ROOT, tag, Path(tmp))

            desktop_artifacts = {artifact.name: artifact for artifact in artifacts if artifact.suffix == ".zip"}
            self.assertIn(f"qiongli-claude-desktop-skill-{tag}.zip", desktop_artifacts)
            self.assertIn(f"qiongli-claude-desktop-skill-business-{tag}.zip", desktop_artifacts)
            self.assertIn(f"qiongli-claude-desktop-skill-core-{tag}.zip", desktop_artifacts)
            self.assertIn(f"qiongli-claude-desktop-skill-economics-{tag}.zip", desktop_artifacts)
            self.assertIn(f"qiongli-claude-desktop-skill-finance-{tag}.zip", desktop_artifacts)

            with zipfile.ZipFile(desktop_artifacts[f"qiongli-claude-desktop-skill-core-{tag}.zip"]) as archive:
                names = set(archive.namelist())
                file_names = [name for name in names if not name.endswith("/")]

                self.assertIn("qiongli/SKILL.md", names)
                self.assertIn("qiongli/VERSION", names)
                self.assertIn("qiongli/SUBJECT", names)
                self.assertIn("qiongli/skills-core.md", names)
                self.assertIn("qiongli/skills-summary.md", names)
                self.assertIn("qiongli/skills/registry.yaml", names)
                self.assertIn("qiongli/workflows/paper.md", names)
                self.assertNotIn("qiongli/.claude-plugin/plugin.json", names)
                self.assertFalse(any(name.startswith("qiongli/commands/") for name in names))
                self.assertLessEqual(len(file_names), self.DESKTOP_FILE_BUDGET)

                detailed_skill_specs = [
                    name
                    for name in file_names
                    if name.startswith("qiongli/skills/")
                    and name != "qiongli/skills/registry.yaml"
                    and name.endswith(".md")
                ]
                self.assertEqual([], detailed_skill_specs)

                skill_text = archive.read("qiongli/SKILL.md").decode("utf-8")
                subject_text = archive.read("qiongli/SUBJECT").decode("utf-8").strip()
                version_text = archive.read("qiongli/VERSION").decode("utf-8").strip()

        self.assertIn("name: qiongli", skill_text)
        self.assertIn("Core Workflow Map", skill_text)
        self.assertIn("provider_connected", skill_text)
        self.assertIn("strategy_only", skill_text)
        self.assertIn("qiongli-literature-provider", skill_text)
        self.assertIn(".mcpb", skill_text)
        self.assertIn("OpenAlex", skill_text)
        self.assertIn("Semantic Scholar", skill_text)
        self.assertIn("If no MCPB or platform-native search is available", skill_text)
        self.assertIn("workflows/prompts/templates", skill_text)
        self.assertNotIn("skills/[stage]/[skill-name].md", skill_text)
        self.assertEqual("core", subject_text)
        self.assertEqual(tag, version_text)

    def test_build_artifacts_creates_economics_desktop_skill_zip(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()

        with tempfile.TemporaryDirectory() as tmp:
            artifacts = self.build_module.build_artifacts(REPO_ROOT, tag, Path(tmp))
            economics_artifact = next(
                artifact for artifact in artifacts if artifact.name == f"qiongli-claude-desktop-skill-economics-{tag}.zip"
            )

            with zipfile.ZipFile(economics_artifact) as archive:
                names = set(archive.namelist())
                file_names = [name for name in names if not name.endswith("/")]

                self.assertIn("qiongli/SUBJECT", names)
                self.assertIn("qiongli/skills/C_design/econ-identification-auditor.md", names)
                self.assertIn("qiongli/skills/F_writing/manuscript-architect.md", names)
                self.assertIn("qiongli/skills/I_code/stats-engine.md", names)
                self.assertIn("qiongli/skills/domain-profiles/economics.yaml", names)
                self.assertNotIn("qiongli/skills/domain-profiles/cs-ai.yaml", names)
                self.assertNotIn("qiongli/venue-profiles/acl.yaml", names)
                self.assertLessEqual(len(file_names), self.DESKTOP_FILE_BUDGET)

                subject_text = archive.read("qiongli/SUBJECT").decode("utf-8").strip()
                manuscript_text = archive.read("qiongli/skills/F_writing/manuscript-architect.md").decode("utf-8")
                stats_text = archive.read("qiongli/skills/I_code/stats-engine.md").decode("utf-8")

        self.assertEqual("economics", subject_text)
        self.assertIn("## Economics Overlay", manuscript_text)
        self.assertIn("Naive TWFE under staggered adoption", stats_text)

    def test_build_artifacts_creates_business_and_finance_desktop_skill_zips(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()

        with tempfile.TemporaryDirectory() as tmp:
            artifacts = self.build_module.build_artifacts(REPO_ROOT, tag, Path(tmp))
            desktop_artifacts = {artifact.name: artifact for artifact in artifacts if artifact.suffix == ".zip"}

            with zipfile.ZipFile(desktop_artifacts[f"qiongli-claude-desktop-skill-business-{tag}.zip"]) as archive:
                business_names = set(archive.namelist())
                business_file_names = [name for name in business_names if not name.endswith("/")]
                business_subject = archive.read("qiongli/SUBJECT").decode("utf-8").strip()
                business_skill = archive.read(
                    "qiongli/skills/C_design/business-journal-positioning-auditor.md"
                ).decode("utf-8")

            with zipfile.ZipFile(desktop_artifacts[f"qiongli-claude-desktop-skill-finance-{tag}.zip"]) as archive:
                finance_names = set(archive.namelist())
                finance_file_names = [name for name in finance_names if not name.endswith("/")]
                finance_subject = archive.read("qiongli/SUBJECT").decode("utf-8").strip()
                finance_skill = archive.read(
                    "qiongli/skills/C_design/finance-identification-risk-auditor.md"
                ).decode("utf-8")

        self.assertEqual("business", business_subject)
        self.assertIn("qiongli/skills/domain-profiles/business-management.yaml", business_names)
        self.assertIn("qiongli/venue-profiles/academy-of-management-journal.yaml", business_names)
        self.assertIn("doctoral-level journal contribution", business_skill)
        self.assertLessEqual(len(business_file_names), self.DESKTOP_FILE_BUDGET)

        self.assertEqual("finance", finance_subject)
        self.assertIn("qiongli/skills/domain-profiles/finance.yaml", finance_names)
        self.assertIn("qiongli/venue-profiles/journal-of-finance.yaml", finance_names)
        self.assertIn("risk-adjusted", finance_skill)
        self.assertLessEqual(len(finance_file_names), self.DESKTOP_FILE_BUDGET)


if __name__ == "__main__":
    unittest.main()
