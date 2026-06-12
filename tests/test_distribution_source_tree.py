from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)

GENERATED_OUTPUT_ROOTS = tuple(str(path) for path in LAYOUT.generated_output_roots)

CANONICAL_SOURCE_PATHS = (
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "SKILL.md"),
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "references" / "workflow-contract.md"),
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "workflows" / "paper.md"),
    str(LAYOUT.skills.relative_to(REPO_ROOT) / "registry.yaml"),
    str(LAYOUT.templates.relative_to(REPO_ROOT) / "idea-funnel.md"),
    str(LAYOUT.standards.relative_to(REPO_ROOT) / "research-workflow-contract.yaml"),
    str(LAYOUT.roles.relative_to(REPO_ROOT) / "pi.yaml"),
    str(LAYOUT.venue_profiles.relative_to(REPO_ROOT) / "nature.yaml"),
    str(LAYOUT.subjects.relative_to(REPO_ROOT) / "catalog.yaml"),
)


def _git_ls_files(paths: tuple[str, ...]) -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "--", *paths],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return [line for line in result.stdout.splitlines() if line.strip()]


class DistributionSourceTreeTests(unittest.TestCase):
    def test_plugin_package_mirrors_are_generated_outputs(self) -> None:
        self.assertIn("packages/qiongli-plugin", GENERATED_OUTPUT_ROOTS)
        self.assertIn("packages/qiongli-next-plugin", GENERATED_OUTPUT_ROOTS)

    def test_generated_distribution_outputs_are_not_tracked(self) -> None:
        tracked = _git_ls_files(GENERATED_OUTPUT_ROOTS)

        self.assertEqual([], tracked)

    def test_canonical_sources_remain_tracked(self) -> None:
        tracked = set(_git_ls_files(CANONICAL_SOURCE_PATHS))

        self.assertEqual(set(CANONICAL_SOURCE_PATHS), tracked)


if __name__ == "__main__":
    unittest.main()
