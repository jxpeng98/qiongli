from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

from qiongli.workflow_contract_doc import generate_workflow_contract_reference


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_CONTRACT_DOC = RepoLayout(REPO_ROOT).workflow / "references" / "workflow-contract.md"


class WorkflowContractDocTests(unittest.TestCase):
    def test_generated_workflow_contract_matches_repo_copy(self) -> None:
        generated = generate_workflow_contract_reference(REPO_ROOT)
        actual = WORKFLOW_CONTRACT_DOC.read_text(encoding="utf-8")

        self.assertEqual(actual, generated)

    def test_generated_workflow_contract_includes_generated_marker_and_k_stage(self) -> None:
        generated = generate_workflow_contract_reference(REPO_ROOT)

        self.assertIn(
            "Auto-generated from `standards/research-workflow-contract.yaml`",
            generated,
        )
        self.assertIn("| `K4` | K | Beamer build | `presentation/beamer/`, `presentation/slides.bib` |", generated)
        self.assertIn("`references/stage-K-presentation.md`", generated)

    def test_stage_playbooks_surface_semantic_quality_gate_ids(self) -> None:
        required_tokens = {
            "content/workflow/references/stage-C-design.md": [
                "q1_rq_method_alignment",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-F-writing.md": [
                "q2_claim_evidence_traceability",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-G-compliance.md": [
                "q3_reporting_completeness",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-I-code.md": [
                "q4_reproducibility_baseline",
                "quality-gate-report.md",
            ],
        }

        missing: list[str] = []
        for relative_path, tokens in required_tokens.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            missing.extend(f"{relative_path}: {token}" for token in tokens if token not in text)

        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
