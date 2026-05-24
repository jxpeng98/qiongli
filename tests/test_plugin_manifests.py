from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PLUGIN_ROOT = REPO_ROOT / "plugins" / "qiongli"
CODEX_PLUGIN_MANIFEST = PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
CLAUDE_PLUGIN_MANIFEST = PLUGIN_ROOT / ".claude-plugin" / "plugin.json"
GEMINI_EXTENSION_MANIFEST = PLUGIN_ROOT / "gemini-extension.json"
PLUGIN_SKILL = PLUGIN_ROOT / "skills" / "qiongli-workflow"
WORKFLOW_VERSION = REPO_ROOT / "qiongli-workflow" / "VERSION"


class PluginManifestTests(unittest.TestCase):
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
        self.assertTrue((PLUGIN_SKILL / "SKILL.md").is_file())
        skill_text = (PLUGIN_SKILL / "SKILL.md").read_text(encoding="utf-8")
        self.assertIn("name: qiongli\n", skill_text)
        self.assertIn("# Qiongli Academic Workflow", skill_text)
        self.assertTrue((PLUGIN_SKILL / "workflows" / "paper.md").is_file())
        self.assertTrue((PLUGIN_SKILL / "references" / "workflow-contract.md").is_file())


if __name__ == "__main__":
    unittest.main()
