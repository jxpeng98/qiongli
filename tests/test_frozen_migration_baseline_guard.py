from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts import check_frozen_migration_baseline as guard


REPO_ROOT = Path(__file__).resolve().parents[1]


class FrozenMigrationBaselineGuardTests(unittest.TestCase):
    def _git(self, repo: Path, *arguments: str) -> str:
        completed = subprocess.run(
            ["git", "-C", str(repo), *arguments],
            check=True,
            stdout=subprocess.PIPE,
            text=True,
        )
        return completed.stdout.strip()

    def test_frozen_surface_includes_baseline_plan_and_schemas(self) -> None:
        protected = (
            "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
            "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json",
            "tooling/migration/qiongli-1x-baseline-plan.json",
            "tooling/migration/baseline-plan.schema.json",
            "tooling/migration/baseline-manifest.schema.json",
            "tooling/migration/oracle-fixture.schema.json",
        )
        self.assertTrue(all(guard.is_frozen_path(path) for path in protected))
        self.assertFalse(guard.is_frozen_path("tooling/migration/qiongli-2x-report.json"))

    def test_existing_anchor_rejects_modify_delete_and_rename_paths(self) -> None:
        changes = guard.frozen_changes(
            [
                "README.md",
                "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
                "tooling/migration/baselines/v1.19.0-beta.1/oracles/rust-lite.json",
                "tooling/migration/qiongli-1x-baseline-plan.json",
            ],
            base_has_frozen_anchor=True,
        )
        self.assertEqual(
            changes,
            [
                "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
                "tooling/migration/baselines/v1.19.0-beta.1/oracles/rust-lite.json",
                "tooling/migration/qiongli-1x-baseline-plan.json",
            ],
        )

    def test_base_without_anchor_allows_only_the_bootstrap_transition(self) -> None:
        self.assertEqual(
            guard.frozen_changes(
                [guard.FROZEN_BASELINE_ANCHOR],
                base_has_frozen_anchor=False,
            ),
            [],
        )

    def test_anchor_detection_requires_the_exact_manifest_path(self) -> None:
        responses = (
            guard.subprocess.CompletedProcess(["git"], 0, b"commit\n", b""),
            guard.subprocess.CompletedProcess(
                ["git"],
                0,
                (guard.FROZEN_BASELINE_ANCHOR + "\0").encode(),
                b"",
            ),
        )
        with mock.patch.object(guard, "_git", side_effect=responses):
            self.assertTrue(guard.base_contains_frozen_anchor(REPO_ROOT, "base"))

    def test_invalid_comparison_base_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            with self.assertRaisesRegex(guard.GuardError, "not a commit"):
                guard.base_contains_frozen_anchor(
                    Path(temp_name), "missing-comparison-base"
                )

    def test_git_comparison_rejects_a_rename_out_of_the_frozen_surface(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            repo = Path(temp_name)
            self._git(repo, "init", "--quiet")
            anchor = repo / guard.FROZEN_BASELINE_ANCHOR
            anchor.parent.mkdir(parents=True)
            anchor.write_text("{}\n", encoding="utf-8")
            self._git(repo, "add", ".")
            self._git(
                repo,
                "-c",
                "user.name=Qiongli Test",
                "-c",
                "user.email=qiongli-test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "freeze baseline",
            )
            base = self._git(repo, "rev-parse", "HEAD")
            moved = repo / "moved-manifest.json"
            anchor.rename(moved)
            self._git(repo, "add", "-A")
            self._git(
                repo,
                "-c",
                "user.name=Qiongli Test",
                "-c",
                "user.email=qiongli-test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "move baseline",
            )

            self.assertTrue(guard.base_contains_frozen_anchor(repo, base))
            changed = guard.changed_paths_from_git(repo, base)
            self.assertIn(guard.FROZEN_BASELINE_ANCHOR, changed)
            self.assertEqual(
                guard.frozen_changes(changed, base_has_frozen_anchor=True),
                [guard.FROZEN_BASELINE_ANCHOR],
            )


if __name__ == "__main__":
    unittest.main()
