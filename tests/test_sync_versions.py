from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
SYNC_VERSIONS_PATH = RepoLayout(REPO_ROOT).scripts / "sync_versions.py"
SPEC = importlib.util.spec_from_file_location("sync_versions_module", SYNC_VERSIONS_PATH)
assert SPEC is not None and SPEC.loader is not None
sync_versions_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sync_versions_module)


class SyncVersionsTests(unittest.TestCase):
    def test_parse_version_normalizes_beta_layers(self) -> None:
        package_version, skill_version, repo_version, npm_version = sync_versions_module.parse_version(
            "0.2.0b3"
        )

        self.assertEqual(package_version, "0.2.0b3")
        self.assertEqual(skill_version, "0.2.0-beta.3")
        self.assertEqual(repo_version, "v0.2.0-beta.3")
        self.assertEqual(npm_version, "0.2.0-beta.3")

    def test_main_print_field_outputs_repo_version_without_syncing(self) -> None:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = sync_versions_module.main(
                ["0.2.0b1", "--print-field", "repo_version"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue().strip(), "v0.2.0-beta.1")

    def test_sync_versions_updates_expected_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            layout = RepoLayout(root)
            layout.python_package.mkdir(parents=True)
            (root / "skills" / "F_writing").mkdir(parents=True)
            (root / "qiongli-workflow" / "skills").mkdir(parents=True)
            (root / "packages" / "npm-qiongli").mkdir(parents=True)

            (root / "pyproject.toml").write_text(
                'name = "qiongli"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            (layout.python_package / "__init__.py").write_text(
                '__version__ = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "qiongli-workflow" / "VERSION").write_text(
                "v0.1.0\n",
                encoding="utf-8",
            )
            (root / "qiongli-workflow" / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "package.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "uv.lock").write_text(
                'version = 1\n'
                '\n'
                '[[package]]\n'
                'name = "qiongli"\n'
                'version = "0.1.0"\n'
                'source = { editable = "." }\n',
                encoding="utf-8",
            )
            (root / "package-lock.json").write_text(
                '{\n'
                '  "name": "qiongli-docs",\n'
                '  "lockfileVersion": 3,\n'
                '  "packages": {\n'
                '    "": {"name": "qiongli-docs"},\n'
                '    "node_modules/qiongli": {"resolved": "packages/npm-qiongli", "link": true},\n'
                '    "packages/npm-qiongli": {"name": "qiongli", "version": "0.1.0"}\n'
                '  }\n'
                '}\n',
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow").mkdir(parents=True)
            (root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "VERSION").write_text(
                "v0.1.0\n",
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "skills").mkdir()
            (
                root
                / "packages"
                / "npm-qiongli"
                / "payload"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml"
            ).write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli").mkdir(parents=True)
            (root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli" / "__init__.py").write_text(
                '__version__ = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "python-runtime" / "skills").mkdir(parents=True)
            (root / "packages" / "npm-qiongli" / "python-runtime" / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "plugins" / "qiongli" / ".codex-plugin").mkdir(parents=True)
            (root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "packages" / "qiongli-plugin" / ".codex-plugin").mkdir(parents=True)
            (root / "packages" / "qiongli-plugin" / ".codex-plugin" / "plugin.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "packages" / "qiongli-next-plugin" / ".codex-plugin").mkdir(parents=True)
            (root / "packages" / "qiongli-next-plugin" / ".codex-plugin" / "plugin.json").write_text(
                '{\n  "name": "qiongli-next",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "packages" / "qiongli-next-plugin" / "skills" / "qiongli-workflow" / "skills").mkdir(
                parents=True
            )
            (root / "packages" / "qiongli-next-plugin" / "skills" / "qiongli-workflow" / "VERSION").write_text(
                "v0.1.0\n",
                encoding="utf-8",
            )
            (
                root
                / "packages"
                / "qiongli-next-plugin"
                / "skills"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml"
            ).write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "skills").mkdir(parents=True)
            (root / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "VERSION").write_text(
                "v0.1.0\n",
                encoding="utf-8",
            )
            (
                root
                / "plugins"
                / "qiongli"
                / "skills"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml"
            ).write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "plugins" / "qiongli" / ".claude-plugin").mkdir()
            (root / "plugins" / "qiongli" / ".claude-plugin" / "plugin.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "plugins" / "qiongli" / "gemini-extension.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "packages" / "qiongli-plugin" / ".claude-plugin").mkdir()
            (root / "packages" / "qiongli-plugin" / ".claude-plugin" / "plugin.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "packages" / "qiongli-plugin" / "gemini-extension.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "skills" / "F_writing" / "demo.md").write_text(
                '---\nid: "demo"\nstage: "F_writing"\n---\n',
                encoding="utf-8",
            )

            changed = sync_versions_module.sync_versions(root, "0.2.0b2")

            self.assertIn(root / "pyproject.toml", changed)
            self.assertIn(layout.python_package / "__init__.py", changed)
            self.assertIn(root / "skills" / "registry.yaml", changed)
            self.assertIn(root / "qiongli-workflow" / "VERSION", changed)
            self.assertIn(root / "qiongli-workflow" / "skills" / "registry.yaml", changed)
            self.assertIn(root / "packages" / "npm-qiongli" / "package.json", changed)
            self.assertIn(root / "package-lock.json", changed)
            self.assertIn(root / "uv.lock", changed)
            self.assertIn(
                root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "VERSION",
                changed,
            )
            self.assertIn(
                root
                / "packages"
                / "npm-qiongli"
                / "payload"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml",
                changed,
            )
            self.assertIn(
                root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli" / "__init__.py",
                changed,
            )
            self.assertIn(
                root / "packages" / "npm-qiongli" / "python-runtime" / "skills" / "registry.yaml",
                changed,
            )
            self.assertIn(
                root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json",
                changed,
            )
            self.assertIn(
                root / "packages" / "qiongli-plugin" / ".codex-plugin" / "plugin.json",
                changed,
            )
            self.assertIn(
                root / "packages" / "qiongli-next-plugin" / ".codex-plugin" / "plugin.json",
                changed,
            )
            self.assertIn(
                root / "packages" / "qiongli-next-plugin" / "skills" / "qiongli-workflow" / "VERSION",
                changed,
            )
            self.assertIn(
                root
                / "packages"
                / "qiongli-next-plugin"
                / "skills"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml",
                changed,
            )
            self.assertIn(root / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "VERSION", changed)
            self.assertIn(
                root
                / "plugins"
                / "qiongli"
                / "skills"
                / "qiongli-workflow"
                / "skills"
                / "registry.yaml",
                changed,
            )
            self.assertIn(
                root / "plugins" / "qiongli" / ".claude-plugin" / "plugin.json",
                changed,
            )
            self.assertIn(root / "plugins" / "qiongli" / "gemini-extension.json", changed)
            self.assertIn(
                root / "packages" / "qiongli-plugin" / ".claude-plugin" / "plugin.json",
                changed,
            )
            self.assertIn(root / "packages" / "qiongli-plugin" / "gemini-extension.json", changed)
            self.assertIn('version = "0.2.0b2"', (root / "pyproject.toml").read_text())
            self.assertIn(
                '__version__ = "0.2.0b2"',
                (layout.python_package / "__init__.py").read_text(),
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (root / "skills" / "registry.yaml").read_text(),
            )
            self.assertEqual(
                (root / "qiongli-workflow" / "VERSION").read_text().strip(),
                "v0.2.0-beta.2",
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (root / "qiongli-workflow" / "skills" / "registry.yaml").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "packages" / "npm-qiongli" / "package.json").read_text(),
            )
            self.assertIn(
                'version = "0.2.0b2"',
                (root / "uv.lock").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "package-lock.json").read_text(),
            )
            self.assertEqual(
                (root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "VERSION").read_text().strip(),
                "v0.2.0-beta.2",
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (
                    root
                    / "packages"
                    / "npm-qiongli"
                    / "payload"
                    / "qiongli-workflow"
                    / "skills"
                    / "registry.yaml"
                ).read_text(),
            )
            self.assertIn(
                '__version__ = "0.2.0b2"',
                (root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli" / "__init__.py").read_text(),
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (root / "packages" / "npm-qiongli" / "python-runtime" / "skills" / "registry.yaml").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (
                    root
                    / "packages"
                    / "qiongli-plugin"
                    / ".codex-plugin"
                    / "plugin.json"
                ).read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (
                    root
                    / "packages"
                    / "qiongli-next-plugin"
                    / ".codex-plugin"
                    / "plugin.json"
                ).read_text(),
            )
            self.assertEqual(
                (
                    root
                    / "packages"
                    / "qiongli-next-plugin"
                    / "skills"
                    / "qiongli-workflow"
                    / "VERSION"
                ).read_text().strip(),
                "v0.2.0-beta.2",
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (
                    root
                    / "packages"
                    / "qiongli-next-plugin"
                    / "skills"
                    / "qiongli-workflow"
                    / "skills"
                    / "registry.yaml"
                ).read_text(),
            )
            self.assertEqual(
                (root / "plugins" / "qiongli" / "skills" / "qiongli-workflow" / "VERSION").read_text().strip(),
                "v0.2.0-beta.2",
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (
                    root
                    / "plugins"
                    / "qiongli"
                    / "skills"
                    / "qiongli-workflow"
                    / "skills"
                    / "registry.yaml"
                ).read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "plugins" / "qiongli" / ".claude-plugin" / "plugin.json").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "plugins" / "qiongli" / "gemini-extension.json").read_text(),
            )
            self.assertNotIn(root / "skills" / "F_writing" / "demo.md", changed)
