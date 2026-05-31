from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DOC_PATH = REPO_ROOT / "docs" / "development" / "distribution-materialization.md"


class DistributionMaterializationDocsTests(unittest.TestCase):
    def test_docs_define_source_and_generated_boundaries(self) -> None:
        content = DOC_PATH.read_text(encoding="utf-8")

        for token in (
            "Canonical source",
            "Generated outputs",
            "`skills/`",
            "`templates/`",
            "`subjects/`",
            "`qiongli/payload/`",
            "`packages/npm-qiongli/payload/`",
            "`plugins/qiongli/skills/qiongli-workflow/`",
        ):
            with self.subTest(token=token):
                self.assertIn(token, content)

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


if __name__ == "__main__":
    unittest.main()
