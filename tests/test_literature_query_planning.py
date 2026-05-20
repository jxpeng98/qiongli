import unittest

from bridges.providers.literature_query import (
    build_structured_query_plan,
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


if __name__ == "__main__":
    unittest.main()
