from __future__ import annotations

from collections.abc import Callable, Mapping
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.provider_config import (
    provider_config_summary,
    redact_provider_config,
    resolve_provider_config,
)
from bridges.hybrid_search_router import build_hybrid_search_plan
from bridges.providers import arxiv_client, crossref_client, openalex_client, pubmed_client
from bridges.providers.s2_client import search_paper


PROVIDER_STATUS_ORDER = ("openalex", "semantic_scholar", "crossref", "pubmed", "arxiv")
PROVIDER_SEARCH_ORDER = ("semantic_scholar", "openalex", "crossref", "pubmed", "arxiv")
PROVIDER_ACTIVATION_FIELDS = (
    ("openalex", "openalex.api_key"),
    ("semantic_scholar", "semantic_scholar.api_key"),
    ("crossref", "crossref.email"),
    ("pubmed", "pubmed.api_key"),
)
SEARCH_MODES = ("auto", "topic", "title", "doi", "review", "systematic_review")


LITERATURE_STATUS_INPUT_SCHEMA: dict[str, Any] = {
    "type": "object",
    "properties": {
        "cwd": {
            "type": "string",
            "minLength": 1,
            "maxLength": 4096,
            "pattern": ".*\\S.*",
            "description": (
                "Project-config context used by Full and accepted as a compatibility "
                "context by Lite."
            ),
        }
    },
    "additionalProperties": False,
}


SEARCH_PLAN_INPUT_SCHEMA: dict[str, Any] = {
    "type": "object",
    "required": ["query"],
    "properties": {
        "cwd": {
            "type": "string",
            "minLength": 1,
            "maxLength": 4096,
            "pattern": ".*\\S.*",
            "description": (
                "Project-config context used by Full and accepted as a compatibility "
                "context by Lite."
            ),
        },
        "query": {
            "type": "string",
            "minLength": 1,
            "maxLength": 4096,
            "pattern": ".*\\S.*",
        },
        "platform": {
            "type": "string",
            "minLength": 1,
            "maxLength": 64,
            "pattern": "^[A-Za-z0-9]+(?:[ _-]+[A-Za-z0-9]+)*[ _-]*$",
        },
        "native_search_available": {"type": "boolean", "default": False},
        "native_search_usable": {
            "type": "boolean",
            "deprecated": True,
            "description": "Compatibility alias for native_search_available.",
        },
        "nativeSearchAvailable": {
            "type": "boolean",
            "deprecated": True,
            "description": "Compatibility alias for native_search_available.",
        },
        "native_search_tools": {"$ref": "#/$defs/toolList"},
        "nativeSearchTools": {
            "$ref": "#/$defs/toolList",
            "deprecated": True,
            "description": "Compatibility alias for native_search_tools.",
        },
        "query_variants": {"$ref": "#/$defs/queryList"},
        "queryVariants": {
            "$ref": "#/$defs/queryList",
            "deprecated": True,
            "description": "Compatibility alias for query_variants.",
        },
        "include_working_papers": {"type": "boolean"},
        "includeWorkingPapers": {
            "type": "boolean",
            "deprecated": True,
            "description": "Compatibility alias for include_working_papers.",
        },
        "from_year": {"$ref": "#/$defs/year"},
        "fromYear": {
            "$ref": "#/$defs/year",
            "deprecated": True,
            "description": "Compatibility alias for from_year.",
        },
        "to_year": {"$ref": "#/$defs/year"},
        "toYear": {
            "$ref": "#/$defs/year",
            "deprecated": True,
            "description": "Compatibility alias for to_year.",
        },
        "search_mode": {"$ref": "#/$defs/searchMode"},
        "searchMode": {
            "$ref": "#/$defs/searchMode",
            "deprecated": True,
            "description": "Compatibility alias for search_mode.",
        },
        "venue_filter": {"type": "string", "maxLength": 256},
        "venueFilter": {
            "type": "string",
            "maxLength": 256,
            "deprecated": True,
            "description": "Compatibility alias for venue_filter.",
        },
        "document_types": {"$ref": "#/$defs/documentTypeList"},
        "documentTypes": {
            "$ref": "#/$defs/documentTypeList",
            "deprecated": True,
            "description": "Compatibility alias for document_types.",
        },
    },
    "additionalProperties": False,
    "$defs": {
        "year": {
            "oneOf": [
                {"type": "integer", "minimum": 1000, "maximum": 9999},
                {
                    "type": "string",
                    "minLength": 4,
                    "maxLength": 4,
                    "pattern": "^[0-9]{4}$",
                },
            ]
        },
        "searchMode": {"type": "string", "enum": list(SEARCH_MODES)},
        "toolList": {
            "type": "array",
            "description": (
                "Identifiers are trimmed, lowercased, and separator-normalized; "
                "duplicates after normalization are invalid."
            ),
            "items": {
                "type": "string",
                "minLength": 1,
                "maxLength": 256,
                "pattern": ".*\\S.*",
            },
            "maxItems": 8,
            "uniqueItems": True,
        },
        "queryList": {
            "type": "array",
            "description": (
                "Queries are trimmed; duplicates after Unicode lowercasing are invalid."
            ),
            "items": {
                "type": "string",
                "minLength": 1,
                "maxLength": 4096,
                "pattern": ".*\\S.*",
            },
            "maxItems": 16,
            "uniqueItems": True,
        },
        "documentTypeList": {
            "type": "array",
            "description": (
                "Document types are trimmed; duplicates after Unicode lowercasing "
                "are invalid."
            ),
            "items": {
                "type": "string",
                "minLength": 1,
                "maxLength": 256,
                "pattern": ".*\\S.*",
            },
            "maxItems": 32,
            "uniqueItems": True,
        },
    },
}


LITERATURE_PROVIDER_CAPABILITIES: dict[str, dict[str, Any]] = {
    "openalex": {
        "status": "implemented",
        "max_per_provider_limit": 200,
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
        "max_per_provider_limit": 200,
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
        "max_per_provider_limit": 200,
        "capabilities": [
            "topic_search",
            "doi_lookup",
            "year_filter",
            "document_type_filter",
            "venue_metadata",
            "reference_metadata",
        ],
    },
    "pubmed": {
        "status": "implemented",
        "max_per_provider_limit": 200,
        "capabilities": [
            "topic_search",
            "doi_lookup",
            "biomedical_topic_search",
            "year_filter",
        ],
    },
    "arxiv": {
        "status": "implemented",
        "max_per_provider_limit": 200,
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
        "inputSchema": LITERATURE_STATUS_INPUT_SCHEMA,
    },
    {
        "name": "qiongli_search_plan",
        "description": "Plan provider and platform-native literature search routing without executing search.",
        "inputSchema": SEARCH_PLAN_INPUT_SCHEMA,
    },
    {
        "name": "qiongli_literature_search",
        "description": (
            "Search academic literature using the providers available in the active "
            "Qiongli runtime profile."
        ),
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
        "description": (
            "Export an auditable provider capability, search plan, diagnostics, "
            "and result snapshot."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {
                    "type": "string",
                    "description": (
                        "Compatibility context path used by the Full runtime and ignored by Lite."
                    ),
                },
                "query": {"type": "string"},
                "provider_status": {"type": "object"},
                "search_plan": {"type": "object"},
                "results": {"type": "array", "items": {"type": "object"}},
                "diagnostics": {"type": "object"},
                "query_plan": {
                    "type": "object",
                    "deprecated": True,
                    "description": "Compatibility alias for search_plan.",
                },
                "search_results": {
                    "type": "array",
                    "items": {"type": "object"},
                    "deprecated": True,
                    "description": "Compatibility alias for results.",
                },
                "search_diagnostics": {
                    "type": "object",
                    "deprecated": True,
                    "description": "Compatibility alias for diagnostics.",
                },
            },
            "additionalProperties": False,
        },
    },
]

ProviderSearchFn = Callable[[dict[str, object], int], dict[str, object]]


class MCPToolInputError(ValueError):
    """Raised when a tool call violates the public capability input contract."""


def handle_literature_status(args: dict[str, Any]) -> dict[str, Any]:
    normalized = _normalize_literature_status_args(args)
    cwd = _cwd_from_args(normalized)
    config = resolve_provider_config(cwd=cwd)
    summary = provider_config_summary(config)
    active_providers = _active_provider_names(config)
    missing = _missing_provider_fields(summary)
    payload = {
        "status": "ok",
        "providers": summary,
        "active_providers": active_providers,
        "capability_mode": "provider_connected" if active_providers else "strategy_only",
        "missing": missing,
        "provider_capabilities": LITERATURE_PROVIDER_CAPABILITIES,
        "capabilities": LITERATURE_PROVIDER_CAPABILITIES,
        "redacted_config": redact_provider_config(config),
    }
    next_action = _provider_setup_next_action(missing)
    if next_action is not None:
        payload["next_action"] = next_action
    return payload


def handle_search_plan(args: dict[str, Any]) -> dict[str, Any]:
    normalized = _normalize_search_plan_args(args)
    config = resolve_provider_config(cwd=_cwd_from_args(normalized))
    active_providers = set(_active_provider_names(config))
    provider_status = {
        provider: "configured" if provider in active_providers else "missing"
        for provider in PROVIDER_STATUS_ORDER
    }
    return build_hybrid_search_plan(
        normalized,
        provider_capability_mode=(
            "provider_connected" if active_providers else "strategy_only"
        ),
        provider_status=provider_status,
    )


def _normalize_literature_status_args(args: dict[str, Any]) -> dict[str, Any]:
    _reject_unknown_arguments(args, set(LITERATURE_STATUS_INPUT_SCHEMA["properties"]))
    normalized: dict[str, Any] = {}
    if "cwd" in args:
        normalized["cwd"] = _bounded_string(
            args["cwd"],
            "cwd",
            maximum=4096,
            require_nonblank=True,
        )
    return normalized


def _normalize_search_plan_args(args: dict[str, Any]) -> dict[str, Any]:
    _reject_unknown_arguments(args, set(SEARCH_PLAN_INPUT_SCHEMA["properties"]))
    if "query" not in args:
        raise MCPToolInputError("query is required")

    normalized: dict[str, Any] = {
        "query": _bounded_string(
            args["query"],
            "query",
            maximum=4096,
            require_nonblank=True,
        )
    }
    if "cwd" in args:
        normalized["cwd"] = _bounded_string(
            args["cwd"],
            "cwd",
            maximum=4096,
            require_nonblank=True,
        )
    if "platform" in args:
        platform = _bounded_string(
            args["platform"],
            "platform",
            maximum=64,
            require_nonblank=True,
        )
        if not _valid_platform_identifier(platform):
            raise MCPToolInputError("platform must be an ASCII identifier")
        normalized["platform"] = platform

    present, value = _single_alias_value(
        args,
        ("native_search_available", "native_search_usable", "nativeSearchAvailable"),
        "native_search_available",
    )
    normalized["native_search_available"] = (
        _boolean_value(value, "native_search_available") if present else False
    )

    present, value = _single_alias_value(
        args,
        ("native_search_tools", "nativeSearchTools"),
        "native_search_tools",
    )
    if present:
        normalized["native_search_tools"] = _bounded_string_list(
            value,
            "native_search_tools",
            maximum_items=8,
            maximum_item_length=256,
            normalize_identifiers=True,
        )

    present, value = _single_alias_value(
        args,
        ("query_variants", "queryVariants"),
        "query_variants",
    )
    if present:
        normalized["query_variants"] = _bounded_string_list(
            value,
            "query_variants",
            maximum_items=16,
            maximum_item_length=4096,
        )

    present, value = _single_alias_value(
        args,
        ("include_working_papers", "includeWorkingPapers"),
        "include_working_papers",
    )
    if present:
        normalized["include_working_papers"] = _boolean_value(
            value,
            "include_working_papers",
        )

    for canonical, alias in (("from_year", "fromYear"), ("to_year", "toYear")):
        present, value = _single_alias_value(args, (canonical, alias), canonical)
        if present:
            normalized[canonical] = _year_value(value, canonical)

    present, value = _single_alias_value(
        args,
        ("search_mode", "searchMode"),
        "search_mode",
    )
    if present:
        if not isinstance(value, str):
            raise MCPToolInputError("search_mode must be a string")
        if value not in SEARCH_MODES:
            raise MCPToolInputError("unsupported search_mode")
        normalized["search_mode"] = value
    else:
        normalized["search_mode"] = "topic"

    present, value = _single_alias_value(
        args,
        ("venue_filter", "venueFilter"),
        "venue_filter",
    )
    if present:
        venue_filter = _bounded_string(
            value,
            "venue_filter",
            maximum=256,
            require_nonblank=False,
        )
        if venue_filter:
            normalized["venue_filter"] = venue_filter

    present, value = _single_alias_value(
        args,
        ("document_types", "documentTypes"),
        "document_types",
    )
    if present:
        normalized["document_types"] = _bounded_string_list(
            value,
            "document_types",
            maximum_items=32,
            maximum_item_length=256,
        )

    start = normalized.get("from_year")
    end = normalized.get("to_year")
    if isinstance(start, int) and isinstance(end, int) and start > end:
        raise MCPToolInputError("from_year must not exceed to_year")
    return normalized


def _single_alias_value(
    args: Mapping[str, Any],
    names: tuple[str, ...],
    canonical: str,
) -> tuple[bool, Any]:
    present = [name for name in names if name in args]
    if len(present) > 1:
        raise MCPToolInputError(f"{canonical} aliases must not be combined")
    if not present:
        return False, None
    return True, args[present[0]]


def _bounded_string(
    value: Any,
    field: str,
    *,
    maximum: int,
    require_nonblank: bool,
) -> str:
    if not isinstance(value, str):
        raise MCPToolInputError(f"{field} must be a string")
    if len(value) > maximum:
        raise MCPToolInputError(f"{field} must be at most {maximum} characters")
    normalized = value.strip()
    if require_nonblank and not normalized:
        raise MCPToolInputError(f"{field} must not be empty")
    return normalized


def _boolean_value(value: Any, field: str) -> bool:
    if not isinstance(value, bool):
        raise MCPToolInputError(f"{field} must be a boolean")
    return value


def _bounded_string_list(
    value: Any,
    field: str,
    *,
    maximum_items: int,
    maximum_item_length: int,
    normalize_identifiers: bool = False,
) -> list[str]:
    if not isinstance(value, list):
        raise MCPToolInputError(f"{field} must be an array")
    if len(value) > maximum_items:
        raise MCPToolInputError(f"{field} must contain at most {maximum_items} items")
    normalized: list[str] = []
    seen: set[str] = set()
    for item in value:
        if not isinstance(item, str):
            raise MCPToolInputError(f"{field} must contain strings")
        if len(item) > maximum_item_length:
            raise MCPToolInputError(
                f"{field} items must be at most {maximum_item_length} characters"
            )
        cleaned = item.strip()
        if not cleaned:
            raise MCPToolInputError(f"{field} must not contain empty strings")
        if normalize_identifiers:
            cleaned = _normalize_identifier(cleaned)
            if not cleaned:
                raise MCPToolInputError(f"{field} must contain valid identifiers")
        dedupe_key = cleaned.lower()
        if dedupe_key in seen:
            raise MCPToolInputError(f"{field} must contain unique items")
        seen.add(dedupe_key)
        normalized.append(cleaned)
    return normalized


def _normalize_identifier(value: str) -> str:
    normalized = value.strip().lower().replace("-", " ").replace("_", " ")
    return "_".join(normalized.split())


def _valid_platform_identifier(value: str) -> bool:
    return (
        bool(value)
        and value[0].isascii()
        and value[0].isalnum()
        and all(
            character.isascii()
            and (character.isalnum() or character in {" ", "_", "-"})
            for character in value
        )
    )


def _year_value(value: Any, field: str) -> int:
    if isinstance(value, bool):
        raise MCPToolInputError(f"{field} must be a four-digit year")
    if isinstance(value, int):
        year = value
    elif isinstance(value, str) and len(value) == 4 and value.isascii() and value.isdigit():
        year = int(value)
    else:
        raise MCPToolInputError(f"{field} must be a four-digit year")
    if year < 1000 or year > 9999:
        raise MCPToolInputError(f"{field} must be between 1000 and 9999")
    return year


def _reject_unknown_arguments(args: Mapping[str, Any], allowed: set[str]) -> None:
    unknown = sorted(set(args) - allowed)
    if unknown:
        raise MCPToolInputError("arguments contain unsupported fields")


def _active_provider_names(config: Mapping[str, Any]) -> list[str]:
    providers = config.get("providers", {})
    providers = providers if isinstance(providers, Mapping) else {}
    active: list[str] = []
    for provider in PROVIDER_STATUS_ORDER:
        raw = providers.get(provider, {})
        if (
            isinstance(raw, Mapping)
            and raw.get("enabled") is True
            and raw.get("configured") is True
        ):
            active.append(provider)
    return active


def _missing_provider_fields(summary: Mapping[str, str]) -> list[str]:
    return [
        field
        for provider, field in PROVIDER_ACTIVATION_FIELDS
        if summary.get(provider) != "configured"
    ]


def _provider_setup_next_action(missing: list[str]) -> dict[str, Any] | None:
    for field, provider in (
        ("openalex.api_key", "openalex"),
        ("semantic_scholar.api_key", "semantic_scholar"),
        ("crossref.email", "crossref"),
        ("pubmed.api_key", "pubmed"),
    ):
        if field in missing:
            return {
                "tool": "qiongli_configure_provider",
                "args": {"provider": provider},
                "message": (
                    "Run qiongli_configure_provider to open a local setup page. "
                    "Do not paste API keys in chat."
                ),
            }
    return None


def handle_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    return run_literature_search(args)


def handle_literature_export_evidence(args: dict[str, Any]) -> dict[str, Any]:
    normalized = _normalize_evidence_export_args(args)
    results = normalized["results"]
    return {
        "artifact_type": "qiongli_literature_evidence_snapshot",
        "exported_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "query": normalized["query"],
        "provider_status": normalized["provider_status"],
        "search_plan": normalized["search_plan"],
        "diagnostics": normalized["diagnostics"],
        "result_count": len(results),
        "results": results,
    }


def _normalize_evidence_export_args(args: dict[str, Any]) -> dict[str, Any]:
    allowed = {
        "cwd",
        "query",
        "provider_status",
        "search_plan",
        "results",
        "diagnostics",
        "query_plan",
        "search_results",
        "search_diagnostics",
    }
    unknown = sorted(set(args) - allowed)
    if unknown:
        raise MCPToolInputError("arguments contain unsupported fields")

    expected_types: dict[str, type[Any]] = {
        "cwd": str,
        "query": str,
        "provider_status": dict,
        "search_plan": dict,
        "results": list,
        "diagnostics": dict,
        "query_plan": dict,
        "search_results": list,
        "search_diagnostics": dict,
    }
    for field, expected_type in expected_types.items():
        if field in args and not isinstance(args[field], expected_type):
            raise MCPToolInputError(f"{field} must be a {expected_type.__name__}")
    for field in ("results", "search_results"):
        if field in args and any(not isinstance(item, dict) for item in args[field]):
            raise MCPToolInputError(f"{field} must contain objects")

    return {
        "query": str(args.get("query", "") or "").strip(),
        "provider_status": args.get("provider_status", {}),
        "search_plan": args.get("search_plan", args.get("query_plan", {})),
        "results": args.get("results", args.get("search_results", [])),
        "diagnostics": args.get("diagnostics", args.get("search_diagnostics", {})),
    }


def run_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.providers.literature_search import run_scholarly_search

    task_packet = _task_packet_from_search_args(args)
    provider_fns = _configured_provider_fns(args)
    return run_scholarly_search(task_packet, search_paper, provider_fns=provider_fns)


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
    return active_provider_search_fns(config)


def active_provider_search_fns(
    config: Mapping[str, object],
) -> dict[str, ProviderSearchFn]:
    providers = config.get("providers", {})
    providers = providers if isinstance(providers, Mapping) else {}
    provider_fns: dict[str, ProviderSearchFn] = {}

    for provider_name in PROVIDER_SEARCH_ORDER:
        raw_provider = providers.get(provider_name, {})
        if not isinstance(raw_provider, Mapping):
            continue
        if not raw_provider.get("enabled") or not raw_provider.get("configured"):
            continue
        provider_fn = _provider_search_fn(provider_name, raw_provider)
        if provider_fn is not None:
            provider_fns[provider_name] = provider_fn
    return provider_fns


def _provider_search_fn(
    provider_name: str,
    provider_config: Mapping[str, object],
) -> ProviderSearchFn | None:
    if provider_name == "semantic_scholar":
        api_key = str(provider_config.get("api_key", "") or "").strip()
        return lambda translation, limit: _s2_provider_search(
            translation,
            limit,
            api_key=api_key,
        )
    if provider_name == "openalex":
        api_key = str(provider_config.get("api_key", "") or "").strip()
        email = str(provider_config.get("email", "") or "").strip()
        return lambda translation, limit: openalex_client.search(
            translation,
            limit,
            api_key=api_key,
            email=email,
        )
    if provider_name == "crossref":
        email = str(provider_config.get("email", "") or "").strip()
        return lambda translation, limit: crossref_client.search(
            translation,
            limit,
            email=email,
        )
    if provider_name == "pubmed":
        api_key = str(provider_config.get("api_key", "") or "").strip()
        return lambda translation, limit: pubmed_client.search(
            translation,
            limit,
            api_key=api_key,
        )
    if provider_name == "arxiv":
        return arxiv_client.search
    return None


def _s2_provider_search(
    translation: dict[str, object],
    limit: int,
    *,
    api_key: str,
) -> dict[str, object]:
    filters = translation.get("filters", {})
    filters = filters if isinstance(filters, Mapping) else {}
    return search_paper(
        str(translation.get("translated_query", "") or ""),
        limit,
        year_start=filters.get("year_start"),
        year_end=filters.get("year_end"),
        publication_type=str(filters.get("publication_type", "") or "") or None,
        venue=str(filters.get("venue", "") or "") or None,
        api_key=api_key,
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
