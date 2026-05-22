from __future__ import annotations

import importlib.util
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "audit_literature_search_quality.py"


def load_audit_module():
    if not MODULE_PATH.exists():
        raise AssertionError(f"Missing audit script: {MODULE_PATH}")
    spec = importlib.util.spec_from_file_location(
        "audit_literature_search_quality",
        MODULE_PATH,
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def write_diagnostics(path: Path, metadata: str, *, omit_sections: tuple[str, ...] = ()) -> None:
    sections = {
        "Search Scope": "Mode, sources, dates, and review-grade intent are recorded.",
        "Known-Item Recall": "No seed or benchmark studies are missing.",
        "Provider Coverage": "Provider counts and failures are reconciled.",
        "Query Coverage": "Every concept block has at least one productive query.",
        "Deduplication Summary": "Duplicate decisions are tied to match bases.",
        "Coverage Gaps": "No unresolved database or concept gaps remain.",
        "Next Search Actions": "No additional search action is required.",
    }
    body = [
        "# Search Diagnostics",
        "",
        "```yaml",
        textwrap.dedent(metadata).strip(),
        "```",
        "",
    ]
    for heading, content in sections.items():
        if heading in omit_sections:
            continue
        body.extend([f"## {heading}", "", content, ""])
    path.write_text("\n".join(body), encoding="utf-8")


class LiteratureSearchQualityAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.audit_module = load_audit_module()

    def test_systematic_review_blocks_single_provider_search(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            diagnostics_path = Path(tmp_dir) / "search_diagnostics.md"
            write_diagnostics(
                diagnostics_path,
                """
                mode: systematic_review
                review_grade: true
                status: ok
                provider_coverage:
                  semantic_scholar: 12
                query_coverage:
                  q1: 12
                known_item_recall:
                  missing: []
                flags: []
                dedup_ratio: 0.08
                """,
            )

            result = self.audit_module.audit_literature_search_quality(
                diagnostics_path,
                task_id="B1",
            )

        joined = "\n".join(result.errors)
        self.assertIn("systematic_review", joined)
        self.assertIn("at least two providers", joined)
        self.assertEqual([], result.warnings)

    def test_targeted_search_warns_on_single_provider_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            diagnostics_path = Path(tmp_dir) / "search_diagnostics.md"
            write_diagnostics(
                diagnostics_path,
                """
                mode: targeted_search
                review_grade: false
                status: warning
                provider_coverage:
                  semantic_scholar: 7
                query_coverage:
                  q1: 7
                known_item_recall:
                  missing: []
                flags: []
                dedup_ratio: 0.0
                """,
            )

            result = self.audit_module.audit_literature_search_quality(
                diagnostics_path,
                task_id="B2",
            )

        self.assertEqual([], result.errors)
        self.assertIn("single provider", "\n".join(result.warnings))

    def test_missing_search_diagnostics_is_b1_failure(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir)

            result = self.audit_module.audit_literature_search_quality(
                project_root,
                task_id="B1",
            )

        joined = "\n".join(result.errors)
        self.assertIn("search_diagnostics.md", joined)
        self.assertIn("B1", joined)

    def test_valid_systematic_review_diagnostics_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            diagnostics_path = Path(tmp_dir) / "search_diagnostics.md"
            write_diagnostics(
                diagnostics_path,
                """
                mode: systematic_review
                review_grade: true
                status: ok
                provider_coverage:
                  semantic_scholar: 12
                  openalex: 9
                query_coverage:
                  q1: 12
                  q2: 5
                known_item_recall:
                  missing: []
                flags: []
                dedup_ratio: 0.15
                """,
            )

            result = self.audit_module.audit_literature_search_quality(
                diagnostics_path,
                task_id="B1",
            )

        self.assertEqual([], result.errors)
        self.assertEqual([], result.warnings)
        self.assertGreater(result.passed, 0)

    def test_v2_json_diagnostics_pass_for_valid_systematic_review(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            diagnostics_path = Path(tmp_dir) / "search_diagnostics.md"
            diagnostics_path.write_text(
                textwrap.dedent(
                    """\
                    # Search Diagnostics

                    ## Search Scope
                    Search scope recorded.

                    ## Known-Item Recall
                    Known items recalled.

                    ## Provider Coverage
                    Providers recorded.

                    ## Query Coverage
                    Query health recorded.

                    ## Deduplication Summary
                    Deduplication recorded.

                    ## Coverage Gaps
                    No unresolved gaps.

                    ## Next Search Actions
                    None.

                    ## Machine-Readable Diagnostics

                    ```json
                    {
                      "search_mode": "systematic_review",
                      "gate_status": "pass",
                      "blocking_reasons": [],
                      "warnings": [],
                      "provider_coverage": {
                        "success_count": 2,
                        "hit_counts": {
                          "semantic_scholar": 12,
                          "openalex": 8
                        }
                      },
                      "concept_coverage": {
                        "all_required_covered": true,
                        "missing_required_concepts": []
                      },
                      "known_item_recall": {
                        "missing_items": []
                      },
                      "screening_readiness": {
                        "usable": true
                      },
                      "snowball_readiness": {
                        "usable": true
                      }
                    }
                    ```
                    """
                ),
                encoding="utf-8",
            )

            result = self.audit_module.audit_literature_search_quality(
                diagnostics_path,
                task_id="B1",
            )

        self.assertEqual([], result.errors)
        self.assertEqual([], result.warnings)


if __name__ == "__main__":
    unittest.main()
