from __future__ import annotations

import json
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class QiongliNamingTests(unittest.TestCase):
    def test_python_distribution_and_cli_use_qiongli(self) -> None:
        metadata = tomllib.loads((REPO_ROOT / "pyproject.toml").read_text(encoding="utf-8"))

        self.assertEqual(metadata["project"]["name"], "qiongli-installer")
        scripts = metadata["project"]["scripts"]
        self.assertEqual(scripts["qiongli"], "qiongli.cli:main")
        self.assertEqual(scripts["ql"], "qiongli.cli:main")
        self.assertEqual(scripts["research-skills"], "qiongli.cli:main")
        self.assertEqual(scripts["rsk"], "qiongli.cli:main")
        self.assertEqual(scripts["rsw"], "qiongli.cli:main")

    def test_plugin_manifests_use_qiongli_public_identity(self) -> None:
        marketplace = json.loads((REPO_ROOT / ".agents" / "plugins" / "marketplace.json").read_text(encoding="utf-8"))
        codex_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude_marketplace = json.loads((REPO_ROOT / ".claude-plugin" / "marketplace.json").read_text(encoding="utf-8"))
        claude_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        gemini_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / "gemini-extension.json").read_text(encoding="utf-8"))

        self.assertEqual(marketplace["name"], "qiongli")
        self.assertEqual(marketplace["interface"]["displayName"], "Qiongli")
        self.assertEqual(marketplace["plugins"][0]["name"], "qiongli")
        self.assertEqual(marketplace["plugins"][0]["source"], {"source": "local", "path": "./plugins/qiongli"})

        for manifest in (codex_manifest, claude_manifest, gemini_manifest):
            self.assertEqual(manifest["name"], "qiongli")
            self.assertIn("Qiongli", manifest["description"])

        self.assertEqual(claude_marketplace["name"], "qiongli")
        self.assertEqual(claude_marketplace["plugins"][0]["source"], "./plugins/qiongli")

    def test_portable_skill_id_uses_qiongli_workflow(self) -> None:
        skill_root = REPO_ROOT / "qiongli-workflow"
        skill_text = (skill_root / "SKILL.md").read_text(encoding="utf-8")

        self.assertIn("name: qiongli-workflow", skill_text)
        self.assertTrue((REPO_ROOT / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "SKILL.md").is_file())

        manifest = (REPO_ROOT / "install" / "install_manifest.tsv").read_text(encoding="utf-8")
        self.assertIn("qiongli-workflow\t${CODEX_HOME}/skills/qiongli-workflow", manifest)
        self.assertNotIn("research-paper-workflow\t${CODEX_HOME}/skills/research-paper-workflow", manifest)


if __name__ == "__main__":
    unittest.main()
