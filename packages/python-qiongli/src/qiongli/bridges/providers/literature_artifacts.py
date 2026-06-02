from __future__ import annotations

import csv
import io
import json
import re
from pathlib import Path
from typing import Any

from bridges.providers.literature_schema import SEARCH_LOG_FIELDS, SEARCH_RESULT_FIELDS


SEARCH_BUNDLE_FILENAMES = {
    "search_strategy": "search_strategy.md",
    "search_log": "search_log.md",
    "search_results": "search_results.csv",
    "dedup_log": "dedup_log.csv",
    "search_diagnostics": "search_diagnostics.md",
}
DEDUP_LOG_FIELDS = (
    "candidate_record_id",
    "canonical_record_id",
    "decision",
    "match_basis",
    "resolver",
    "notes",
)
JSON_BLOCK_RE = re.compile(r"```json\s*\n(.*?)\n```", re.DOTALL)


def materialize_search_bundle(
    project_root: str | Path,
    search_bundle: dict[str, Any],
) -> dict[str, Path]:
    """Write the canonical literature search bundle under ``project_root``."""
    root = Path(project_root)
    root.mkdir(parents=True, exist_ok=True)
    data = _bundle_data(search_bundle)

    paths = {
        name: root / filename for name, filename in SEARCH_BUNDLE_FILENAMES.items()
    }
    paths["search_strategy"].write_text(_render_search_strategy(data), encoding="utf-8")
    paths["search_log"].write_text(_render_search_log(data), encoding="utf-8")
    _write_csv(
        paths["search_results"],
        data.get("search_results", []),
        base_fields=SEARCH_RESULT_FIELDS,
    )
    _write_csv(
        paths["dedup_log"],
        data.get("dedup_log", []),
        base_fields=DEDUP_LOG_FIELDS,
    )
    paths["search_diagnostics"].write_text(_render_search_diagnostics(data), encoding="utf-8")
    return paths


def read_search_diagnostics(project_root: str | Path) -> dict[str, Any]:
    path = Path(project_root) / SEARCH_BUNDLE_FILENAMES["search_diagnostics"]
    if not path.exists():
        return {}
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return {}

    match = JSON_BLOCK_RE.search(text)
    if not match:
        return _parse_diagnostics_fallback(text)
    try:
        parsed = json.loads(match.group(1))
    except json.JSONDecodeError:
        return _parse_diagnostics_fallback(text)
    return parsed if isinstance(parsed, dict) else {}


def _bundle_data(search_bundle: dict[str, Any]) -> dict[str, Any]:
    data = search_bundle.get("data", search_bundle)
    return data if isinstance(data, dict) else {}


def _render_search_strategy(data: dict[str, Any]) -> str:
    query_plan = _mapping(data.get("query_plan"))
    query_variants = _rows(data.get("query_variants"))
    lines = [
        "# Search Strategy",
        "",
        f"- Provider mode: {_scalar(data.get('provider_mode'))}",
        f"- Per-query limit: {_scalar(data.get('per_query_limit'))}",
        f"- Query count: {len(query_variants)}",
        "",
        "## Query Variants",
        "",
        _markdown_table(
            rows=query_variants,
            fields=("query_id", "query"),
            empty_text="No query variants recorded.",
        ),
        "",
        "## Machine-Readable Search Plan",
        "",
        "```json",
        _json_dump(query_plan),
        "```",
        "",
    ]
    return "\n".join(lines)


def _render_search_log(data: dict[str, Any]) -> str:
    rows = [_normalize_search_log_row(row) for row in _rows(data.get("search_log"))]
    lines = [
        "# Search Log",
        "",
        _markdown_table(
            rows=rows,
            fields=SEARCH_LOG_FIELDS,
            empty_text="No search executions recorded.",
        ),
        "",
    ]
    return "\n".join(lines)


def _render_search_diagnostics(data: dict[str, Any]) -> str:
    diagnostics = _mapping(data.get("search_diagnostics"))
    screening_readiness = diagnostics.get("screening_readiness", {})
    bundle_gate = _bundle_gate(diagnostics)
    lines = [
        "# Search Diagnostics",
        "",
        "## Search Scope",
        "",
        f"- Attempted providers: {_join_list(diagnostics.get('attempted_providers'))}",
        f"- Attempted queries: {_scalar(diagnostics.get('attempted_query_count'))}",
        f"- Unique results: {_scalar(diagnostics.get('unique_result_count'))}",
        f"- Status reason: {_scalar(diagnostics.get('status_reason') or diagnostics.get('status'))}",
        "",
        "## Known-Item Recall",
        "",
        _render_nested_summary(diagnostics.get("known_item_recall")),
        "",
        "## Provider Coverage",
        "",
        _render_nested_summary(diagnostics.get("provider_coverage")),
        "",
        "## Query Coverage",
        "",
        _render_nested_summary(diagnostics.get("query_coverage")),
        "",
        "## Deduplication Summary",
        "",
        f"- Duplicate count: {_scalar(diagnostics.get('duplicate_count'))}",
        f"- Dedup ratio: {_scalar(diagnostics.get('dedup_ratio'))}",
        "",
        "## Coverage Gaps",
        "",
        _render_nested_summary(diagnostics.get("coverage_gaps") or diagnostics.get("recommended_actions")),
        "",
        "## Next Search Actions",
        "",
        _render_nested_summary(diagnostics.get("recommended_actions")),
        "",
        "## Screening Readiness",
        "",
        _render_nested_summary(screening_readiness),
        "",
        "## Bundle Gate State",
        "",
        _render_nested_summary(bundle_gate),
        "",
        "## Machine-Readable Diagnostics",
        "",
        "```json",
        _json_dump(diagnostics),
        "```",
        "",
    ]
    return "\n".join(lines)


def _write_csv(path: Path, rows: Any, *, base_fields: tuple[str, ...]) -> None:
    normalized_rows = _rows(rows)
    fieldnames = list(base_fields)
    extra_fields = sorted(
        {
            key
            for row in normalized_rows
            for key in row
            if key not in fieldnames
        }
    )
    fieldnames.extend(extra_fields)

    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for row in normalized_rows:
            writer.writerow({field: _csv_value(row.get(field, "")) for field in fieldnames})


def _normalize_search_log_row(row: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(row)
    if "raw_count" not in normalized:
        normalized["raw_count"] = normalized.get("retrieved_count", normalized.get("raw_hit_count", ""))
    if "normalized_count" not in normalized:
        normalized["normalized_count"] = normalized.get(
            "normalized_hits",
            normalized.get("retrieved_count", ""),
        )
    return normalized


def _markdown_table(
    *,
    rows: list[dict[str, Any]],
    fields: tuple[str, ...],
    empty_text: str,
) -> str:
    if not rows:
        return empty_text
    output = io.StringIO()
    output.write("| " + " | ".join(fields) + " |\n")
    output.write("| " + " | ".join("---" for _ in fields) + " |\n")
    for row in rows:
        output.write(
            "| "
            + " | ".join(_markdown_cell(row.get(field, "")) for field in fields)
            + " |\n"
        )
    return output.getvalue().rstrip()


def _render_nested_summary(value: Any) -> str:
    if value in ({}, [], None, ""):
        return "No diagnostics recorded."
    if isinstance(value, dict):
        return "\n".join(
            f"- {key}: {_scalar(value[key])}" for key in sorted(value)
        )
    if isinstance(value, list):
        return "\n".join(f"- {_scalar(item)}" for item in value) or "No diagnostics recorded."
    return f"- {_scalar(value)}"


def _bundle_gate(diagnostics: dict[str, Any]) -> Any:
    if "bundle_gate" in diagnostics:
        return diagnostics["bundle_gate"]
    if "bundle_gate_state" in diagnostics:
        return diagnostics["bundle_gate_state"]
    return {}


def _parse_diagnostics_fallback(text: str) -> dict[str, Any]:
    diagnostics: dict[str, Any] = {}
    lower_text = text.lower()
    if "known_item_missing" in lower_text:
        diagnostics["known_item_recall"] = {"status": "known_item_missing"}
    bundle_state = re.search(r"bundle[_\s-]*gate(?:[_\s-]*state)?\s*[:=]\s*([A-Za-z0-9_-]+)", text, re.I)
    if bundle_state:
        diagnostics["bundle_gate"] = {"state": bundle_state.group(1)}
    readiness = re.search(r"screening[_\s-]*readiness\s*[:=]\s*([A-Za-z0-9_-]+)", text, re.I)
    if readiness:
        diagnostics["screening_readiness"] = {"state": readiness.group(1)}
    return diagnostics


def _rows(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [dict(item) for item in value if isinstance(item, dict)]


def _mapping(value: Any) -> dict[str, Any]:
    return dict(value) if isinstance(value, dict) else {}


def _json_dump(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True)


def _join_list(value: Any) -> str:
    if isinstance(value, list):
        cleaned = [str(item) for item in value if str(item).strip()]
        return ", ".join(cleaned) if cleaned else "not recorded"
    return _scalar(value)


def _scalar(value: Any) -> str:
    if value in (None, ""):
        return "not recorded"
    if isinstance(value, (dict, list)):
        return json.dumps(value, ensure_ascii=False, sort_keys=True)
    return str(value)


def _csv_value(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, (dict, list)):
        return json.dumps(value, ensure_ascii=False, sort_keys=True)
    return str(value)


def _markdown_cell(value: Any) -> str:
    text = _scalar(value)
    return text.replace("|", "\\|").replace("\n", " ")
