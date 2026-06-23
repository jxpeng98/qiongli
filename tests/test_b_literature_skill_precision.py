from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.audit_b_literature_skill_precision import (
    audit_b_literature_skill_precision,
    render_markdown_report,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class BLiteratureSkillPrecisionTests(unittest.TestCase):
    def _write_file(self, root: Path, rel: str, body: str) -> Path:
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(body).strip() + "\n", encoding="utf-8")
        return path

    def test_fixture_flags_direct_api_defaults_and_missing_evidence_limits(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_file(
                root,
                "content/skills/B_literature/academic-searcher.md",
                """\
                ---
                id: academic-searcher
                stage: B_literature
                ---

                # Academic Searcher

                ## Purpose

                Search databases.

                ## Inputs

                Use a research question.

                ## Process

                Call the Semantic Scholar API directly, then fall back to
                Google Scholar when results look thin.

                ## Output Contract

                Write `RESEARCH/[topic]/search_results.csv`.

                ## Quality Bar

                Results should be relevant.

                ## Common Pitfalls

                Searches may miss papers.
                """,
            )
            self._write_file(
                root,
                "content/skills/B_literature/paper-extractor.md",
                """\
                ---
                id: paper-extractor
                stage: B_literature
                ---

                # Paper Extractor

                ## Purpose

                Extract paper details.

                ## Inputs

                Use papers.

                ## Process

                Summarize findings and limitations.

                ## Output Contract

                Write `RESEARCH/[topic]/extraction_table.md`.

                ## Quality Bar

                Extract useful information.

                ## Common Pitfalls

                Missing details.
                """,
            )
            self._write_file(
                root,
                "content/skills-core.md",
                """\
                # Skills Core Reference

                ## academic-searcher

                Search Semantic Scholar API directly and use Google Scholar as
                fallback.
                """,
            )

            result = audit_b_literature_skill_precision(root)
            report = render_markdown_report(result)

        self.assertTrue(result.has_gaps)
        self.assertIn("provider ownership", report)
        self.assertIn("evidence limits", report)
        self.assertIn("skills-core direct API defaults", report)

    def test_canonical_repository_initially_has_precision_gaps(self) -> None:
        result = audit_b_literature_skill_precision(REPO_ROOT)
        paths = {item.path.as_posix() for item in result.skill_results if item.issue_count}

        self.assertIn("content/skills/B_literature/academic-searcher.md", paths)
        self.assertIn("content/skills/B_literature/reference-manager-bridge.md", paths)


if __name__ == "__main__":
    unittest.main()
