from __future__ import annotations

from typing import Any, Mapping


def build_resource_activation_plan(
    *,
    decision: str,
    active_subject: str,
    primary_subject: str,
    loaded_resources: dict[str, Any],
    method_lenses: list[str],
    borrowed_lenses: list[dict[str, Any]],
    persistence: dict[str, Any],
) -> dict[str, Any]:
    status = str(persistence.get("status", "none") or "none")
    resources: list[dict[str, Any]] = []
    activation = _resource_activation(decision, status)

    for path in _string_list(loaded_resources.get("overlays")):
        resources.append(
            {
                "kind": "subject_overlay",
                "subject": primary_subject,
                "path": path,
                "activation": activation,
            }
        )

    for path in _string_list(loaded_resources.get("subject_skills")):
        resources.append(
            {
                "kind": "subject_skill",
                "subject": primary_subject,
                "path": path,
                "activation": activation,
            }
        )

    method_pack_paths = _string_list(loaded_resources.get("method_packs"))
    for index, lens in enumerate(method_lenses):
        path = _path_for_lens(lens, method_pack_paths, index)
        if not path:
            continue
        resources.append(
            {
                "kind": "method_pack",
                "subject": primary_subject,
                "lens": lens,
                "path": path,
                "activation": activation,
            }
        )

    borrowed_offset = len(method_lenses)
    for index, record in enumerate(borrowed_lenses):
        if not isinstance(record, Mapping):
            continue
        lens = str(record.get("lens", "") or "")
        source_subject = str(record.get("source_subject", primary_subject) or primary_subject)
        if not lens:
            continue
        path = _path_for_lens(lens, method_pack_paths, borrowed_offset + index)
        if not path:
            continue
        resources.append(
            {
                "kind": "method_pack_only",
                "subject": source_subject,
                "lens": lens,
                "path": path,
                "activation": "temporary",
            }
        )

    return {
        "decision": decision,
        "active_subject": active_subject,
        "primary_subject": primary_subject,
        "levels": _normalize_levels(_string_list(loaded_resources.get("levels"))),
        "resources": _unique_resource_records(resources),
        "persistence_recommendation": {
            "status": status,
            "write_manifest": False,
            "recommended_subject_mode": _recommended_subject_mode(decision),
        },
        "contract_warnings": _string_list(loaded_resources.get("contract_warnings")),
    }


def _normalize_levels(levels: list[str]) -> list[str]:
    if "core_only" in levels:
        return ["core"]
    normalized = ["core"]
    for level in levels:
        if level in {"core", "core_only"} or level in normalized:
            continue
        normalized.append(level)
    return normalized


def _resource_activation(decision: str, status: str) -> str:
    if decision == "suggest_subject":
        return "proposed"
    if decision == "borrow_lens" or status == "temporary":
        return "temporary"
    if status in {"locked", "applied", "proposed"}:
        return status
    return "available"


def _recommended_subject_mode(decision: str) -> str:
    if decision == "suggest_subject":
        return "suggested"
    if decision == "confirm_subject":
        return "confirmed"
    if decision == "lock_subject":
        return "locked"
    return "auto"


def _path_for_lens(lens: str, paths: list[str], index: int) -> str:
    marker = f"/{lens}."
    for path in paths:
        if marker in path or path.endswith(f"/{lens}.yaml"):
            return path
    if 0 <= index < len(paths):
        return paths[index]
    return ""


def _string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item.strip() for item in value if isinstance(item, str) and item.strip()]


def _unique_resource_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    seen: set[tuple[str, str, str]] = set()
    unique: list[dict[str, Any]] = []
    for record in records:
        marker = (
            str(record.get("kind", "")),
            str(record.get("subject", "")),
            str(record.get("path", "")),
        )
        if marker in seen:
            continue
        seen.add(marker)
        unique.append(record)
    return unique
