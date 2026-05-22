from __future__ import annotations

import unittest

from bridges.providers.literature_diagnostics import build_search_diagnostics_v2


def _query_plan(search_mode: str = "review_grade") -> dict[str, object]:
    return {
        "search_mode": search_mode,
        "concept_blocks": [
            {
                "id": "c_population",
                "label": "Population",
                "terms": ["older adults"],
                "phrases": [],
                "required": True,
                "exclusions": [],
                "controlled_vocab": [],
            },
            {
                "id": "c_construct",
                "label": "Construct",
                "terms": ["conversational agents"],
                "phrases": [],
                "required": True,
                "exclusions": [],
                "controlled_vocab": [],
            },
            {
                "id": "c_context",
                "label": "Context",
                "terms": ["home health"],
                "phrases": [],
                "required": False,
                "exclusions": [],
                "controlled_vocab": [],
            },
        ],
        "provider_translations": [
            {
                "provider": "semantic_scholar",
                "query_id": "q1",
                "translated_query": "older adults conversational agents",
                "concept_ids": ["c_population", "c_construct"],
                "filters": {},
                "rationale": "primary concept query",
            },
            {
                "provider": "openalex",
                "query_id": "q2",
                "translated_query": "conversational agents home health",
                "concept_ids": ["c_construct", "c_context"],
                "filters": {},
                "rationale": "context sensitivity query",
            },
        ],
        "filters": {},
        "known_items": [
            {
                "title": "Known Medication Agents Study",
                "doi": "10.1234/known",
                "paper_id": "S2-KNOWN",
            }
        ],
        "stopping_rules": {"max_rounds": 2},
    }


def _passing_search_log() -> list[dict[str, object]]:
    return [
        {
            "query_id": "q1",
            "provider": "semantic_scholar",
            "status": "ok",
            "retrieved_count": 2,
        },
        {
            "query_id": "q2",
            "provider": "openalex",
            "status": "ok",
            "retrieved_count": 1,
        },
    ]


def _passing_results() -> list[dict[str, object]]:
    return [
        {
            "record_id": "s2:known",
            "source": "semantic_scholar",
            "query_id": "q1",
            "paper_id": "S2-KNOWN",
            "title": "Known Medication Agents Study",
            "doi": "https://doi.org/10.1234/Known",
        },
        {
            "record_id": "openalex:known",
            "source": "openalex",
            "query_id": "q2",
            "paper_id": "OA-KNOWN",
            "title": "Known Medication Agents Study",
            "doi": "10.1234/known",
        },
    ]


class LiteratureDiagnosticsV2Tests(unittest.TestCase):
    def test_targeted_search_warns_but_does_not_fail_for_single_successful_provider(self) -> None:
        plan = _query_plan(search_mode="targeted_search")
        plan["provider_translations"] = [plan["provider_translations"][0]]

        diagnostics = build_search_diagnostics_v2(
            plan,
            search_log=[
                {
                    "query_id": "q1",
                    "provider": "semantic_scholar",
                    "status": "ok",
                    "retrieved_count": 1,
                }
            ],
            search_results=[_passing_results()[0]],
            dedup_log=[],
            provider_summaries={
                "semantic_scholar": {
                    "status": "ok",
                    "normalized_hits": 1,
                    "failures": [],
                }
            },
        )

        self.assertEqual("targeted_search", diagnostics["search_mode"])
        self.assertEqual("warning", diagnostics["gate_status"])
        self.assertEqual([], diagnostics["blocking_reasons"])
        self.assertEqual(1, diagnostics["provider_coverage"]["success_count"])
        self.assertIn("single_successful_provider", diagnostics["warnings"])

    def test_targeted_search_fails_only_for_hard_blockers(self) -> None:
        diagnostics = build_search_diagnostics_v2(
            {"search_mode": "targeted_search", "concept_blocks": [], "provider_translations": []},
            search_log=[
                {
                    "query_id": "q1",
                    "provider": "semantic_scholar",
                    "status": "error",
                    "retrieved_count": 0,
                    "error": "rate limited",
                }
            ],
            search_results=[],
            dedup_log=[],
            provider_summaries={
                "semantic_scholar": {
                    "status": "error",
                    "normalized_hits": 0,
                    "failures": [{"query_id": "q1", "error": "rate limited"}],
                }
            },
            raw_diagnostics={
                "all_providers_failed": True,
                "zero_hit": True,
                "normalized_result_count": 0,
            },
        )

        self.assertEqual("fail", diagnostics["gate_status"])
        self.assertIn("invalid_query_plan", diagnostics["blocking_reasons"])
        self.assertIn("all_providers_failed", diagnostics["blocking_reasons"])
        self.assertIn("zero_hits", diagnostics["blocking_reasons"])

    def test_review_grade_passes_with_two_providers_required_concepts_and_known_item(self) -> None:
        diagnostics = build_search_diagnostics_v2(
            _query_plan(search_mode="review_grade"),
            search_log=_passing_search_log(),
            search_results=_passing_results(),
            dedup_log=[
                {
                    "candidate_record_id": "openalex:known",
                    "canonical_record_id": "s2:known",
                    "match_basis": "doi",
                }
            ],
            provider_summaries={
                "semantic_scholar": {"status": "ok", "normalized_hits": 1, "failures": []},
                "openalex": {"status": "ok", "normalized_hits": 1, "failures": []},
            },
            raw_diagnostics={"normalized_result_count": 2, "duplicate_count": 1},
        )

        self.assertEqual("pass", diagnostics["gate_status"])
        self.assertEqual([], diagnostics["blocking_reasons"])
        self.assertTrue(diagnostics["concept_coverage"]["all_required_covered"])
        self.assertEqual([], diagnostics["concept_coverage"]["missing_required_concepts"])
        self.assertEqual(1, diagnostics["known_item_recall"]["recalled_count"])
        self.assertEqual([], diagnostics["known_item_recall"]["missing_items"])
        self.assertEqual(2, diagnostics["provider_coverage"]["success_count"])
        self.assertEqual(1, diagnostics["provider_overlap"]["overlap_count"])
        self.assertEqual(1, diagnostics["dedup_health"]["duplicate_count"])
        self.assertEqual(0.5, diagnostics["dedup_health"]["duplicate_rate"])

    def test_review_grade_blocks_missing_provider_concept_and_known_item(self) -> None:
        plan = _query_plan(search_mode="review_grade")
        plan["provider_translations"] = [plan["provider_translations"][0]]
        plan["provider_translations"][0]["concept_ids"] = ["c_population"]

        diagnostics = build_search_diagnostics_v2(
            plan,
            search_log=[
                {
                    "query_id": "q1",
                    "provider": "semantic_scholar",
                    "status": "ok",
                    "retrieved_count": 1,
                }
            ],
            search_results=[
                {
                    "record_id": "s2:other",
                    "source": "semantic_scholar",
                    "query_id": "q1",
                    "paper_id": "OTHER",
                    "title": "A Different Study",
                    "doi": "10.9999/other",
                }
            ],
            dedup_log=[],
            provider_summaries={
                "semantic_scholar": {"status": "ok", "normalized_hits": 1, "failures": []},
            },
        )

        self.assertEqual("fail", diagnostics["gate_status"])
        self.assertIn("less_than_two_successful_providers", diagnostics["blocking_reasons"])
        self.assertIn("missing_required_concepts", diagnostics["blocking_reasons"])
        self.assertIn("missing_known_items", diagnostics["blocking_reasons"])
        self.assertEqual(["c_construct"], diagnostics["concept_coverage"]["missing_required_concepts"])
        self.assertEqual(0, diagnostics["known_item_recall"]["recalled_count"])

    def test_systematic_review_inherits_review_grade_and_requires_b1_readiness(self) -> None:
        diagnostics = build_search_diagnostics_v2(
            _query_plan(search_mode="systematic_review"),
            search_log=_passing_search_log(),
            search_results=_passing_results(),
            dedup_log=[],
            provider_summaries={
                "semantic_scholar": {"status": "ok", "normalized_hits": 1, "failures": []},
                "openalex": {"status": "ok", "normalized_hits": 1, "failures": []},
            },
            raw_diagnostics={
                "screening_readiness": {"usable": True, "reason": "stable records"},
                "snowball_readiness": {"usable": False, "reason": "citation graph missing"},
            },
        )

        self.assertEqual("fail", diagnostics["gate_status"])
        self.assertTrue(diagnostics["screening_readiness"]["usable"])
        self.assertFalse(diagnostics["snowball_readiness"]["usable"])
        self.assertIn("snowball_not_ready", diagnostics["blocking_reasons"])


if __name__ == "__main__":
    unittest.main()
