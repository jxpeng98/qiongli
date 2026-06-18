from __future__ import annotations

import importlib
import re
from dataclasses import asdict, is_dataclass
from datetime import datetime, timezone
from typing import Any, Callable, Mapping

from bridges.providers.literature_query import (
    build_legacy_query_variants,
    build_structured_query_plan,
    translate_query_for_provider,
)


SearchFn = Callable[[str, int], dict[str, Any]]
ProviderSearchFn = Callable[[dict[str, object], int], dict[str, object]]

MAX_QUERY_VARIANTS = 4
DEFAULT_PER_QUERY_LIMIT = 20
MAX_PER_QUERY_LIMIT = 50
STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "be",
    "by",
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
    "what",
    "when",
    "where",
    "which",
    "why",
    "with",
}
DOI_PREFIX_RE = re.compile(r"^(?:https?://(?:dx\.)?doi\.org/|doi:\s*)", re.I)
NON_ALNUM_RE = re.compile(r"[^a-z0-9]+")


def build_query_variants(task_packet: dict[str, Any]) -> list[dict[str, str]]:
    return build_legacy_query_variants(task_packet)


def run_scholarly_search(
    task_packet: dict[str, Any],
    search_fn: SearchFn,
    *,
    retrieved_at: str | None = None,
    provider_fns: dict[str, ProviderSearchFn] | None = None,
) -> dict[str, Any]:
    query_plan = build_structured_query_plan(task_packet)
    query_variants = build_query_variants(task_packet)
    if not query_variants:
        diagnostics = _build_search_diagnostics(
            attempted_providers=[],
            attempted_query_count=0,
            failed_query_count=0,
            raw_result_count=0,
            unique_result_count=0,
            duplicate_count=0,
        )
        diagnostics = _append_search_diagnostics_v2(
            diagnostics,
            query_plan=query_plan,
            unique_results=[],
            search_log=[],
            dedup_log=[],
            provider_summaries={},
            failures=[],
            raw_result_count=0,
            normalized_result_count=0,
            duplicate_count=0,
        )
        return {
            "status": "warning",
            "summary": "Empty topic/query context, no scholarly search performed.",
            "provenance": [],
            "data": {
                "provider_mode": "builtin_semantic_scholar_baseline",
                "query_plan": query_plan,
                "provider_summaries": {},
                "search_diagnostics": diagnostics,
                "query_variants": [],
                "search_results": [],
                "dedup_log": [],
                "search_log": [],
                "artifact_bundle": _artifact_bundle(),
            },
        }

    per_query_limit = _resolve_per_query_limit(task_packet)
    timestamp = retrieved_at or datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    normalized_results: list[dict[str, Any]] = []
    search_log: list[dict[str, Any]] = []
    failures: list[dict[str, str]] = []
    provider_summaries: dict[str, dict[str, Any]] = {}
    raw_result_count = 0

    if provider_fns is None:
        provider_mode = "builtin_semantic_scholar_baseline"
        provenance = ["https://api.semanticscholar.org/graph/v1"]
        attempted_providers = ["semantic_scholar"]

        for variant in query_variants:
            translation = {
                "query_id": variant["query_id"],
                "provider": "semantic_scholar",
                "translated_query": variant["query"],
                "filters": {},
            }
            response = search_fn(variant["query"], per_query_limit)
            raw_result_count += _handle_search_response(
                response,
                translation=translation,
                provider="semantic_scholar",
                timestamp=timestamp,
                per_query_limit=per_query_limit,
                normalized_results=normalized_results,
                search_log=search_log,
                failures=failures,
                provider_summaries=provider_summaries,
            )
    else:
        provider_mode = "provider_translations"
        provenance = []
        attempted_providers = []
        for provider, provider_fn in provider_fns.items():
            provider_name = str(provider).strip().casefold()
            if not provider_name:
                continue
            attempted_providers.append(provider_name)
            translation = translate_query_for_provider(query_plan, provider_name)
            try:
                response = provider_fn(translation, per_query_limit)
            except Exception as exc:  # pragma: no cover - exact exception type is provider-owned.
                response = {"error": str(exc), "data": []}
            raw_result_count += _handle_search_response(
                response,
                translation=translation,
                provider=provider_name,
                timestamp=timestamp,
                per_query_limit=per_query_limit,
                normalized_results=normalized_results,
                search_log=search_log,
                failures=failures,
                provider_summaries=provider_summaries,
            )

    for summary in provider_summaries.values():
        if summary["failures"] and summary["normalized_hits"]:
            summary["status"] = "warning"
        elif summary["failures"] and not summary["normalized_hits"]:
            summary["status"] = "error"
        else:
            summary["status"] = "ok"

    unique_results, dedup_log = dedupe_search_results(normalized_results)
    duplicate_count = len(dedup_log)
    attempted_query_count = len(search_log)
    failed_query_count = len(failures)
    diagnostics = _build_search_diagnostics(
        attempted_providers=attempted_providers,
        attempted_query_count=attempted_query_count,
        failed_query_count=failed_query_count,
        raw_result_count=raw_result_count,
        normalized_result_count=len(normalized_results),
        unique_result_count=len(unique_results),
        duplicate_count=duplicate_count,
    )
    diagnostics = _append_search_diagnostics_v2(
        diagnostics,
        query_plan=query_plan,
        unique_results=unique_results,
        search_log=search_log,
        dedup_log=dedup_log,
        provider_summaries=provider_summaries,
        failures=failures,
        raw_result_count=raw_result_count,
        normalized_result_count=len(normalized_results),
        duplicate_count=duplicate_count,
    )

    if unique_results:
        status = "warning" if failures else "ok"
        summary = (
            f"Found {len(unique_results)} unique papers across {attempted_query_count} query attempts "
            f"({raw_result_count} raw hits, {duplicate_count} deduplicated)."
        )
    elif diagnostics["all_providers_failed"]:
        status = "error"
        summary = (
            f"Scholarly search failed for all {attempted_query_count} query attempts; "
            f"last error: {failures[-1]['error']}"
        )
    else:
        status = "warning"
        summary = (
            f"No papers returned across {attempted_query_count} query attempts for the current topic."
        )

    return {
        "status": status,
        "summary": summary,
        "provenance": provenance,
        "data": {
            "provider_mode": provider_mode,
            "query_plan": query_plan,
            "provider_summaries": provider_summaries,
            "search_diagnostics": diagnostics,
            "query_variants": query_variants,
            "per_query_limit": per_query_limit,
            "raw_result_count": raw_result_count,
            "normalized_result_count": len(normalized_results),
            "unique_result_count": len(unique_results),
            "duplicate_count": duplicate_count,
            "search_results": unique_results,
            "dedup_log": dedup_log,
            "search_log": search_log,
            "failures": failures,
            "artifact_bundle": _artifact_bundle(),
        },
    }


def _handle_search_response(
    response: dict[str, Any],
    *,
    translation: dict[str, Any],
    provider: str,
    timestamp: str,
    per_query_limit: int,
    normalized_results: list[dict[str, Any]],
    search_log: list[dict[str, Any]],
    failures: list[dict[str, str]],
    provider_summaries: dict[str, dict[str, Any]],
) -> int:
    query_id = str(translation.get("query_id", "")).strip()
    translated_query = str(translation.get("translated_query", "")).strip()
    filters = translation.get("filters", {})
    if not isinstance(filters, dict):
        filters = {}

    if not isinstance(response, dict):
        response = {"error": "provider returned a non-mapping response", "data": []}

    error = str(response.get("error", "")).strip()
    hits = response.get("data", [])
    if not isinstance(hits, list):
        hits = []
    raw_hit_count = len(hits)

    if error:
        failures.append({"query_id": query_id, "query": translated_query, "provider": provider, "error": error})

    search_log.append(
        {
            "query_id": query_id,
            "query": translated_query,
            "translated_query": translated_query,
            "filters": dict(filters),
            "provider": provider,
            "retrieved_at": timestamp,
            "retrieved_count": len(hits),
            "limit": per_query_limit,
            "status": "error" if error else "ok",
            "error": error,
        }
    )

    summary = provider_summaries.setdefault(
        provider,
        {
            "provider": provider,
            "attempted_queries": [],
            "raw_hits": 0,
            "normalized_hits": 0,
            "failures": [],
            "status": "ok",
        },
    )
    summary["attempted_queries"].append(
        {
            "query_id": query_id,
            "translated_query": translated_query,
            "filters": dict(filters),
        }
    )
    summary["raw_hits"] += raw_hit_count
    if error:
        summary["failures"].append({"query_id": query_id, "error": error})

    normalized_for_attempt = 0
    for offset, hit in enumerate(hits, start=1):
        if not isinstance(hit, dict):
            continue
        normalized_results.append(
            normalize_search_hit(
                hit,
                query_id=query_id,
                query_text=translated_query,
                retrieved_at=timestamp,
                ordinal=offset,
                source=provider,
            )
        )
        normalized_for_attempt += 1
    summary["normalized_hits"] += normalized_for_attempt
    return raw_hit_count


def _build_search_diagnostics(
    *,
    attempted_providers: list[str],
    attempted_query_count: int,
    failed_query_count: int,
    raw_result_count: int,
    normalized_result_count: int = 0,
    unique_result_count: int,
    duplicate_count: int,
) -> dict[str, Any]:
    all_providers_failed = (
        bool(attempted_query_count)
        and failed_query_count == attempted_query_count
        and normalized_result_count == 0
    )
    zero_hit = normalized_result_count == 0
    if all_providers_failed:
        status_reason = "all_attempted_queries_failed"
    elif failed_query_count:
        status_reason = "partial_provider_failure"
    elif zero_hit:
        status_reason = "zero_hits"
    else:
        status_reason = "hits_returned"

    return {
        "attempted_providers": attempted_providers,
        "attempted_query_count": attempted_query_count,
        "failed_query_count": failed_query_count,
        "raw_result_count": raw_result_count,
        "normalized_result_count": normalized_result_count,
        "unique_result_count": unique_result_count,
        "duplicate_count": duplicate_count,
        "all_providers_failed": all_providers_failed,
        "zero_hit": zero_hit,
        "status_reason": status_reason,
    }


def _append_search_diagnostics_v2(
    diagnostics: dict[str, Any],
    *,
    query_plan: dict[str, Any],
    unique_results: list[dict[str, Any]],
    search_log: list[dict[str, Any]],
    dedup_log: list[dict[str, Any]],
    provider_summaries: dict[str, dict[str, Any]],
    failures: list[dict[str, str]],
    raw_result_count: int,
    normalized_result_count: int,
    duplicate_count: int,
) -> dict[str, Any]:
    builder = _load_search_diagnostics_v2_builder()
    if builder is None:
        return diagnostics

    try:
        v2_diagnostics = builder(
            query_plan=query_plan,
            search_log=search_log,
            search_results=unique_results,
            dedup_log=dedup_log,
            provider_summaries=provider_summaries,
            raw_diagnostics={
                **diagnostics,
                "failures": failures,
                "raw_result_count": raw_result_count,
                "normalized_result_count": normalized_result_count,
                "duplicate_count": duplicate_count,
            },
        )
    except TypeError:
        try:
            v2_diagnostics = builder(
                query_plan,
                search_log,
                unique_results,
                dedup_log,
                provider_summaries,
                diagnostics,
            )
        except Exception as exc:  # pragma: no cover - defensive compatibility path.
            return {**diagnostics, "diagnostics_v2_error": str(exc)}
    except Exception as exc:  # pragma: no cover - defensive compatibility path.
        return {**diagnostics, "diagnostics_v2_error": str(exc)}

    v2_mapping = _coerce_diagnostics_mapping(v2_diagnostics)
    if not v2_mapping:
        return diagnostics
    return _merge_without_overwriting(diagnostics, v2_mapping)


def _load_search_diagnostics_v2_builder() -> Callable[..., Any] | None:
    try:
        module = importlib.import_module("bridges.providers.literature_diagnostics")
    except ImportError:
        return None
    builder = getattr(module, "build_search_diagnostics_v2", None)
    return builder if callable(builder) else None


def _coerce_diagnostics_mapping(value: Any) -> dict[str, Any]:
    if isinstance(value, Mapping):
        return dict(value)
    if is_dataclass(value) and not isinstance(value, type):
        return asdict(value)
    to_dict = getattr(value, "to_dict", None)
    if callable(to_dict):
        converted = to_dict()
        if isinstance(converted, Mapping):
            return dict(converted)
    return {}


def _merge_without_overwriting(
    baseline: dict[str, Any],
    extra: dict[str, Any],
) -> dict[str, Any]:
    merged = dict(baseline)
    for key, value in extra.items():
        if key not in merged:
            merged[key] = value
            continue
        if isinstance(merged[key], dict) and isinstance(value, Mapping):
            merged[key] = _merge_without_overwriting(merged[key], dict(value))
    return merged


def normalize_search_hit(
    hit: dict[str, Any],
    *,
    query_id: str,
    query_text: str,
    retrieved_at: str,
    ordinal: int,
    source: str = "semantic_scholar",
) -> dict[str, Any]:
    paper_id = str(hit.get("paperId") or "").strip()
    external_ids = hit.get("externalIds", {})
    if not isinstance(external_ids, dict):
        external_ids = {}
    doi = _normalize_doi(external_ids.get("DOI"))

    title = " ".join(str(hit.get("title", "")).split())
    authors = _flatten_authors(hit.get("authors"))
    year = _safe_int(hit.get("year"))
    venue = " ".join(str(hit.get("venue", "")).split())
    abstract = " ".join(str(hit.get("abstract", "")).split())
    url = str(hit.get("url", "")).strip()
    citation_count = _safe_int(hit.get("citationCount"))
    open_access_url = _extract_open_access_url(hit)
    record_id = paper_id or f"{query_id}-{ordinal}"

    return {
        "record_id": f"s2:{record_id}",
        "source": source,
        "query_id": query_id,
        "query_text": query_text,
        "retrieved_at": retrieved_at,
        "paper_id": paper_id,
        "title": title,
        "authors": authors,
        "year": year,
        "venue": venue,
        "doi": doi,
        "url": url,
        "abstract": abstract,
        "citation_count": citation_count,
        "open_access_pdf_url": open_access_url,
    }


def dedupe_search_results(records: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, str]]]:
    canonical_records: list[dict[str, Any]] = []
    canonical_by_key: dict[str, dict[str, Any]] = {}
    dedup_log: list[dict[str, str]] = []

    for record in records:
        match_key, match_basis = record_match_key(record)
        existing = canonical_by_key.get(match_key)
        if existing is None:
            record["query_ids"] = [record.get("query_id", "")]
            canonical_records.append(record)
            canonical_by_key[match_key] = record
            continue

        _merge_record(existing, record)
        dedup_log.append(
            {
                "candidate_record_id": str(record.get("record_id", "")),
                "canonical_record_id": str(existing.get("record_id", "")),
                "decision": "merge_duplicate",
                "match_basis": match_basis,
                "resolver": "builtin_scholarly_search",
                "notes": f"Merged query {record.get('query_id', '')} into canonical record.",
            }
        )

    for record in canonical_records:
        query_ids = [query_id for query_id in record.pop("query_ids", []) if query_id]
        if query_ids:
            record["query_ids"] = ";".join(sorted(set(query_ids)))

    return canonical_records, dedup_log


def _artifact_bundle() -> dict[str, str]:
    return {
        "search_strategy": "search_strategy.md",
        "search_log": "search_log.md",
        "search_results": "search_results.csv",
        "dedup_log": "dedup_log.csv",
        "search_diagnostics": "search_diagnostics.md",
    }


def _resolve_per_query_limit(task_packet: dict[str, Any]) -> int:
    for key in ("per_query_limit", "limit", "search_limit"):
        value = task_packet.get(key)
        try:
            parsed = int(str(value).strip())
        except (TypeError, ValueError):
            continue
        return max(1, min(parsed, MAX_PER_QUERY_LIMIT))
    return DEFAULT_PER_QUERY_LIMIT


def _build_keyword_bundle(raw_keywords: Any) -> str:
    if not isinstance(raw_keywords, list):
        return ""
    cleaned: list[str] = []
    for item in raw_keywords:
        text = " ".join(str(item).strip().split())
        if not text:
            continue
        cleaned.append(f"\"{text}\"" if " " in text else text)
    return " ".join(cleaned[:6])


def _distill_question(question: str) -> str:
    terms: list[str] = []
    seen: set[str] = set()
    for token in re.findall(r"[A-Za-z0-9][A-Za-z0-9-]{2,}", question.lower()):
        if token in STOPWORDS:
            continue
        if token in seen:
            continue
        seen.add(token)
        terms.append(token)
        if len(terms) >= 8:
            break
    return " ".join(terms)


def _flatten_authors(raw_authors: Any) -> str:
    if not isinstance(raw_authors, list):
        return ""
    names: list[str] = []
    for author in raw_authors:
        if isinstance(author, dict):
            name = " ".join(str(author.get("name", "")).split())
            if name:
                names.append(name)
    return "; ".join(names)


def _normalize_doi(raw: Any) -> str:
    value = " ".join(str(raw or "").strip().split())
    if not value:
        return ""
    value = DOI_PREFIX_RE.sub("", value)
    return value.rstrip(".,);]").lower()


def _safe_int(raw: Any) -> int | None:
    try:
        return int(raw)
    except (TypeError, ValueError):
        return None


def _extract_open_access_url(hit: dict[str, Any]) -> str:
    payload = hit.get("openAccessPdf")
    if isinstance(payload, dict):
        return str(payload.get("url", "")).strip()
    return ""


def record_match_key(record: dict[str, Any]) -> tuple[str, str]:
    doi = str(record.get("doi", "")).strip().lower()
    if doi:
        return f"doi:{doi}", "doi"

    paper_id = str(record.get("paper_id", "")).strip()
    if paper_id:
        return f"paper_id:{paper_id}", "paper_id"

    title = _normalize_title(record.get("title"))
    year = str(record.get("year") or "").strip()
    if title and year:
        return f"title_year:{title}:{year}", "title+year"
    if title:
        return f"title:{title}", "title"
    return f"record:{record.get('record_id', '')}", "record_id"


def _normalize_title(raw: Any) -> str:
    value = " ".join(str(raw or "").strip().lower().split())
    return NON_ALNUM_RE.sub("", value)


def _merge_record(canonical: dict[str, Any], candidate: dict[str, Any]) -> None:
    query_ids = canonical.setdefault("query_ids", [])
    if isinstance(query_ids, list):
        query_ids.append(str(candidate.get("query_id", "")))

    for key in ("doi", "url", "abstract", "venue", "open_access_pdf_url"):
        if not canonical.get(key) and candidate.get(key):
            canonical[key] = candidate[key]

    if not canonical.get("authors") and candidate.get("authors"):
        canonical["authors"] = candidate["authors"]

    if not canonical.get("paper_id") and candidate.get("paper_id"):
        canonical["paper_id"] = candidate["paper_id"]

    canonical_citations = canonical.get("citation_count")
    candidate_citations = candidate.get("citation_count")
    if isinstance(candidate_citations, int):
        if not isinstance(canonical_citations, int) or candidate_citations > canonical_citations:
            canonical["citation_count"] = candidate_citations
