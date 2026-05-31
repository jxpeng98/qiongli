from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
GUARD_PATH = REPO_ROOT / "scripts" / "check_generated_payload_edits.py"


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
            "qiongli/payload/qiongli-workflow/SKILL.md",
            "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md",
            "packages/npm-qiongli/python-runtime/qiongli/__init__.py",
            "plugins/qiongli/skills/qiongli-workflow/SKILL.md",
            "qiongli-workflow/skills/registry.yaml",
            "qiongli-workflow/templates/paper-note.md",
        ]
        for path in generated_paths:
            with self.subTest(path=path):
                self.assertTrue(self.guard.is_generated_payload_path(path))

    def test_keeps_canonical_source_paths_allowed(self) -> None:
        source_paths = [
            "qiongli-workflow/SKILL.md",
            "qiongli-workflow/workflows/paper-read.md",
            "qiongli-workflow/references/stage-B-literature.md",
            "skills/B_literature/academic-searcher.md",
            "templates/paper-note.md",
            "standards/research-workflow-contract.yaml",
            "roles/literature-ra.yaml",
            "venue-profiles/nature.yaml",
            "subjects/catalog.yaml",
            "subjects/finance/skills/finance-identification-risk-auditor.md",
            "skills-core.md",
            "skills-summary.md",
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


if __name__ == "__main__":
    unittest.main()
