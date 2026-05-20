"""Stable schema constants for literature search planning and results."""

SEARCH_RESULT_FIELDS = (
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
)

SEARCH_LOG_FIELDS = (
    "query_id",
    "provider",
    "translated_query",
    "filters",
    "retrieved_at",
    "raw_count",
    "normalized_count",
    "status",
    "error",
)

QUERY_PLAN_REQUIRED_KEYS = (
    "search_mode",
    "concept_blocks",
    "provider_translations",
    "filters",
    "known_items",
    "stopping_rules",
)
