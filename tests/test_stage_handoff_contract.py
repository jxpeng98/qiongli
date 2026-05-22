from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.audit_stage_handoffs import audit_stage_handoff


REPO_ROOT = Path(__file__).resolve().parents[1]


class StageHandoffContractTests(unittest.TestCase):
    def test_bundled_stage_handoff_assets_exist(self) -> None:
        for path in (
            REPO_ROOT / "qiongli-workflow" / "references" / "stage-handoff-contract.md",
            REPO_ROOT / "templates" / "stage-handoff.md",
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


if __name__ == "__main__":
    unittest.main()
