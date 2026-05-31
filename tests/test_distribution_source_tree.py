from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

GENERATED_OUTPUT_ROOTS = (
    "qiongli/payload",
    "packages/npm-qiongli/payload",
    "packages/npm-qiongli/python-runtime",
    "plugins/qiongli/skills/qiongli-workflow",
    "qiongli-workflow/venue-profiles",
)

CANONICAL_SOURCE_PATHS = (
    "qiongli-workflow/SKILL.md",
    "qiongli-workflow/references/workflow-contract.md",
    "qiongli-workflow/workflows/paper.md",
    "skills/registry.yaml",
    "templates/idea-funnel.md",
    "standards/research-workflow-contract.yaml",
    "roles/pi.yaml",
    "venue-profiles/nature.yaml",
    "subjects/catalog.yaml",
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
    def test_generated_distribution_outputs_are_not_tracked(self) -> None:
        tracked = _git_ls_files(GENERATED_OUTPUT_ROOTS)

        self.assertEqual([], tracked)

    def test_canonical_sources_remain_tracked(self) -> None:
        tracked = set(_git_ls_files(CANONICAL_SOURCE_PATHS))

        self.assertEqual(set(CANONICAL_SOURCE_PATHS), tracked)


if __name__ == "__main__":
    unittest.main()
