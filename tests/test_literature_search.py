from __future__ import annotations

import unittest

from bridges.providers.literature_search import (
    build_query_variants,
    dedupe_search_results,
    run_scholarly_search,
)


class LiteratureSearchBaselineTests(unittest.TestCase):
    def test_build_query_variants_uses_topic_question_and_keywords(self) -> None:
        task_packet = {
            "topic": "qualitative process research in management",
            "research_question": "How do founders use narratives to mobilize stakeholder support?",
            "keywords": ["qualitative research", "process theory", "stakeholder support"],
        }

        variants = build_query_variants(task_packet)

        self.assertGreaterEqual(len(variants), 3)
        self.assertEqual(variants[0]["query"], "qualitative process research in management")
        self.assertIn("How do founders use narratives", variants[1]["query"])
        self.assertTrue(any('"qualitative research"' in item["query"] for item in variants))

    def test_dedupe_search_results_merges_records_by_doi(self) -> None:
        records = [
            {
                "record_id": "s2:1",
                "query_id": "q1",
                "paper_id": "1",
                "title": "A Study of Platforms",
                "year": 2024,
                "doi": "10.1000/example",
                "authors": "A. Author",
                "citation_count": 4,
            },
            {
                "record_id": "s2:2",
                "query_id": "q2",
                "paper_id": "2",
                "title": "A Study of Platforms",
                "year": 2024,
                "doi": "10.1000/example",
                "authors": "",
                "citation_count": 8,
            },
        ]

        unique_records, dedup_log = dedupe_search_results(records)

        self.assertEqual(len(unique_records), 1)
        self.assertEqual(len(dedup_log), 1)
        self.assertEqual(dedup_log[0]["match_basis"], "doi")
        self.assertEqual(unique_records[0]["citation_count"], 8)
        self.assertEqual(unique_records[0]["query_ids"], "q1;q2")

    def test_run_scholarly_search_returns_bundle_aware_output(self) -> None:
        responses = {
            "qualitative governance": {
                "data": [
                    {
                        "paperId": "abc",
                        "title": "Qualitative Governance Research",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2023,
                        "abstract": "Study of governance in firms.",
                        "url": "https://example.com/paper-1",
                        "citationCount": 12,
                        "venue": "Academy of Management Journal",
                        "externalIds": {"DOI": "10.1000/xyz"},
                        "openAccessPdf": {"url": "https://example.com/paper-1.pdf"},
                    }
                ]
            },
            "governance firms": {
                "data": [
                    {
                        "paperId": "dup",
                        "title": "Qualitative Governance Research",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2023,
                        "abstract": "",
                        "url": "https://example.com/paper-1b",
                        "citationCount": 13,
                        "venue": "AMJ",
                        "externalIds": {"DOI": "https://doi.org/10.1000/xyz"},
                    }
                ]
            },
        }

        def fake_search(query: str, limit: int) -> dict[str, object]:
            del limit
            return responses.get(query, {"error": f"unexpected query: {query}", "data": []})

        task_packet = {
            "topic": "qualitative governance",
            "keywords": ["governance", "firms"],
            "per_query_limit": 5,
        }

        result = run_scholarly_search(
            task_packet,
            fake_search,
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["data"]["raw_result_count"], 2)
        self.assertEqual(result["data"]["unique_result_count"], 1)
        self.assertEqual(result["data"]["duplicate_count"], 1)
        self.assertEqual(result["data"]["artifact_bundle"]["dedup_log"], "dedup_log.csv")
        self.assertEqual(result["data"]["search_results"][0]["doi"], "10.1000/xyz")
        self.assertEqual(result["data"]["search_results"][0]["query_ids"], "q1;q2")
        self.assertEqual(result["data"]["search_log"][0]["query_id"], "q1")

    def test_run_scholarly_search_includes_baseline_execution_diagnostics(self) -> None:
        def fake_search(query: str, limit: int) -> dict[str, object]:
            self.assertEqual(limit, 5)
            return {
                "data": [
                    {
                        "paperId": f"paper-{query[:4]}",
                        "title": f"Result for {query}",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2024,
                    }
                ]
            }

        result = run_scholarly_search(
            {
                "topic": "qualitative governance",
                "keywords": ["governance", "firms"],
                "per_query_limit": 5,
            },
            fake_search,
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        data = result["data"]
        self.assertIn("query_plan", data)
        self.assertEqual(data["query_plan"]["legacy_query_variants"], data["query_variants"])
        self.assertIn("semantic_scholar", data["provider_summaries"])
        self.assertEqual(data["provider_summaries"]["semantic_scholar"]["status"], "ok")
        self.assertEqual(
            data["search_diagnostics"]["attempted_providers"],
            ["semantic_scholar"],
        )
        self.assertEqual(data["search_diagnostics"]["attempted_query_count"], 2)
        self.assertEqual(data["search_diagnostics"]["failed_query_count"], 0)
        self.assertEqual(data["search_diagnostics"]["raw_result_count"], 2)
        self.assertEqual(data["search_diagnostics"]["unique_result_count"], 2)
        self.assertFalse(data["search_diagnostics"]["all_providers_failed"])
        self.assertFalse(data["search_diagnostics"]["zero_hit"])
        self.assertEqual(data["search_log"][0]["translated_query"], "qualitative governance")
        self.assertEqual(data["search_log"][0]["filters"], {})

    def test_run_scholarly_search_with_provider_fns_reports_partial_failure_diagnostics(self) -> None:
        provider_calls: list[tuple[str, dict[str, object], int]] = []

        def semantic_scholar_provider(
            translation: dict[str, object],
            limit: int,
        ) -> dict[str, object]:
            provider_calls.append(("semantic_scholar", translation, limit))
            return {
                "data": [
                    {
                        "paperId": "s2-hit",
                        "title": "Provider Translated Result",
                        "authors": [{"name": "Taylor Chen"}],
                        "year": 2025,
                    }
                ]
            }

        def openalex_provider(
            translation: dict[str, object],
            limit: int,
        ) -> dict[str, object]:
            provider_calls.append(("openalex", translation, limit))
            return {"error": "openalex unavailable", "data": []}

        result = run_scholarly_search(
            {
                "topic": "platform governance",
                "keywords": ["platform governance", "capability building"],
                "year_start": 2020,
                "per_query_limit": 3,
            },
            lambda query, limit: {"error": "legacy path should not run", "data": []},
            retrieved_at="2026-03-25T12:00:00+00:00",
            provider_fns={
                "semantic_scholar": semantic_scholar_provider,
                "openalex": openalex_provider,
            },
        )

        data = result["data"]
        self.assertEqual(result["status"], "warning")
        self.assertEqual([call[0] for call in provider_calls], ["semantic_scholar", "openalex"])
        self.assertTrue(all(call[2] == 3 for call in provider_calls))
        self.assertEqual(
            data["search_diagnostics"]["attempted_providers"],
            ["semantic_scholar", "openalex"],
        )
        self.assertEqual(data["search_diagnostics"]["attempted_query_count"], 2)
        self.assertEqual(data["search_diagnostics"]["failed_query_count"], 1)
        self.assertFalse(data["search_diagnostics"]["all_providers_failed"])
        self.assertFalse(data["search_diagnostics"]["zero_hit"])
        self.assertEqual(data["provider_summaries"]["openalex"]["status"], "error")
        self.assertEqual(data["provider_summaries"]["semantic_scholar"]["normalized_hits"], 1)
        self.assertEqual(data["search_log"][0]["translated_query"], provider_calls[0][1]["translated_query"])
        self.assertEqual(data["search_log"][1]["filters"]["year_start"], "2020")
        self.assertEqual(data["search_results"][0]["source"], "semantic_scholar")
        self.assertEqual(data["search_results"][0]["query_id"], provider_calls[0][1]["query_id"])
        self.assertEqual(
            data["search_results"][0]["query_text"],
            provider_calls[0][1]["translated_query"],
        )

    def test_run_scholarly_search_reports_error_when_all_attempts_fail(self) -> None:
        result = run_scholarly_search(
            {
                "topic": "platform governance",
            },
            lambda query, limit: {"error": f"failed: {query}", "data": []},
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        diagnostics = result["data"]["search_diagnostics"]
        self.assertEqual(result["status"], "error")
        self.assertEqual(diagnostics["status_reason"], "all_attempted_queries_failed")
        self.assertTrue(diagnostics["all_providers_failed"])
        self.assertTrue(diagnostics["zero_hit"])
        self.assertEqual(diagnostics["failed_query_count"], diagnostics["attempted_query_count"])

    def test_run_scholarly_search_reports_warning_when_successful_attempts_return_zero_hits(self) -> None:
        result = run_scholarly_search(
            {
                "topic": "platform governance",
            },
            lambda query, limit: {"data": []},
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        diagnostics = result["data"]["search_diagnostics"]
        self.assertEqual(result["status"], "warning")
        self.assertEqual(diagnostics["status_reason"], "zero_hits")
        self.assertFalse(diagnostics["all_providers_failed"])
        self.assertTrue(diagnostics["zero_hit"])
        self.assertEqual(diagnostics["failed_query_count"], 0)

    def test_run_scholarly_search_does_not_mark_error_response_with_hits_as_all_failed(self) -> None:
        result = run_scholarly_search(
            {
                "topic": "platform governance",
            },
            lambda query, limit: {
                "error": "rate limited after partial page",
                "data": [
                    {
                        "paperId": "partial-hit",
                        "title": "Partial Result",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2024,
                    }
                ],
            },
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        diagnostics = result["data"]["search_diagnostics"]
        self.assertEqual(result["status"], "warning")
        self.assertEqual(diagnostics["status_reason"], "partial_provider_failure")
        self.assertFalse(diagnostics["all_providers_failed"])
        self.assertFalse(diagnostics["zero_hit"])
        self.assertEqual(diagnostics["raw_result_count"], 1)
        self.assertEqual(diagnostics["normalized_result_count"], 1)

    def test_run_scholarly_search_keeps_raw_and_normalized_hit_counts_separate(self) -> None:
        result = run_scholarly_search(
            {
                "topic": "platform governance",
            },
            lambda query, limit: {
                "data": [
                    {
                        "paperId": "valid-hit",
                        "title": "Valid Result",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2024,
                    },
                    "non-mapping-provider-row",
                ],
            },
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        data = result["data"]
        diagnostics = data["search_diagnostics"]
        self.assertEqual(data["raw_result_count"], 2)
        self.assertEqual(data["normalized_result_count"], 1)
        self.assertEqual(diagnostics["raw_result_count"], 2)
        self.assertEqual(diagnostics["normalized_result_count"], 1)
        self.assertEqual(data["provider_summaries"]["semantic_scholar"]["raw_hits"], 2)
        self.assertEqual(data["provider_summaries"]["semantic_scholar"]["normalized_hits"], 1)


if __name__ == "__main__":
    unittest.main()
