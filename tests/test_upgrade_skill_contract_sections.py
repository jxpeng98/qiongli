from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.audit_skill_sections import audit_skills
from scripts.upgrade_skill_contract_sections import upgrade_skill_file


class UpgradeSkillContractSectionsTests(unittest.TestCase):
    def test_upgrade_inserts_inputs_and_output_contract_from_frontmatter(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "skills" / "A_framing" / "demo.md"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(
                textwrap.dedent(
                    """\
                    ---
                    id: demo
                    stage: A_framing
                    inputs:
                      - type: Topic
                        description: Raw topic
                    outputs:
                      - type: DemoArtifact
                        artifact: "framing/demo.md"
                    ---

                    # Demo Skill

                    ## Purpose

                    Refine a topic.

                    ## Process

                    Do the work.

                    ## Quality Bar

                    - Clear.

                    ## Common Pitfalls

                    - Avoid weak scope.
                    """
                ),
                encoding="utf-8",
            )

            changed = upgrade_skill_file(path)
            result = audit_skills(Path(tmp_dir))

            self.assertTrue(changed)
            text = path.read_text(encoding="utf-8")
            self.assertIn("## Inputs", text)
            self.assertIn("Raw topic", text)
            self.assertIn("## Output Contract", text)
            self.assertIn("RESEARCH/[topic]/framing/demo.md", text)
            self.assertTrue(result.skill_results[0].sections["Inputs"])
            self.assertTrue(result.skill_results[0].sections["Output Contract"])

    def test_upgrade_is_idempotent(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "skills" / "F_writing" / "demo.md"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(
                textwrap.dedent(
                    """\
                    ---
                    id: demo
                    stage: F_writing
                    outputs:
                      - artifact: "manuscript/demo.md"
                    ---

                    # Demo Skill

                    ## Purpose

                    Write.

                    ## Inputs

                    Use existing artifacts.
                    If a required input is missing or insufficient, write a gap note.

                    ## Process

                    Work.

                    ## Output Contract

                    Write `RESEARCH/[topic]/manuscript/demo.md`.
                    Separate finding, interpretation, and implication.
                    Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
                    Apply `references/academic-output-rubric.md`.

                    ## Quality Bar

                    - Clear.

                    ## Common Pitfalls

                    - Avoid drift.
                    """
                ),
                encoding="utf-8",
            )

            self.assertFalse(upgrade_skill_file(path))


if __name__ == "__main__":
    unittest.main()
