from __future__ import annotations

from collections.abc import Callable, Mapping
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.provider_config import (
    provider_capability_mode,
    provider_config_summary,
    redact_provider_config,
    resolve_provider_config,
)
from bridges.hybrid_search_router import build_hybrid_search_plan
from bridges.providers import arxiv_client, crossref_client, openalex_client, pubmed_client
from bridges.providers.s2_client import search_paper


LITERATURE_PROVIDER_CAPABILITIES: dict[str, dict[str, Any]] = {
    "openalex": {
        "status": "implemented",
        "capabilities": [
            "topic_search",
            "doi_lookup",
            "year_filter",
            "document_type_filter",
            "venue_metadata",
        ],
    },
    "semantic_scholar": {
        "status": "implemented",
        "capabilities": [
            "topic_search",
            "title_lookup",
            "doi_lookup",
            "year_filter",
            "venue_metadata",
        ],
    },
    "crossref": {
        "status": "implemented",
        "capabilities": [
            "topic_search",
            "doi_lookup",
            "year_filter",
            "document_type_filter",
            "reference_metadata",
        ],
    },
    "pubmed": {
        "status": "implemented",
        "capabilities": [
            "topic_search",
            "doi_lookup",
            "biomedical_topic_search",
            "year_filter",
        ],
    },
    "arxiv": {
        "status": "implemented",
        "capabilities": [
            "topic_search",
            "preprint_search",
            "arxiv_id_lookup",
            "year_filter",
            "category_filter",
        ],
    },
}


LITERATURE_TOOL_DEFINITIONS: list[dict[str, Any]] = [
    {
        "name": "qiongli_literature_status",
        "description": "Report configured literature providers and capability mode without exposing secrets.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": False},
    },
    {
        "name": "qiongli_search_plan",
        "description": "Plan provider MCP and platform-native literature search routing without executing search.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "platform": {"type": "string"},
                "native_search_available": {"type": "boolean"},
                "nativeSearchAvailable": {"type": "boolean"},
                "native_search_tools": {"type": "array", "items": {"type": "string"}},
                "nativeSearchTools": {"type": "array", "items": {"type": "string"}},
                "query_variants": {"type": "array", "items": {"type": "string"}},
                "queryVariants": {"type": "array", "items": {"type": "string"}},
                "include_working_papers": {"type": "boolean"},
                "includeWorkingPapers": {"type": "boolean"},
                "fromYear": {"type": ["integer", "string"]},
                "toYear": {"type": ["integer", "string"]},
                "search_mode": {
                    "type": "string",
                    "enum": ["auto", "topic", "title", "doi", "review", "systematic_review"],
                },
                "searchMode": {
                    "type": "string",
                    "enum": ["auto", "topic", "title", "doi", "review", "systematic_review"],
                },
                "venue_filter": {"type": "string"},
                "venueFilter": {"type": "string"},
                "document_types": {"type": "array", "items": {"type": "string"}},
                "documentTypes": {"type": "array", "items": {"type": "string"}},
            },
            "additionalProperties": True,
        },
    },
    {
        "name": "qiongli_literature_search",
        "description": "Search academic literature using the full Qiongli CLI MCP provider stack.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "number"},
                "per_query_limit": {"type": "number"},
                "per_provider_limit": {"type": "number"},
                "search_limit": {"type": "number"},
                "total_limit": {"type": "number"},
                "search_depth": {"type": "string"},
                "searchDepth": {"type": "string"},
                "search_mode": {
                    "type": "string",
                    "enum": ["auto", "topic", "title", "doi", "review", "systematic_review"],
                },
                "exact_title": {"type": "boolean"},
                "fromYear": {"type": ["integer", "string"]},
                "toYear": {"type": ["integer", "string"]},
                "venue_filter": {"type": "string"},
                "document_types": {"type": "array", "items": {"type": "string"}},
                "query_variants": {"type": "array", "items": {"type": "string"}},
            },
            "additionalProperties": True,
        },
    },
    {
        "name": "qiongli_literature_export_evidence",
        "description": "Export an auditable provider capability and search evidence snapshot.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": True},
    },
]

ProviderSearchFn = Callable[[dict[str, object], int], dict[str, object]]
PROVIDER_SEARCH_ORDER = ("semantic_scholar", "openalex", "crossref", "pubmed", "arxiv")


def handle_literature_status(args: dict[str, Any]) -> dict[str, Any]:
    cwd = _cwd_from_args(args)
    config = resolve_provider_config(cwd=cwd)
    summary = provider_config_summary(config)
    return {
        "providers": summary,
        "capability_mode": provider_capability_mode(summary),
        "capabilities": LITERATURE_PROVIDER_CAPABILITIES,
        "redacted_config": redact_provider_config(config),
    }


def handle_search_plan(args: dict[str, Any]) -> dict[str, Any]:
    status = handle_literature_status(args)
    capability_mode = str(status.get("capability_mode", "strategy_only") or "strategy_only")
    provider_status = status.get("providers")
    return build_hybrid_search_plan(
        args,
        provider_capability_mode=capability_mode,
        provider_status=provider_status if isinstance(provider_status, Mapping) else None,
    )


def handle_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    return run_literature_search(args)


def handle_literature_export_evidence(args: dict[str, Any]) -> dict[str, Any]:
    results = args.get("results", args.get("search_results", []))
    if not isinstance(results, list):
        results = []
    return {
        "artifact_type": "qiongli_literature_evidence_snapshot",
        "exported_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "query": str(args.get("query", "") or "").strip(),
        "provider_status": args.get("provider_status", {}),
        "search_plan": args.get("search_plan", args.get("query_plan", {})),
        "diagnostics": args.get("diagnostics", args.get("search_diagnostics", {})),
        "result_count": len(results),
        "results": results,
    }


def run_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.providers.literature_search import run_scholarly_search

    task_packet = _task_packet_from_search_args(args)
    provider_fns = _configured_provider_fns(args)
    if provider_fns:
        return run_scholarly_search(task_packet, search_paper, provider_fns=provider_fns)
    return run_scholarly_search(task_packet, search_paper)


def _task_packet_from_search_args(args: dict[str, Any]) -> dict[str, Any]:
    query = str(args.get("query", "") or "").strip()
    variants = args.get("query_variants", args.get("queryVariants", []))
    keywords = [query] if query else []
    if isinstance(variants, list):
        keywords.extend(str(item).strip() for item in variants if str(item).strip())
    per_query_limit = args.get("per_query_limit", args.get("perQueryLimit"))
    per_provider_limit = args.get("per_provider_limit", args.get("perProviderLimit"))
    return {
        "topic": query or "literature-search",
        "research_question": query,
        "keywords": keywords,
        "paper_type": _paper_type_from_search_mode(args.get("search_mode", args.get("searchMode"))),
        "search_mode": args.get("search_mode", args.get("searchMode", "auto")),
        "search_depth": args.get("search_depth", args.get("searchDepth")),
        "year_start": args.get("fromYear"),
        "year_end": args.get("toYear"),
        "venue_profile": args.get("venue_filter", args.get("venueFilter", "")),
        "publication_type": _first_document_type(args.get("document_types", args.get("documentTypes"))),
        "limit": args.get("limit"),
        "search_limit": args.get("search_limit", args.get("searchLimit")),
        "per_provider_limit": per_provider_limit,
        "per_query_limit": per_query_limit,
    }


def _configured_provider_fns(args: dict[str, Any]) -> dict[str, ProviderSearchFn]:
    config = resolve_provider_config(cwd=_cwd_from_args(args))
    providers = config.get("providers", {})
    providers = providers if isinstance(providers, Mapping) else {}
    provider_fns: dict[str, ProviderSearchFn] = {}

    for provider_name in PROVIDER_SEARCH_ORDER:
        raw_provider = providers.get(provider_name, {})
        if not isinstance(raw_provider, Mapping):
            continue
        if not raw_provider.get("enabled") or not raw_provider.get("configured"):
            continue
        provider_fn = _provider_search_fn(provider_name)
        if provider_fn is not None:
            provider_fns[provider_name] = provider_fn
    return provider_fns


def _provider_search_fn(provider_name: str) -> ProviderSearchFn | None:
    if provider_name == "semantic_scholar":
        return _s2_provider_search
    if provider_name == "openalex":
        return openalex_client.search
    if provider_name == "crossref":
        return crossref_client.search
    if provider_name == "pubmed":
        return pubmed_client.search
    if provider_name == "arxiv":
        return arxiv_client.search
    return None


def _s2_provider_search(translation: dict[str, object], limit: int) -> dict[str, object]:
    filters = translation.get("filters", {})
    filters = filters if isinstance(filters, Mapping) else {}
    return search_paper(
        str(translation.get("translated_query", "") or ""),
        limit,
        year_start=filters.get("year_start"),
        year_end=filters.get("year_end"),
        publication_type=str(filters.get("publication_type", "") or "") or None,
        venue=str(filters.get("venue", "") or "") or None,
    )


def _paper_type_from_search_mode(search_mode: Any) -> str:
    return "systematic-review" if str(search_mode).strip() == "systematic_review" else "empirical"


def _first_document_type(raw: Any) -> str:
    if isinstance(raw, list) and raw:
        return str(raw[0])
    return str(raw or "")


def _cwd_from_args(args: dict[str, Any]) -> Path:
    raw = str(args.get("cwd", "") or "").strip()
    return Path(raw).expanduser().resolve() if raw else Path.cwd()
