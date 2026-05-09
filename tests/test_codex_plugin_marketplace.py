from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MARKETPLACE_PATH = REPO_ROOT / ".agents" / "plugins" / "marketplace.json"
CLAUDE_MARKETPLACE_PATH = REPO_ROOT / ".claude-plugin" / "marketplace.json"
PLUGIN_ROOT = REPO_ROOT / "plugins" / "qiongli"
CODEX_PLUGIN_MANIFEST = PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
CLAUDE_PLUGIN_MANIFEST = PLUGIN_ROOT / ".claude-plugin" / "plugin.json"
GEMINI_EXTENSION_MANIFEST = PLUGIN_ROOT / "gemini-extension.json"
PLUGIN_SKILL = PLUGIN_ROOT / "skills" / "qiongli-workflow"
WORKFLOW_VERSION = REPO_ROOT / "qiongli-workflow" / "VERSION"


class CodexPluginMarketplaceTests(unittest.TestCase):
    def test_repo_marketplace_declares_qiongli_plugin(self) -> None:
        marketplace = json.loads(MARKETPLACE_PATH.read_text(encoding="utf-8"))

        self.assertEqual(marketplace["name"], "qiongli")
        self.assertEqual(marketplace["interface"]["displayName"], "Qiongli")

        entries = {entry["name"]: entry for entry in marketplace["plugins"]}
        entry = entries["qiongli"]
        self.assertEqual(entry["source"], {"source": "local", "path": "./plugins/qiongli"})
        self.assertEqual(
            entry["policy"],
            {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
        )
        self.assertEqual(entry["category"], "Education")

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

    def test_claude_marketplace_declares_qiongli_plugin(self) -> None:
        marketplace = json.loads(CLAUDE_MARKETPLACE_PATH.read_text(encoding="utf-8"))

        self.assertEqual(marketplace["name"], "qiongli")
        self.assertEqual(marketplace["owner"]["name"], "Jiaxin Peng")
        entries = {entry["name"]: entry for entry in marketplace["plugins"]}
        entry = entries["qiongli"]
        self.assertEqual(entry["source"], "./plugins/qiongli")
        self.assertEqual(entry["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(entry["category"], "education")

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
        self.assertIn("name: qiongli-workflow", skill_text)
        self.assertTrue((PLUGIN_SKILL / "workflows" / "paper.md").is_file())
        self.assertTrue((PLUGIN_SKILL / "references" / "workflow-contract.md").is_file())


if __name__ == "__main__":
    unittest.main()
