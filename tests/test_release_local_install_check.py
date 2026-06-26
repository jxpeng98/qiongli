from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
SCRIPT_PATH = LAYOUT.scripts / "release_local_install_check.py"


def load_release_local_install_check():
    spec = importlib.util.spec_from_file_location("release_local_install_check", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReleaseLocalInstallCheckTests(unittest.TestCase):
    def test_validate_install_tree_accepts_isolated_plugin_and_mcp_surfaces(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            self._write_codex_plugin_tree(sandbox, manifest_category=False)
            self._write_claude_plugin_tree(sandbox)
            self._write_antigravity_plugin_tree(sandbox)
            self._write_json(
                sandbox.antigravity_config_path,
                {"mcpServers": {}},
            )
            self._write_json(
                sandbox.hermes_config_path,
                {"mcpServers": {"qiongli": {"command": "qiongli", "args": module.QIONGLI_MCP_ARGS}}},
            )
            payload = {
                "installed": {
                    "codex": {"installed": True, "surface": "plugin"},
                    "claude": {"installed": True, "surface": "plugin"},
                    "antigravity": {
                        "installed": True,
                        "surface": "plugin",
                        "mcp": {
                            "path": str(sandbox.antigravity_plugin_root / "mcp_config.json"),
                            "source": "plugin",
                        },
                    },
                    "hermes": {"installed": True, "surface": "mcp"},
                }
            }

            module.validate_install_tree(repo_root, sandbox, payload)

            self._write_codex_plugin_tree(sandbox, manifest_category=True)
            with self.assertRaisesRegex(module.LocalInstallCheckError, "top-level category"):
                module.validate_install_tree(repo_root, sandbox, payload)

    def test_build_env_points_clients_at_sandbox_paths(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")

            env = module.build_env(repo_root, sandbox)

        self.assertEqual(env["QIONGLI_CODEX_MARKETPLACE_PATH"], str(sandbox.codex_marketplace_path))
        self.assertEqual(env["QIONGLI_CLAUDE_MARKETPLACE_ROOT"], str(sandbox.claude_plugin_parent))
        self.assertEqual(env["QIONGLI_ANTIGRAVITY_PLUGIN_PARENT"], str(sandbox.antigravity_plugin_parent))
        self.assertEqual(env["ANTIGRAVITY_CONFIG_PATH"], str(sandbox.antigravity_config_path))
        self.assertEqual(env["HERMES_CONFIG_PATH"], str(sandbox.hermes_config_path))
        self.assertEqual(sandbox.codex_marketplace_path, sandbox.home / ".agents" / "plugins" / "marketplace.json")
        self.assertEqual(sandbox.codex_plugin_root, sandbox.home / "plugins" / "qiongli")
        self.assertEqual(sandbox.claude_plugin_root, sandbox.claude_plugin_parent / "plugins" / "qiongli")
        self.assertEqual(sandbox.antigravity_config_path, sandbox.home / ".gemini" / "config" / "mcp_config.json")
        self.assertTrue(env["PYTHONPATH"].startswith(str(repo_root / "packages" / "python-qiongli" / "src")))

    def _write_repo_version(self, repo_root: Path, version: str) -> None:
        version_path = repo_root / "content" / "workflow" / "VERSION"
        version_path.parent.mkdir(parents=True, exist_ok=True)
        version_path.write_text(f"{version}\n", encoding="utf-8")

    def _write_codex_plugin_tree(self, sandbox, *, manifest_category: bool) -> None:
        manifest = {
            "name": "qiongli",
            "version": "9.9.9",
            "skills": "./skills/",
            "mcpServers": "./.mcp.json",
            "interface": {"category": "Education"},
        }
        if manifest_category:
            manifest["category"] = "Education"
        self._write_json(sandbox.codex_plugin_root / ".codex-plugin" / "plugin.json", manifest)
        self._write_json(
            sandbox.codex_plugin_root / ".mcp.json",
            {"mcpServers": {"qiongli": {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}}},
        )
        self._write_json(
            sandbox.codex_marketplace_path,
            {"plugins": [{"name": "qiongli", "source": {"source": "local", "path": "./plugins/qiongli"}}]},
        )
        self._write_skill(sandbox.codex_plugin_root / "skills" / "qiongli-workflow")
        self._write_json(
            sandbox.codex_plugin_root / ".qiongli-managed.json",
            {"managed_by": "qiongli-cli", "surface": "plugin"},
        )

    def _write_claude_plugin_tree(self, sandbox) -> None:
        self._write_json(
            sandbox.claude_plugin_root / ".claude-plugin" / "plugin.json",
            {
                "name": "qiongli",
                "version": "9.9.9",
                "mcpServers": {"qiongli": {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}},
            },
        )
        self._write_skill(sandbox.claude_plugin_root / "skills" / "qiongli-workflow")
        self._write_json(
            sandbox.claude_plugin_root / ".qiongli-managed.json",
            {"managed_by": "qiongli-cli", "surface": "plugin"},
        )

    def _write_antigravity_plugin_tree(self, sandbox) -> None:
        self._write_json(
            sandbox.antigravity_plugin_root / "plugin.json",
            {"name": "qiongli", "version": "9.9.9"},
        )
        self._write_json(
            sandbox.antigravity_plugin_root / "mcp_config.json",
            {"mcpServers": {"qiongli": {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}}},
        )
        self._write_skill(sandbox.antigravity_plugin_root / "skills" / "qiongli-workflow")
        self._write_json(
            sandbox.antigravity_plugin_root / ".qiongli-managed.json",
            {"managed_by": "qiongli-cli", "surface": "plugin"},
        )

    def _write_skill(self, skill_dir: Path) -> None:
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
        (skill_dir / "SKILL.md").write_text(
            "---\nname: qiongli\ndescription: Valid release local install check fixture\n---\n",
            encoding="utf-8",
        )
        self._write_json(skill_dir / "SUBJECT_MANIFEST.json", {"subject": "core", "coverage": "complete"})

    def _write_json(self, path: Path, payload: object) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
