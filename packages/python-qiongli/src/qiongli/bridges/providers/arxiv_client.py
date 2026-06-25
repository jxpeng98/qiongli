from __future__ import annotations

import re
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from typing import Any


ARXIV_QUERY_URL = "http://export.arxiv.org/api/query"
DEFAULT_TIMEOUT_SECONDS = 15
ATOM_NS = "{http://www.w3.org/2005/Atom}"
ARXIV_NS = "{http://arxiv.org/schemas/atom}"


def search(translation: dict[str, object], limit: int) -> dict[str, object]:
    query = _search_query(translation)
    if not query:
        return {"data": []}

    params: dict[str, str | int] = {
        "search_query": query,
        "start": 0,
        "max_results": _normalize_limit(limit),
        "sortBy": "relevance",
        "sortOrder": "descending",
    }

    try:
        payload = _get_text(ARXIV_QUERY_URL, params)
    except Exception as exc:  # noqa: BLE001 - provider client returns structured errors.
        return {"error": str(exc), "data": []}

    return {"data": _normalize_feed(payload)}


def _search_query(translation: dict[str, object]) -> str:
    payload = translation.get("payload", {})
    if isinstance(payload, dict):
        search_query = str(payload.get("search_query", "") or "").strip()
        if search_query:
            return search_query
    return str(translation.get("translated_query", "") or "").strip()


def _normalize_limit(limit: int) -> int:
    try:
        return max(1, int(limit))
    except (TypeError, ValueError):
        return 1


def _get_text(url: str, params: dict[str, str | int]) -> str:
    query_string = urllib.parse.urlencode(params, quote_via=urllib.parse.quote)
    request = urllib.request.Request(
        f"{url}?{query_string}",
        headers={"Accept": "application/atom+xml", "User-Agent": "Qiongli-CLI-MCP/1.0"},
    )
    try:
        with urllib.request.urlopen(request, timeout=DEFAULT_TIMEOUT_SECONDS) as response:
            return response.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as exc:
        raise RuntimeError(f"HTTP Error {exc.code}: {exc.reason}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(str(exc)) from exc


def _normalize_feed(payload: str) -> list[dict[str, Any]]:
    try:
        root = ET.fromstring(payload)
    except ET.ParseError as exc:
        raise RuntimeError(f"invalid arXiv Atom response: {exc}") from exc

    records: list[dict[str, Any]] = []
    for entry in root.findall(f"{ATOM_NS}entry"):
        records.append(_normalize_entry(entry))
    return records


def _normalize_entry(entry: ET.Element) -> dict[str, Any]:
    entry_id = _text(entry, "id")
    arxiv_id = _arxiv_id(entry_id)
    pdf_url = _link(entry, rel="related", mime_type="application/pdf")
    doi = _text(entry, "doi", namespace=ARXIV_NS)
    record: dict[str, Any] = {
        "paperId": arxiv_id,
        "title": _collapse_ws(_text(entry, "title")),
        "authors": [{"name": author} for author in _authors(entry)],
        "year": _year(_text(entry, "published")),
        "abstract": _collapse_ws(_text(entry, "summary")),
        "url": _link(entry, rel="alternate") or entry_id,
        "venue": "arXiv",
        "externalIds": {"ArXiv": arxiv_id},
        "citationCount": None,
        "provider": "arxiv",
    }
    if doi:
        record["externalIds"]["DOI"] = doi
    if pdf_url:
        record["openAccessPdf"] = {"url": pdf_url}
    return record


def _text(entry: ET.Element, tag: str, *, namespace: str = ATOM_NS) -> str:
    node = entry.find(f"{namespace}{tag}")
    return "" if node is None or node.text is None else str(node.text).strip()


def _authors(entry: ET.Element) -> list[str]:
    names: list[str] = []
    for author in entry.findall(f"{ATOM_NS}author"):
        name = _text(author, "name")
        if name:
            names.append(_collapse_ws(name))
    return names


def _link(entry: ET.Element, *, rel: str, mime_type: str | None = None) -> str:
    for link in entry.findall(f"{ATOM_NS}link"):
        if link.attrib.get("rel") != rel:
            continue
        if mime_type and link.attrib.get("type") != mime_type:
            continue
        href = str(link.attrib.get("href", "")).strip()
        if href:
            return href
    return ""


def _arxiv_id(value: str) -> str:
    cleaned = str(value or "").strip()
    if not cleaned:
        return ""
    cleaned = cleaned.rsplit("/", 1)[-1]
    cleaned = cleaned.removeprefix("arXiv:")
    return cleaned


def _year(value: str) -> int | None:
    match = re.search(r"\b(19|20)\d{2}\b", str(value or ""))
    return int(match.group(0)) if match else None


def _collapse_ws(value: str) -> str:
    return " ".join(str(value or "").split())
