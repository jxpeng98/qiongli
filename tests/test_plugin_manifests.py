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
PLUGIN_ROOT = LAYOUT.plugin_package
NEXT_PLUGIN_ROOT = LAYOUT.next_plugin_package
CODEX_PLUGIN_MANIFEST = PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
CLAUDE_PLUGIN_MANIFEST = PLUGIN_ROOT / ".claude-plugin" / "plugin.json"
GEMINI_EXTENSION_MANIFEST = PLUGIN_ROOT / "gemini-extension.json"
NEXT_CODEX_PLUGIN_MANIFEST = NEXT_PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
WORKFLOW_VERSION = LAYOUT.workflow / "VERSION"
EXPECTED_AUTHOR = {"name": "Jiaxin Peng", "url": "https://github.com/jxpeng98"}
EXPECTED_CATEGORY = "Education"
EXPECTED_REPOSITORY = "https://github.com/jxpeng98/qiongli"
EXPECTED_LICENSE = "MIT"
EXPECTED_WORKFLOW_DESCRIPTION = (
    "Qiongli academic research workflow for paper planning, literature review, writing, "
    "compliance, submission, presentation, and research code."
)


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
        self.assertEqual(manifest["description"], EXPECTED_WORKFLOW_DESCRIPTION)
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(manifest["category"], EXPECTED_CATEGORY)

        interface = manifest["interface"]
        self.assertEqual(interface["displayName"], "Qiongli")
        self.assertEqual(interface["developerName"], "Jiaxin Peng")
        self.assertEqual(interface["category"], EXPECTED_CATEGORY)
        self.assertIn("academic research workflow", interface["shortDescription"].lower())
        self.assertIn("paper planning", interface["longDescription"].lower())
        self.assertLessEqual(len(interface["defaultPrompt"]), 3)
        for prompt in interface["defaultPrompt"]:
            self.assertLessEqual(len(prompt), 128)
            self.assertNotIn(" /", prompt)
        self.assertTrue(any("$qiongli" in prompt for prompt in interface["defaultPrompt"]))

    def test_codex_plugin_bundles_qiongli_mcp_server(self) -> None:
        manifest = json.loads(CODEX_PLUGIN_MANIFEST.read_text(encoding="utf-8"))
        mcp_manifest_path = PLUGIN_ROOT / ".mcp.json"
        mcp_manifest = json.loads(mcp_manifest_path.read_text(encoding="utf-8"))

        self.assertEqual(manifest["mcpServers"], "./.mcp.json")
        server = mcp_manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "node")
        self.assertEqual(server["args"], ["./mcp/qiongli-literature-provider/index.mjs"])
        self.assertEqual(server["cwd"], ".")
        self.assertEqual(server["startup_timeout_sec"], 20)
        self.assertEqual(server["tool_timeout_sec"], 60)
        self.assertNotIn("env", server)
        self.assertTrue((PLUGIN_ROOT / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file())
        self.assertTrue((PLUGIN_ROOT / "mcp" / "qiongli-literature-provider" / "query.mjs").is_file())
        self.assertNotIn("QIONGLI_OPENALEX_EMAIL", json.dumps(mcp_manifest))
        self.assertNotIn("SEMANTIC_SCHOLAR_API_KEY", json.dumps(mcp_manifest))
        self.assertNotIn("qiongli mcp", json.dumps(mcp_manifest))

    def test_claude_plugin_manifest_exposes_workflow_skill(self) -> None:
        manifest = json.loads(CLAUDE_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["description"], EXPECTED_WORKFLOW_DESCRIPTION)
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(manifest["license"], EXPECTED_LICENSE)

    def test_claude_plugin_bundles_qiongli_mcp_server(self) -> None:
        manifest = json.loads(CLAUDE_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        self.assertIn("mcpServers", manifest)
        self.assertIn("qiongli", manifest["mcpServers"])
        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
        self.assertNotIn("env", server)
        self.assertTrue((PLUGIN_ROOT / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file())
        self.assertTrue((PLUGIN_ROOT / "mcp" / "qiongli-literature-provider" / "query.mjs").is_file())
        manifest_text = json.dumps(manifest)
        self.assertNotIn("QIONGLI_OPENALEX_EMAIL", manifest_text)
        self.assertNotIn("SEMANTIC_SCHOLAR_API_KEY", manifest_text)
        self.assertNotIn("qiongli mcp", manifest_text)

    def test_gemini_extension_manifest_exposes_workflow_skill(self) -> None:
        manifest = json.loads(GEMINI_EXTENSION_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["description"], EXPECTED_WORKFLOW_DESCRIPTION)
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(manifest["license"], EXPECTED_LICENSE)

    def test_next_codex_plugin_manifest_uses_release_metadata(self) -> None:
        manifest = json.loads(NEXT_CODEX_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli-next")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(manifest["license"], EXPECTED_LICENSE)
        self.assertIn("prerelease academic research workflow", manifest["description"].lower())
        self.assertEqual(manifest["interface"]["developerName"], EXPECTED_AUTHOR["name"])
        self.assertEqual(manifest["interface"]["category"], EXPECTED_CATEGORY)

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
