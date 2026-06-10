from __future__ import annotations

import importlib.util
import json
import os
import shutil
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
WORKFLOW_ROOT = LAYOUT.workflow
WORKFLOW_VERSION = (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")
VALIDATOR_SCRIPT_PATH = LAYOUT.scripts / "validate_marketplace_install.py"
if str(LAYOUT.scripts) not in sys.path:
    sys.path.insert(0, str(LAYOUT.scripts))
VALIDATOR_SPEC = importlib.util.spec_from_file_location("validate_marketplace_install", VALIDATOR_SCRIPT_PATH)
assert VALIDATOR_SPEC is not None and VALIDATOR_SPEC.loader is not None
validator = importlib.util.module_from_spec(VALIDATOR_SPEC)
sys.modules["validate_marketplace_install"] = validator
VALIDATOR_SPEC.loader.exec_module(validator)


def find_usable_bash() -> str | None:
    candidates: list[str] = []
    for value in (os.environ.get("BASH"), shutil.which("bash")):
        if value:
            candidates.append(value)

    candidates.extend(
        [
            "/bin/bash",
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files\Git\usr\bin\bash.exe",
        ]
    )

    seen: set[str] = set()
    for candidate in candidates:
        normalized = str(Path(candidate))
        if normalized in seen:
            continue
        seen.add(normalized)
        if not Path(candidate).exists():
            continue

        result = subprocess.run(
            [candidate, "--version"],
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode == 0:
            return candidate

    return None


class PluginDistributionContractTests(unittest.TestCase):
    def materialize_payload_root(self, tmp_dir: str) -> Path:
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
        return out

    def materialize_plugin_payload(self, tmp_dir: str) -> Path:
        return self.materialize_payload_root(tmp_dir) / "plugins" / "qiongli"

    def test_platform_manifests_share_workflow_version(self) -> None:
        codex = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude = json.loads((PLUGIN_ROOT / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        gemini = json.loads((PLUGIN_ROOT / "gemini-extension.json").read_text(encoding="utf-8"))

        self.assertEqual(codex["version"], WORKFLOW_VERSION)
        self.assertEqual(claude["version"], WORKFLOW_VERSION)
        self.assertEqual(gemini["version"], WORKFLOW_VERSION)

    def test_codex_plugin_exposes_skill_directory(self) -> None:
        manifest = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["skills"], "./skills/")
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            self.assertTrue((materialized_plugin / "skills").is_dir())
            self.assertTrue((materialized_plugin / ".mcp.json").is_file())
            self.assertTrue(
                (materialized_plugin / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file()
            )

    def test_git_backed_next_codex_plugin_source_is_installable(self) -> None:
        manifest_path = NEXT_PLUGIN_ROOT / ".codex-plugin" / "plugin.json"
        validator._assert_manifest(
            "codex",
            manifest_path,
            WORKFLOW_VERSION,
            expected_plugin_name="qiongli-next",
            expected_skill_name="qiongli-next",
        )

        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        self.assertEqual(manifest["name"], "qiongli-next")
        self.assertEqual(manifest["skills"], "./skills/")
        self.assertEqual(manifest["mcpServers"], "./.mcp.json")

        mcp_manifest = json.loads((NEXT_PLUGIN_ROOT / ".mcp.json").read_text(encoding="utf-8"))
        self.assertEqual(set(mcp_manifest["mcpServers"]), {"qiongli-next"})
        validator._assert_bundled_literature_mcp(
            NEXT_PLUGIN_ROOT,
            "codex",
            mcp_server_name="qiongli-next",
        )

        skill_root = NEXT_PLUGIN_ROOT / "skills" / "qiongli-workflow"
        workflow_names = validator._assert_skill_invocation(
            skill_root,
            f"v{WORKFLOW_VERSION}",
            skill_name="qiongli-next",
        )
        validator._assert_subject_marker(skill_root, "core")
        validator._assert_subject_manifest(skill_root, "core", "complete")
        validator._assert_command_invocation(NEXT_PLUGIN_ROOT, workflow_names, skill_name="qiongli-next")

        self.assertFalse((NEXT_PLUGIN_ROOT / ".claude-plugin").exists())
        self.assertFalse((NEXT_PLUGIN_ROOT / "gemini-extension.json").exists())

    def test_codex_plugin_materializes_bundled_mcp_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8")
            )
            mcp_manifest = json.loads((materialized_plugin / ".mcp.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["mcpServers"], "./.mcp.json")
        self.assertEqual(mcp_manifest["mcpServers"]["qiongli"]["command"], "node")
        self.assertEqual(
            mcp_manifest["mcpServers"]["qiongli"]["args"],
            ["./mcp/qiongli-literature-provider/index.mjs"],
        )

    def test_claude_plugin_materializes_bundled_mcp_server(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8")
            )

            self.assertIn("mcpServers", manifest)
            self.assertIn("qiongli", manifest["mcpServers"])
            server = manifest["mcpServers"]["qiongli"]
            self.assertEqual(server["command"], "node")
            self.assertEqual(
                server["args"],
                ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
            )
            self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
            self.assertTrue(
                (materialized_plugin / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file()
            )

    def test_codex_bundled_mcp_validation_requires_plugin_manifest_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest_path = materialized_plugin / ".codex-plugin" / "plugin.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            del manifest["mcpServers"]
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "mcpServers"):
                validator._assert_bundled_literature_mcp(materialized_plugin, "codex")

    def test_plugin_package_contains_real_portable_skill_copy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            plugin_skill_root = materialized_plugin / "skills" / "qiongli-workflow"

            self.assertTrue((plugin_skill_root / "SKILL.md").is_file())
            self.assertTrue((plugin_skill_root / "skills" / "registry.yaml").is_file())
            self.assertFalse(plugin_skill_root.is_symlink(), "plugin package must be a real copy, not a symlink")
            self.assertEqual(
                (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8"),
                (plugin_skill_root / "VERSION").read_text(encoding="utf-8"),
            )
            self.assertEqual(
                (LAYOUT.skills / "registry.yaml").read_text(encoding="utf-8"),
                (plugin_skill_root / "skills" / "registry.yaml").read_text(encoding="utf-8"),
            )

    def test_sync_script_accepts_all_target_in_dry_run(self) -> None:
        bash = find_usable_bash()
        if bash is None:
            self.skipTest("usable bash is not available")

        result = subprocess.run(
            [bash, "scripts/sync_skill_package.sh", "--target", "all", "--dry-run"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        self.assertIn("qiongli-workflow", result.stdout)
        self.assertIn("plugins/qiongli/skills/qiongli-workflow", result.stdout)

    def test_marketplace_validator_builds_platform_artifacts_and_checks_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/validate_marketplace_install.py",
                    "--dist-dir",
                    tmp_dir,
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        self.assertIn(
            "[OK] codex marketplace artifact (core-next): "
            "qiongli-next invocation checked; bundled literature MCP checked",
            result.stdout,
        )
        self.assertIn(
            "[OK] claude marketplace artifact (core-next): "
            "qiongli-next invocation checked; bundled literature MCP checked",
            result.stdout,
        )
        self.assertIn(
            "[OK] claude marketplace ZIP artifact (core-next): "
            "qiongli-next invocation checked; bundled literature MCP checked",
            result.stdout,
        )
        self.assertIn("[OK] claude-desktop skill artifact (core-next)", result.stdout)
        self.assertIn("under desktop file budget", result.stdout)
        self.assertNotIn("[OK] gemini marketplace artifact", result.stdout)
        self.assertNotIn("artifact (economics)", result.stdout)
        self.assertIn("qiongli-next invocation", result.stdout)
        self.assertIn("bundled literature MCP checked", result.stdout)


if __name__ == "__main__":
    unittest.main()
