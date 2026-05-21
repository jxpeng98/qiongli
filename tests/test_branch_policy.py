from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


class BranchPolicyTests(unittest.TestCase):
    def test_ci_workflows_cover_development_branch(self) -> None:
        for workflow in (".github/workflows/ci.yml", ".github/workflows/install-check.yml"):
            content = read(workflow)
            self.assertIn('branches: ["main", "master", "dev"]', content)

    def test_release_workflow_allows_beta_from_dev_but_stable_from_primary(self) -> None:
        content = read("scripts/release_automation.sh")
        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn("Stable releases use primary branch ($primary_branch); prerelease releases may run from $DEV_PRERELEASE_BRANCH", content)
        self.assertIn('push_branch="$current_branch"', content)

    def test_maintainer_policy_documents_official_plugin_and_branch_roles(self) -> None:
        content = read("docs/maintainer/release-branch-policy.md")
        self.assertIn("official plugin marketplace", content)
        self.assertIn("`dev`", content)
        self.assertIn("`main`", content)
        self.assertIn("stable release", content)

    def test_maintainer_policy_documents_naming_decision(self) -> None:
        content = read("docs/maintainer/naming-policy.md")
        self.assertIn("**Qiongli**", content)
        self.assertIn("**Qiongli Zhengche**", content)
        self.assertIn("**Zhengche**", content)
        self.assertIn("qiongli", content)


if __name__ == "__main__":
    unittest.main()
