from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

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
            "Gemini",
            "CLI",
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
            boundary_text = read(plugin_skill / "skills" / "Z_cross_cutting" / "boundary-interviewer.md")

        for phrase in (
            "Cross-Platform Trigger Contract",
            "Ambiguity Trigger",
            "Stage-Aware Grill Contract",
            "Cross-Stage Grill Memory",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, skill_text + "\n" + boundary_text)


if __name__ == "__main__":
    unittest.main()
