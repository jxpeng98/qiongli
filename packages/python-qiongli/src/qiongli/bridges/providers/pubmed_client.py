from __future__ import annotations

import json
import re
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

from bridges.provider_config import resolve_provider_config


PUBMED_ESEARCH_URL = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi"
PUBMED_ESUMMARY_URL = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi"
DEFAULT_TIMEOUT_SECONDS = 15
YEAR_RE = re.compile(r"\b(19|20)\d{2}\b")


def search(translation: dict[str, object], limit: int) -> dict[str, object]:
    query = str(translation.get("translated_query", "") or "").strip()
    if not query:
        return {"data": []}

    try:
        search_payload = _get_json(
            PUBMED_ESEARCH_URL,
            {
                "db": "pubmed",
                "retmode": "json",
                "retmax": max(1, int(limit)),
                "term": query,
                **_api_key_param(),
            },
        )
        ids = _ids_from_search(search_payload)
        if not ids:
            return {"data": []}
        summary_payload = _get_json(
            PUBMED_ESUMMARY_URL,
            {
                "db": "pubmed",
                "retmode": "json",
                "id": ",".join(ids),
                **_api_key_param(),
            },
        )
    except Exception as exc:  # noqa: BLE001 - provider client returns structured errors.
        return {"error": str(exc), "data": []}

    return {"data": [_normalize_summary(summary_payload, uid) for uid in ids]}


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


def _ids_from_search(payload: dict[str, Any]) -> list[str]:
    result = payload.get("esearchresult", {})
    ids = result.get("idlist", []) if isinstance(result, dict) else []
    if not isinstance(ids, list):
        return []
    return [str(uid).strip() for uid in ids if str(uid).strip()]


def _normalize_summary(payload: dict[str, Any], uid: str) -> dict[str, Any]:
    result = payload.get("result", {})
    item = result.get(uid, {}) if isinstance(result, dict) else {}
    item = item if isinstance(item, dict) else {}
    title = str(item.get("title", "") or "").strip()
    pubdate = str(item.get("pubdate", "") or "").strip()
    return {
        "paperId": uid,
        "title": title,
        "authors": [{"name": author} for author in _authors(item.get("authors"))],
        "year": _year_from_pubdate(pubdate),
        "abstract": "",
        "url": f"https://pubmed.ncbi.nlm.nih.gov/{uid}/",
        "venue": str(item.get("source", "") or "").strip(),
        "externalIds": {"PubMed": uid},
        "citationCount": None,
        "provider": "pubmed",
    }


def _authors(raw: Any) -> list[str]:
    if not isinstance(raw, list):
        return []
    names: list[str] = []
    for item in raw:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name", "") or "").strip()
        if name:
            names.append(name)
    return names


def _year_from_pubdate(pubdate: str) -> int | None:
    match = YEAR_RE.search(pubdate)
    if not match:
        return None
    return int(match.group(0))


def _api_key_param() -> dict[str, str]:
    providers = resolve_provider_config(cwd=Path.cwd()).get("providers", {})
    if not isinstance(providers, dict):
        return {}
    pubmed = providers.get("pubmed", {})
    if not isinstance(pubmed, dict):
        return {}
    api_key = str(pubmed.get("api_key", "") or "").strip()
    return {"api_key": api_key} if api_key else {}
