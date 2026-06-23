from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)


class QiongliNamingTests(unittest.TestCase):
    def test_python_distribution_and_cli_use_qiongli(self) -> None:
        metadata = tomllib.loads((REPO_ROOT / "pyproject.toml").read_text(encoding="utf-8"))

        self.assertEqual(metadata["project"]["name"], "qiongli")
        scripts = metadata["project"]["scripts"]
        self.assertEqual(scripts["qiongli"], "qiongli.cli:main")
        self.assertEqual(scripts["ql"], "qiongli.cli:main")
        self.assertEqual(scripts["research-skills"], "qiongli.cli:main")
        self.assertEqual(scripts["rsk"], "qiongli.cli:main")
        self.assertEqual(scripts["rsw"], "qiongli.cli:main")

    def test_pypi_version_check_uses_qiongli_distribution(self) -> None:
        cli_source = (LAYOUT.python_package / "cli.py").read_text(encoding="utf-8")

        self.assertIn("https://pypi.org/pypi/qiongli/json", cli_source)
        self.assertIn("pipx upgrade qiongli", cli_source)
        self.assertNotIn("qiongli-installer/json", cli_source)
        self.assertNotIn("pipx upgrade qiongli-installer", cli_source)

    def test_plugin_manifests_use_qiongli_public_identity(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
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
            plugin_root = out / "plugins" / "qiongli"
            codex_manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
            claude_manifest = json.loads((plugin_root / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))

        for manifest in (codex_manifest, claude_manifest):
            self.assertEqual(manifest["name"], "qiongli")
            self.assertIn("Qiongli", manifest["description"])
        self.assertFalse((plugin_root / "gemini-extension.json").exists())

    def test_next_codex_plugin_uses_prerelease_identity(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
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
            plugin_root = out / "plugins" / "qiongli-next"
            manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
            skill_text = (plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").read_text(encoding="utf-8")
            mcp_manifest = json.loads((plugin_root / ".mcp.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli-next")
        self.assertEqual(manifest["interface"]["displayName"], "Qiongli Next")
        self.assertIn("$qiongli-next", "\n".join(manifest["interface"]["defaultPrompt"]))
        self.assertIn("name: qiongli-next\n", skill_text)
        self.assertEqual(set(mcp_manifest["mcpServers"]), {"qiongli-next"})

    def test_portable_skill_identity_is_visible_as_qiongli(self) -> None:
        skill_root = LAYOUT.workflow
        skill_text = (skill_root / "SKILL.md").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as tmp_dir:
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
            plugin_skill = out / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "SKILL.md"
            self.assertTrue(plugin_skill.is_file())
            plugin_skill_text = plugin_skill.read_text(encoding="utf-8")

        self.assertIn("name: qiongli\n", skill_text)
        self.assertIn("# Qiongli Academic Workflow", skill_text)
        self.assertNotIn("name: research-paper-workflow", skill_text)
        self.assertIn("name: qiongli\n", plugin_skill_text)

        manifest = (LAYOUT.install / "install_manifest.tsv").read_text(encoding="utf-8")
        self.assertIn("qiongli-workflow\t${CODEX_HOME}/skills/qiongli-workflow", manifest)
        self.assertNotIn("research-paper-workflow\t${CODEX_HOME}/skills/research-paper-workflow", manifest)


if __name__ == "__main__":
    unittest.main()
