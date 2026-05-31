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
            self.assertIn("release/acceptance/**", content)

    def test_ci_workflow_cancels_stale_runs_and_splits_test_tiers(self) -> None:
        content = read(".github/workflows/ci.yml")

        self.assertIn("concurrency:", content)
        self.assertIn("cancel-in-progress: true", content)
        self.assertIn("cache: pip", content)
        self.assertIn("test-tier: full", content)
        self.assertIn("test-tier: windows-smoke", content)
        self.assertIn("if: matrix.test-tier == 'full'", content)
        self.assertIn("if: matrix.test-tier == 'windows-smoke'", content)
        self.assertIn("python -m unittest tests.test_install_qiongli tests.test_bootstrap_qiongli tests.test_universal_installer tests.test_command_runtime tests.test_release_automation tests.test_branch_policy", content)
        self.assertIn("./scripts/release_preflight.sh --quick", content)

    def test_ci_syncs_skill_package_before_strict_research_validation(self) -> None:
        content = read(".github/workflows/ci.yml")
        sync_cmd = "python scripts/materialize_distribution_payloads.py --target all --in-place"
        validate_cmd = "python scripts/validate_research_standard.py --strict"

        self.assertIn(sync_cmd, content)
        self.assertIn(validate_cmd, content)
        self.assertLess(content.index(sync_cmd), content.index(validate_cmd))

    def test_ci_rejects_generated_payload_edits_before_sync_steps(self) -> None:
        content = read(".github/workflows/ci.yml")
        guard_cmd = "python scripts/check_generated_payload_edits.py --base-ref origin/dev"
        sync_cmd = "python scripts/materialize_distribution_payloads.py --target all --in-place"

        self.assertIn(guard_cmd, content)
        self.assertIn(sync_cmd, content)
        self.assertLess(content.index(guard_cmd), content.index(sync_cmd))

    def test_ci_syncs_npm_payload_after_injected_project_defaults(self) -> None:
        content = read(".github/workflows/ci.yml")
        inject_cmd = "bash scripts/inject_project_toml.sh"
        payload_cmd = "python scripts/materialize_distribution_payloads.py --target all --in-place"
        validate_cmd = "python scripts/validate_research_standard.py --strict"
        unit_cmd = "python -m unittest discover -s tests -v"

        self.assertIn(inject_cmd, content)
        self.assertIn(payload_cmd, content)
        self.assertIn(validate_cmd, content)
        self.assertIn(unit_cmd, content)
        self.assertLess(content.index(inject_cmd), content.index(payload_cmd))
        self.assertLess(content.index(payload_cmd), content.index(validate_cmd))
        self.assertLess(content.index(payload_cmd), content.index(unit_cmd))

    def test_release_workflow_allows_beta_from_dev_but_stable_from_primary(self) -> None:
        content = read("scripts/release_automation.sh")
        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn("Stable releases use primary branch ($primary_branch); prerelease releases may run from $DEV_PRERELEASE_BRANCH", content)
        self.assertIn('push_branch="$current_branch"', content)

    def test_maintainer_policy_documents_official_plugin_and_branch_roles(self) -> None:
        content = read("docs/maintainer/release-branch-policy.md")
        self.assertIn("official public marketplace", content)
        self.assertIn("jxpeng98/skillsplace", content)
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
