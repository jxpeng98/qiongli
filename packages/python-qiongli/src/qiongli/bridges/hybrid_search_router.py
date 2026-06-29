from __future__ import annotations

from collections.abc import Mapping
from typing import Any


PROVIDER_PROVENANCE_LABELS = [
    "mcp:semantic_scholar",
    "mcp:openalex",
    "mcp:crossref",
    "mcp:pubmed",
    "mcp:arxiv",
]
PROVIDER_NAMES = [label.removeprefix("mcp:") for label in PROVIDER_PROVENANCE_LABELS]

AGENT_INSTRUCTIONS = [
    "MCP servers must not call Codex or Claude native search directly.",
    "The active agent executes native_search_queries only when the platform exposes native search.",
    "Do not treat native-search results as provider-reproducible records.",
    "Write provider, native, and user-corpus records with distinct provenance labels.",
]


def build_hybrid_search_plan(
    args: Mapping[str, Any],
    *,
    provider_capability_mode: str,
    provider_status: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    query = _string_arg(args, "query")
    platform = _platform_arg(args)
    native_search_available = _bool_arg(_first_arg(args, ("native_search_available", "nativeSearchAvailable"), False))
    native_search_tools = _native_search_tools(
        _first_arg(args, ("native_search_tools", "nativeSearchTools")),
        platform,
        native_search_available,
    )
    normalized_provider_mode = _provider_capability_mode(provider_capability_mode)
    provider_names = _provider_names(provider_status)
    if provider_status is not None and not provider_names:
        normalized_provider_mode = "strategy_only"
    provider_available = normalized_provider_mode == "provider_connected" and bool(provider_names)

    search_execution_mode, limitations = _search_execution_mode(
        query=query,
        provider_available=provider_available,
        native_search_available=native_search_available,
    )
    filters = _search_filters(args)
    query_entries = _query_entries(args, query)
    provider_queries = (
        _provider_queries(query_entries, filters, provider_names)
        if search_execution_mode in {"hybrid_search", "provider_connected"}
        else []
    )
    native_search_queries = (
        _native_search_queries(query_entries, platform, native_search_tools, filters)
        if search_execution_mode in {"hybrid_search", "native_only"}
        else []
    )

    return {
        "artifact_type": "qiongli_hybrid_search_plan",
        "query": query,
        "platform": platform,
        "search_execution_mode": search_execution_mode,
        "provider_capability_mode": normalized_provider_mode,
        "native_search_available": native_search_available,
        "native_search_tools": native_search_tools,
        "provider_queries": provider_queries,
        "native_search_queries": native_search_queries,
        "provenance_labels": {
            "provider": [f"mcp:{provider}" for provider in provider_names] if provider_available else [],
            "native": [f"native:{tool}" for tool in native_search_tools],
            "user_corpus": ["user_corpus"],
        },
        "execution_sequence": _execution_sequence(provider_queries, native_search_queries),
        "agent_instructions": list(AGENT_INSTRUCTIONS),
        "merge_policy": {
            "dedupe_keys": ["doi", "title", "year", "provider_record_id", "native_url"],
            "provider_records": "Prefer provider MCP metadata for reproducible bibliographic fields.",
            "native_records": "Keep native-search records only with native provenance labels and source URLs.",
            "user_corpus_records": "Keep user-corpus records separate from provider and native search records.",
            "search_log": "Record provider and native query execution separately before merge and dedupe.",
        },
        "limitations": limitations,
    }


def _search_execution_mode(
    *,
    query: str,
    provider_available: bool,
    native_search_available: bool,
) -> tuple[str, list[str]]:
    if not query:
        return "strategy_only", ["Search query is empty."]
    if provider_available and native_search_available:
        return "hybrid_search", []
    if provider_available:
        return "provider_connected", ["Platform-native search was not declared available."]
    if native_search_available:
        return (
            "native_only",
            ["Provider MCP search is unavailable; native results require explicit provenance labels."],
        )
    return "strategy_only", ["No provider MCP search or platform-native search is available."]


def _provider_capability_mode(value: str) -> str:
    return "provider_connected" if value == "provider_connected" else "strategy_only"


def _provider_queries(
    query_entries: list[dict[str, str]],
    filters: dict[str, Any],
    provider_names: list[str],
) -> list[dict[str, Any]]:
    return [
        {
            "provider": provider,
            "query_id": entry["query_id"],
            "query": entry["query"],
            "source": entry["source"],
            "filters": dict(filters),
            "provenance_label": f"mcp:{provider}",
        }
        for provider in PROVIDER_NAMES
        if provider in provider_names
        for entry in query_entries
    ]


def _native_search_queries(
    query_entries: list[dict[str, str]],
    platform: str,
    native_search_tools: list[str],
    filters: dict[str, Any],
) -> list[dict[str, Any]]:
    return [
        {
            "tool": tool,
            "platform": platform,
            "query_id": entry["query_id"],
            "query": entry["query"],
            "source": entry["source"],
            "filters": dict(filters),
            "provenance_label": f"native:{tool}",
        }
        for tool in native_search_tools
        for entry in query_entries
    ]


def _execution_sequence(
    provider_queries: list[dict[str, Any]],
    native_search_queries: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    sequence = [
        {
            "actor": "agent",
            "action": "call qiongli_literature_status",
            "tool": "qiongli_literature_status",
        },
        {
            "actor": "agent",
            "action": "call qiongli_search_plan",
            "tool": "qiongli_search_plan",
        },
    ]
    if provider_queries:
        sequence.append(
            {
                "actor": "agent",
                "action": "call qiongli_literature_search",
                "tool": "qiongli_literature_search",
                "queries": "provider_queries",
            }
        )
    if native_search_queries:
        sequence.append(
            {
                "actor": "agent",
                "action": "execute platform-native search",
                "queries": "native_search_queries",
            }
        )
    sequence.append(
        {
            "actor": "agent",
            "action": "merge/dedupe/search_log",
            "inputs": ["provider_queries", "native_search_queries", "user_corpus"],
        }
    )
    return sequence


def _search_filters(args: Mapping[str, Any]) -> dict[str, Any]:
    filters: dict[str, Any] = {}
    for output_key, aliases in (
        ("include_working_papers", ("include_working_papers", "includeWorkingPapers")),
        ("fromYear", ("fromYear",)),
        ("toYear", ("toYear",)),
        ("search_mode", ("search_mode", "searchMode")),
        ("venue_filter", ("venue_filter", "venueFilter")),
    ):
        value = _first_arg(args, aliases)
        if value is not None:
            filters[output_key] = value
    document_types = _string_list(_first_arg(args, ("document_types", "documentTypes")))
    if document_types:
        filters["document_types"] = document_types
    return filters


def _native_search_tools(raw: Any, platform: str, native_search_available: bool) -> list[str]:
    if not native_search_available:
        return []
    tools: list[str] = []
    if isinstance(raw, str):
        tools = [_normalize_tool(raw)] if raw.strip() else []
    elif isinstance(raw, list):
        tools = [_normalize_tool(item) for item in raw if str(item or "").strip()]
    if not tools:
        tools = [_default_native_search_tool(platform)]
    return list(dict.fromkeys(tools))


def _query_entries(args: Mapping[str, Any], query: str) -> list[dict[str, str]]:
    if not query:
        return []
    queries = [query, *_string_list(_first_arg(args, ("query_variants", "queryVariants")))]
    entries: list[dict[str, str]] = []
    seen: set[str] = set()
    for candidate in queries:
        key = candidate.lower()
        if key in seen:
            continue
        seen.add(key)
        entries.append(
            {
                "query_id": f"Q{len(entries) + 1}",
                "query": candidate,
                "source": "primary" if not entries else "variant",
            }
        )
    return entries


def _provider_names(provider_status: Mapping[str, Any] | None) -> list[str]:
    if provider_status is None:
        return list(PROVIDER_NAMES)
    names: list[str] = []
    for provider in PROVIDER_NAMES:
        value = provider_status.get(provider)
        if value == "configured":
            names.append(provider)
        elif isinstance(value, Mapping) and value.get("configured"):
            names.append(provider)
        elif value is True:
            names.append(provider)
    return names


def _default_native_search_tool(platform: str) -> str:
    normalized = platform.lower().replace("-", "_")
    if normalized == "codex":
        return "codex_web_search"
    if normalized in {"claude", "claude_code", "claudecode"}:
        return "claude_web_search"
    if normalized == "antigravity":
        return "antigravity_search"
    return "platform_native_search"


def _string_list(raw: Any) -> list[str]:
    if isinstance(raw, str):
        value = raw.strip()
        return [value] if value else []
    if isinstance(raw, list):
        return [str(item).strip() for item in raw if str(item or "").strip()]
    return []


def _string_arg(args: Mapping[str, Any], key: str) -> str:
    return str(args.get(key, "") or "").strip()


def _first_arg(args: Mapping[str, Any], keys: tuple[str, ...], default: Any = None) -> Any:
    for key in keys:
        if key in args and args[key] is not None:
            return args[key]
    return default


def _platform_arg(args: Mapping[str, Any]) -> str:
    platform = str(args.get("platform", "unknown") or "unknown").strip()
    return platform.replace("-", "_") or "unknown"


def _bool_arg(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "y", "available"}
    return bool(value)


def _normalize_tool(value: Any) -> str:
    return str(value or "").strip().replace(" ", "_")
