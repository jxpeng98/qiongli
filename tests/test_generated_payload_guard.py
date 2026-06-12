from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
GUARD_PATH = LAYOUT.scripts / "check_generated_payload_edits.py"


def _load_guard_module():
    spec = importlib.util.spec_from_file_location("check_generated_payload_edits", GUARD_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {GUARD_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class GeneratedPayloadGuardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.guard = _load_guard_module()

    def test_classifies_generated_payload_paths(self) -> None:
        generated_paths = [
            ".agent/workflows/paper.md",
            ".gemini/qiongli.md",
            "packages/python-qiongli/src/qiongli/payload/qiongli-workflow/SKILL.md",
            "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md",
            "packages/npm-qiongli/python-runtime/qiongli/__init__.py",
            "packages/qiongli-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-plugin/commands/paper.md",
            "packages/qiongli-plugin/platforms/agent/workflows/paper.md",
            "packages/qiongli-next-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-next-plugin/skills/qiongli-workflow/SKILL.md",
            "plugins/qiongli/.codex-plugin/plugin.json",
            "plugins/qiongli/skills/qiongli-workflow/SKILL.md",
            "qiongli-workflow/SKILL.md",
            "qiongli-workflow/skills/registry.yaml",
            "qiongli-workflow/templates/paper-note.md",
        ]
        for path in generated_paths:
            with self.subTest(path=path):
                self.assertTrue(self.guard.is_generated_payload_path(path))

    def test_keeps_canonical_source_paths_allowed(self) -> None:
        source_paths = [
            "content/workflow/SKILL.md",
            "content/workflow/workflows/paper-read.md",
            "content/workflow/references/stage-B-literature.md",
            "content/skills/B_literature/academic-searcher.md",
            "content/templates/paper-note.md",
            "content/standards/research-workflow-contract.yaml",
            "content/roles/literature-ra.yaml",
            "content/venue-profiles/nature.yaml",
            "content/subjects/catalog.yaml",
            "content/subjects/finance/skills/finance-identification-risk-auditor.md",
            "content/skills-core.md",
            "content/skills-summary.md",
        ]
        for path in source_paths:
            with self.subTest(path=path):
                self.assertFalse(self.guard.is_generated_payload_path(path))

    def test_cli_rejects_changed_generated_paths(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(GUARD_PATH),
                "--changed-file",
                "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(1, result.returncode)
        self.assertIn("Generated distribution payload files changed", result.stderr)
        self.assertIn("canonical source", result.stderr)

    def test_cli_allows_generated_paths_with_explicit_release_override(self) -> None:
        env = os.environ.copy()
        env["QIONGLI_ALLOW_GENERATED_PAYLOAD_CHANGES"] = "1"
        result = subprocess.run(
            [
                sys.executable,
                str(GUARD_PATH),
                "--changed-file",
                "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md",
            ],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(0, result.returncode)
        self.assertIn("override enabled", result.stdout)

    def test_diff_mode_uses_base_ref_to_find_changed_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.PIPE)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
            (repo / "skills").mkdir()
            (repo / "skills" / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-m", "initial"], cwd=repo, check=True, stdout=subprocess.PIPE)
            base = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=repo,
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            ).stdout.strip()

            changed = repo / "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md"
            changed.parent.mkdir(parents=True)
            changed.write_text("generated\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-m", "generated edit"], cwd=repo, check=True, stdout=subprocess.PIPE)

            result = subprocess.run(
                [sys.executable, str(GUARD_PATH), "--base-ref", base],
                cwd=repo,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertEqual(1, result.returncode)
        self.assertIn("packages/npm-qiongli/payload/qiongli-workflow/SKILL.md", result.stderr)

    def test_diff_mode_allows_deleted_generated_payload_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.PIPE)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
            generated = repo / "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md"
            generated.parent.mkdir(parents=True)
            generated.write_text("generated\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-m", "initial generated output"], cwd=repo, check=True, stdout=subprocess.PIPE)
            base = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=repo,
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            ).stdout.strip()

            generated.unlink()
            subprocess.run(["git", "add", "-u"], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-m", "drop generated output"], cwd=repo, check=True, stdout=subprocess.PIPE)

            result = subprocess.run(
                [sys.executable, str(GUARD_PATH), "--base-ref", base],
                cwd=repo,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertEqual(0, result.returncode)
        self.assertIn("no generated distribution payload edits detected", result.stdout)


if __name__ == "__main__":
    unittest.main()
