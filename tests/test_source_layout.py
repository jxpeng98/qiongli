from __future__ import annotations

import unittest
import tempfile
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
        self.assertNotIn(Path(".gemini"), layout.generated_output_roots)
        self.assertIn(Path("packages/python-qiongli/src/qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("packages/npm-qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("packages/qiongli-plugin"), layout.generated_output_roots)
        self.assertIn(Path("packages/qiongli-next-plugin"), layout.generated_output_roots)
        self.assertIn(Path("plugins/qiongli"), layout.generated_output_roots)
        self.assertIn(Path("plugins/qiongli-next"), layout.generated_output_roots)
        self.assertIn(Path("qiongli-workflow"), layout.generated_output_roots)
        self.assertIn(Path("content/workflow/skills"), layout.generated_output_roots)
        self.assertIn(Path("content/workflow/templates"), layout.generated_output_roots)

    def test_payload_skills_prefers_root_registry_over_partial_content_mirror(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "skills").mkdir()
            (root / "skills" / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
            (root / "content" / "skills" / "domain-profiles").mkdir(parents=True)
            (root / "content" / "skills" / "domain-profiles" / "finance.yaml").write_text(
                "id: finance\n",
                encoding="utf-8",
            )

            layout = RepoLayout(root)

            self.assertEqual(root.resolve() / "skills", layout.skills)

    def test_resolves_legacy_source_paths_to_current_content_tree(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        self.assertEqual(layout.skills / "registry.yaml", layout.resolve_source_path("skills/registry.yaml"))
        self.assertEqual(
            layout.workflow / "references" / "workflow-contract.md",
            layout.resolve_source_path("qiongli-workflow/references/workflow-contract.md"),
        )
        self.assertEqual(
            layout.workflow / "workflows" / "paper.md",
            layout.resolve_source_path(".agent/workflows/paper.md"),
        )
        self.assertEqual(
            layout.workflow / "workflows",
            layout.resolve_source_path(".agent/workflows"),
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
