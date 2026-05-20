from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class DisagreementAdjudicationTemplateTests(unittest.TestCase):
    def test_disagreement_matrix_includes_required_fields(self) -> None:
        template = REPO_ROOT / "templates" / "disagreement-matrix.md"
        content = template.read_text(encoding="utf-8")

        for field in (
            "issue_id",
            "codex_position",
            "claude_position",
            "evidence_refs",
            "risk_level",
            "final_decision",
        ):
            with self.subTest(field=field):
                self.assertIn(field, content)


if __name__ == "__main__":
    unittest.main()
