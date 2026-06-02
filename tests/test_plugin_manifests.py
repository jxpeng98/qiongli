from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
PLUGIN_ROOT = REPO_ROOT / "plugins" / "qiongli"
CODEX_PLUGIN_MANIFEST = PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
CLAUDE_PLUGIN_MANIFEST = PLUGIN_ROOT / ".claude-plugin" / "plugin.json"
GEMINI_EXTENSION_MANIFEST = PLUGIN_ROOT / "gemini-extension.json"
WORKFLOW_VERSION = LAYOUT.workflow / "VERSION"


class PluginManifestTests(unittest.TestCase):
    def materialize_plugin_skill(self, tmp_dir: str) -> Path:
        out = Path(tmp_dir) / "dist-source"
        result = subprocess.run(
            [
                sys.executable,
                "scripts/materialize_distribution_payloads.py",
                "--target",
                "plugin",
                "--out",
                str(out),
                "--force",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        return out / "plugins" / "qiongli" / "skills" / "qiongli-workflow"

    def test_plugin_manifest_exposes_workflow_skill(self) -> None:
        manifest = json.loads(CODEX_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["skills"], "./skills/")
        self.assertEqual(manifest["repository"], "https://github.com/jxpeng98/qiongli")
        self.assertEqual(manifest["license"], "MIT")

        interface = manifest["interface"]
        self.assertEqual(interface["displayName"], "Qiongli")
        self.assertEqual(interface["developerName"], "Jiaxin Peng")
        self.assertEqual(interface["category"], "Education")
        self.assertLessEqual(len(interface["defaultPrompt"]), 3)
        for prompt in interface["defaultPrompt"]:
            self.assertLessEqual(len(prompt), 128)
            self.assertNotIn(" /", prompt)
        self.assertTrue(any("$qiongli" in prompt for prompt in interface["defaultPrompt"]))

    def test_claude_plugin_manifest_exposes_workflow_skill(self) -> None:
        manifest = json.loads(CLAUDE_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["repository"], "https://github.com/jxpeng98/qiongli")
        self.assertEqual(manifest["license"], "MIT")

    def test_gemini_extension_manifest_exposes_workflow_skill(self) -> None:
        manifest = json.loads(GEMINI_EXTENSION_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertIn("qiongli-workflow", manifest["description"])

    def test_plugin_contains_discoverable_research_paper_workflow_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_skill = self.materialize_plugin_skill(tmp_dir)

            self.assertTrue((plugin_skill / "SKILL.md").is_file())
            skill_text = (plugin_skill / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn("name: qiongli\n", skill_text)
            self.assertIn("# Qiongli Academic Workflow", skill_text)
            self.assertTrue((plugin_skill / "workflows" / "paper.md").is_file())
            self.assertTrue((plugin_skill / "references" / "workflow-contract.md").is_file())


if __name__ == "__main__":
    unittest.main()
