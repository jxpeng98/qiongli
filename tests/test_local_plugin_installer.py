from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import yaml

from qiongli import __version__ as QIONGLI_VERSION
from qiongli.local_plugin_installer import (
    LocalPluginOptions,
    _build_materialize_source,
    install_local_plugin,
    remove_local_plugin,
    resolve_antigravity_plugin_root,
    resolve_claude_plugin_paths,
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

    def test_resolve_codex_plugin_paths_uses_codex_personal_marketplace_root(self) -> None:
        marketplace = Path("/tmp/qiongli-home/.agents/plugins/marketplace.json")

        paths = resolve_codex_plugin_paths(marketplace_path=marketplace)

        self.assertEqual(paths.marketplace_path, marketplace)
        self.assertEqual(paths.plugin_root, Path("/tmp/qiongli-home/plugins/qiongli"))
        self.assertEqual(paths.marketplace_source_path, "./plugins/qiongli")

    def test_resolve_codex_plugin_paths_honors_env_marketplace_override(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            marketplace = Path(tmp) / "codex" / "marketplace.json"

            with mock.patch.dict(os.environ, {"QIONGLI_CODEX_MARKETPLACE_PATH": str(marketplace)}):
                paths = resolve_codex_plugin_paths()

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
            self.assertTrue((plugin_root / ".qiongli-managed.json").is_file())
            self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())
            self.assertTrue((plugin_root / "commands" / "paper.md").is_file())

            codex_manifest = self._read_json(plugin_root / ".codex-plugin" / "plugin.json")
            self.assertEqual(codex_manifest["name"], "qiongli")
            self.assertNotIn("category", codex_manifest)
            self.assertEqual(codex_manifest["skills"], "./skills/")
            self.assertEqual(codex_manifest["mcpServers"], "./.mcp.json")
            self.assertEqual(codex_manifest["interface"]["displayName"], "Qiongli")
            self.assertEqual(codex_manifest["interface"]["category"], "Education")
            skill_text = (plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").read_text(encoding="utf-8")
            frontmatter = skill_text.split("---", 2)[1]
            skill_metadata = yaml.safe_load(frontmatter)
            self.assertEqual(skill_metadata["name"], "qiongli")
            self.assertIn("Qiongli version:", skill_metadata["description"])

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
            marker = self._read_json(plugin_root / ".qiongli-managed.json")
            self.assertEqual(marker["managed_by"], "qiongli-cli")
            self.assertEqual(marker["plugin"], "qiongli")
            self.assertEqual(marker["surface"], "plugin")
            self.assertEqual(marker["platform"], "codex")
            self.assertEqual(marker["mcp"]["command"], "qiongli")

            command_text = (plugin_root / "commands" / "paper.md").read_text(encoding="utf-8")
            self.assertIn("Load the `qiongli` skill", command_text)
            self.assertIn("skills/qiongli-workflow/workflows/paper.md", command_text)

            marketplace_manifest = self._read_json(marketplace)
            self.assertEqual(marketplace_manifest["name"], "personal")
            self.assertEqual(marketplace_manifest["interface"], {"displayName": "Personal"})
            self.assertIsInstance(marketplace_manifest["plugins"], list)
            qiongli_entry = self._marketplace_entry(marketplace_manifest)
            self.assertEqual(qiongli_entry["name"], "qiongli")
            self.assertEqual(qiongli_entry["source"], {"source": "local", "path": "./plugins/qiongli"})
            self.assertEqual(qiongli_entry["policy"]["installation"], "AVAILABLE")
            self.assertEqual(qiongli_entry["policy"]["authentication"], "ON_INSTALL")
            self.assertEqual(qiongli_entry["category"], "Education")
            self.assertEqual(qiongli_entry["metadata"], {"managedBy": "qiongli-cli", "surface": "plugin"})

    def test_install_codex_plugin_preserves_existing_marketplace_name_and_interface(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            marketplace.parent.mkdir(parents=True)
            marketplace.write_text(
                json.dumps(
                    {
                        "name": "mine",
                        "interface": {"displayName": "Mine"},
                        "plugins": [{"name": "other", "source": {"source": "local", "path": "./plugins/other"}}],
                    }
                ),
                encoding="utf-8",
            )

            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    codex_marketplace_path=marketplace,
                )
            )

            marketplace_manifest = self._read_json(marketplace)
            self.assertEqual(marketplace_manifest["name"], "mine")
            self.assertEqual(marketplace_manifest["interface"], {"displayName": "Mine"})
            self.assertEqual([entry["name"] for entry in marketplace_manifest["plugins"]], ["other", "qiongli"])

    def test_install_claude_plugin_writes_full_mcp_manifest_and_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            claude_marketplace = root / "claude-code"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="claude",
                    claude_plugin_parent=claude_marketplace,
                )
            )

            plugin_root = claude_marketplace / "plugins" / "qiongli"
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
            marker = self._read_json(plugin_root / ".qiongli-managed.json")
            self.assertEqual(marker["managed_by"], "qiongli-cli")
            self.assertEqual(marker["plugin"], "qiongli")
            self.assertEqual(marker["surface"], "plugin")
            self.assertEqual(marker["platform"], "claude")
            self.assertEqual(marker["mcp"]["command"], "qiongli")

            command_text = (plugin_root / "commands" / "paper.md").read_text(encoding="utf-8")
            self.assertIn("Load the `qiongli` skill", command_text)
            self.assertIn("skills/qiongli-workflow/workflows/paper.md", command_text)

            marketplace = self._read_json(claude_marketplace / ".claude-plugin" / "marketplace.json")
            self.assertEqual(marketplace["name"], "qiongli-local")
            self.assertEqual(marketplace["plugins"][0]["name"], "qiongli")
            self.assertEqual(marketplace["plugins"][0]["source"], "./plugins/qiongli")
            self.assertEqual(marketplace["plugins"][0]["category"], "Education")

    def test_resolve_claude_plugin_paths_uses_marketplace_plugins_root(self) -> None:
        marketplace_root = Path("/tmp/qiongli-claude-marketplace")

        paths = resolve_claude_plugin_paths(marketplace_root=marketplace_root)

        self.assertEqual(paths.marketplace_root, marketplace_root)
        self.assertEqual(paths.marketplace_path, marketplace_root / ".claude-plugin" / "marketplace.json")
        self.assertEqual(paths.plugin_root, marketplace_root / "plugins" / "qiongli")
        self.assertEqual(paths.marketplace_name, "qiongli-local")

    def test_install_antigravity_plugin_writes_root_plugin_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            antigravity_parent = root / "antigravity"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="antigravity",
                    antigravity_plugin_parent=antigravity_parent,
                )
            )

            plugin_root = antigravity_parent / "qiongli"
            self.assertTrue(result.changed)
            self.assertEqual(result.installed_roots, {"antigravity": plugin_root})
            manifest = self._read_json(plugin_root / "plugin.json")
            self.assertEqual(manifest["name"], "qiongli")
            self.assertEqual(manifest["version"], QIONGLI_VERSION)
            mcp_manifest = self._read_json(plugin_root / "mcp_config.json")
            self.assertEqual(
                mcp_manifest,
                {
                    "mcpServers": {
                        "qiongli": {
                            "command": "qiongli",
                            "args": ["mcp", "serve", "--transport", "stdio"],
                        }
                    }
                },
            )
            self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())
            self.assertTrue((plugin_root / "commands" / "paper.md").is_file())

    def test_target_all_installs_codex_and_claude_only(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            claude_parent = root / "claude-code"
            antigravity_parent = root / "antigravity-plugins"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="all",
                    codex_marketplace_path=marketplace,
                    claude_plugin_parent=claude_parent,
                    antigravity_plugin_parent=antigravity_parent,
                )
            )

            self.assertTrue(result.changed)
            self.assertEqual(
                result.installed_roots,
                {
                    "codex": marketplace.parent / "plugins" / "qiongli",
                    "claude": claude_parent / "plugins" / "qiongli",
                    "antigravity": resolve_antigravity_plugin_root(antigravity_parent),
                },
            )
            self.assertTrue((result.installed_roots["codex"] / ".codex-plugin" / "plugin.json").is_file())
            self.assertTrue((result.installed_roots["claude"] / ".claude-plugin" / "plugin.json").is_file())
            self.assertTrue((result.installed_roots["antigravity"] / "plugin.json").is_file())
            self.assertTrue((result.installed_roots["antigravity"] / "mcp_config.json").is_file())
            self.assertEqual(set(result.installed_roots), {"codex", "claude", "antigravity"})

    def test_materialize_source_accepts_self_contained_payload_workflow(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            payload = root / "payload"
            (payload / "qiongli-workflow" / "skills").mkdir(parents=True)
            (payload / "qiongli-workflow" / "skills" / "registry.yaml").write_text("workflow\n", encoding="utf-8")
            (payload / "skills").mkdir(parents=True)
            (payload / "skills" / "registry.yaml").write_text("payload\n", encoding="utf-8")
            (payload / "subjects").mkdir(parents=True)
            (payload / "subjects" / "catalog.yaml").write_text("subjects: {}\n", encoding="utf-8")

            materialized = _build_materialize_source(payload, root / "work")

            self.assertEqual(
                "workflow",
                (materialized / "qiongli-workflow" / "skills" / "registry.yaml").read_text(encoding="utf-8").strip(),
            )
            self.assertEqual("payload", (materialized / "skills" / "registry.yaml").read_text(encoding="utf-8").strip())

    def test_non_plugin_target_returns_empty_unchanged_result(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="hermes",
                    codex_marketplace_path=root / "agents" / "marketplace.json",
                    claude_plugin_parent=root / "claude-code",
                )
            )

            self.assertFalse(result.changed)
            self.assertEqual(result.installed_roots, {})
            self.assertEqual(list(root.rglob("*")), [])

    def test_dry_run_returns_planned_roots_without_writing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            claude_parent = root / "claude-code"
            antigravity_parent = root / "antigravity-plugins"

            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="all",
                    dry_run=True,
                    codex_marketplace_path=marketplace,
                    claude_plugin_parent=claude_parent,
                    antigravity_plugin_parent=antigravity_parent,
                )
            )

            self.assertFalse(result.changed)
            self.assertEqual(
                result.installed_roots,
                {
                    "codex": marketplace.parent / "plugins" / "qiongli",
                    "claude": claude_parent / "plugins" / "qiongli",
                    "antigravity": antigravity_parent / "qiongli",
                },
            )
            self.assertFalse(marketplace.exists())
            self.assertFalse(result.installed_roots["codex"].exists())
            self.assertFalse(result.installed_roots["claude"].exists())
            self.assertFalse(result.installed_roots["antigravity"].exists())

    def test_managed_overwrite_replaces_existing_managed_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            options = LocalPluginOptions(
                repo_root=REPO_ROOT,
                target="codex",
                codex_marketplace_path=marketplace,
            )
            install_local_plugin(options)
            plugin_root = marketplace.parent / "plugins" / "qiongli"
            stale = plugin_root / "stale.txt"
            stale.write_text("remove me", encoding="utf-8")

            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    overwrite=True,
                    codex_marketplace_path=marketplace,
                )
            )

            self.assertFalse(stale.exists())
            self.assertTrue((plugin_root / ".codex-plugin" / "plugin.json").is_file())

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
            marketplace_manifest = self._read_json(marketplace)
            self.assertEqual(marketplace_manifest["name"], "personal")
            self.assertEqual(marketplace_manifest["interface"], {"displayName": "Personal"})
            self.assertEqual(marketplace_manifest["plugins"], [])

    def test_dry_run_remove_does_not_delete_managed_root_or_marketplace_entry(self) -> None:
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

            removed = remove_local_plugin(target="codex", dry_run=True, codex_marketplace_path=marketplace)

            self.assertEqual(removed, 1)
            self.assertTrue(plugin_root.exists())
            self.assertEqual(self._marketplace_entry(self._read_json(marketplace))["name"], "qiongli")

    def test_remove_preserves_marketplace_entry_when_plugin_root_is_unmanaged(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            plugin_root = marketplace.parent / "plugins" / "qiongli"
            plugin_root.mkdir(parents=True)
            (plugin_root / "notes.txt").write_text("user data", encoding="utf-8")
            entry = {
                "name": "qiongli",
                "source": {"source": "local", "path": "./plugins/qiongli"},
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": "Education",
            }
            self._write_marketplace(marketplace, [entry])

            removed = remove_local_plugin(target="codex", codex_marketplace_path=marketplace)

            self.assertEqual(removed, 0)
            self.assertTrue(plugin_root.exists())
            self.assertEqual(self._read_json(marketplace)["plugins"], [entry])

    def test_remove_preserves_marketplace_lite_plugin_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            plugin_root = marketplace.parent / "plugins" / "qiongli"
            (plugin_root / ".codex-plugin").mkdir(parents=True)
            (plugin_root / ".codex-plugin" / "plugin.json").write_text(
                json.dumps({"name": "qiongli", "mcpServers": "./.mcp.json"}),
                encoding="utf-8",
            )
            (plugin_root / ".mcp.json").write_text(
                json.dumps(
                    {
                        "mcpServers": {
                            "qiongli": {
                                "command": "node",
                                "args": ["./mcp/qiongli-literature-provider/index.mjs"],
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            entry = {
                "name": "qiongli",
                "source": {"source": "local", "path": "./plugins/qiongli"},
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": "Education",
            }
            self._write_marketplace(marketplace, [entry])

            removed = remove_local_plugin(target="codex", codex_marketplace_path=marketplace)

            self.assertEqual(removed, 0)
            self.assertTrue(plugin_root.exists())
            self.assertEqual(self._read_json(marketplace)["plugins"], [entry])

    def test_remove_managed_root_preserves_non_local_qiongli_marketplace_entry(self) -> None:
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
            non_local_entry = {
                "name": "qiongli",
                "source": {"source": "github", "path": "jxpeng98/qiongli"},
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": "Education",
            }
            self._write_marketplace(marketplace, [non_local_entry])

            removed = remove_local_plugin(target="codex", codex_marketplace_path=marketplace)

            self.assertEqual(removed, 1)
            self.assertFalse(plugin_root.exists())
            self.assertEqual(self._read_json(marketplace)["plugins"], [non_local_entry])

    def test_remove_stale_marketplace_only_managed_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"
            entry = {
                "name": "qiongli",
                "source": {"source": "local", "path": "./plugins/qiongli"},
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": "Education",
                "metadata": {"managedBy": "qiongli-cli", "surface": "plugin"},
            }
            self._write_marketplace(marketplace, [entry])

            removed = remove_local_plugin(target="codex", codex_marketplace_path=marketplace)

            self.assertEqual(removed, 1)
            self.assertFalse((marketplace.parent / "plugins" / "qiongli").exists())
            self.assertEqual(self._read_json(marketplace)["plugins"], [])

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

    def _marketplace_entry(self, marketplace_manifest: object) -> dict[str, object]:
        self.assertIsInstance(marketplace_manifest, dict)
        plugins = marketplace_manifest["plugins"]  # type: ignore[index]
        self.assertIsInstance(plugins, list)
        matches = [entry for entry in plugins if isinstance(entry, dict) and entry.get("name") == "qiongli"]
        self.assertEqual(len(matches), 1)
        return matches[0]

    def _write_marketplace(self, path: Path, plugins: list[dict[str, object]]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(
                {
                    "name": "personal",
                    "interface": {"displayName": "Personal"},
                    "plugins": plugins,
                }
            ),
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
