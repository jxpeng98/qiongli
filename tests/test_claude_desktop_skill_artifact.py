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
        is_prerelease = self.build_module._is_prerelease_tag(tag)
        skill_name = "qiongli-next" if is_prerelease else "qiongli"
        artifact_prefix = "qiongli-next" if is_prerelease else "qiongli"
        core_artifact_name = f"{artifact_prefix}-claude-desktop-skill-core-{tag}.zip"

        with tempfile.TemporaryDirectory() as tmp:
            artifacts = self.build_module.build_artifacts(REPO_ROOT, tag, Path(tmp))

            desktop_artifacts = {
                artifact.name: artifact
                for artifact in artifacts
                if "claude-desktop-skill" in artifact.name and artifact.suffix == ".zip"
            }
            if is_prerelease:
                expected_desktop_artifacts = [core_artifact_name]
            else:
                expected_desktop_artifacts = [
                    f"qiongli-claude-desktop-skill-{subject}-{tag}.zip"
                    for subject in self.build_module._desktop_subjects(REPO_ROOT)
                ]
                expected_desktop_artifacts.append(f"qiongli-claude-desktop-skill-{tag}.zip")
            self.assertEqual(
                sorted(expected_desktop_artifacts),
                sorted(desktop_artifacts),
            )

            with zipfile.ZipFile(desktop_artifacts[core_artifact_name]) as archive:
                names = set(archive.namelist())
                file_names = [name for name in names if not name.endswith("/")]

                self.assertIn(f"{skill_name}/SKILL.md", names)
                self.assertIn(f"{skill_name}/VERSION", names)
                self.assertIn(f"{skill_name}/SUBJECT", names)
                self.assertIn(f"{skill_name}/skills-core.md", names)
                self.assertIn(f"{skill_name}/skills-summary.md", names)
                self.assertIn(f"{skill_name}/skills/registry.yaml", names)
                self.assertIn(f"{skill_name}/workflows/paper.md", names)
                self.assertNotIn(f"{skill_name}/.claude-plugin/plugin.json", names)
                self.assertFalse(any(name.startswith(f"{skill_name}/commands/") for name in names))
                self.assertLessEqual(len(file_names), self.DESKTOP_FILE_BUDGET)

                detailed_skill_specs = [
                    name
                    for name in file_names
                    if name.startswith(f"{skill_name}/skills/")
                    and name != f"{skill_name}/skills/registry.yaml"
                    and name.endswith(".md")
                ]
                self.assertEqual([], detailed_skill_specs)

                skill_text = archive.read(f"{skill_name}/SKILL.md").decode("utf-8")
                subject_text = archive.read(f"{skill_name}/SUBJECT").decode("utf-8").strip()
                version_text = archive.read(f"{skill_name}/VERSION").decode("utf-8").strip()

        self.assertIn(f"name: {skill_name}", skill_text)
        if is_prerelease:
            self.assertIn(f"${skill_name}", skill_text)
        self.assertIn("Core Workflow Map", skill_text)
        self.assertIn("provider_connected", skill_text)
        self.assertIn("strategy_only", skill_text)
        self.assertIn("MCP settings UI", skill_text)
        self.assertIn("qiongli_config_status", skill_text)
        self.assertIn("qiongli_configure_provider", skill_text)
        self.assertIn("qiongli_save_provider_config", skill_text)
        self.assertIn("qiongli-literature-provider", skill_text)
        self.assertIn(".mcpb", skill_text)
        self.assertIn("manual Desktop install", skill_text)
        self.assertIn("manual MCP install", skill_text)
        self.assertIn("qiongli mcp serve --transport stdio", skill_text)
        self.assertIn("qiongli_task_run", skill_text)
        self.assertIn("agent runtime", skill_text)
        self.assertIn("OpenAlex", skill_text)
        self.assertIn("Semantic Scholar", skill_text)
        self.assertIn("Crossref", skill_text)
        self.assertIn("PubMed", skill_text)
        self.assertIn("OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv", skill_text)
        self.assertNotIn("provider for OpenAlex and Semantic Scholar configuration", skill_text)
        self.assertIn("Platform-native search alone is `native_only`, not `provider_connected`", skill_text)
        self.assertIn("if no provider MCP/MCPB and no platform-native search is available", skill_text)
        self.assertIn("workflows/prompts/templates", skill_text)
        self.assertNotIn("skills/[stage]/[skill-name].md", skill_text)
        self.assertEqual("core", subject_text)
        self.assertEqual(tag, version_text)

    def test_build_artifacts_creates_economics_desktop_skill_zip(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            economics_artifact = self.build_module._build_claude_desktop_skill(
                REPO_ROOT, tag, tmp_path, tmp_path / "work", "economics"
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
            tmp_path = Path(tmp)
            business_artifact = self.build_module._build_claude_desktop_skill(
                REPO_ROOT, tag, tmp_path, tmp_path / "business-work", "business"
            )
            finance_artifact = self.build_module._build_claude_desktop_skill(
                REPO_ROOT, tag, tmp_path, tmp_path / "finance-work", "finance"
            )

            with zipfile.ZipFile(business_artifact) as archive:
                business_names = set(archive.namelist())
                business_file_names = [name for name in business_names if not name.endswith("/")]
                business_subject = archive.read("qiongli/SUBJECT").decode("utf-8").strip()
                business_skill = archive.read(
                    "qiongli/skills/C_design/business-journal-positioning-auditor.md"
                ).decode("utf-8")

            with zipfile.ZipFile(finance_artifact) as archive:
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
