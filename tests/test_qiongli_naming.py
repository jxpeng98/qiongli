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
        cli_source = (REPO_ROOT / "qiongli" / "cli.py").read_text(encoding="utf-8")

        self.assertIn("https://pypi.org/pypi/qiongli/json", cli_source)
        self.assertIn("pipx upgrade qiongli", cli_source)
        self.assertNotIn("qiongli-installer/json", cli_source)
        self.assertNotIn("pipx upgrade qiongli-installer", cli_source)

    def test_plugin_manifests_use_qiongli_public_identity(self) -> None:
        codex_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        gemini_manifest = json.loads((REPO_ROOT / "plugins" / "qiongli" / "gemini-extension.json").read_text(encoding="utf-8"))

        for manifest in (codex_manifest, claude_manifest, gemini_manifest):
            self.assertEqual(manifest["name"], "qiongli")
            self.assertIn("Qiongli", manifest["description"])

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

        manifest = (REPO_ROOT / "install" / "install_manifest.tsv").read_text(encoding="utf-8")
        self.assertIn("qiongli-workflow\t${CODEX_HOME}/skills/qiongli-workflow", manifest)
        self.assertNotIn("research-paper-workflow\t${CODEX_HOME}/skills/research-paper-workflow", manifest)


if __name__ == "__main__":
    unittest.main()
