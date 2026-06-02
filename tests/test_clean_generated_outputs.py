from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
CLEANER_PATH = REPO_ROOT / "scripts" / "clean_generated_outputs.py"
PATHS_PATH = REPO_ROOT / "scripts" / "generated_output_paths.py"


def _load_paths_module():
    spec = importlib.util.spec_from_file_location("generated_output_paths", PATHS_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {PATHS_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _init_git_repo(repo: Path, *, ignore_generated: bool = True) -> None:
    subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.PIPE)
    subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo, check=True)
    subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
    if ignore_generated:
        (repo / ".gitignore").write_text(
            "\n".join(
                [
                    "/qiongli/payload/",
                    "/packages/npm-qiongli/payload/",
                    "/packages/npm-qiongli/python-runtime/",
                    "/plugins/qiongli/skills/qiongli-workflow/",
                    "/qiongli-workflow/",
                    "/content/workflow/skills/",
                    "/content/workflow/skills-core.md",
                    "/content/workflow/skills-summary.md",
                    "/content/workflow/templates/",
                    "/content/workflow/standards/",
                    "/content/workflow/roles/",
                    "/content/workflow/venue-profiles/",
                    "",
                ]
            ),
            encoding="utf-8",
        )


def _write(path: Path, content: str = "content\n") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


class CleanGeneratedOutputsTests(unittest.TestCase):
    def test_generated_output_paths_are_covered_by_gitignore(self) -> None:
        paths = _load_paths_module()

        for rel in paths.GENERATED_OUTPUT_PATHS:
            candidate = f"{rel}/.generated-output" if rel in paths.GENERATED_OUTPUT_DIRECTORIES else rel
            with self.subTest(path=rel):
                result = subprocess.run(
                    ["git", "check-ignore", "--no-index", "--", candidate],
                    cwd=REPO_ROOT,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                    check=False,
                )
                self.assertEqual(0, result.returncode, result.stderr)

    def test_dry_run_lists_generated_outputs_without_removing_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _init_git_repo(repo)
            generated = repo / "content/workflow/templates/paper-note.md"
            generated_file = repo / "content/workflow/skills-core.md"
            source = repo / "content/templates/paper-note.md"
            _write(generated)
            _write(generated_file)
            _write(source)

            result = subprocess.run(
                [sys.executable, str(CLEANER_PATH), "--root", str(repo)],
                cwd=REPO_ROOT,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(0, result.returncode, result.stderr)
            self.assertIn("Would remove content/workflow/templates", result.stdout)
            self.assertIn("Would remove content/workflow/skills-core.md", result.stdout)
            self.assertTrue(generated.exists())
            self.assertTrue(generated_file.exists())
            self.assertTrue(source.exists())

    def test_apply_removes_generated_outputs_and_keeps_canonical_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _init_git_repo(repo)
            generated = repo / "packages/npm-qiongli/python-runtime/qiongli/__pycache__/module.pyc"
            mirrored_skill = repo / "content/workflow/skills/registry.yaml"
            source_skill = repo / "content/skills/registry.yaml"
            source_template = repo / "content/templates/paper-note.md"
            _write(generated)
            _write(mirrored_skill)
            _write(source_skill)
            _write(source_template)

            result = subprocess.run(
                [sys.executable, str(CLEANER_PATH), "--root", str(repo), "--apply"],
                cwd=REPO_ROOT,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(0, result.returncode, result.stderr)
            self.assertIn("Removed packages/npm-qiongli/python-runtime", result.stdout)
            self.assertIn("Removed content/workflow/skills", result.stdout)
            self.assertFalse((repo / "packages/npm-qiongli/python-runtime").exists())
            self.assertFalse((repo / "content/workflow/skills").exists())
            self.assertTrue(source_skill.exists())
            self.assertTrue(source_template.exists())

    def test_apply_refuses_existing_targets_that_are_not_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _init_git_repo(repo, ignore_generated=False)
            generated = repo / "qiongli-workflow/templates/paper-note.md"
            _write(generated)

            result = subprocess.run(
                [sys.executable, str(CLEANER_PATH), "--root", str(repo), "--apply"],
                cwd=REPO_ROOT,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(1, result.returncode)
            self.assertIn("not ignored by git", result.stderr)
            self.assertTrue(generated.exists())


if __name__ == "__main__":
    unittest.main()
