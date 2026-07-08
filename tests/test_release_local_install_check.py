from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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
    def test_run_install_check_verifies_lifecycle_mcp_tools_after_tree_validation(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            payload = {
                "installed": {
                    "codex": {"installed": True, "surface": "plugin"},
                    "claude": {"installed": True, "surface": "plugin"},
                    "antigravity": {"installed": True, "surface": "plugin"},
                    "hermes": {"installed": True, "surface": "mcp"},
                }
            }

            with mock.patch.object(module, "run_cli", side_effect=["", json.dumps(payload)]):
                with mock.patch.object(module, "validate_install_tree") as validate_tree:
                    with mock.patch.object(module, "validate_lifecycle_mcp_tools", create=True) as validate_mcp:
                        result = module.run_install_check(repo_root, sandbox, python="python")

        self.assertEqual(result, payload)
        validate_tree.assert_called_once()
        validate_mcp.assert_called_once()
        self.assertEqual(validate_mcp.call_args.kwargs["python"], "python")

    def test_lifecycle_mcp_tool_name_validator_requires_subject_tools(self) -> None:
        module = load_release_local_install_check()

        module.validate_lifecycle_mcp_tool_names(
            ["qiongli_config_status", "qiongli_subject_status", "qiongli_subject_update"],
        )

        with self.assertRaisesRegex(
            module.LocalInstallCheckError,
            "missing lifecycle MCP tools: qiongli_subject_status, qiongli_subject_update",
        ):
            module.validate_lifecycle_mcp_tool_names(["qiongli_config_status"])

    def test_validate_install_tree_accepts_isolated_plugin_and_mcp_surfaces(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            self._write_platform_target_registry(repo_root)
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

    def test_validate_install_tree_requires_registry_target_metadata_in_markers(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            self._write_platform_target_registry(repo_root)
            self._write_codex_plugin_tree(
                sandbox,
                manifest_category=False,
                include_target_metadata=False,
            )
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

            with self.assertRaisesRegex(
                module.LocalInstallCheckError,
                "Codex plugin marker platform_target.target_id expected codex-marketplace-plugin",
            ):
                module.validate_install_tree(repo_root, sandbox, payload)

    def test_local_acceptance_client_mapping_uses_registry_recommended_keys(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo_root = Path(tmp_dir) / "repo"
            self._write_renamed_local_acceptance_registry(repo_root)

            targets_by_client = module._local_acceptance_targets_by_client(repo_root)

        self.assertEqual(targets_by_client["codex"].target_id, "fixture-codex-target")
        self.assertEqual(targets_by_client["claude"].target_id, "fixture-claude-target")
        self.assertEqual(targets_by_client["antigravity"].target_id, "fixture-antigravity-target")

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

    def _write_renamed_local_acceptance_registry(self, repo_root: Path) -> None:
        registry = repo_root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_text(
            """
schema_version: "1.0"
targets:
  fixture-codex-target:
    display_name: Codex Fixture
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: codex
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.claude-plugin/]
    bundled_mcp_mode: marketplace-lite-binary
    command_surface: slash-commands
    validator: fixture-codex-validator
    release_download:
      guide_label: Codex
      recommended_key: codex
      asset_groups: []
  fixture-claude-target:
    display_name: Claude Fixture
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: claude
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: marketplace-lite-binary
    command_surface: slash-commands
    validator: fixture-claude-validator
    release_download:
      guide_label: Claude Code
      recommended_key: claude_code
      asset_groups: []
  fixture-antigravity-target:
    display_name: Antigravity Fixture
    artifact_kind: local-plugin
    archive_format: directory
    adapter:
      kind: local-plugin
      plugin_manifest_platform: none
      materializer: local_plugin_installer
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: antigravity-python-runtime
    command_surface: slash-commands
    validator: fixture-antigravity-validator
    release_download:
      guide_label: Antigravity
      recommended_key: antigravity
      asset_groups: []
""".lstrip(),
            encoding="utf-8",
        )

    def _write_repo_version(self, repo_root: Path, version: str) -> None:
        version_path = repo_root / "content" / "workflow" / "VERSION"
        version_path.parent.mkdir(parents=True, exist_ok=True)
        version_path.write_text(f"{version}\n", encoding="utf-8")

    def _write_platform_target_registry(self, repo_root: Path) -> None:
        registry = repo_root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_text(
            """
schema_version: "1.0"
targets:
  codex-marketplace-plugin:
    display_name: Codex Marketplace Plugin
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: codex
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.claude-plugin/]
    bundled_mcp_mode: marketplace-lite-binary
    command_surface: slash-commands
    validator: codex-marketplace-plugin
    release_download:
      guide_label: Codex
      recommended_key: codex
      asset_groups: []
  claude-code-marketplace-plugin:
    display_name: Claude Code Marketplace Plugin
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: claude
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: marketplace-lite-binary
    command_surface: slash-commands
    validator: claude-code-marketplace-plugin
    release_download:
      guide_label: Claude Code
      recommended_key: claude_code
      asset_groups: []
  antigravity-local-plugin:
    display_name: Antigravity Local Plugin
    artifact_kind: local-plugin
    archive_format: directory
    adapter:
      kind: local-plugin
      plugin_manifest_platform: none
      materializer: local_plugin_installer
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: antigravity-python-runtime
    command_surface: slash-commands
    validator: antigravity-local-plugin
    release_download:
      guide_label: Antigravity
      recommended_key: antigravity
      asset_groups: []
  claude-desktop-skill-zip:
    display_name: Claude Desktop Skill ZIP
    artifact_kind: skill-zip
    archive_format: zip
    adapter:
      kind: skill-zip
      plugin_manifest_platform: none
      materializer: desktop_skill_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: not_applicable
    source_inputs: [content/workflow/**]
    required_paths: [SKILL.md]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: none
    command_surface: skill-workflows
    validator: claude-desktop-skill-zip
    release_download:
      guide_label: Claude Desktop/Web skills
      recommended_key: claude_desktop_skill
      asset_groups: []
""".lstrip(),
            encoding="utf-8",
        )

    def _write_codex_plugin_tree(
        self,
        sandbox,
        *,
        manifest_category: bool,
        include_target_metadata: bool = True,
    ) -> None:
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
            self._managed_marker("codex") if include_target_metadata else self._managed_marker_without_target(),
        )

    def _write_claude_plugin_tree(self, sandbox, *, include_target_metadata: bool = True) -> None:
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
            self._managed_marker("claude") if include_target_metadata else self._managed_marker_without_target(),
        )

    def _write_antigravity_plugin_tree(self, sandbox, *, include_target_metadata: bool = True) -> None:
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
            self._managed_marker("antigravity") if include_target_metadata else self._managed_marker_without_target(),
        )

    def _managed_marker_without_target(self) -> dict[str, str]:
        return {"managed_by": "qiongli-cli", "surface": "plugin"}

    def _managed_marker(self, platform: str) -> dict[str, object]:
        return {
            "managed_by": "qiongli-cli",
            "surface": "plugin",
            "platform_target": self._platform_target_marker(platform),
        }

    def _platform_target_marker(self, platform: str) -> dict[str, str]:
        targets = {
            "codex": {
                "target_id": "codex-marketplace-plugin",
                "artifact_kind": "marketplace-plugin",
                "archive_format": "tar.gz",
                "bundled_mcp_mode": "marketplace-lite-binary",
                "command_surface": "slash-commands",
                "validator": "codex-marketplace-plugin",
            },
            "claude": {
                "target_id": "claude-code-marketplace-plugin",
                "artifact_kind": "marketplace-plugin",
                "archive_format": "tar.gz",
                "bundled_mcp_mode": "marketplace-lite-binary",
                "command_surface": "slash-commands",
                "validator": "claude-code-marketplace-plugin",
            },
            "antigravity": {
                "target_id": "antigravity-local-plugin",
                "artifact_kind": "local-plugin",
                "archive_format": "directory",
                "bundled_mcp_mode": "antigravity-python-runtime",
                "command_surface": "slash-commands",
                "validator": "antigravity-local-plugin",
            },
        }
        return targets[platform]

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
