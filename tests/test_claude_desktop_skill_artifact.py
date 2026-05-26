from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
BUILD_ARTIFACTS_PATH = REPO_ROOT / "scripts" / "build_plugin_artifacts.py"


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
        tag = (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip()

        with tempfile.TemporaryDirectory() as tmp:
            artifacts = self.build_module.build_artifacts(REPO_ROOT, tag, Path(tmp))

            desktop_artifacts = [
                artifact for artifact in artifacts if artifact.name == f"qiongli-claude-desktop-skill-{tag}.zip"
            ]
            self.assertEqual(1, len(desktop_artifacts))

            with zipfile.ZipFile(desktop_artifacts[0]) as archive:
                names = set(archive.namelist())
                file_names = [name for name in names if not name.endswith("/")]

                self.assertIn("qiongli/SKILL.md", names)
                self.assertIn("qiongli/VERSION", names)
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
                version_text = archive.read("qiongli/VERSION").decode("utf-8").strip()

        self.assertIn("name: qiongli", skill_text)
        self.assertIn("Claude Desktop / Claude.ai slim package", skill_text)
        self.assertNotIn("skills/[stage]/[skill-name].md", skill_text)
        self.assertEqual(tag, version_text)


if __name__ == "__main__":
    unittest.main()
