from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]


class PaperReadingSummaryContractTests(unittest.TestCase):
    def test_b2_contract_declares_project_summary_outputs(self) -> None:
        contract = yaml.safe_load(
            (REPO_ROOT / "standards" / "research-workflow-contract.yaml").read_text(
                encoding="utf-8"
            )
        )

        outputs = set(contract["task_catalog"]["B2"]["outputs"])

        self.assertIn("literature/paper_reading_summary.md", outputs)
        self.assertIn("literature/paper_reading_matrix.md", outputs)

    def test_paper_read_workflow_enforces_truthful_summary_boundaries(self) -> None:
        content = (
            REPO_ROOT / "qiongli-workflow" / "workflows" / "paper-read.md"
        ).read_text(encoding="utf-8")

        for token in (
            "literature/paper_reading_summary.md",
            "literature/paper_reading_matrix.md",
            "evidence_limit: abstract_only",
            "Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications.",
            "direct_evidence",
            "reasonable_inference",
            "unsupported_gap",
        ):
            self.assertIn(token, content)

    def test_paper_note_template_requires_source_anchors_and_uncertainty(self) -> None:
        content = (REPO_ROOT / "templates" / "paper-note.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "Evidence Limit",
            "Source Anchors",
            "Author Claims",
            "Agent Interpretation",
            "Reusable Citation Points",
            "Uncertainty Register",
        ):
            self.assertIn(token, content)

    def test_project_summary_templates_exist_and_block_unsupported_claims(self) -> None:
        summary = (REPO_ROOT / "templates" / "paper-reading-summary.md").read_text(
            encoding="utf-8"
        )
        matrix = (REPO_ROOT / "templates" / "paper-reading-matrix.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "source_anchor",
            "evidence_limit",
            "unsupported_gap",
            "Do not upgrade an inference into a fact",
        ):
            self.assertIn(token, summary)
            self.assertIn(token, matrix)


if __name__ == "__main__":
    unittest.main()
