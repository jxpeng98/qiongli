from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

from bridges.provider_config import resolve_provider_config


OPENALEX_WORKS_URL = "https://api.openalex.org/works"
DEFAULT_TIMEOUT_SECONDS = 15


def search(
    translation: dict[str, object],
    limit: int,
    *,
    api_key: str | None = None,
    email: str | None = None,
) -> dict[str, object]:
    query = str(translation.get("translated_query", "") or "").strip()
    if not query:
        return {"data": []}

    filters = translation.get("filters", {})
    filters = filters if isinstance(filters, dict) else {}
    params: dict[str, str | int] = {
        "search": query,
        "per-page": max(1, int(limit)),
        "sort": "relevance_score:desc",
    }
    filter_value = _filter_value(filters)
    if filter_value:
        params["filter"] = filter_value
    resolved_api_key = _openalex_api_key() if api_key is None else api_key.strip()
    if resolved_api_key:
        params["api_key"] = resolved_api_key
    resolved_email = _openalex_email() if email is None else email.strip()
    if resolved_email:
        params["mailto"] = resolved_email

    try:
        payload = _get_json(OPENALEX_WORKS_URL, params)
    except Exception as exc:  # noqa: BLE001 - provider client returns structured errors.
        return {"error": str(exc), "data": []}

    results = payload.get("results", [])
    if not isinstance(results, list):
        results = []
    return {"data": [_normalize_work(item) for item in results if isinstance(item, dict)]}


def _get_json(url: str, params: dict[str, str | int]) -> dict[str, Any]:
    query_string = urllib.parse.urlencode(params, quote_via=urllib.parse.quote)
    request = urllib.request.Request(
        f"{url}?{query_string}",
        headers={"Accept": "application/json", "User-Agent": "Qiongli-CLI-MCP/1.0"},
    )
    try:
        with urllib.request.urlopen(request, timeout=DEFAULT_TIMEOUT_SECONDS) as response:
            return dict(json.loads(response.read()))
    except urllib.error.HTTPError as exc:
        raise RuntimeError(f"HTTP Error {exc.code}: {exc.reason}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(str(exc)) from exc


def _normalize_work(item: dict[str, Any]) -> dict[str, Any]:
    doi = str(item.get("doi", "") or "").removeprefix("https://doi.org/").strip()
    authors = [
        str(authorship.get("author", {}).get("display_name", "")).strip()
        for authorship in item.get("authorships", [])
        if isinstance(authorship, dict)
    ]
    return {
        "paperId": str(item.get("id", "") or "").strip(),
        "title": str(item.get("display_name", "") or "").strip(),
        "authors": [{"name": author} for author in authors if author],
        "year": item.get("publication_year"),
        "abstract": _abstract_from_inverted_index(item.get("abstract_inverted_index")),
        "url": str(item.get("id", "") or "").strip(),
        "venue": str(
            (item.get("primary_location") or {}).get("source", {}).get("display_name", "")
            if isinstance(item.get("primary_location"), dict)
            else ""
        ).strip(),
        "externalIds": {"DOI": doi} if doi else {},
        "citationCount": item.get("cited_by_count"),
        "provider": "openalex",
    }


def _abstract_from_inverted_index(raw: Any) -> str:
    if not isinstance(raw, dict):
        return ""
    positions: list[tuple[int, str]] = []
    for word, indexes in raw.items():
        if not isinstance(indexes, list):
            continue
        positions.extend((int(index), str(word)) for index in indexes if isinstance(index, int))
    return " ".join(word for _, word in sorted(positions))


def _filter_value(filters: dict[str, Any]) -> str:
    parts: list[str] = []
    if filters.get("year_start"):
        parts.append(f"from_publication_date:{filters['year_start']}-01-01")
    if filters.get("year_end"):
        parts.append(f"to_publication_date:{filters['year_end']}-12-31")
    if filters.get("publication_type"):
        parts.append(f"type:{filters['publication_type']}")
    return ",".join(parts)


def _openalex_email() -> str:
    providers = resolve_provider_config(cwd=Path.cwd()).get("providers", {})
    if not isinstance(providers, dict):
        return ""
    openalex = providers.get("openalex", {})
    if not isinstance(openalex, dict):
        return ""
    return str(openalex.get("email", "") or "").strip()


def _openalex_api_key() -> str:
    providers = resolve_provider_config(cwd=Path.cwd()).get("providers", {})
    if not isinstance(providers, dict):
        return ""
    openalex = providers.get("openalex", {})
    if not isinstance(openalex, dict):
        return ""
    return str(openalex.get("api_key", "") or "").strip()
