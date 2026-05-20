from __future__ import annotations

import re
from typing import Any


MAX_QUERY_VARIANTS = 4
STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "be",
    "by",
    "do",
    "for",
    "from",
    "how",
    "in",
    "into",
    "is",
    "of",
    "on",
    "or",
    "that",
    "the",
    "their",
    "this",
    "to",
    "use",
    "what",
    "when",
    "where",
    "which",
    "why",
    "with",
}


def build_structured_query_plan(task_packet: dict[str, Any]) -> dict[str, Any]:
    """Build a reproducible, provider-neutral query plan from a task packet."""
    keywords = _clean_keywords(task_packet.get("keywords"))
    research_question = _clean_text(task_packet.get("research_question"))
    concept_blocks = _build_concept_blocks(
        keywords=keywords,
        research_question=research_question,
        venue_profile=_clean_text(task_packet.get("venue_profile")).casefold(),
    )
    legacy_variants = _build_legacy_query_variants(task_packet)
    provider_translations = _build_default_translations(
        legacy_variants=legacy_variants,
        concept_blocks=concept_blocks,
        research_question=research_question,
        keywords=keywords,
    )

    return {
        "search_mode": _search_mode(task_packet.get("paper_type")),
        "concept_blocks": concept_blocks,
        "provider_translations": provider_translations,
        "filters": _build_filters(task_packet),
        "known_items": _normalize_known_items(task_packet.get("known_items")),
        "stopping_rules": {
            "max_rounds": 2,
            "stop_when_new_included_below": 3,
        },
        "legacy_query_variants": legacy_variants,
    }


def validate_query_plan(plan: dict[str, Any]) -> None:
    concept_blocks = plan.get("concept_blocks")
    if not isinstance(concept_blocks, list) or not concept_blocks:
        raise ValueError("Query plan must include concept blocks.")

    concept_ids: set[str] = set()
    has_required = False
    for block in concept_blocks:
        if not isinstance(block, dict):
            raise ValueError("Concept block must be a mapping.")
        block_id = _clean_text(block.get("id"))
        if not block_id:
            raise ValueError("Concept block must include an id.")
        concept_ids.add(block_id)
        has_required = has_required or bool(block.get("required"))

    if not has_required:
        raise ValueError("Query plan must mark at least one required concept block.")

    translations = plan.get("provider_translations", [])
    if not isinstance(translations, list):
        raise ValueError("Provider translations must be a list.")
    for translation in translations:
        if not isinstance(translation, dict):
            raise ValueError("Provider translation must be a mapping.")
        for concept_id in translation.get("concept_ids", []):
            if concept_id not in concept_ids:
                raise ValueError(f"Provider translation references unknown concept id: {concept_id}")

    for index, known_item in enumerate(plan.get("known_items", []), start=1):
        if not isinstance(known_item, dict):
            raise ValueError(f"Known item {index} must be a mapping.")
        title = _clean_text(known_item.get("title"))
        doi = _clean_text(known_item.get("doi"))
        paper_id = _clean_text(known_item.get("paper_id") or known_item.get("paperId"))
        if not title or not (doi or paper_id):
            raise ValueError(
                f"Known item {index} must include title and either DOI or paper id."
            )


def build_legacy_query_variants(task_packet: dict[str, Any]) -> list[dict[str, str]]:
    plan = build_structured_query_plan(task_packet)
    return [
        {
            "query_id": item["query_id"],
            "query": item["query"],
            "rationale": item["rationale"],
        }
        for item in plan["legacy_query_variants"]
    ]


def _build_concept_blocks(
    *,
    keywords: list[str],
    research_question: str,
    venue_profile: str,
) -> list[dict[str, Any]]:
    distilled_terms = _distill_question(research_question).split()
    population_source = keywords[:1] or distilled_terms[:2] or ["participants"]
    construct_source = keywords[1:3] or distilled_terms[2:5] or ["phenomenon"]
    context_source = keywords[3:5] or distilled_terms[5:8] or ["research context"]

    blocks = [
        _concept_block("c1_population", "Population or corpus", population_source, required=True),
        _concept_block("c2_construct", "Construct or intervention", construct_source, required=True),
        _concept_block("c3_context", "Context or setting", context_source, required=False),
    ]

    if venue_profile == "chi":
        blocks[2]["controlled_vocab"] = [
            "ACM CHI",
            "HCI",
            "human-computer interaction",
        ]

    return blocks


def _concept_block(
    block_id: str,
    label: str,
    source_items: list[str],
    *,
    required: bool,
) -> dict[str, Any]:
    phrases = [item for item in source_items if " " in item]
    terms: list[str] = []
    for item in source_items:
        if " " not in item:
            terms.append(item)
            continue
        terms.extend(_tokenize(item))

    return {
        "id": block_id,
        "label": label,
        "terms": _dedupe(terms),
        "phrases": _dedupe(phrases),
        "required": required,
        "exclusions": [],
        "controlled_vocab": [],
    }


def _build_default_translations(
    *,
    legacy_variants: list[dict[str, str]],
    concept_blocks: list[dict[str, Any]],
    research_question: str,
    keywords: list[str],
) -> list[dict[str, Any]]:
    concept_ids = [block["id"] for block in concept_blocks]
    translations: list[dict[str, Any]] = []

    broad_query = _first_query(legacy_variants) or " ".join(keywords) or research_question
    precise_query = _keyword_bundle(keywords) or research_question or broad_query
    sensitivity_query = _distill_question(research_question) or " ".join(keywords[:4]) or broad_query

    for query_type, query in (
        ("broad", broad_query),
        ("precise", precise_query),
        ("sensitivity_probe", sensitivity_query),
    ):
        cleaned = _clean_text(query)
        if not cleaned:
            continue
        translations.append(
            {
                "provider": "semantic_scholar",
                "query_id": f"q{len(translations) + 1}",
                "query_type": query_type,
                "translated_query": cleaned,
                "concept_ids": concept_ids,
                "filters": {},
            }
        )

    return translations


def _build_legacy_query_variants(task_packet: dict[str, Any]) -> list[dict[str, str]]:
    seen: set[str] = set()
    variants: list[dict[str, str]] = []

    def add_variant(query: Any, rationale: str) -> None:
        cleaned = _clean_text(query)
        if not cleaned:
            return
        key = cleaned.casefold()
        if key in seen:
            return
        seen.add(key)
        variants.append(
            {
                "query_id": f"q{len(variants) + 1}",
                "query": cleaned,
                "rationale": rationale,
            }
        )

    topic = _clean_text(task_packet.get("topic"))
    add_variant(topic, "topic seed")

    direct_query = _clean_text(task_packet.get("query"))
    add_variant(direct_query, "explicit query")

    research_question = _clean_text(task_packet.get("research_question"))
    if research_question:
        add_variant(research_question, "research question")
        distilled = _distill_question(research_question)
        if distilled and distilled.casefold() != research_question.casefold():
            add_variant(distilled, "distilled research question keywords")

    keyword_bundle = _keyword_bundle(_clean_keywords(task_packet.get("keywords")))
    add_variant(keyword_bundle, "keyword bundle")

    for alias_key in ("target_title", "title"):
        alias_value = _clean_text(task_packet.get(alias_key))
        add_variant(alias_value, f"{alias_key} seed")

    return variants[:MAX_QUERY_VARIANTS]


def _build_filters(task_packet: dict[str, Any]) -> dict[str, Any]:
    filters: dict[str, Any] = {}
    for key in ("year_start", "year_end", "language", "publication_type", "venue"):
        value = task_packet.get(key)
        if value not in (None, ""):
            filters[key] = value
    return filters


def _normalize_known_items(raw_known_items: Any) -> list[dict[str, str]]:
    if not isinstance(raw_known_items, list):
        return []

    known_items: list[dict[str, str]] = []
    for item in raw_known_items:
        if not isinstance(item, dict):
            known_items.append({})
            continue
        known_items.append(
            {
                "title": _clean_text(item.get("title")),
                "doi": _clean_text(item.get("doi") or item.get("DOI")),
                "paper_id": _clean_text(item.get("paper_id") or item.get("paperId")),
            }
        )
    return known_items


def _search_mode(raw_paper_type: Any) -> str:
    paper_type = _clean_text(raw_paper_type).casefold().replace("_", "-")
    if paper_type == "systematic-review":
        return "systematic_review"
    return "targeted_search"


def _clean_keywords(raw_keywords: Any) -> list[str]:
    if not isinstance(raw_keywords, list):
        return []
    return _dedupe([_clean_text(item) for item in raw_keywords if _clean_text(item)])


def _keyword_bundle(keywords: list[str]) -> str:
    cleaned: list[str] = []
    for item in keywords:
        cleaned.append(f"\"{item}\"" if " " in item else item)
    return " ".join(cleaned[:6])


def _distill_question(question: str) -> str:
    terms: list[str] = []
    seen: set[str] = set()
    for token in _tokenize(question):
        if token in STOPWORDS or token in seen:
            continue
        seen.add(token)
        terms.append(token)
        if len(terms) >= 8:
            break
    return " ".join(terms)


def _tokenize(text: str) -> list[str]:
    return re.findall(r"[A-Za-z0-9][A-Za-z0-9-]{2,}", text.lower())


def _first_query(variants: list[dict[str, str]]) -> str:
    if not variants:
        return ""
    return variants[0].get("query", "")


def _clean_text(value: Any) -> str:
    return " ".join(str(value or "").strip().split())


def _dedupe(items: list[str]) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for item in items:
        cleaned = _clean_text(item)
        if not cleaned:
            continue
        key = cleaned.casefold()
        if key in seen:
            continue
        seen.add(key)
        deduped.append(cleaned)
    return deduped
