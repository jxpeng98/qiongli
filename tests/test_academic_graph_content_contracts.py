from __future__ import annotations

import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
TEMPLATES = ROOT / "content" / "templates"
STANDARDS = ROOT / "content" / "standards"
SKILLS = ROOT / "content" / "skills"


class AcademicGraphContentContractTests(unittest.TestCase):
    def test_idea_and_boundary_contracts_freeze_stable_ids(self) -> None:
        idea = yaml.safe_load(
            (STANDARDS / "idea-funnel-contract.yaml").read_text(encoding="utf-8")
        )
        boundary = yaml.safe_load(
            (STANDARDS / "boundary-review-contract.yaml").read_text(encoding="utf-8")
        )

        self.assertEqual(idea["stable_identity"]["field"], "idea_id")
        self.assertEqual(idea["stable_identity"]["format"], "IF-###")
        self.assertEqual(
            boundary["stable_identity"]["locked_decision_field"], "decision_id"
        )
        self.assertEqual(
            boundary["stable_identity"]["question_format"], "BQ-###"
        )

        boundary_template = (TEMPLATES / "boundary-review.md").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "| Decision ID | Decision | Rationale | Confidence | Evidence Basis | Downstream Impact |",
            boundary_template,
        )

    def test_literature_map_freezes_machine_readable_tables(self) -> None:
        template = (TEMPLATES / "literature-map.md").read_text(encoding="utf-8")
        for header in (
            "| Citekey | Primary Cluster ID | Secondary Cluster IDs | Evidence Limit | Source Anchor |",
            "| Cluster ID | Cluster Label | Basis | Core Argument | Representative Papers | Evidence Limits |",
            "| Gap ID | Open Problem | Cluster IDs | Source Anchors | Project Relevance | Status |",
            "| Source Cluster ID | Relation | Target Cluster ID | Source Anchor | Evidence Limit | Status |",
        ):
            self.assertIn(header, template)
        self.assertIn("Never renumber or reuse", template)

        skill = (SKILLS / "B_literature" / "literature-mapper.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("templates/literature-map.md", skill)
        self.assertIn("exact machine-readable", skill)

    def test_manuscript_claim_map_separates_citations_from_evidence(self) -> None:
        template = (TEMPLATES / "claim-evidence-map.md").read_text(encoding="utf-8")
        self.assertIn(
            "| Claim ID | Claim | Claim Type | Evidence Pointer | Citation Keys | Manuscript Location | Confidence | Action |",
            template,
        )
        self.assertIn("citation link records", template)
        self.assertIn("Never renumber or reuse", template)

        skill = (SKILLS / "F_writing" / "manuscript-architect.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("stable `CLM-###` claim IDs", skill)
        self.assertIn("citation edges record attribution", skill)


if __name__ == "__main__":
    unittest.main()
