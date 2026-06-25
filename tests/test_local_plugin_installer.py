from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from qiongli.local_plugin_installer import (
    LocalPluginOptions,
    install_local_plugin,
    remove_local_plugin,
    resolve_codex_plugin_paths,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class LocalPluginInstallerTests(unittest.TestCase):
    def test_resolve_codex_plugin_paths_uses_marketplace_relative_plugin_root(self) -> None:
        marketplace = Path("/tmp/qiongli-marketplace/marketplace.json")

        paths = resolve_codex_plugin_paths(marketplace_path=marketplace)

        self.assertEqual(paths.marketplace_path, marketplace)
        self.assertEqual(paths.plugin_root, marketplace.parent / "plugins" / "qiongli")
        self.assertEqual(paths.marketplace_source_path, "./plugins/qiongli")

    def test_install_codex_plugin_writes_full_mcp_payload_and_marketplace_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    codex_marketplace_path=marketplace,
                )
            )

            plugin_root = marketplace.parent / "plugins" / "qiongli"
            self.assertTrue(result.changed)
            self.assertEqual(result.installed_roots, {"codex": plugin_root})
            self.assertTrue((plugin_root / ".codex-plugin" / "plugin.json").is_file())
            self.assertTrue((plugin_root / ".mcp.json").is_file())
            self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())
            self.assertTrue((plugin_root / "commands" / "paper.md").is_file())

            codex_manifest = self._read_json(plugin_root / ".codex-plugin" / "plugin.json")
            self.assertEqual(codex_manifest["name"], "qiongli")
            self.assertEqual(codex_manifest["skills"], "./skills/")
            self.assertEqual(codex_manifest["mcpServers"], "./.mcp.json")
            self.assertEqual(codex_manifest["interface"]["displayName"], "Qiongli")
            self.assertEqual(codex_manifest["interface"]["category"], "Education")

            mcp_manifest = self._read_json(plugin_root / ".mcp.json")
            self.assertEqual(
                mcp_manifest,
                {
                    "mcpServers": {
                        "qiongli": {
                            "command": "qiongli",
                            "args": ["mcp", "serve", "--transport", "stdio"],
                            "startup_timeout_sec": 20,
                            "tool_timeout_sec": 120,
                        }
                    }
                },
            )
            self.assertNotIn("env", mcp_manifest["mcpServers"]["qiongli"])
            self.assertNotIn("qiongli-literature-provider", json.dumps(mcp_manifest))

            command_text = (plugin_root / "commands" / "paper.md").read_text(encoding="utf-8")
            self.assertIn("Load the `qiongli` skill", command_text)
            self.assertIn("skills/qiongli-workflow/workflows/paper.md", command_text)

            marketplace_manifest = self._read_json(marketplace)
            qiongli_entry = marketplace_manifest["plugins"]["qiongli"]
            self.assertEqual(qiongli_entry["name"], "qiongli")
            self.assertEqual(qiongli_entry["source"], {"type": "local", "path": "./plugins/qiongli"})
            self.assertEqual(qiongli_entry["policy"]["installation"], "AVAILABLE")
            self.assertEqual(qiongli_entry["policy"]["authentication"], "ON_INSTALL")
            self.assertEqual(qiongli_entry["category"], "Education")

    def test_install_claude_plugin_writes_full_mcp_manifest_and_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            claude_parent = root / "claude-code"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="claude",
                    claude_plugin_parent=claude_parent,
                )
            )

            plugin_root = claude_parent / "qiongli"
            self.assertTrue(result.changed)
            self.assertEqual(result.installed_roots, {"claude": plugin_root})
            self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())

            manifest = self._read_json(plugin_root / ".claude-plugin" / "plugin.json")
            self.assertEqual(manifest["name"], "qiongli")
            self.assertEqual(manifest["mcpServers"]["qiongli"]["type"], "stdio")
            self.assertEqual(manifest["mcpServers"]["qiongli"]["command"], "qiongli")
            self.assertEqual(
                manifest["mcpServers"]["qiongli"]["args"],
                ["mcp", "serve", "--transport", "stdio"],
            )
            self.assertNotIn("env", manifest["mcpServers"]["qiongli"])
            self.assertNotIn("qiongli-literature-provider", json.dumps(manifest))

    def test_remove_local_plugin_removes_managed_codex_root_and_marketplace_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    codex_marketplace_path=marketplace,
                )
            )
            plugin_root = marketplace.parent / "plugins" / "qiongli"

            removed = remove_local_plugin(target="codex", codex_marketplace_path=marketplace)

            self.assertEqual(removed, 1)
            self.assertFalse(plugin_root.exists())
            self.assertEqual(self._read_json(marketplace), {"plugins": {}})

    def test_install_refuses_unmanaged_existing_plugin_root_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            plugin_root = marketplace.parent / "plugins" / "qiongli"
            plugin_root.mkdir(parents=True)
            (plugin_root / "notes.txt").write_text("user data", encoding="utf-8")

            with self.assertRaises(FileExistsError):
                install_local_plugin(
                    LocalPluginOptions(
                        repo_root=REPO_ROOT,
                        target="codex",
                        codex_marketplace_path=marketplace,
                    )
                )

            self.assertEqual((plugin_root / "notes.txt").read_text(encoding="utf-8"), "user data")

    def _read_json(self, path: Path) -> object:
        return json.loads(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
