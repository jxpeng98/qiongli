from __future__ import annotations

import json
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PLUGIN_ROOT = REPO_ROOT / "plugins" / "research-skills"
PLUGIN_SKILL_ROOT = PLUGIN_ROOT / "skills" / "research-paper-workflow"
WORKFLOW_ROOT = REPO_ROOT / "research-paper-workflow"
WORKFLOW_VERSION = (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")


class PluginDistributionContractTests(unittest.TestCase):
    def test_platform_manifests_share_workflow_version(self) -> None:
        codex = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude = json.loads((PLUGIN_ROOT / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude_marketplace = json.loads((REPO_ROOT / ".claude-plugin" / "marketplace.json").read_text(encoding="utf-8"))
        gemini = json.loads((PLUGIN_ROOT / "gemini-extension.json").read_text(encoding="utf-8"))

        self.assertEqual(codex["version"], WORKFLOW_VERSION)
        self.assertEqual(claude["version"], WORKFLOW_VERSION)
        self.assertEqual(claude_marketplace["metadata"]["version"], WORKFLOW_VERSION)
        self.assertEqual(claude_marketplace["plugins"][0]["version"], WORKFLOW_VERSION)
        self.assertEqual(gemini["version"], WORKFLOW_VERSION)

    def test_codex_marketplace_points_to_local_research_skills_plugin(self) -> None:
        marketplace = json.loads((REPO_ROOT / ".agents" / "plugins" / "marketplace.json").read_text(encoding="utf-8"))
        entries = {entry["name"]: entry for entry in marketplace["plugins"]}

        self.assertIn("research-skills", entries)
        self.assertEqual(entries["research-skills"]["source"], {"source": "local", "path": "./plugins/research-skills"})

    def test_codex_plugin_exposes_skill_directory(self) -> None:
        manifest = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["skills"], "./skills/")
        self.assertTrue((PLUGIN_ROOT / "skills").is_dir())

    def test_plugin_package_contains_real_portable_skill_copy(self) -> None:
        self.assertTrue((PLUGIN_SKILL_ROOT / "SKILL.md").is_file())
        self.assertTrue((PLUGIN_SKILL_ROOT / "skills" / "registry.yaml").is_file())
        self.assertFalse(PLUGIN_SKILL_ROOT.is_symlink(), "plugin package must be a real copy, not a symlink")

    def test_sync_script_accepts_all_target_in_dry_run(self) -> None:
        result = subprocess.run(
            ["bash", "scripts/sync_skill_package.sh", "--target", "all", "--dry-run"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        self.assertIn("research-paper-workflow", result.stdout)
        self.assertIn("plugins/research-skills/skills/research-paper-workflow", result.stdout)


if __name__ == "__main__":
    unittest.main()
