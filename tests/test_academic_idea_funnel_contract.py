from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = RepoLayout(REPO_ROOT).standards / "idea-funnel-contract.yaml"
TEMPLATE = RepoLayout(REPO_ROOT).templates / "idea-funnel.md"
ARTIFACT_TYPES = RepoLayout(REPO_ROOT).schemas / "artifact-types.yaml"
REGISTRY = RepoLayout(REPO_ROOT).skills / "registry.yaml"
SKILL = RepoLayout(REPO_ROOT).skills / "Z_cross_cutting" / "boundary-interviewer.md"
SKILLS_CORE = RepoLayout(REPO_ROOT).skills_core
SKILLS_SUMMARY = RepoLayout(REPO_ROOT).skills_summary
README = REPO_ROOT / "README.md"
QIONGLI_SKILL = RepoLayout(REPO_ROOT).workflow / "SKILL.md"
PAPER_WORKFLOW = RepoLayout(REPO_ROOT).workflow / "workflows" / "paper.md"
FIND_GAP_WORKFLOW = RepoLayout(REPO_ROOT).workflow / "workflows" / "find-gap.md"
STAGE_A_REFERENCE = RepoLayout(REPO_ROOT).workflow / "references" / "stage-A-framing.md"


class AcademicIdeaFunnelContractTests(unittest.TestCase):
    def test_contract_declares_academic_idea_funnel_artifact(self) -> None:
        self.assertTrue(CONTRACT.is_file())
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))

        self.assertEqual(contract["name"], "academic-idea-funnel-contract")
        self.assertEqual(contract["artifact"], "context/idea_funnel.md")
        self.assertEqual(contract["purpose"], "academic-idea-discovery-and-triage")
        self.assertEqual(contract["stage"], "A")
        self.assertIn("paper", contract["entrypoints"])
        self.assertIn("find-gap", contract["entrypoints"])
        self.assertIn("brainstorm", contract["entrypoints"])

    def test_contract_preserves_grill_me_credit_as_academic_adaptation(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        credit = contract["source_credit"]

        self.assertEqual(credit["inspired_by"], "Matt Pocock's grill-me skill")
        self.assertIn("https://github.com/mattpocock/skills", credit["url"])
        self.assertIn("academic adaptation", credit["adaptation_note"])
        self.assertIn("not a copied workflow", credit["adaptation_note"])

    def test_contract_requires_scholarly_triage_fields(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        required = set(contract["required_fields"])

        for expected in (
            "source_prompt",
            "candidate_ideas",
            "recommended_idea",
            "core_claim",
            "research_question",
            "candidate_gap",
            "contribution_type",
            "evidence_plan",
            "weakest_assumption",
            "rival_explanation",
            "venue_reviewer_risk",
            "next_stage_recommendation",
            "boundary_review_handoff",
        ):
            self.assertIn(expected, required)

    def test_template_records_complete_idea_funnel(self) -> None:
        self.assertTrue(TEMPLATE.is_file())
        template = TEMPLATE.read_text(encoding="utf-8")

        for heading in (
            "# Academic Idea Funnel",
            "Save to: `RESEARCH/[topic]/context/idea_funnel.md`",
            "## Source Prompt And Existing Artifacts",
            "## Candidate Idea Triage",
            "## Recommended Research Idea",
            "## Claim, Gap, And Contribution",
            "## Evidence Plan",
            "## Weakest Assumption And Rival Risk",
            "## Reviewer And Venue Fit",
            "## Next Stage Recommendation",
            "## Boundary Review Handoff",
        ):
            self.assertIn(heading, template)

    def test_artifact_type_and_registry_expose_idea_funnel(self) -> None:
        artifact_types = {
            item["name"]: item
            for item in yaml.safe_load(ARTIFACT_TYPES.read_text(encoding="utf-8"))["artifact_types"]
        }
        self.assertIn("AcademicIdeaFunnel", artifact_types)
        idea_funnel = artifact_types["AcademicIdeaFunnel"]
        self.assertEqual(idea_funnel["format"], "markdown")
        self.assertIn("boundary-interviewer", idea_funnel["produced_by"])
        self.assertIn("question-refiner", idea_funnel["consumed_by"])
        self.assertIn("gap-analyzer", idea_funnel["consumed_by"])

        entries = {
            item["id"]: item
            for item in yaml.safe_load(REGISTRY.read_text(encoding="utf-8"))["skills"]
        }
        self.assertIn("AcademicIdeaFunnel", entries["boundary-interviewer"]["outputs"])

    def test_skill_and_quick_references_document_idea_funnel(self) -> None:
        skill_text = SKILL.read_text(encoding="utf-8")

        for phrase in (
            "context/idea_funnel.md",
            "Candidate Idea Triage",
            "weakest assumption",
            "next_stage_recommendation",
            "boundary_review_handoff",
        ):
            self.assertIn(phrase, skill_text)

        self.assertIn("AcademicIdeaFunnel", SKILLS_CORE.read_text(encoding="utf-8"))
        self.assertIn("context/idea_funnel.md", SKILLS_SUMMARY.read_text(encoding="utf-8"))

    def test_workflows_trigger_idea_funnel_before_stage_a_outputs(self) -> None:
        for label, path in (
            ("qiongli skill", QIONGLI_SKILL),
            ("paper workflow", PAPER_WORKFLOW),
            ("find-gap workflow", FIND_GAP_WORKFLOW),
            ("stage-A reference", STAGE_A_REFERENCE),
        ):
            content = path.read_text(encoding="utf-8")
            self.assertIn("Academic Idea Funnel", content, label)
            self.assertIn("context/idea_funnel.md", content, label)
            self.assertIn("context/boundary_review.md", content, label)

    def test_readme_credits_academic_adaptation(self) -> None:
        readme = README.read_text(encoding="utf-8")

        self.assertIn("Academic Idea Funnel", readme)
        self.assertIn("Matt Pocock", readme)
        self.assertIn("academic adaptation", readme)
