from __future__ import annotations

import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.audit_skill_sections import REQUIRED_SECTIONS, audit_skills, render_markdown_report


class AuditSkillSectionsTests(unittest.TestCase):
    def _write_skill(self, root: Path, rel: str, body: str) -> Path:
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(body).strip() + "\n", encoding="utf-8")
        return path

    def test_audit_reports_required_section_and_constraint_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_skill(
                root,
                "skills/A_framing/demo-skill.md",
                """\
                ---
                id: demo-skill
                stage: A_framing
                outputs:
                  - artifact: "framing/demo.md"
                ---

                # Demo Skill

                ## Purpose

                Refine a research question.

                ## Inputs

                Use `RESEARCH/[topic]/framing/research_question.md`; if inputs are missing, write a gap note.

                ## Process

                1. Separate finding, interpretation, and implication.
                2. Bind every claim to evidence.

                ## Output Contract

                Write `RESEARCH/[topic]/framing/demo.md`.

                ## Quality Bar

                - Do not invent citations, data, or statistical results.

                ## Common Pitfalls

                - Avoid platform-specific instructions.
                """,
            )

            result = audit_skills(root)

        self.assertEqual(result.total_skills, 1)
        self.assertEqual(result.section_coverage["Purpose"].present, 1)
        self.assertTrue(result.skill_results[0].is_complete)
        self.assertEqual(result.skill_results[0].missing_sections, [])
        self.assertEqual(result.skill_results[0].missing_constraints, [])

    def test_audit_prioritizes_core_stage_skill_with_missing_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_skill(
                root,
                "skills/F_writing/weak-skill.md",
                """\
                ---
                id: weak-skill
                stage: F_writing
                ---

                # Weak Skill

                ## Purpose

                Improve writing.
                """,
            )

            result = audit_skills(root)
            report = render_markdown_report(result)

        weak = result.skill_results[0]
        self.assertEqual(set(weak.missing_sections), set(REQUIRED_SECTIONS) - {"Purpose"})
        self.assertIn("F_writing", result.stage_coverage)
        self.assertIn("weak-skill.md", report)
        self.assertIn("First Batch Priority", report)

    def test_cli_writes_report_and_strict_fails_when_gaps_exist(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report_path = root / "report.md"
            self._write_skill(
                root,
                "skills/B_literature/weak-skill.md",
                """\
                ---
                id: weak-skill
                stage: B_literature
                ---

                # Weak Skill

                ## Purpose

                Search literature.
                """,
            )

            result = subprocess.run(
                [
                    "python3",
                    str(Path(__file__).resolve().parents[1] / "scripts" / "audit_skill_sections.py"),
                    "--root",
                    str(root),
                    "--output",
                    str(report_path),
                    "--strict",
                ],
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 1)
            self.assertTrue(report_path.is_file())
            self.assertIn("missing required skill quality coverage", result.stderr)


if __name__ == "__main__":
    unittest.main()
