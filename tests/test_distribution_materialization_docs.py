from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DOC_PATH = REPO_ROOT / "docs" / "development" / "distribution-materialization.md"
PLUGIN_ARCHITECTURE_DOC = REPO_ROOT / "docs" / "advanced" / "plugin-first-architecture.md"
README_PATH = REPO_ROOT / "README.md"
CLI_DOC = REPO_ROOT / "docs" / "reference" / "cli.md"
CLI_ZH_DOC = REPO_ROOT / "docs" / "zh" / "reference" / "cli.md"
PUBLISH_PYPI_DOC = REPO_ROOT / "docs" / "advanced" / "publish-pypi.md"
PUBLISH_PYPI_ZH_DOC = REPO_ROOT / "docs" / "zh" / "advanced" / "publish-pypi.md"


def _normalize_whitespace(content: str) -> str:
    return " ".join(content.split())


class DistributionMaterializationDocsTests(unittest.TestCase):
    def test_docs_define_source_and_generated_boundaries(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        for token in (
            "Canonical source",
            "Repository structure",
            "Generated and materialized outputs",
            "`skills/`",
            "`templates/`",
            "`subjects/`",
            "`qiongli/payload/`",
            "`packages/npm-qiongli/payload/`",
            "`plugins/qiongli/skills/qiongli-workflow/`",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)

    def test_docs_describe_clean_checkout_qiongli_workflow_shape(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")
        normalized = _normalize_whitespace(content)

        for token in (
            "Clean checkout `qiongli-workflow/` shape",
            "`qiongli-workflow/SKILL.md`",
            "`qiongli-workflow/VERSION`",
            "`qiongli-workflow/agents/`",
            "`qiongli-workflow/references/`",
            "`qiongli-workflow/workflows/`",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)

        self.assertIn(
            "does not contain `qiongli-workflow/templates/`, "
            "`qiongli-workflow/standards/`, `qiongli-workflow/roles/`, "
            "or `qiongli-workflow/venue-profiles/`",
            normalized,
        )

    def test_docs_explain_output_free_checkout(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")
        normalized = _normalize_whitespace(content)

        self.assertIn("Output-free checkout", content)

        for token in (
            "Generated outputs are intentionally untracked",
            "clean checkout remains output-free",
            "release artifacts keep the same installed structure",
        ):
            with self.subTest(token=token):
                self.assertIn(token, normalized)

    def test_docs_explain_how_to_add_future_skills(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        for token in (
            "Adding a new skill",
            "`skills/<stage>/<skill-id>.md`",
            "`skills/registry.yaml`",
            "`subjects/catalog.yaml`",
            "Do not edit materialized copies",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)

    def test_docs_explain_how_to_add_subject_specific_packages(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        for token in (
            "Adding or extending a subject package",
            "`subjects/<subject-id>/skills/`",
            "`subjects/<subject-id>/overlays/`",
            "`python scripts/materialize_subject_package.py`",
            "`complete`",
            "`focused`",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)

    def test_docs_state_ci_and_pr_generation_policy(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        self.assertIn("Feature PRs should not commit generated outputs", content)
        self.assertIn("GitHub Actions may materialize payloads in a temporary workspace", content)
        self.assertIn("Release automation may materialize payloads in a staging workspace", content)

    def test_docs_define_source_only_feature_pr_contract(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")
        normalized = _normalize_whitespace(content)

        self.assertIn("Source-only feature development", content)
        self.assertIn(
            "Normal feature PRs should update canonical source, tests, and documentation only",
            normalized,
        )
        self.assertIn("package mirror directories are not review targets", normalized)

    def test_plugin_architecture_uses_staged_release_materialization(self) -> None:
        content = PLUGIN_ARCHITECTURE_DOC.read_text(encoding="utf-8")
        normalized = _normalize_whitespace(content)

        self.assertIn("release staging workspace", normalized)
        self.assertIn("does not write generated payloads back to the source checkout", normalized)
        self.assertNotIn("--target all --in-place` inside the release workspace", content)

    def test_subject_development_docs_point_to_staged_package_contract_tests(self) -> None:
        docs = {
            "README.md": README_PATH.read_text(encoding="utf-8"),
            "docs/reference/cli.md": CLI_DOC.read_text(encoding="utf-8"),
            "docs/zh/reference/cli.md": CLI_ZH_DOC.read_text(encoding="utf-8"),
        }

        for path, content in docs.items():
            with self.subTest(path=path):
                self.assertNotIn("npm payload tests", content)
                self.assertIn("staged materialization", content)
                self.assertIn("npm package contract tests", content)

    def test_publish_docs_use_staged_preflight_roots_for_manual_checks(self) -> None:
        docs = {
            "docs/advanced/publish-pypi.md": PUBLISH_PYPI_DOC.read_text(encoding="utf-8"),
            "docs/zh/advanced/publish-pypi.md": PUBLISH_PYPI_ZH_DOC.read_text(encoding="utf-8"),
        }

        for path, content in docs.items():
            with self.subTest(path=path):
                self.assertIn("--out /tmp/qiongli-dist --force", content)
                self.assertIn("bash scripts/pypi_preflight.sh --root /tmp/qiongli-dist", content)
                self.assertIn("bash scripts/npm_preflight.sh --root /tmp/qiongli-dist", content)
                self.assertNotIn("bash scripts/pypi_preflight.sh\n", content)
                self.assertNotIn("bash scripts/pypi_preflight.sh --no-build\n", content)
                self.assertNotIn("inject_project_toml.sh --root", content)

    def test_docs_show_unified_materializer_commands(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        for token in (
            "`python scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force`",
            "`python scripts/materialize_distribution_payloads.py --target plugin --out /tmp/qiongli-plugin --force`",
            "`python scripts/materialize_distribution_payloads.py --target all --in-place`",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)


if __name__ == "__main__":
    unittest.main()
