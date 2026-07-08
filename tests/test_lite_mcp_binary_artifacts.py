from __future__ import annotations

import json
import os
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

from qiongli.source_layout import RepoLayout
from scripts.build_plugin_artifacts import build_artifacts


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteMCPBinaryArtifactTests(unittest.TestCase):
    def test_build_lite_mcp_stages_current_platform_binary(self) -> None:
        from tooling.scripts.build_lite_mcp import build_current_platform

        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            self.assertTrue(binary.is_file())
            self.assertEqual(binary.name, "qiongli-literature-provider")
            self.assertTrue(os.access(binary, os.X_OK))

    def test_codex_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        plugin_name = "qiongli-next" if "beta" in tag else "qiongli"
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            codex = next(
                path for path in artifacts if path.name.startswith(f"{plugin_name}-codex-plugin-")
            )
            with tarfile.open(codex, "r:gz") as archive:
                names = set(archive.getnames())
                member = next(
                    name for name in names if name.endswith(f"/plugins/{plugin_name}/.mcp.json")
                )
                extracted = archive.extractfile(member)
                self.assertIsNotNone(extracted, msg=f"missing tar member: {member}")
                assert extracted is not None
                manifest = json.loads(extracted.read().decode("utf-8"))

        self.assertTrue(
            any(
                name.endswith(f"/plugins/{plugin_name}/bin/qiongli-literature-provider")
                for name in names
            )
        )
        server = manifest["mcpServers"][plugin_name]
        self.assertEqual(server["command"], "./bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])

    def test_direct_desktop_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        plugin_name = "qiongli-next" if "beta" in tag else "qiongli"
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            desktop = next(
                path
                for path in artifacts
                if path.name.startswith(f"{plugin_name}-claude-desktop-plugin-")
            )
            with zipfile.ZipFile(desktop) as archive:
                names = set(archive.namelist())
                manifest_member = next(
                    name for name in names if name.endswith("/.claude-plugin/plugin.json")
                )
                manifest = json.loads(
                    archive.read(manifest_member).decode("utf-8")
                )

        self.assertTrue(
            any(name.endswith("/bin/qiongli-literature-provider") for name in names)
        )
        server = manifest["mcpServers"][plugin_name]
        self.assertEqual(
            server["command"],
            "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
        )
        self.assertEqual(server["args"], ["--transport", "stdio"])


if __name__ == "__main__":
    unittest.main()
