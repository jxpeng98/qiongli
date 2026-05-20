import unittest

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


if __name__ == "__main__":
    unittest.main()
