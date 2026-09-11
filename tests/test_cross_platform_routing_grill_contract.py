from __future__ import annotations

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class CrossPlatformRoutingGrillContractTests(unittest.TestCase):
    def test_canonical_workflow_declares_cross_platform_trigger_contract(self) -> None:
        skill_text = read(LAYOUT.workflow / "SKILL.md")
        routing_text = read(LAYOUT.workflow / "references" / "platform-routing.md")
        combined = skill_text + "\n" + routing_text

        for phrase in (
            "Cross-Platform Trigger Contract",
            "Ambiguity Trigger",
            "academic research lifecycle",
            "does not require explicit",
            "Codex",
            "Claude",
            "CLI",
            "qiongli_orchestrator_route",
            "不知道怎么做",
            "not sure",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

    def test_stage_grill_contract_is_all_stage_and_cross_stage(self) -> None:
        boundary_text = read(LAYOUT.skills / "Z_cross_cutting" / "boundary-interviewer.md")
        critique_text = read(LAYOUT.skills / "Z_cross_cutting" / "self-critique.md")
        handoff_text = read(LAYOUT.workflow / "references" / "stage-handoff-contract.md")
        combined = boundary_text + "\n" + critique_text + "\n" + handoff_text

        for phrase in (
            "Stage-Aware Grill Contract",
            "Cross-Stage Grill Memory",
            "light automatic grill",
            "deep grill",
            "Open Grill Issues",
            "Resolved Grill Decisions",
            "Revisit Triggers",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

        for stage in ("Stage A", "Stage B", "Stage C", "Stage D", "Stage E", "Stage F", "Stage G", "Stage J", "Stage H", "Stage I", "Stage K"):
            with self.subTest(stage=stage):
                self.assertIn(stage, combined)

        self.assertNotIn("Future trigger stages", boundary_text)

    def test_writing_consumers_resolve_the_shared_contract(self) -> None:
        reference = "references/stage-F-writing.md"
        self.assertTrue((LAYOUT.workflow / reference).is_file())
        paths = (
            REPO_ROOT / "content" / "skills-core.md",
            LAYOUT.workflow / "SKILL.md",
            LAYOUT.workflow / "workflows" / "paper-write.md",
            LAYOUT.workflow / "workflows" / "academic-write.md",
            LAYOUT.skills / "F_writing" / "manuscript-architect.md",
            *(LAYOUT.roles / name for name in (
                "science-writer.yaml", "research-orchestrator.yaml", "pi.yaml"
            )),
        )
        for path in paths:
            with self.subTest(consumer=path):
                self.assertIn(reference, read(path))

    def test_writing_consumers_do_not_restore_retired_process_requirements(self) -> None:
        # These are the retired instructions, not desired prose to reproduce.
        paths = (
            REPO_ROOT / "content" / "skills-core.md",
            LAYOUT.workflow / "SKILL.md",
            LAYOUT.workflow / "references" / "stage-F-writing.md",
            LAYOUT.workflow / "workflows" / "paper-write.md",
            LAYOUT.workflow / "workflows" / "academic-write.md",
            LAYOUT.skills / "F_writing" / "manuscript-architect.md",
            LAYOUT.skills / "Z_cross_cutting" / "self-critique.md",
            *(LAYOUT.roles / name for name in (
                "science-writer.yaml", "research-orchestrator.yaml", "pi.yaml"
            )),
        )
        retired = (
            "do not draft the whole artifact in one uninterrupted pass",
            "require_chunk_level_confirmation: true",
            "every substantive paragraph must advance at least two",
            "every substantive paragraph must move beyond description into at least two",
            "standard runs require at least 2 review passes",
            "standard 2 passes, deep 3",
        )
        for path in paths:
            text = read(path).lower()
            for instruction in retired:
                with self.subTest(consumer=path, instruction=instruction):
                    self.assertNotIn(instruction, text)

    def test_design_consumers_resolve_the_shared_contract(self) -> None:
        reference = "references/stage-C-design.md"
        self.assertTrue((LAYOUT.workflow / reference).is_file())
        paths = (
            REPO_ROOT / "content" / "skills-core.md",
            LAYOUT.workflow / "workflows" / "study-design.md",
            *(LAYOUT.skills / "C_design" / name for name in (
                "study-designer.md", "rival-hypothesis-designer.md",
                "robustness-planner.md", "prereg-writer.md",
            )),
            *(LAYOUT.roles / name for name in (
                "pi.yaml", "methods-lead.yaml", "statistician.yaml",
                "compliance-officer.yaml",
            )),
            *(LAYOUT.templates / name for name in (
                "study-design.md", "preregistration-template.md",
            )),
        )
        for path in paths:
            with self.subTest(consumer=path):
                self.assertIn(reference, read(path))

    def test_preregistration_card_routes_to_its_contract_output(self) -> None:
        card = read(LAYOUT.skills / "C_design" / "prereg-writer.md")
        metadata = yaml.safe_load(card.split("---", 2)[1])
        task_ids = re.findall(
            r"`(C[0-9_]+)`",
            card.split("## Related Task IDs", 1)[1].split("## ", 1)[0],
        )
        contract = yaml.safe_load(read(LAYOUT.standards / "research-workflow-contract.yaml"))
        self.assertEqual(len(task_ids), 1)
        self.assertEqual(
            [output["artifact"] for output in metadata["outputs"]],
            contract["task_catalog"][task_ids[0]]["outputs"],
        )

    def test_structured_writing_and_history_tables_resolve_canonical_templates(self) -> None:
        for reference, template in (
            ("stage-F-writing.md", "claim-evidence-map.md"),
            ("stage-consolidation.md", "research-state.md"),
        ):
            with self.subTest(reference=reference):
                self.assertIn(
                    f"templates/{template}",
                    read(LAYOUT.workflow / "references" / reference),
                )
                self.assertTrue((LAYOUT.templates / template).is_file())

    def test_writing_role_uses_academic_writer_name_with_legacy_alias(self) -> None:
        role_text = read(LAYOUT.roles / "science-writer.yaml")
        capability_text = read(LAYOUT.standards / "mcp-agent-capability-map.yaml")

        for phrase in (
            "id: academic-writer",
            'display_name: "Academic Writer"',
            "legacy_ids:",
            "science-writer",
            "aliases:",
            "scholarly-writer",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, role_text)

        self.assertIn('mapped_role: "academic-writer"', capability_text)

    def test_stage_i_declares_academic_analysis_code_constraints(self) -> None:
        paths = (
            LAYOUT.workflow / "references" / "stage-I-code.md",
            LAYOUT.workflow / "workflows" / "code-build.md",
            LAYOUT.skills / "I_code" / "code-builder.md",
            LAYOUT.skills / "I_code" / "code-specification.md",
            LAYOUT.skills / "I_code" / "code-planning.md",
        )
        combined = "\n".join(read(path) for path in paths)

        for phrase in (
            "Academic Analysis Code",
            "estimand",
            "dataset lineage",
            "model diagnostics",
            "manuscript-facing",
            "service layers",
            "controllers",
            "analysis_plan_source",
            "manuscript_outputs",
            "robustness_checks",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

    def test_materialized_plugin_contains_routing_and_grill_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "dist-source"
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/materialize_distribution_payloads.py",
                    "--target",
                    "plugin",
                    "--out",
                    str(out),
                    "--force",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)

            plugin_skill = out / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
            skill_text = read(plugin_skill / "SKILL.md")
            self.assertEqual(skill_text, read(LAYOUT.workflow / "SKILL.md"))
            self.assertIn("references/platform-routing.md", skill_text)
            routing_text = read(plugin_skill / "references" / "platform-routing.md")
            boundary_text = read(plugin_skill / "skills" / "Z_cross_cutting" / "boundary-interviewer.md")
            paper_write_text = read(plugin_skill / "workflows" / "paper-write.md")
            manuscript_text = read(plugin_skill / "skills" / "F_writing" / "manuscript-architect.md")
            science_writer_text = read(plugin_skill / "roles" / "science-writer.yaml")
            orchestrator_role_text = read(plugin_skill / "roles" / "research-orchestrator.yaml")
            pi_role_text = read(plugin_skill / "roles" / "pi.yaml")
            writing_contract = read(plugin_skill / "references" / "stage-F-writing.md")
            self.assertEqual(
                writing_contract,
                read(LAYOUT.workflow / "references" / "stage-F-writing.md"),
            )
            for source in ("qiongli.md", "paper-read.md", "academic-write.md", "paper-write.md"):
                with self.subTest(workflow=source):
                    self.assertEqual(
                        read(plugin_skill / "workflows" / source),
                        read(LAYOUT.workflow / "workflows" / source),
                    )
            self.assertEqual(
                read(plugin_skill / "references" / "codex-workflow-wrapper.md"),
                read(LAYOUT.workflow / "references" / "codex-workflow-wrapper.md"),
            )

        for phrase in (
            "Cross-Platform Trigger Contract",
            "Ambiguity Trigger",
            "Stage-Aware Grill Contract",
            "Cross-Stage Grill Memory",
            "Writing Harness Contract",
            "mainline drift",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(
                    phrase,
                    "\n".join(
                        [
                            skill_text,
                            routing_text,
                            boundary_text,
                            paper_write_text,
                            manuscript_text,
                            science_writer_text,
                            orchestrator_role_text,
                            pi_role_text,
                            writing_contract,
                        ]
                    ),
                )


if __name__ == "__main__":
    unittest.main()
