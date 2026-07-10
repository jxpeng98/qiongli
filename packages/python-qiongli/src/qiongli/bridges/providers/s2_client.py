import json
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.provider_config import resolve_provider_config

S2_GRAPH_BASE = "https://api.semanticscholar.org/graph/v1"
DEFAULT_TIMEOUT_SECONDS = 15
DEFAULT_MAX_ATTEMPTS = 3
RETRYABLE_HTTP_CODES = {429, 500, 502, 503, 504}
DEFAULT_SEARCH_FIELDS = "paperId,title,authors,year,abstract,url,citationCount,venue,externalIds,openAccessPdf"
MIN_SEARCH_YEAR = 1900


def search_paper(
    query: str,
    limit: int = 10,
    *,
    year_start: int | str | None = None,
    year_end: int | str | None = None,
    fields: str | list[str] | tuple[str, ...] | None = None,
    publication_type: str | None = None,
    venue: str | None = None,
    api_key: str | None = None,
) -> dict[str, Any]:
    """Search for papers by keyword."""
    if not query.strip():
        return {"data": []}

    search_query = _search_query_with_keyword_filters(query, venue, publication_type)
    params: dict[str, str | int] = {
        "query": search_query,
        "limit": limit,
        "fields": _format_fields(fields),
    }
    year_filter = _format_year_filter(year_start, year_end)
    if year_filter:
        params["year"] = year_filter

    query_string = urllib.parse.urlencode(
        params,
        quote_via=urllib.parse.quote,
    )
    url = f"{S2_GRAPH_BASE}/paper/search?{query_string}"

    return _make_request(url) if api_key is None else _make_request(url, api_key=api_key)

def get_paper_details(paper_id: str) -> dict[str, Any]:
    """Get detailed information about a specific paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}?fields=title,authors,year,abstract,url,citationCount,referenceCount,venue"
    return _make_request(url)

def get_citations(paper_id: str, limit: int = 20) -> dict[str, Any]:
    """Get papers that cite the target paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}/citations?limit={limit}&fields=title,authors,year,venue,url,citationCount"
    return _make_request(url)

def get_references(paper_id: str, limit: int = 20) -> dict[str, Any]:
    """Get papers referenced by the target paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}/references?limit={limit}&fields=title,authors,year,venue,url,citationCount"
    return _make_request(url)

def _make_request(url: str, *, api_key: str | None = None) -> dict[str, Any]:
    headers = {
        "User-Agent": "Research-Skills-MCP/1.0",
        "Accept": "application/json",
    }
    resolved_api_key = _semantic_scholar_api_key() if api_key is None else api_key.strip()
    if resolved_api_key:
        headers["x-api-key"] = resolved_api_key

    for attempt in range(1, DEFAULT_MAX_ATTEMPTS + 1):
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=DEFAULT_TIMEOUT_SECONDS) as response:
                content = response.read()
                return dict(json.loads(content))
        except urllib.error.HTTPError as exc:
            if exc.code in RETRYABLE_HTTP_CODES and attempt < DEFAULT_MAX_ATTEMPTS:
                time.sleep(_retry_delay_seconds(exc, attempt))
                continue
            return {"error": _format_http_error(exc), "data": []}
        except urllib.error.URLError as exc:
            if attempt < DEFAULT_MAX_ATTEMPTS:
                time.sleep(float(attempt))
                continue
            return {"error": str(exc), "data": []}
        except Exception as exc:
            return {"error": str(exc), "data": []}

    return {"error": "Semantic Scholar request exhausted retries.", "data": []}


def _semantic_scholar_api_key() -> str:
    providers = resolve_provider_config(cwd=Path.cwd()).get("providers", {})
    if not isinstance(providers, dict):
        return ""
    semantic_scholar = providers.get("semantic_scholar", {})
    if not isinstance(semantic_scholar, dict):
        return ""
    return str(semantic_scholar.get("api_key", "")).strip()


def _search_query_with_keyword_filters(
    query: str,
    venue: str | None,
    publication_type: str | None,
) -> str:
    keyword_filters = [value.strip() for value in (venue, publication_type) if value and value.strip()]
    if not keyword_filters:
        return query
    # S2 paper search has no safe exact venue/type params; keep these as query keywords.
    return " ".join([query.strip(), *keyword_filters])


def _format_year_filter(year_start: int | str | None, year_end: int | str | None) -> str | None:
    start = _clean_year(year_start)
    end = _clean_year(year_end)
    if start and end:
        if int(start) > int(end):
            return None
        return f"{start}-{end}"
    if start:
        return f"{start}-"
    if end:
        return f"-{end}"
    return None


def _clean_year(year: int | str | None) -> str | None:
    if year is None:
        return None
    cleaned = str(year).strip()
    if not cleaned.isdigit() or len(cleaned) != 4:
        return None
    parsed = int(cleaned)
    max_year = datetime.now(timezone.utc).year + 1
    if parsed < MIN_SEARCH_YEAR or parsed > max_year:
        return None
    return cleaned


def _format_fields(fields: str | list[str] | tuple[str, ...] | None) -> str:
    baseline_fields = _split_fields(DEFAULT_SEARCH_FIELDS)
    if fields is None:
        return ",".join(baseline_fields)
    if isinstance(fields, str):
        override_fields = _split_fields(fields)
    else:
        override_fields = [str(field).strip() for field in fields if str(field).strip()]

    merged_fields = _dedupe_fields([*baseline_fields, *override_fields])
    return ",".join(merged_fields)


def _split_fields(fields: str) -> list[str]:
    return [field.strip() for field in fields.split(",") if field.strip()]


def _dedupe_fields(fields: list[str]) -> list[str]:
    deduped: list[str] = []
    seen: set[str] = set()
    for field in fields:
        if field in seen:
            continue
        seen.add(field)
        deduped.append(field)
    return deduped


def _retry_delay_seconds(exc: urllib.error.HTTPError, attempt: int) -> float:
    retry_after = exc.headers.get("Retry-After") if exc.headers else None
    if retry_after:
        try:
            return max(float(retry_after), 1.0)
        except ValueError:
            pass
    return float(2 ** (attempt - 1))


def _format_http_error(exc: urllib.error.HTTPError) -> str:
    reason = str(exc.reason or "").strip()
    if reason:
        return f"HTTP Error {exc.code}: {reason}"
    return f"HTTP Error {exc.code}"
