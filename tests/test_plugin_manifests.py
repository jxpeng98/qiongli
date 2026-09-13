from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.distribution_metadata import load_plugin_distribution
from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
PLUGIN_ROOT = LAYOUT.plugin_package
NEXT_PLUGIN_ROOT = LAYOUT.next_plugin_package
CODEX_PLUGIN_MANIFEST = PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
CLAUDE_PLUGIN_MANIFEST = PLUGIN_ROOT / ".claude-plugin" / "plugin.json"
NEXT_CODEX_PLUGIN_MANIFEST = NEXT_PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
WORKFLOW_VERSION = LAYOUT.workflow / "VERSION"
EXPECTED_AUTHOR = {"name": "Jiaxin Peng", "url": "https://github.com/jxpeng98"}
EXPECTED_CATEGORY = "Education"
EXPECTED_REPOSITORY = "https://github.com/jxpeng98/qiongli"
EXPECTED_LICENSE = "MIT"
EXPECTED_WORKFLOW_DESCRIPTION = (
    "Qiongli academic research workflow for literature, manuscripts, statistics, "
    "analysis code, reproducibility, rebuttal, submission, presentation, and stage-aware grill."
)
EXPECTED_DISCOVERY_TERMS = (
    "academic",
    "research",
    "literature",
    "manuscript",
    "analysis",
    "statistics",
    "reproducibility",
    "rebuttal",
)


class PluginDistributionMetadataTests(unittest.TestCase):
    def test_canonical_distribution_metadata_defines_stable_and_next_plugins(self) -> None:
        metadata = load_plugin_distribution(REPO_ROOT)

        self.assertEqual(set(metadata.plugins), {"qiongli", "qiongli-next"})
        self.assertEqual(metadata.plugins["qiongli"].skill_name, "qiongli")
        self.assertEqual(metadata.plugins["qiongli-next"].skill_name, "qiongli-next")
        self.assertEqual(metadata.plugins["qiongli"].mcp_server_name, "qiongli")
        self.assertEqual(metadata.plugins["qiongli-next"].mcp_server_name, "qiongli-next")
        self.assertEqual(metadata.plugins["qiongli"].release_lines, ("legacy-1x",))
        self.assertEqual(metadata.plugins["qiongli"].release_channels, ("stable",))
        self.assertEqual(
            metadata.plugins["qiongli-next"].release_lines,
            ("legacy-1x",),
        )
        self.assertEqual(metadata.plugins["qiongli-next"].release_channels, ("beta",))
        self.assertEqual(
            metadata.plugins["qiongli-next"].planned_release_lines,
            ("native-2x",),
        )
        self.assertEqual(
            metadata.plugins["qiongli-next"].planned_release_channels,
            ("alpha", "beta"),
        )

    def test_canonical_distribution_metadata_carries_discovery_terms(self) -> None:
        metadata = load_plugin_distribution(REPO_ROOT)
        stable = metadata.plugins["qiongli"]
        searchable_text = " ".join([stable.description, *stable.keywords, *stable.default_prompts]).lower()

        for term in EXPECTED_DISCOVERY_TERMS:
            with self.subTest(term=term):
                self.assertIn(term, searchable_text)


class PluginManifestTests(unittest.TestCase):
    def materialize_plugin_root(self, tmp_dir: str) -> Path:
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
        return out / "plugins" / "qiongli"

    def materialize_next_plugin_root(self, tmp_dir: str) -> Path:
        out = Path(tmp_dir) / "dist-source"
        result = subprocess.run(
            [
                sys.executable,
                "scripts/materialize_distribution_payloads.py",
                "--target",
                "next-plugin",
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
        return out / "plugins" / "qiongli-next"

    def materialize_plugin_skill(self, tmp_dir: str) -> Path:
        return self.materialize_plugin_root(tmp_dir) / "skills" / "qiongli-workflow"

    def test_plugin_manifest_exposes_workflow_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["skills"], "./skills/")
        self.assertEqual(manifest["repository"], "https://github.com/jxpeng98/qiongli")
        self.assertEqual(manifest["license"], "MIT")
        self.assertEqual(manifest["description"], EXPECTED_WORKFLOW_DESCRIPTION)
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertNotIn("category", manifest)

        interface = manifest["interface"]
        self.assertEqual(interface["displayName"], "Qiongli")
        self.assertEqual(interface["developerName"], "Jiaxin Peng")
        self.assertEqual(interface["category"], EXPECTED_CATEGORY)
        self.assertIn("academic research workflow", interface["shortDescription"].lower())
        self.assertIn("manuscript", interface["longDescription"].lower())
        self.assertIn("analysis code", interface["longDescription"].lower())
        self.assertLessEqual(len(interface["defaultPrompt"]), 3)
        for prompt in interface["defaultPrompt"]:
            self.assertLessEqual(len(prompt), 128)
            self.assertNotIn(" /", prompt)
        self.assertTrue(any("$qiongli" in prompt for prompt in interface["defaultPrompt"]))

    def test_codex_plugin_manifest_exposes_academic_discovery_terms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        searchable_text = " ".join(
            [
                manifest["description"],
                " ".join(manifest["keywords"]),
                manifest["interface"]["longDescription"],
                " ".join(manifest["interface"]["defaultPrompt"]),
            ]
        ).lower()

        for term in EXPECTED_DISCOVERY_TERMS:
            with self.subTest(term=term):
                self.assertIn(term, searchable_text)

    def test_codex_plugin_bundles_qiongli_mcp_server(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
            mcp_manifest = json.loads((plugin_root / ".mcp.json").read_text(encoding="utf-8"))
            mcp_entrypoint_exists = (plugin_root / "bin" / "qiongli-literature-provider").is_file()

        self.assertEqual(manifest["mcpServers"], "./.mcp.json")
        server = mcp_manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "./bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])
        self.assertNotEqual(server["command"], "qiongli")
        self.assertNotIn("mcp serve", json.dumps(server))
        self.assertEqual(server["cwd"], ".")
        self.assertEqual(server["startup_timeout_sec"], 20)
        self.assertEqual(server["tool_timeout_sec"], 60)
        self.assertNotIn("env", server)
        self.assertTrue(mcp_entrypoint_exists)
        self.assertNotIn("QIONGLI_OPENALEX_EMAIL", json.dumps(mcp_manifest))
        self.assertNotIn("SEMANTIC_SCHOLAR_API_KEY", json.dumps(mcp_manifest))
        self.assertNotIn("qiongli mcp", json.dumps(mcp_manifest))

    def test_claude_plugin_manifest_exposes_workflow_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["description"], EXPECTED_WORKFLOW_DESCRIPTION)
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(manifest["license"], EXPECTED_LICENSE)

    def test_claude_plugin_manifest_exposes_academic_discovery_terms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        searchable_text = " ".join([manifest["description"], " ".join(manifest["keywords"])]).lower()

        for term in EXPECTED_DISCOVERY_TERMS:
            with self.subTest(term=term):
                self.assertIn(term, searchable_text)

    def test_claude_plugin_bundles_qiongli_mcp_server(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
            mcp_entrypoint_exists = (plugin_root / "bin" / "qiongli-literature-provider").is_file()

        self.assertIn("mcpServers", manifest)
        self.assertIn("qiongli", manifest["mcpServers"])
        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider")
        self.assertEqual(
            server["args"],
            ["--transport", "stdio"],
        )
        self.assertNotEqual(server["command"], "qiongli")
        self.assertNotIn("mcp serve", json.dumps(server))
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
        self.assertNotIn("env", server)
        self.assertTrue(mcp_entrypoint_exists)
        manifest_text = json.dumps(manifest)
        self.assertNotIn("QIONGLI_OPENALEX_EMAIL", manifest_text)
        self.assertNotIn("SEMANTIC_SCHOLAR_API_KEY", manifest_text)
        self.assertNotIn("qiongli mcp", manifest_text)

    def test_plugin_package_does_not_generate_gemini_extension_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_plugin_root(tmp_dir)

        self.assertFalse((plugin_root / "gemini-extension.json").exists())

    def test_next_codex_plugin_manifest_uses_release_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_next_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli-next")
        self.assertEqual(manifest["version"], WORKFLOW_VERSION.read_text(encoding="utf-8").strip().lstrip("v"))
        self.assertEqual(manifest["author"], EXPECTED_AUTHOR)
        self.assertNotIn("category", manifest)
        self.assertEqual(manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(manifest["license"], EXPECTED_LICENSE)
        self.assertIn("prerelease academic research workflow", manifest["description"].lower())
        self.assertEqual(manifest["interface"]["developerName"], EXPECTED_AUTHOR["name"])
        self.assertEqual(manifest["interface"]["category"], EXPECTED_CATEGORY)

    def test_next_codex_plugin_manifest_exposes_academic_discovery_terms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_root = self.materialize_next_plugin_root(tmp_dir)
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        searchable_text = " ".join(
            [
                manifest["description"],
                " ".join(manifest["keywords"]),
                manifest["interface"]["longDescription"],
                " ".join(manifest["interface"]["defaultPrompt"]),
            ]
        ).lower()

        for term in EXPECTED_DISCOVERY_TERMS:
            with self.subTest(term=term):
                self.assertIn(term, searchable_text)

    def test_plugin_contains_discoverable_research_paper_workflow_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            plugin_skill = self.materialize_plugin_skill(tmp_dir)

            self.assertTrue((plugin_skill / "SKILL.md").is_file())
            skill_text = (plugin_skill / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn("name: qiongli\n", skill_text)
            frontmatter = skill_text.split("---", 2)[1]
            skill_metadata = yaml.safe_load(frontmatter)
            self.assertEqual(skill_metadata["name"], "qiongli")
            self.assertIn("Qiongli version:", skill_metadata["description"])
            self.assertIn("# Qiongli Academic Workflow", skill_text)
            self.assertTrue((plugin_skill / "workflows" / "paper.md").is_file())
            self.assertTrue((plugin_skill / "references" / "workflow-contract.md").is_file())


if __name__ == "__main__":
    unittest.main()
