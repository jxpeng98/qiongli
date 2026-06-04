from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = REPO_ROOT / "packages" / "qiongli-literature-mcpb"


class LiteratureMCPBArtifactTests(unittest.TestCase):
    def test_literature_mcpb_manifest_declares_sensitive_config(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["manifest_version"], "0.3")
        self.assertEqual(manifest["name"], "qiongli-literature-provider")
        self.assertEqual(manifest["display_name"], "Qiongli Literature Provider")
        self.assertEqual(manifest["server"]["type"], "node")
        self.assertEqual(manifest["server"]["entry_point"], "server/index.mjs")
        self.assertIs(manifest["user_config"]["semantic_scholar_api_key"]["sensitive"], True)
        self.assertEqual(manifest["user_config"]["openalex_email"]["type"], "string")
        self.assertEqual(manifest["user_config"]["default_result_limit"]["default"], 10)
        self.assertIn("qiongli_literature_search", {tool["name"] for tool in manifest["tools"]})

    def test_literature_mcpb_manifest_server_entry_exists(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
        server_entry = PACKAGE_ROOT / manifest["server"]["entry_point"]

        self.assertTrue(server_entry.is_file())

    def test_literature_mcpb_manifest_declares_expected_tools(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))

        self.assertEqual(
            {tool["name"] for tool in manifest["tools"]},
            {
                "qiongli_literature_status",
                "qiongli_config_status",
                "qiongli_save_provider_config",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
            },
        )

    def test_literature_mcpb_manifest_package_structure_is_consistent(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
        package = json.loads((PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli-literature-provider")
        self.assertEqual(manifest["version"], package["version"])
        self.assertEqual(package["type"], "module")
        self.assertIs(package["private"], True)
        self.assertEqual(package["scripts"]["start"], "node server/index.mjs")
        self.assertEqual(package.get("dependencies", {}), {})

    def test_literature_mcpb_runs_without_qiongli_cli_or_npm_install(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
        package = json.loads((PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))
        server_index = (PACKAGE_ROOT / "server" / "index.mjs").read_text(encoding="utf-8")

        mcp_config = manifest["server"]["mcp_config"]
        serialized_manifest = json.dumps(manifest)
        self.assertEqual(mcp_config["command"], "node")
        self.assertNotIn("qiongli mcp", serialized_manifest)
        self.assertNotIn("qiongli", mcp_config["command"])
        self.assertEqual(package.get("dependencies", {}), {})
        self.assertNotIn("@modelcontextprotocol/sdk", server_index)

    def test_build_literature_mcpb_contains_required_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist = Path(tmp_dir) / "dist"
            result = subprocess.run(
                [sys.executable, "scripts/build_literature_mcpb.py", "--dist-dir", str(dist)],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            artifact = next(dist.glob("qiongli-literature-provider-*.mcpb"))
            self.assertIn(str(artifact), result.stdout)

            with zipfile.ZipFile(artifact) as zf:
                names = set(zf.namelist())

        self.assertIn("manifest.json", names)
        self.assertIn("package.json", names)
        self.assertIn("README.md", names)
        self.assertIn("server/index.mjs", names)
        self.assertIn("server/config.mjs", names)
        self.assertIn("server/stdio.mjs", names)

    def test_build_literature_mcpb_excludes_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist = Path(tmp_dir) / "dist"
            result = subprocess.run(
                [sys.executable, "scripts/build_literature_mcpb.py", "--dist-dir", str(dist)],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            artifact = next(dist.glob("qiongli-literature-provider-*.mcpb"))

            with zipfile.ZipFile(artifact) as zf:
                names = set(zf.namelist())

        self.assertFalse(any(name.startswith("test/") or name.startswith("tests/") for name in names))


if __name__ == "__main__":
    unittest.main()
