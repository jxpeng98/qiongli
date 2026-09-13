from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

from bridges.provider_config import resolve_provider_config


CROSSREF_WORKS_URL = "https://api.crossref.org/works"
DEFAULT_TIMEOUT_SECONDS = 15


def search(
    translation: dict[str, object],
    limit: int,
    *,
    email: str | None = None,
) -> dict[str, object]:
    query = str(translation.get("translated_query", "") or "").strip()
    if not query:
        return {"data": []}

    filters = translation.get("filters", {})
    filters = filters if isinstance(filters, dict) else {}
    params: dict[str, str | int] = {
        "query.bibliographic": query,
        "rows": max(1, int(limit)),
    }
    filter_value = _filter_value(filters)
    if filter_value:
        params["filter"] = filter_value
    mailto = _crossref_email() if email is None else email.strip()
    if mailto:
        params["mailto"] = mailto

    try:
        payload = _get_json(CROSSREF_WORKS_URL, params)
    except Exception as exc:  # noqa: BLE001 - provider client returns structured errors.
        return {"error": str(exc), "data": []}

    message = payload.get("message", {})
    items = message.get("items", []) if isinstance(message, dict) else []
    if not isinstance(items, list):
        items = []
    return {"data": [_normalize_item(item) for item in items if isinstance(item, dict)]}


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


def _normalize_item(item: dict[str, Any]) -> dict[str, Any]:
    doi = str(item.get("DOI", "") or "").strip()
    return {
        "paperId": doi,
        "title": _first_text(item.get("title")),
        "authors": [{"name": author} for author in _authors(item.get("author"))],
        "year": _issued_year(item.get("issued")),
        "abstract": str(item.get("abstract", "") or "").strip(),
        "url": str(item.get("URL", "") or "").strip(),
        "venue": _first_text(item.get("container-title")),
        "externalIds": {"DOI": doi} if doi else {},
        "citationCount": item.get("is-referenced-by-count"),
        "provider": "crossref",
        "doi": doi,
    }


def _filter_value(filters: dict[str, Any]) -> str:
    parts: list[str] = []
    if filters.get("year_start"):
        parts.append(f"from-pub-date:{filters['year_start']}-01-01")
    if filters.get("year_end"):
        parts.append(f"until-pub-date:{filters['year_end']}-12-31")
    if filters.get("publication_type"):
        parts.append(f"type:{filters['publication_type']}")
    return ",".join(parts)


def _first_text(raw: Any) -> str:
    if isinstance(raw, list) and raw:
        return str(raw[0] or "").strip()
    return str(raw or "").strip()


def _authors(raw: Any) -> list[str]:
    if not isinstance(raw, list):
        return []
    names: list[str] = []
    for item in raw:
        if not isinstance(item, dict):
            continue
        given = str(item.get("given", "") or "").strip()
        family = str(item.get("family", "") or "").strip()
        name = " ".join(part for part in (given, family) if part)
        if name:
            names.append(name)
    return names


def _issued_year(raw: Any) -> int | None:
    if not isinstance(raw, dict):
        return None
    date_parts = raw.get("date-parts", [])
    if not isinstance(date_parts, list) or not date_parts:
        return None
    first = date_parts[0]
    if not isinstance(first, list) or not first:
        return None
    try:
        return int(first[0])
    except (TypeError, ValueError):
        return None


def _crossref_email() -> str:
    providers = resolve_provider_config(cwd=Path.cwd()).get("providers", {})
    if not isinstance(providers, dict):
        return ""
    crossref = providers.get("crossref", {})
    if not isinstance(crossref, dict):
        return ""
    return str(crossref.get("email", "") or "").strip()
