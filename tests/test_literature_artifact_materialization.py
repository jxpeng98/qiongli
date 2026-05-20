from __future__ import annotations

import csv
import json
import tempfile
import unittest
from pathlib import Path

from bridges.providers.literature_artifacts import materialize_search_bundle
from bridges.providers.literature_schema import SEARCH_RESULT_FIELDS
from scripts.materialize_literature_search_bundle import (
    materialize_literature_search_bundle as materialize_search_bundle_cli,
)
from scripts.audit_literature_search_quality import audit_literature_search_quality


class LiteratureArtifactMaterializationTests(unittest.TestCase):
    def _search_output(self) -> dict[str, object]:
        return {
            "status": "warning",
            "summary": "Found one result with one diagnostics gate.",
            "data": {
                "provider_mode": "provider_translations",
                "query_plan": {
                    "search_mode": "systematic_review",
                    "concept_blocks": [
                        {"id": "c1", "label": "Topic", "terms": ["platform governance"]}
                    ],
                    "provider_translations": [
                        {
                            "query_id": "q1",
                            "provider": "semantic_scholar",
                            "translated_query": "platform governance",
                            "filters": {"year_start": "2020"},
                        }
                    ],
                    "filters": {"year_start": "2020"},
                    "known_items": [{"doi": "10.1000/seed"}],
                    "stopping_rules": {"max_rounds": 2},
                },
                "query_variants": [{"query_id": "q1", "query": "platform governance"}],
                "search_log": [
                    {
                        "query_id": "q1",
                        "provider": "semantic_scholar",
                        "translated_query": "platform governance",
                        "filters": {"year_start": "2020"},
                        "retrieved_at": "2026-03-25T00:00:00+00:00",
                        "retrieved_count": 1,
                        "status": "ok",
                        "error": "",
                    }
                ],
                "search_results": [
                    {
                        "record_id": "s2:seed",
                        "source": "semantic_scholar",
                        "query_id": "q1",
                        "retrieved_at": "2026-03-25T00:00:00+00:00",
                        "paper_id": "seed",
                        "title": "Platform Governance in Practice",
                        "authors": "Alex Smith",
                        "year": 2024,
                        "venue": "Organization Science",
                        "doi": "10.1000/seed",
                        "url": "https://example.com/seed",
                        "abstract": "Routines shape governance.",
                        "citation_count": 42,
                        "open_access_pdf_url": "https://example.com/seed.pdf",
                        "query_ids": "q1",
                    }
                ],
                "dedup_log": [
                    {
                        "candidate_record_id": "s2:dup",
                        "canonical_record_id": "s2:seed",
                        "decision": "merge_duplicate",
                        "match_basis": "doi",
                        "resolver": "builtin_scholarly_search",
                        "notes": "Merged duplicate DOI.",
                    }
                ],
                "search_diagnostics": {
                    "attempted_query_count": 1,
                    "unique_result_count": 1,
                    "status_reason": "hits_returned",
                    "screening_readiness": {
                        "ready": False,
                        "reason": "known item recall requires review",
                    },
                    "bundle_gate": {
                        "state": "needs_review",
                        "reason": "diagnostics require librarian review",
                    },
                },
            },
        }

    def test_materialize_search_bundle_writes_reproducible_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir) / "RESEARCH" / "demo-topic"
            search_output = self._search_output()

            written = materialize_search_bundle(project_root, search_output)

            self.assertEqual(
                set(written),
                {
                    "search_strategy",
                    "search_log",
                    "search_results",
                    "dedup_log",
                    "search_diagnostics",
                },
            )
            for filename in (
                "search_strategy.md",
                "search_log.md",
                "search_results.csv",
                "dedup_log.csv",
                "search_diagnostics.md",
            ):
                self.assertTrue((project_root / filename).exists(), filename)

            strategy_text = (project_root / "search_strategy.md").read_text(encoding="utf-8")
            self.assertIn("## Machine-Readable Search Plan", strategy_text)
            self.assertIn('"provider_translations"', strategy_text)

            diagnostics_text = (project_root / "search_diagnostics.md").read_text(encoding="utf-8")
            self.assertIn("## Screening Readiness", diagnostics_text)
            self.assertIn("## Bundle Gate State", diagnostics_text)
            self.assertIn("## Next Search Actions", diagnostics_text)
            diagnostics_json = diagnostics_text.split("```json", 1)[1].split("```", 1)[0]
            self.assertEqual(
                json.loads(diagnostics_json)["bundle_gate"]["state"],
                "needs_review",
            )
            audit_result = audit_literature_search_quality(
                project_root / "search_diagnostics.md",
                task_id="B2",
            )
            self.assertEqual([], audit_result.errors)

            with (project_root / "search_results.csv").open(
                encoding="utf-8",
                newline="",
            ) as handle:
                reader = csv.DictReader(handle)
                self.assertEqual(reader.fieldnames[: len(SEARCH_RESULT_FIELDS)], list(SEARCH_RESULT_FIELDS))
                rows = list(reader)
            self.assertEqual(rows[0]["doi"], "10.1000/seed")
            self.assertEqual(rows[0]["query_ids"], "q1")

    def test_cli_materializer_uses_canonical_bundle_writer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_root = Path(tmp_dir) / "RESEARCH" / "demo-topic"

            written = materialize_search_bundle_cli(self._search_output(), project_root)

            self.assertIn(project_root / "search_strategy.md", written)
            self.assertIn(project_root / "search_diagnostics.md", written)
            self.assertTrue((project_root / "search_strategy.md").exists())
            self.assertTrue((project_root / "dedup_log.csv").exists())


if __name__ == "__main__":
    unittest.main()
