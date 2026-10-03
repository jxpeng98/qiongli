from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

from scripts.audit_stage_handoffs import audit_stage_handoff


REPO_ROOT = Path(__file__).resolve().parents[1]


class StageHandoffContractTests(unittest.TestCase):
    def test_bundled_stage_handoff_assets_exist(self) -> None:
        for path in (
            RepoLayout(REPO_ROOT).workflow / "references" / "stage-handoff-contract.md",
            RepoLayout(REPO_ROOT).templates / "stage-handoff.md",
        ):
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"Missing {path}")

    def test_complete_handoff_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            handoff = Path(tmp_dir) / "stage-handoff.md"
            handoff.write_text(
                textwrap.dedent(
                    """\
                    # Stage Handoff

                    ## Completed Artifacts
                    - `framing/research_question.md`

                    ## Decision Summary
                    - D1: Scope narrowed.

                    ## Unresolved Questions
                    - None.

                    ## Evidence Dependencies
                    - `evidence/claim-evidence-ledger.csv`

                    ## Assumptions Passed Forward
                    - Venue remains CHI.

                    ## Risks For Next Stage
                    - Measurement validity needs review.

                    ## Recommended Next Tasks
                    - C1
                    """
                ),
                encoding="utf-8",
            )

            result = audit_stage_handoff(handoff)

        self.assertEqual([], result.errors)

    def test_missing_required_section_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            handoff = Path(tmp_dir) / "stage-handoff.md"
            handoff.write_text("# Stage Handoff\n\n## Completed Artifacts\n- file\n", encoding="utf-8")

            result = audit_stage_handoff(handoff)

        self.assertIn("Missing section: Decision Summary", "\n".join(result.errors))

    def test_nested_handoff_sections_pass(self) -> None:
        content = """# Stage Handoff
## Reviewed handoff
### Completed Artifacts
### Decision Summary
### Unresolved Questions
### Evidence Dependencies
### Assumptions Passed Forward
### Risks For Next Stage
### Recommended Next Tasks
"""
        with tempfile.TemporaryDirectory() as tmp_dir:
            handoff = Path(tmp_dir) / "stage_handoff.md"
            for level in range(3, 7):
                with self.subTest(level=level):
                    handoff.write_text(content.replace("### ", "#" * level + " "), encoding="utf-8")
                    self.assertEqual([], audit_stage_handoff(handoff).errors)


if __name__ == "__main__":
    unittest.main()
