import unittest

from bridges.providers.literature_query import (
    build_structured_query_plan,
    translate_query_for_provider,
    validate_query_plan,
)
from bridges.providers.literature_schema import (
    QUERY_PLAN_REQUIRED_KEYS,
    SEARCH_LOG_FIELDS,
    SEARCH_RESULT_FIELDS,
)


class LiteratureSchemaTests(unittest.TestCase):
    def test_search_result_fields_include_stable_contract_columns(self):
        expected_fields = {
            "record_id",
            "source",
            "query_id",
            "retrieved_at",
            "paper_id",
            "title",
            "authors",
            "year",
            "venue",
            "doi",
            "url",
            "abstract",
            "citation_count",
            "open_access_pdf_url",
            "provider_rank",
            "relevance_reason",
        }

        self.assertTrue(expected_fields.issubset(SEARCH_RESULT_FIELDS))

    def test_search_log_fields_include_stable_contract_columns(self):
        expected_fields = {
            "query_id",
            "provider",
            "translated_query",
            "filters",
            "retrieved_at",
            "raw_count",
            "normalized_count",
            "status",
            "error",
        }

        self.assertTrue(expected_fields.issubset(SEARCH_LOG_FIELDS))

    def test_query_plan_required_keys_include_reproducibility_keys(self):
        expected_keys = {
            "search_mode",
            "concept_blocks",
            "provider_translations",
            "filters",
            "known_items",
            "stopping_rules",
        }

        self.assertTrue(expected_keys.issubset(QUERY_PLAN_REQUIRED_KEYS))


class LiteratureQueryPlanningTests(unittest.TestCase):
    def test_systematic_review_chi_packet_builds_structured_query_plan(self):
        task_packet = {
            "paper_type": "systematic-review",
            "research_question": (
                "How do older adults use conversational agents to manage medication "
                "adherence in home health contexts?"
            ),
            "keywords": [
                "older adults",
                "conversational agents",
                "medication adherence",
                "home health",
            ],
            "venue_profile": "chi",
        }

        plan = build_structured_query_plan(task_packet)

        self.assertEqual(plan["search_mode"], "systematic_review")
        concept_blocks = plan["concept_blocks"]
        self.assertGreaterEqual(len(concept_blocks), 2)
        self.assertLessEqual(len(concept_blocks), 5)
        self.assertEqual(concept_blocks[0]["id"], "c1_population")
        self.assertEqual(concept_blocks[1]["id"], "c2_construct")
        self.assertEqual(concept_blocks[2]["id"], "c3_context")

        for block in concept_blocks:
            self.assertTrue(
                {"terms", "phrases", "required", "exclusions", "controlled_vocab"}.issubset(block)
            )

        query_types = {item["query_type"] for item in plan["provider_translations"]}
        self.assertIn("broad", query_types)
        self.assertIn("precise", query_types)
        self.assertIn("sensitivity_probe", query_types)
        validate_query_plan(plan)

    def test_task_packet_can_request_review_grade_search_mode(self):
        plan = build_structured_query_plan(
            {
                "paper_type": "empirical",
                "search_mode": "review_grade",
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
            }
        )

        self.assertEqual(plan["search_mode"], "review_grade")
        validate_query_plan(plan)

    def test_validate_query_plan_fails_without_required_concept(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
            }
        )
        for block in plan["concept_blocks"]:
            block["required"] = False

        with self.assertRaisesRegex(ValueError, "required concept block"):
            validate_query_plan(plan)

    def test_validate_query_plan_fails_for_unknown_translation_concept(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
            }
        )
        plan["provider_translations"][0]["concept_ids"] = ["c99_missing"]

        with self.assertRaisesRegex(ValueError, "unknown concept id"):
            validate_query_plan(plan)

    def test_validate_query_plan_fails_for_incomplete_known_item(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
                "known_items": [{"title": "Known Paper"}],
            }
        )

        with self.assertRaisesRegex(ValueError, "Known item"):
            validate_query_plan(plan)

    def test_validate_query_plan_requires_translation_reproducibility_fields(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
            }
        )
        plan["provider_translations"][0].pop("translated_query")

        with self.assertRaisesRegex(ValueError, "translated_query"):
            validate_query_plan(plan)

    def test_validate_query_plan_requires_at_least_one_provider_translation(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How do teams adopt AI assistants?",
                "keywords": ["teams", "AI assistants"],
            }
        )

        for missing_value in ([], None):
            with self.subTest(provider_translations=missing_value):
                plan["provider_translations"] = missing_value
                with self.assertRaisesRegex(ValueError, "Provider translations"):
                    validate_query_plan(plan)


class LiteratureQueryTranslationTests(unittest.TestCase):
    def _plan(self):
        return build_structured_query_plan(
            {
                "paper_type": "systematic-review",
                "research_question": (
                    "How do graph neural networks support uncertainty estimation "
                    "for clinical decision support?"
                ),
                "keywords": [
                    "graph neural networks",
                    "uncertainty estimation",
                    "clinical decision support",
                    "computer science",
                    "statistics",
                ],
                "year_start": 2020,
                "year_end": 2025,
                "publication_type": "journal-article",
            }
        )

    def assert_common_translation_fields(self, translation, provider):
        self.assertEqual(translation["provider"], provider)
        self.assertTrue(translation["query_id"])
        self.assertTrue(translation["translated_query"])
        self.assertIsInstance(translation["filters"], dict)
        self.assertTrue(translation["rationale"])

    def test_semantic_scholar_translation_is_readable_keyword_query(self):
        translation = translate_query_for_provider(self._plan(), "semantic_scholar")

        self.assert_common_translation_fields(translation, "semantic_scholar")
        self.assertIn("graph neural networks", translation["translated_query"])
        self.assertIn("uncertainty estimation", translation["translated_query"])
        self.assertNotIn(" AND ", translation["translated_query"])
        self.assertNotIn(" OR ", translation["translated_query"])
        self.assertNotIn("(", translation["translated_query"])
        self.assertNotIn(")", translation["translated_query"])
        self.assertNotIn("payload", translation)

    def test_openalex_translation_includes_search_filter_and_sort_payload(self):
        translation = translate_query_for_provider(self._plan(), "openalex")

        self.assert_common_translation_fields(translation, "openalex")
        payload = translation["payload"]
        self.assertEqual(payload["search"], translation["translated_query"])
        self.assertEqual(payload["sort"], "relevance_score:desc")
        self.assertIn("from_publication_date:2020-01-01", payload["filter"])
        self.assertIn("to_publication_date:2025-12-31", payload["filter"])
        self.assertIn("type:journal-article", payload["filter"])

    def test_crossref_translation_includes_bibliographic_date_and_type_payload(self):
        translation = translate_query_for_provider(self._plan(), "crossref")

        self.assert_common_translation_fields(translation, "crossref")
        payload = translation["payload"]
        self.assertEqual(payload["query.bibliographic"], translation["translated_query"])
        self.assertEqual(payload["filter"]["from-pub-date"], "2020-01-01")
        self.assertEqual(payload["filter"]["until-pub-date"], "2025-12-31")
        self.assertEqual(payload["filter"]["type"], "journal-article")

    def test_arxiv_translation_uses_fielded_clauses_for_cs_math_stat_topics(self):
        translation = translate_query_for_provider(self._plan(), "arxiv")

        self.assert_common_translation_fields(translation, "arxiv")
        query = translation["translated_query"]
        self.assertIn("all:", query)
        self.assertIn("ti:", query)
        self.assertIn("abs:", query)
        self.assertIn("cat:cs.*", translation["payload"]["filters"])
        self.assertIn("cat:math.*", translation["payload"]["filters"])
        self.assertIn("cat:stat.*", translation["payload"]["filters"])
        executable_query = translation["payload"]["search_query"]
        self.assertIn("cat:cs.*", executable_query)
        self.assertIn("submittedDate:[202001010000 TO *]", executable_query)
        self.assertIn("submittedDate:[* TO 202512312359]", executable_query)

    def test_provider_translation_sanitizes_query_and_filter_values(self):
        plan = build_structured_query_plan(
            {
                "research_question": "How does AI support clinical workflow?",
                "keywords": ['AI "assistant"', "clinical:workflow", "human, computer interaction"],
                "year_start": "2020-01-01",
                "year_end": "20x5",
                "publication_type": "journal,article",
            }
        )

        arxiv_translation = translate_query_for_provider(plan, "arxiv")
        self.assertIn('"AI assistant"', arxiv_translation["translated_query"])
        self.assertNotIn('"assistant""', arxiv_translation["translated_query"])
        self.assertNotIn("clinical:workflow", arxiv_translation["translated_query"])
        self.assertIn("submittedDate:[202001010000 TO *]", arxiv_translation["payload"]["search_query"])
        self.assertNotIn("20x5", arxiv_translation["payload"]["search_query"])

        openalex_translation = translate_query_for_provider(plan, "openalex")
        self.assertIn("from_publication_date:2020-01-01", openalex_translation["payload"]["filter"])
        self.assertNotIn("to_publication_date", openalex_translation["payload"]["filter"])
        self.assertNotIn("journal,article", openalex_translation["payload"]["filter"])

    def test_unknown_provider_returns_validation_error(self):
        with self.assertRaisesRegex(ValueError, "Unsupported provider"):
            translate_query_for_provider(self._plan(), "unknown_index")


if __name__ == "__main__":
    unittest.main()
