from __future__ import annotations

import re
from itertools import combinations
from typing import Any

from bridges.providers.literature_query import validate_query_plan


DOI_PREFIX_RE = re.compile(r"^(?:https?://(?:dx\.)?doi\.org/|doi:\s*)", re.I)
NON_ALNUM_RE = re.compile(r"[^a-z0-9]+")
FAILURE_STATUSES = {"error", "fail", "failed"}
SUCCESS_STATUSES = {"ok", "success", "warning", "partial"}
COUNT_FIELDS = ("normalized_count", "retrieved_count", "raw_count", "result_count", "hit_count")


def build_search_diagnostics_v2(
    query_plan: dict[str, Any],
    search_log: list[dict[str, Any]],
    search_results: list[dict[str, Any]],
    dedup_log: list[dict[str, Any]],
    provider_summaries: dict[str, dict[str, Any]],
    raw_diagnostics: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Build literature-search quality gates without owning provider execution."""
    plan = _as_dict(query_plan)
    logs = _list_of_dicts(search_log)
    results = _list_of_dicts(search_results)
    dedup_entries = _list_of_dicts(dedup_log)
    summaries = _provider_summaries(provider_summaries)
    raw = _as_dict(raw_diagnostics)

    search_mode = _normalize_search_mode(plan.get("search_mode"))
    validation_errors = _validate_plan(plan)
    provider_coverage = _build_provider_coverage(logs, results, summaries, raw)
    query_health = _build_query_health(plan, logs, results)
    concept_coverage = _build_concept_coverage(plan, logs, results)
    known_item_recall = _build_known_item_recall(plan, results)
    provider_overlap = _build_provider_overlap(results)
    dedup_health = _build_dedup_health(results, dedup_entries, raw)
    screening_readiness = _build_screening_readiness(results, raw)
    snowball_readiness = _build_snowball_readiness(results, raw)

    blocking_reasons: list[str] = []
    warnings: list[str] = []

    if validation_errors:
        blocking_reasons.append("invalid_query_plan")

    all_providers_failed = _all_providers_failed(provider_coverage, raw)
    zero_hits = _zero_hits(results, raw)
    if all_providers_failed:
        blocking_reasons.append("all_providers_failed")
    if zero_hits:
        blocking_reasons.append("zero_hits")

    if provider_coverage["partial_failure_providers"]:
        warnings.append("partial_provider_failure")

    if search_mode == "targeted_search":
        if provider_coverage["success_count"] == 1:
            warnings.append("single_successful_provider")
        if not concept_coverage["all_required_covered"]:
            warnings.append("missing_required_concept_coverage")
        if known_item_recall["total_count"] and not known_item_recall["all_recalled"]:
            warnings.append("missing_known_items")
    else:
        if provider_coverage["success_count"] < 2:
            blocking_reasons.append("less_than_two_successful_providers")
        if not concept_coverage["all_required_covered"]:
            blocking_reasons.append("missing_required_concepts")
        if known_item_recall["total_count"] and not known_item_recall["all_recalled"]:
            blocking_reasons.append("missing_known_items")

    if search_mode == "systematic_review":
        if not screening_readiness["usable"]:
            blocking_reasons.append("screening_not_ready")
        if not snowball_readiness["usable"]:
            blocking_reasons.append("snowball_not_ready")

    blocking_reasons = _dedupe_strings(blocking_reasons)
    warnings = _dedupe_strings(warnings)
    gate_status = _gate_status(blocking_reasons, warnings)

    return {
        "search_mode": search_mode,
        "gate_status": gate_status,
        "blocking_reasons": blocking_reasons,
        "warnings": warnings,
        "concept_coverage": concept_coverage,
        "known_item_recall": known_item_recall,
        "provider_coverage": provider_coverage,
        "provider_overlap": provider_overlap,
        "query_health": query_health,
        "dedup_health": dedup_health,
        "screening_readiness": screening_readiness,
        "snowball_readiness": snowball_readiness,
        "recommended_actions": _recommended_actions(
            blocking_reasons,
            warnings,
            concept_coverage,
            known_item_recall,
        ),
        "raw_diagnostics": raw,
        "query_plan_validation_errors": validation_errors,
    }


def _validate_plan(plan: dict[str, Any]) -> list[str]:
    try:
        validate_query_plan(plan)
    except Exception as exc:
        return [str(exc)]
    return []


def _build_provider_coverage(
    search_log: list[dict[str, Any]],
    search_results: list[dict[str, Any]],
    provider_summaries: dict[str, dict[str, Any]],
    raw_diagnostics: dict[str, Any],
) -> dict[str, Any]:
    attempted: list[str] = []
    hit_counts: dict[str, int] = {}
    failure_counts: dict[str, int] = {}

    for provider in _iter_values(raw_diagnostics.get("attempted_providers")):
        provider_name = _normalize_provider(provider)
        if provider_name:
            _append_unique(attempted, provider_name)

    for provider in _iter_values(raw_diagnostics.get("successful_providers")):
        provider_name = _normalize_provider(provider)
        if provider_name:
            _append_unique(attempted, provider_name)
            hit_counts[provider_name] = max(hit_counts.get(provider_name, 0), 1)

    for provider in _iter_values(raw_diagnostics.get("failed_providers")):
        provider_name = _normalize_provider(provider)
        if provider_name:
            _append_unique(attempted, provider_name)
            failure_counts[provider_name] = failure_counts.get(provider_name, 0) + 1

    for provider, summary in provider_summaries.items():
        provider_name = _normalize_provider(summary.get("provider") or provider)
        if not provider_name:
            continue
        _append_unique(attempted, provider_name)
        hit_counts[provider_name] = hit_counts.get(provider_name, 0) + _summary_hit_count(summary)
        if _summary_failed(summary):
            failure_counts[provider_name] = failure_counts.get(provider_name, 0) + 1

    for entry in search_log:
        provider_name = _normalize_provider(entry.get("provider"))
        if not provider_name:
            continue
        _append_unique(attempted, provider_name)
        if _entry_successful(entry) and _entry_count(entry) > 0:
            hit_counts[provider_name] = hit_counts.get(provider_name, 0) + _entry_count(entry)
        if _entry_failed(entry):
            failure_counts[provider_name] = failure_counts.get(provider_name, 0) + 1

    for result in search_results:
        for provider_name in _providers_from_result(result):
            _append_unique(attempted, provider_name)
            hit_counts[provider_name] = hit_counts.get(provider_name, 0) + 1

    if raw_diagnostics.get("all_providers_failed") is True:
        for provider in attempted:
            if hit_counts.get(provider, 0) == 0:
                failure_counts[provider] = failure_counts.get(provider, 0) + 1

    successful = sorted(provider for provider in attempted if hit_counts.get(provider, 0) > 0)
    failed = sorted(
        provider
        for provider in attempted
        if provider not in successful and failure_counts.get(provider, 0) > 0
    )
    partial = sorted(
        provider
        for provider in attempted
        if provider in successful and failure_counts.get(provider, 0) > 0
    )

    return {
        "attempted_providers": attempted,
        "successful_providers": successful,
        "failed_providers": failed,
        "partial_failure_providers": partial,
        "success_count": len(successful),
        "failure_count": len(failed),
        "hit_counts": {provider: hit_counts.get(provider, 0) for provider in attempted},
    }


def _build_query_health(
    query_plan: dict[str, Any],
    search_log: list[dict[str, Any]],
    search_results: list[dict[str, Any]],
) -> dict[str, Any]:
    planned_query_ids = _planned_query_ids(query_plan)
    observed_result_query_ids = {
        query_id
        for result in search_results
        for query_id in _query_ids_from_result(result)
    }
    successful_query_ids: set[str] = set(observed_result_query_ids)
    failed_query_ids: set[str] = set()
    zero_hit_query_ids: set[str] = set()

    for entry in search_log:
        query_id = _clean_text(entry.get("query_id"))
        if not query_id:
            continue
        count = _entry_count(entry)
        if _entry_failed(entry):
            failed_query_ids.add(query_id)
        elif _entry_successful(entry) and count > 0:
            successful_query_ids.add(query_id)
        elif _entry_successful(entry):
            zero_hit_query_ids.add(query_id)

    attempted_query_ids = [
        query_id
        for query_id in (_clean_text(entry.get("query_id")) for entry in search_log)
        if query_id
    ]

    return {
        "planned_query_count": len(planned_query_ids),
        "attempted_query_count": len(search_log),
        "successful_query_count": len(successful_query_ids),
        "failed_query_count": len(failed_query_ids),
        "zero_hit_query_count": len(zero_hit_query_ids),
        "planned_query_ids": planned_query_ids,
        "attempted_query_ids": _dedupe_strings(attempted_query_ids),
        "successful_query_ids": sorted(successful_query_ids),
        "failed_query_ids": sorted(failed_query_ids),
        "zero_hit_query_ids": sorted(zero_hit_query_ids),
        "unobserved_query_ids": sorted(set(planned_query_ids) - set(attempted_query_ids)),
    }


def _build_concept_coverage(
    query_plan: dict[str, Any],
    search_log: list[dict[str, Any]],
    search_results: list[dict[str, Any]],
) -> dict[str, Any]:
    concepts = [
        block
        for block in query_plan.get("concept_blocks", [])
        if isinstance(block, dict) and _clean_text(block.get("id"))
    ]
    concept_query_ids: dict[str, set[str]] = {
        _clean_text(block.get("id")): set()
        for block in concepts
    }
    for translation in query_plan.get("provider_translations", []):
        if not isinstance(translation, dict):
            continue
        query_id = _clean_text(translation.get("query_id"))
        if not query_id:
            continue
        for concept_id in translation.get("concept_ids", []):
            normalized_id = _clean_text(concept_id)
            if normalized_id in concept_query_ids:
                concept_query_ids[normalized_id].add(query_id)

    observed_query_ids = _observed_successful_query_ids(search_log, search_results)
    blocks: list[dict[str, Any]] = []
    missing_required: list[str] = []
    for block in concepts:
        concept_id = _clean_text(block.get("id"))
        planned_query_ids = sorted(concept_query_ids.get(concept_id, set()))
        observed_for_concept = sorted(set(planned_query_ids) & observed_query_ids)
        covered = bool(observed_for_concept)
        required = bool(block.get("required"))
        if required and not covered:
            missing_required.append(concept_id)
        blocks.append(
            {
                "id": concept_id,
                "label": _clean_text(block.get("label")),
                "required": required,
                "planned_query_ids": planned_query_ids,
                "observed_query_ids": observed_for_concept,
                "covered": covered,
            }
        )

    covered_required = [
        block["id"]
        for block in blocks
        if block["required"] and block["covered"]
    ]
    return {
        "all_required_covered": not missing_required,
        "covered_required_count": len(covered_required),
        "required_count": len([block for block in blocks if block["required"]]),
        "missing_required_concepts": missing_required,
        "covered_concepts": [block["id"] for block in blocks if block["covered"]],
        "blocks": blocks,
    }


def _build_known_item_recall(
    query_plan: dict[str, Any],
    search_results: list[dict[str, Any]],
) -> dict[str, Any]:
    known_items = [
        item
        for item in query_plan.get("known_items", [])
        if isinstance(item, dict)
    ]
    result_dois = {_normalize_doi(_result_doi(result)) for result in search_results}
    result_dois.discard("")
    result_paper_ids = {
        _normalize_identifier(_result_paper_id(result))
        for result in search_results
    }
    result_paper_ids.discard("")
    result_titles = {_normalize_title(result.get("title")) for result in search_results}
    result_titles.discard("")

    recalled_items: list[dict[str, str]] = []
    missing_items: list[dict[str, str]] = []
    for item in known_items:
        title = _clean_text(item.get("title"))
        doi = _normalize_doi(item.get("doi") or item.get("DOI"))
        paper_id = _normalize_identifier(item.get("paper_id") or item.get("paperId"))
        title_key = _normalize_title(title)
        matched_by = ""
        if doi and doi in result_dois:
            matched_by = "doi"
        elif paper_id and paper_id in result_paper_ids:
            matched_by = "paper_id"
        elif title_key and title_key in result_titles:
            matched_by = "title"

        item_summary = {
            "title": title,
            "doi": doi,
            "paper_id": paper_id,
        }
        if matched_by:
            recalled_items.append({**item_summary, "matched_by": matched_by})
        else:
            missing_items.append(item_summary)

    total_count = len(known_items)
    recalled_count = len(recalled_items)
    return {
        "required": total_count > 0,
        "total_count": total_count,
        "recalled_count": recalled_count,
        "missing_count": len(missing_items),
        "recall_rate": (recalled_count / total_count) if total_count else None,
        "all_recalled": recalled_count == total_count,
        "recalled_items": recalled_items,
        "missing_items": missing_items,
    }


def _build_provider_overlap(search_results: list[dict[str, Any]]) -> dict[str, Any]:
    providers_by_key: dict[str, set[str]] = {}
    for result in search_results:
        providers = _providers_from_result(result)
        if not providers:
            continue
        match_key = _overlap_key(result)
        if not match_key:
            continue
        providers_by_key.setdefault(match_key, set()).update(providers)

    shared_keys = [
        {"key": key, "providers": sorted(providers), "provider_count": len(providers)}
        for key, providers in sorted(providers_by_key.items())
        if len(providers) > 1
    ]
    pair_counts: dict[str, int] = {}
    for item in shared_keys:
        for left, right in combinations(item["providers"], 2):
            pair_key = f"{left}|{right}"
            pair_counts[pair_key] = pair_counts.get(pair_key, 0) + 1

    return {
        "overlap_count": len(shared_keys),
        "shared_keys": shared_keys,
        "provider_pair_counts": dict(sorted(pair_counts.items())),
    }


def _build_dedup_health(
    search_results: list[dict[str, Any]],
    dedup_log: list[dict[str, Any]],
    raw_diagnostics: dict[str, Any],
) -> dict[str, Any]:
    raw_duplicate_count = _to_int(raw_diagnostics.get("duplicate_count"))
    duplicate_count = raw_duplicate_count if raw_duplicate_count is not None else len(dedup_log)
    denominator = _to_int(raw_diagnostics.get("normalized_result_count"))
    if denominator is None:
        denominator = len(search_results) + duplicate_count
    duplicate_rate = None
    if denominator and denominator > 0:
        duplicate_rate = duplicate_count / denominator
    elif duplicate_count == 0:
        duplicate_rate = 0.0

    return {
        "duplicate_count": duplicate_count,
        "duplicate_rate": duplicate_rate,
        "dedup_log_count": len(dedup_log),
        "normalized_result_count": denominator,
        "unique_result_count": _to_int(raw_diagnostics.get("unique_result_count")) or len(search_results),
    }


def _build_screening_readiness(
    search_results: list[dict[str, Any]],
    raw_diagnostics: dict[str, Any],
) -> dict[str, Any]:
    raw = raw_diagnostics.get("screening_readiness")
    if raw is not None:
        return _normalize_readiness(raw, default_reason="screening readiness supplied")
    usable = bool(search_results)
    return {
        "usable": usable,
        "reason": "search results available" if usable else "no search results available",
    }


def _build_snowball_readiness(
    search_results: list[dict[str, Any]],
    raw_diagnostics: dict[str, Any],
) -> dict[str, Any]:
    raw = raw_diagnostics.get("snowball_readiness")
    if raw is not None:
        return _normalize_readiness(raw, default_reason="snowball readiness supplied")
    usable = any(
        _normalize_doi(_result_doi(result))
        or _normalize_identifier(_result_paper_id(result))
        for result in search_results
    )
    return {
        "usable": usable,
        "reason": "seed identifiers available" if usable else "no DOI or paper id seeds available",
    }


def _recommended_actions(
    blocking_reasons: list[str],
    warnings: list[str],
    concept_coverage: dict[str, Any],
    known_item_recall: dict[str, Any],
) -> list[str]:
    missing_concepts = ", ".join(concept_coverage.get("missing_required_concepts", []))
    missing_known = ", ".join(
        item.get("title") or item.get("doi") or item.get("paper_id") or "unknown"
        for item in known_item_recall.get("missing_items", [])
    )
    actions_by_reason = {
        "invalid_query_plan": "Fix concept blocks, provider translations, and known item identifiers.",
        "all_providers_failed": "Retry failed providers or switch providers before using these results.",
        "zero_hits": "Broaden search terms or relax filters and rerun the search.",
        "less_than_two_successful_providers": "Add at least one more successful provider search.",
        "missing_required_concepts": f"Add successful queries for required concepts: {missing_concepts}.",
        "missing_known_items": f"Run targeted known-item probes for: {missing_known}.",
        "screening_not_ready": "Prepare stable search result records before B1 screening.",
        "snowball_not_ready": "Run citation snowballing or provide seed identifiers before B1.",
        "single_successful_provider": "Add a second provider to improve search robustness.",
        "partial_provider_failure": "Review failed provider attempts and retry where needed.",
        "missing_required_concept_coverage": f"Consider adding query coverage for: {missing_concepts}.",
    }
    actions: list[str] = []
    for reason in blocking_reasons + warnings:
        action = actions_by_reason.get(reason)
        if action:
            _append_unique(actions, action)
    return actions


def _observed_successful_query_ids(
    search_log: list[dict[str, Any]],
    search_results: list[dict[str, Any]],
) -> set[str]:
    query_ids = {
        query_id
        for result in search_results
        for query_id in _query_ids_from_result(result)
    }
    for entry in search_log:
        query_id = _clean_text(entry.get("query_id"))
        if query_id and _entry_successful(entry) and _entry_count(entry) > 0:
            query_ids.add(query_id)
    return query_ids


def _planned_query_ids(query_plan: dict[str, Any]) -> list[str]:
    query_ids: list[str] = []
    for translation in query_plan.get("provider_translations", []):
        if not isinstance(translation, dict):
            continue
        query_id = _clean_text(translation.get("query_id"))
        if query_id:
            _append_unique(query_ids, query_id)
    return query_ids


def _all_providers_failed(provider_coverage: dict[str, Any], raw_diagnostics: dict[str, Any]) -> bool:
    if raw_diagnostics.get("all_providers_failed") is True and provider_coverage["success_count"] == 0:
        return True
    attempted = provider_coverage.get("attempted_providers", [])
    failed = provider_coverage.get("failed_providers", [])
    return bool(attempted) and provider_coverage["success_count"] == 0 and len(failed) == len(attempted)


def _zero_hits(search_results: list[dict[str, Any]], raw_diagnostics: dict[str, Any]) -> bool:
    normalized_count = _to_int(raw_diagnostics.get("normalized_result_count"))
    if normalized_count is not None:
        return normalized_count == 0 and not search_results
    if raw_diagnostics.get("zero_hit") is True and not search_results:
        return True
    return not search_results


def _entry_successful(entry: dict[str, Any]) -> bool:
    status = _clean_text(entry.get("status")).casefold()
    return not status or status in SUCCESS_STATUSES


def _entry_failed(entry: dict[str, Any]) -> bool:
    status = _clean_text(entry.get("status")).casefold()
    return status in FAILURE_STATUSES or bool(_clean_text(entry.get("error")))


def _entry_count(entry: dict[str, Any]) -> int:
    for field in COUNT_FIELDS:
        value = _to_int(entry.get(field))
        if value is not None:
            return value
    return 0


def _summary_hit_count(summary: dict[str, Any]) -> int:
    for field in ("normalized_hits", "raw_hits", "unique_hits", "hit_count", "result_count"):
        value = _to_int(summary.get(field))
        if value is not None:
            return value
    return 0


def _summary_failed(summary: dict[str, Any]) -> bool:
    status = _clean_text(summary.get("status")).casefold()
    failures = summary.get("failures")
    return status in FAILURE_STATUSES or bool(failures)


def _providers_from_result(result: dict[str, Any]) -> list[str]:
    raw = result.get("source") or result.get("provider") or result.get("providers")
    values: list[Any]
    if isinstance(raw, list):
        values = raw
    else:
        values = re.split(r"[;,]", _clean_text(raw)) if _clean_text(raw) else []
    providers: list[str] = []
    for value in values:
        provider = _normalize_provider(value)
        if provider:
            _append_unique(providers, provider)
    return providers


def _query_ids_from_result(result: dict[str, Any]) -> list[str]:
    values: list[Any] = []
    for field in ("query_id", "query_ids"):
        raw = result.get(field)
        if isinstance(raw, list):
            values.extend(raw)
        elif _clean_text(raw):
            values.extend(re.split(r"[;,]", _clean_text(raw)))
    query_ids: list[str] = []
    for value in values:
        query_id = _clean_text(value)
        if query_id:
            _append_unique(query_ids, query_id)
    return query_ids


def _overlap_key(result: dict[str, Any]) -> str:
    doi = _normalize_doi(_result_doi(result))
    if doi:
        return f"doi:{doi}"
    title = _normalize_title(result.get("title"))
    if title:
        return f"title:{title}"
    return ""


def _normalize_readiness(raw: Any, *, default_reason: str) -> dict[str, Any]:
    if isinstance(raw, dict):
        usable_raw = raw.get("usable")
        if usable_raw is None:
            usable_raw = raw.get("ready")
        return {
            **raw,
            "usable": _to_bool(usable_raw),
            "reason": _clean_text(raw.get("reason")) or default_reason,
        }
    return {
        "usable": _to_bool(raw),
        "reason": default_reason,
    }


def _result_doi(result: dict[str, Any]) -> Any:
    doi = result.get("doi") or result.get("DOI")
    if doi:
        return doi
    external_ids = result.get("externalIds") or result.get("external_ids")
    if isinstance(external_ids, dict):
        return external_ids.get("DOI") or external_ids.get("doi")
    return ""


def _result_paper_id(result: dict[str, Any]) -> Any:
    return result.get("paper_id") or result.get("paperId")


def _provider_summaries(raw: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(raw, dict):
        return {}
    summaries: dict[str, dict[str, Any]] = {}
    for key, value in raw.items():
        if isinstance(value, dict):
            summaries[str(key)] = value
    return summaries


def _normalize_search_mode(raw: Any) -> str:
    value = _clean_text(raw).casefold().replace("-", "_")
    if value in {"review_grade", "systematic_review"}:
        return value
    return "targeted_search"


def _normalize_provider(raw: Any) -> str:
    return _clean_text(raw).casefold()


def _normalize_doi(raw: Any) -> str:
    value = _clean_text(raw)
    if not value:
        return ""
    value = DOI_PREFIX_RE.sub("", value)
    return value.rstrip(".,);]").casefold()


def _normalize_identifier(raw: Any) -> str:
    return _clean_text(raw).casefold()


def _normalize_title(raw: Any) -> str:
    value = _clean_text(raw).casefold()
    return NON_ALNUM_RE.sub("", value)


def _gate_status(blocking_reasons: list[str], warnings: list[str]) -> str:
    if blocking_reasons:
        return "fail"
    if warnings:
        return "warning"
    return "pass"


def _as_dict(raw: Any) -> dict[str, Any]:
    return raw if isinstance(raw, dict) else {}


def _list_of_dicts(raw: Any) -> list[dict[str, Any]]:
    if not isinstance(raw, list):
        return []
    return [item for item in raw if isinstance(item, dict)]


def _to_int(raw: Any) -> int | None:
    if isinstance(raw, bool):
        return None
    try:
        return int(raw)
    except (TypeError, ValueError):
        return None


def _to_bool(raw: Any) -> bool:
    if isinstance(raw, bool):
        return raw
    value = _clean_text(raw).casefold()
    if value in {"", "0", "false", "no", "not_ready", "unusable"}:
        return False
    return True


def _iter_values(raw: Any) -> list[Any]:
    if raw is None:
        return []
    if isinstance(raw, (list, tuple, set)):
        return list(raw)
    return [raw]


def _clean_text(value: Any) -> str:
    return " ".join(str(value or "").strip().split())


def _append_unique(items: list[str], value: str) -> None:
    if value and value not in items:
        items.append(value)


def _dedupe_strings(items: list[str]) -> list[str]:
    deduped: list[str] = []
    for item in items:
        _append_unique(deduped, item)
    return deduped
