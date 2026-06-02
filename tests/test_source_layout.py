from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout, discover_repo_root


REPO_ROOT = Path(__file__).resolve().parents[1]


class SourceLayoutTests(unittest.TestCase):
    def test_discover_repo_root_from_test_file(self) -> None:
        self.assertEqual(REPO_ROOT, discover_repo_root(Path(__file__)))

    def test_current_canonical_content_roots_exist(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        expected_files = (
            layout.workflow / "SKILL.md",
            layout.workflow / "VERSION",
            layout.skills / "registry.yaml",
            layout.templates / "idea-funnel.md",
            layout.standards / "research-workflow-contract.yaml",
            layout.roles / "pi.yaml",
            layout.venue_profiles / "nature.yaml",
            layout.subjects / "catalog.yaml",
            layout.skills_core,
            layout.skills_summary,
        )

        for path in expected_files:
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"{path} should exist")

    def test_current_package_and_tooling_roots_exist(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        expected_dirs = (
            layout.python_package,
            layout.research_skills_package,
            layout.bridges_package,
            layout.bridges_compat_package,
            layout.npm_package,
            layout.plugin_package,
            layout.agent_platform,
            layout.gemini_platform,
            layout.literature_mcpb_package,
            layout.tooling,
            layout.pipelines,
            layout.install,
            layout.scripts,
            layout.release,
            layout.evals,
            layout.eval_cases,
            layout.eval_rubrics,
            layout.eval_runner,
            layout.docs,
            layout.tests,
        )

        for path in expected_dirs:
            with self.subTest(path=path):
                self.assertTrue(path.is_dir(), f"{path} should be a directory")

    def test_materialized_output_roots_are_named(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        self.assertIn(Path(".agent"), layout.generated_output_roots)
        self.assertIn(Path(".gemini"), layout.generated_output_roots)
        self.assertIn(Path("packages/python-qiongli/src/qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("packages/npm-qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("plugins/qiongli"), layout.generated_output_roots)
        self.assertIn(Path("qiongli-workflow"), layout.generated_output_roots)
        self.assertIn(Path("content/workflow/skills"), layout.generated_output_roots)
        self.assertIn(Path("content/workflow/templates"), layout.generated_output_roots)

    def test_resolves_legacy_source_paths_to_current_content_tree(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        self.assertEqual(layout.skills / "registry.yaml", layout.resolve_source_path("skills/registry.yaml"))
        self.assertEqual(
            layout.workflow / "references" / "workflow-contract.md",
            layout.resolve_source_path("qiongli-workflow/references/workflow-contract.md"),
        )
        self.assertEqual(
            layout.agent_platform / "workflows" / "paper.md",
            layout.resolve_source_path(".agent/workflows/paper.md"),
        )
        self.assertEqual(
            layout.gemini_platform / "qiongli.md",
            layout.resolve_source_path(".gemini/qiongli.md"),
        )
        self.assertEqual(
            layout.scripts / "release_preflight.sh",
            layout.resolve_source_path("scripts/release_preflight.sh"),
        )
        self.assertEqual(
            layout.install / "install_manifest.tsv",
            layout.resolve_source_path("install/install_manifest.tsv"),
        )
        self.assertEqual(
            layout.release / "automation.md",
            layout.resolve_source_path("release/automation.md"),
        )
        self.assertEqual(layout.skills_core, layout.resolve_source_path("skills-core.md"))


if __name__ == "__main__":
    unittest.main()
