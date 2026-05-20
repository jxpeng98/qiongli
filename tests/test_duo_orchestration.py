from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class DuoOrchestrationTemplateTests(unittest.TestCase):
    def test_duo_review_report_includes_required_sections(self) -> None:
        template = REPO_ROOT / "templates" / "duo-review-report.md"
        content = template.read_text(encoding="utf-8")

        for section in (
            "Findings",
            "Blocking Issues",
            "Required Revisions",
            "Adjudication",
        ):
            with self.subTest(section=section):
                self.assertIn(f"## {section}", content)


if __name__ == "__main__":
    unittest.main()
