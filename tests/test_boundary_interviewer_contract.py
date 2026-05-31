from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = REPO_ROOT / "standards" / "boundary-review-contract.yaml"
TEMPLATE = REPO_ROOT / "templates" / "boundary-review.md"
ARTIFACT_TYPES = REPO_ROOT / "schemas" / "artifact-types.yaml"
REGISTRY = REPO_ROOT / "skills" / "registry.yaml"
SKILL = REPO_ROOT / "skills" / "Z_cross_cutting" / "boundary-interviewer.md"
SKILLS_CORE = REPO_ROOT / "skills-core.md"
SKILLS_SUMMARY = REPO_ROOT / "skills-summary.md"
README = REPO_ROOT / "README.md"
PAPER_WORKFLOW = REPO_ROOT / "qiongli-workflow" / "workflows" / "paper.md"
FIND_GAP_WORKFLOW = REPO_ROOT / "qiongli-workflow" / "workflows" / "find-gap.md"
STAGE_A_REFERENCE = REPO_ROOT / "qiongli-workflow" / "references" / "stage-A-framing.md"


class BoundaryInterviewerContractTests(unittest.TestCase):
    def test_contract_declares_academic_boundary_model(self) -> None:
        self.assertTrue(CONTRACT.is_file())
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))

        self.assertEqual(contract["name"], "boundary-review-contract")
        self.assertEqual(contract["artifact"], "context/boundary_review.md")
        self.assertEqual(contract["purpose"], "academic-boundary-clarification")
        self.assertEqual(contract["mvp_trigger_stages"], ["A", "C", "F", "H", "I"])
        self.assertEqual(contract["future_trigger_stages"], ["B", "D", "E", "G", "J", "K"])

        dimensions = set(contract["academic_boundary_dimensions"])
        for expected in (
            "phenomenon_boundary",
            "construct_boundary",
            "contribution_boundary",
            "claim_strength_boundary",
            "evidence_threshold_boundary",
            "method_validity_boundary",
            "rival_explanation_boundary",
            "generalizability_boundary",
            "ethics_governance_boundary",
            "venue_reviewer_boundary",
            "research_code_boundary",
            "submission_revision_boundary",
        ):
            self.assertIn(expected, dimensions)

    def test_contract_declares_academic_grill_loop_for_idea_discovery(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        grill_loop = contract["academic_grill_loop"]

        self.assertEqual(grill_loop["name"], "academic-grill-loop")
        self.assertEqual(grill_loop["purpose"], "academic-idea-discovery-and-boundary-critique")
        self.assertEqual(grill_loop["source_credit"]["inspired_by"], "Matt Pocock's grill-me skill")
        self.assertIn("https://github.com/mattpocock/skills", grill_loop["source_credit"]["url"])

        for expected in (
            "inspect_artifacts_before_asking",
            "one_scholarly_question_at_a_time",
            "recommended_answer_required",
            "paper_type_aware",
            "claim_evidence_aware",
            "rival_explanation_aware",
            "venue_reviewer_aware",
        ):
            self.assertIn(expected, grill_loop["adapted_principles"])

    def test_contract_stage_a_grill_prompts_help_find_academic_ideas(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        prompts = contract["academic_grill_loop"]["stage_prompts"]["A"]
        joined = "\n".join(prompts)

        self.assertIn("vague topic", joined)
        self.assertIn("defensible research idea", joined)
        self.assertIn("evidence would make", joined)
        self.assertIn("one paper", joined)

    def test_contract_requires_scholarly_decision_fields(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        required = set(contract["required_fields"])

        for expected in (
            "research_question_or_claim",
            "boundary_dimension",
            "question",
            "recommended_answer",
            "claim_strength",
            "evidence_threshold",
            "rival_explanations",
            "validity_or_trustworthiness_risk",
            "generalizability_limit",
            "venue_or_reviewer_risk",
            "decision_log_update",
            "revisit_trigger",
        ):
            self.assertIn(expected, required)

    def test_contract_has_stage_specific_academic_questions(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        stage_questions = contract["stage_question_sets"]

        self.assertIn("A", stage_questions)
        self.assertIn("C", stage_questions)
        self.assertIn("F", stage_questions)
        self.assertIn("H", stage_questions)
        self.assertIn("I", stage_questions)

        self.assertIn(
            "What evidence would make this research question answerable in one paper?",
            stage_questions["A"],
        )
        self.assertIn(
            "What rival explanation would make the preferred design insufficient?",
            stage_questions["C"],
        )
        self.assertIn(
            "Which central claim would a reviewer say exceeds the available evidence?",
            stage_questions["F"],
        )
        self.assertIn(
            "What promise in the cover letter or rebuttal cannot be truthfully supported?",
            stage_questions["H"],
        )
        self.assertIn(
            "Which code or data decision would change the scientific interpretation of the results?",
            stage_questions["I"],
        )

    def test_v2_contract_covers_all_academic_stages(self) -> None:
        contract = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
        stage_policy = contract["stage_boundary_policy"]

        self.assertEqual(set(stage_policy), set("ABCDEFGHIJK"))
        for stage, policy in stage_policy.items():
            self.assertIn("checkpoint_tasks", policy, stage)
            self.assertIn("default_level", policy, stage)
            self.assertIn(policy["default_level"], {"L1", "L2", "L3"}, stage)
            self.assertGreaterEqual(len(policy["dimensions"]), 2, stage)
            self.assertGreaterEqual(len(policy["questions"]), 2, stage)

        self.assertIn("B1", stage_policy["B"]["checkpoint_tasks"])
        self.assertIn("D1", stage_policy["D"]["checkpoint_tasks"])
        self.assertIn("E5", stage_policy["E"]["checkpoint_tasks"])
        self.assertIn("G3", stage_policy["G"]["checkpoint_tasks"])
        self.assertIn("J4", stage_policy["J"]["checkpoint_tasks"])
        self.assertIn("K4", stage_policy["K"]["checkpoint_tasks"])

    def test_template_preserves_academic_decision_record(self) -> None:
        self.assertTrue(TEMPLATE.is_file())
        template = TEMPLATE.read_text(encoding="utf-8")

        for heading in (
            "# Boundary Review",
            "## Scholarly Decision Context",
            "## Artifact Evidence Checked First",
            "## One-Question Academic Loop",
            "## Academic Boundary Map",
            "## Claim Strength And Evidence Threshold",
            "## Rival Explanations And Counterevidence",
            "## Validity Or Trustworthiness Risk",
            "## Generalizability Limit",
            "## Venue Or Reviewer Risk",
            "## Locked Decision",
            "## Revisit Trigger",
            "## Downstream Sync",
        ):
            self.assertIn(heading, template)

    def test_boundary_review_artifact_type_is_registered_as_academic_output(self) -> None:
        payload = yaml.safe_load(ARTIFACT_TYPES.read_text(encoding="utf-8"))
        artifact_types = {item["name"]: item for item in payload["artifact_types"]}

        self.assertIn("BoundaryReview", artifact_types)
        boundary_review = artifact_types["BoundaryReview"]
        self.assertEqual(boundary_review["format"], "markdown")
        self.assertIn("boundary-interviewer", boundary_review["produced_by"])
        self.assertIn("academic-context-maintainer", boundary_review["consumed_by"])
        self.assertIn("manuscript-architect", boundary_review["consumed_by"])

    def test_boundary_interviewer_skill_is_academic_not_generic_requirements(self) -> None:
        self.assertTrue(SKILL.is_file())
        registry = yaml.safe_load(REGISTRY.read_text(encoding="utf-8"))
        entries = {item["id"]: item for item in registry["skills"]}

        self.assertIn("boundary-interviewer", entries)
        entry = entries["boundary-interviewer"]
        self.assertEqual(entry["stage"], "Z_cross_cutting")
        self.assertEqual(entry["file"], "skills/Z_cross_cutting/boundary-interviewer.md")
        self.assertIn("BoundaryReview", entry["outputs"])
        self.assertIn("claim-strength", entry["tags"])
        self.assertIn("evidence-threshold", entry["tags"])

        skill_text = SKILL.read_text(encoding="utf-8")
        for phrase in (
            "finding, interpretation, and implication",
            "rival explanations",
            "validity or trustworthiness",
            "generalizability",
            "venue or reviewer risk",
        ):
            self.assertIn(phrase, skill_text)

        self.assertIn("## boundary-interviewer", SKILLS_CORE.read_text(encoding="utf-8"))
        self.assertIn("| boundary-interviewer |", SKILLS_SUMMARY.read_text(encoding="utf-8"))

    def test_boundary_interviewer_documents_academic_grill_adaptation_and_credit(self) -> None:
        skill_text = SKILL.read_text(encoding="utf-8")
        readme = README.read_text(encoding="utf-8")

        for phrase in (
            "Academic Grill Loop",
            "academic idea discovery",
            "not a generic grill-me clone",
            "Matt Pocock",
            "https://github.com/mattpocock/skills",
        ):
            self.assertIn(phrase, skill_text)

        self.assertIn("Matt Pocock", readme)
        self.assertIn("grill-me", readme)
        self.assertIn("academic idea-discovery", readme)

    def test_boundary_skill_documents_downstream_continuation(self) -> None:
        content = SKILL.read_text(encoding="utf-8")

        self.assertIn("After the user answers", content)
        self.assertIn("continue within the locked boundary", content)
        self.assertIn("must not broaden", content)
        self.assertIn("revisit_trigger", content)

    def test_mvp_workflows_include_academic_boundary_trigger(self) -> None:
        workflow_paths = [
            REPO_ROOT / "qiongli-workflow" / "workflows" / "paper.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "study-design.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "code-build.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "academic-write.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "submission-prep.md",
        ]

        for path in workflow_paths:
            content = path.read_text(encoding="utf-8")
            self.assertIn("Academic Boundary Review Trigger", content, path.as_posix())
            self.assertIn("boundary-interviewer", content, path.as_posix())
            self.assertIn("claim strength", content, path.as_posix())
            self.assertIn("evidence threshold", content, path.as_posix())

    def test_stage_a_and_find_gap_trigger_academic_grill_loop(self) -> None:
        stage_a = STAGE_A_REFERENCE.read_text(encoding="utf-8")
        paper = PAPER_WORKFLOW.read_text(encoding="utf-8")
        find_gap = FIND_GAP_WORKFLOW.read_text(encoding="utf-8")

        for label, content in (
            ("stage-A", stage_a),
            ("paper workflow", paper),
            ("find-gap workflow", find_gap),
        ):
            self.assertIn("Academic Grill Loop", content, label)
            self.assertIn("one scholarly question at a time", content, label)
            self.assertIn("recommended answer", content, label)
            self.assertIn("idea-discovery", content, label)

    def test_v2_workflows_include_boundary_trigger_for_all_remaining_stages(self) -> None:
        workflow_paths = [
            REPO_ROOT / "qiongli-workflow" / "workflows" / "lit-review.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "ethics-check.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "synthesize.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "compliance-check.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "proofread.md",
            REPO_ROOT / "qiongli-workflow" / "workflows" / "academic-present.md",
        ]

        for path in workflow_paths:
            content = path.read_text(encoding="utf-8")
            self.assertIn("boundary-interviewer", content, path.as_posix())
            self.assertIn("context/boundary_review.md", content, path.as_posix())
            self.assertIn("locked boundary", content, path.as_posix())


if __name__ == "__main__":
    unittest.main()
