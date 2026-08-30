from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPO_ROOT / "scripts" / "check_2x_native_change_boundary.sh"


class NativeChangeBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp_dir.name)
        self.run_git("init", "-b", "main")
        self.run_git("config", "user.name", "Qiongli Test")
        self.run_git("config", "user.email", "qiongli-test@example.invalid")
        self.write("README.md", "baseline\n")
        self.run_git("add", "README.md")
        self.run_git("commit", "-m", "baseline")
        self.base_ref = self.run_git("rev-parse", "HEAD").stdout.strip()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(self.repo), *args],
            check=True,
            capture_output=True,
            text=True,
        )

    def write(self, relative_path: str, content: str) -> None:
        path = self.repo / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def commit_paths(self, *relative_paths: str) -> None:
        self.run_git("add", *relative_paths)
        self.run_git("commit", "-m", "test change")

    def run_guard(
        self, github_output: Path | None = None
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        if github_output is not None:
            env["GITHUB_OUTPUT"] = str(github_output)
        return subprocess.run(
            [
                "bash",
                str(SCRIPT),
                "--repo-root",
                str(self.repo),
                "--base-ref",
                self.base_ref,
            ],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_allows_native_workspace_changes(self) -> None:
        path = "packages/qiongli-native/apps/qiongli/src/main.rs"
        self.write(path, "fn main() {}\n")
        self.commit_paths(path)

        result = self.run_guard()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native 2.x change boundary passed", result.stdout)
        self.assertIn("Native matrix required: true", result.stdout)

    def test_skips_native_matrix_for_evidence_only_changes(self) -> None:
        paths = (
            ".trellis/tasks/08-30-example/prd.md",
            ".trellis/workspace/example/journal.md",
            "docs/superpowers/acceptance/receipt.md",
            "docs/superpowers/roadmaps/qiongli-current-program-index.md",
            "docs/superpowers/roadmaps/qiongli-program-ledger-v1.json",
            "tooling/release/acceptance/rel-999.md",
        )
        for path in paths:
            self.write(path, "evidence\n")
        self.commit_paths(*paths)
        github_output = self.repo / "github-output.txt"

        result = self.run_guard(github_output)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native matrix required: false", result.stdout)
        self.assertTrue(github_output.exists())
        self.assertEqual(
            github_output.read_text(encoding="utf-8"),
            "native-matrix-required=false\n",
        )

    def test_requires_native_matrix_for_acceptance_fixtures(self) -> None:
        path = "tooling/release/acceptance/fixtures/release.json"
        self.write(path, "{}\n")
        self.commit_paths(path)

        result = self.run_guard()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native matrix required: true", result.stdout)

    def test_requires_native_matrix_for_unknown_top_level_acceptance_file(
        self,
    ) -> None:
        path = "tooling/release/acceptance/unexpected.json"
        self.write(path, "{}\n")
        self.commit_paths(path)
        github_output = self.repo / "github-output.txt"

        result = self.run_guard(github_output)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native matrix required: true", result.stdout)
        self.assertEqual(
            github_output.read_text(encoding="utf-8"),
            "native-matrix-required=true\n",
        )

    def test_requires_native_matrix_for_mixed_evidence_and_source_changes(
        self,
    ) -> None:
        paths = (
            ".trellis/tasks/08-30-example/prd.md",
            "packages/qiongli-native/apps/qiongli/src/main.rs",
        )
        for path in paths:
            self.write(path, "changed\n")
        self.commit_paths(*paths)

        result = self.run_guard()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native matrix required: true", result.stdout)

    def test_requires_native_matrix_for_an_empty_diff(self) -> None:
        result = self.run_guard()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native matrix required: true", result.stdout)

    def test_rejects_frozen_legacy_and_architecture_paths(self) -> None:
        paths = (
            "packages/python-qiongli/src/qiongli/__init__.py",
            "packages/qiongli-literature-mcpb/src/index.js",
            "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
            "tooling/migration/2x-branch-point.json",
            "docs/architecture/decisions/0201-native-executable-and-resource-architecture.md",
        )
        for path in paths:
            self.write(path, "changed\n")
        self.commit_paths(*paths)

        result = self.run_guard()

        self.assertEqual(result.returncode, 1)
        for path in paths:
            self.assertIn(path, result.stderr)
        self.assertIn(
            "Frozen 1.x or accepted architecture paths changed", result.stderr
        )


if __name__ == "__main__":
    unittest.main()
