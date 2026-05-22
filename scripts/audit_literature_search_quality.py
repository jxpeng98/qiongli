#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


REQUIRED_DIAGNOSTIC_SECTIONS = (
    "Search Scope",
    "Known-Item Recall",
    "Provider Coverage",
    "Query Coverage",
    "Deduplication Summary",
    "Coverage Gaps",
    "Next Search Actions",
)
REVIEW_GRADE_MODES = {"systematic_review", "systematic-review", "review_grade", "review-grade"}
TARGETED_MODES = {"targeted_search", "targeted-search", "targeted", "scoping", "exploratory"}


@dataclass
class LiteratureSearchQualityResult:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    passed: int = 0

    def check(self, condition: bool, pass_msg: str, fail_msg: str) -> None:
        if condition:
            self.passed += 1
            return
        self.errors.append(fail_msg)

    def warn(self, condition: bool, pass_msg: str, warn_msg: str) -> None:
        if condition:
            self.passed += 1
            return
        self.warnings.append(warn_msg)


def audit_literature_search_quality(
    path: Path | str,
    *,
    task_id: str = "B1",
    mode: str | None = None,
    strict: bool = False,
) -> LiteratureSearchQualityResult:
    result = LiteratureSearchQualityResult()
    task = task_id.strip().upper()
    target = Path(path)
    diagnostics_path = target if target.is_file() else target / "search_diagnostics.md"

    if not diagnostics_path.exists():
        message = f"{task} missing search_diagnostics.md: {diagnostics_path}"
        if task == "B1":
            result.errors.append(message)
        else:
            result.warnings.append(message)
        return result

    try:
        content = diagnostics_path.read_text(encoding="utf-8")
    except OSError as exc:
        result.errors.append(f"Failed to read search_diagnostics.md: {exc}")
        return result

    headings = _extract_markdown_headings(content)
    for section in REQUIRED_DIAGNOSTIC_SECTIONS:
        result.check(
            section.casefold() in headings,
            f"search_diagnostics.md includes {section}",
            f"search_diagnostics.md missing required section: {section}",
        )

    metadata = _extract_metadata(content)
    resolved_mode = _normalize_mode(
        mode or str(metadata.get("mode") or metadata.get("search_mode") or "")
    )
    if not resolved_mode:
        resolved_mode = "systematic_review" if task == "B1" else "targeted_search"

    review_grade = _as_bool(metadata.get("review_grade")) or resolved_mode in REVIEW_GRADE_MODES
    targeted = resolved_mode in TARGETED_MODES

    gate_status = str(metadata.get("gate_status", "")).strip().casefold()
    if gate_status == "fail":
        reasons = metadata.get("blocking_reasons") or []
        reason_text = ", ".join(str(item) for item in reasons) if isinstance(reasons, list) else str(reasons)
        result.errors.append(f"search_diagnostics.md gate_status failed: {reason_text}".rstrip())

    if str(metadata.get("status", "")).strip().casefold() == "error" or _has_flag(
        metadata,
        "all_providers_failed",
    ):
        result.errors.append("search_diagnostics.md reports all providers failed")

    provider_count = _positive_provider_count(metadata.get("provider_coverage"))
    if review_grade:
        result.check(
            provider_count >= 2,
            "review-grade provider coverage uses at least two providers",
            (
                f"{resolved_mode} requires coverage from at least two providers; "
                f"found {provider_count}"
            ),
        )
    elif targeted:
        result.warn(
            provider_count >= 2,
            "targeted search used multiple providers",
            "targeted_search used a single provider; document the scope limit before review-grade claims",
        )

    zero_hit_queries = _zero_hit_queries(metadata.get("query_coverage") or metadata.get("query_health"))
    if zero_hit_queries:
        message = "required concept queries returned zero hits: " + ", ".join(zero_hit_queries)
        if review_grade:
            result.errors.append(message)
        else:
            result.warnings.append(message)
    else:
        result.passed += 1

    missing_known_items = _missing_known_items(metadata)
    if missing_known_items:
        message = "known_item_missing: " + ", ".join(missing_known_items)
        if review_grade:
            result.errors.append(message)
        else:
            result.warnings.append(message)
    else:
        result.passed += 1

    if _has_flag(metadata, "provider_undercoverage") and not review_grade:
        result.warnings.append("provider_undercoverage flagged for targeted_search")
    weak_screening = _has_flag(metadata, "weak_screening_readiness") or _readiness_unusable(
        metadata.get("screening_readiness")
    )
    if weak_screening:
        message = "weak_screening_readiness flagged in search_diagnostics.md"
        if review_grade:
            result.errors.append(message)
        else:
            result.warnings.append(message)

    if strict and result.warnings:
        result.errors.extend(f"[strict-warning] {warning}" for warning in result.warnings)

    return result


def _extract_markdown_headings(content: str) -> set[str]:
    return {
        match.group(1).strip().casefold()
        for match in re.finditer(r"^##\s+(.+?)\s*$", content, flags=re.MULTILINE)
    }


def _extract_metadata(content: str) -> dict[str, Any]:
    fenced = re.search(r"```ya?ml\s*\n(.*?)\n```", content, flags=re.DOTALL | re.IGNORECASE)
    if fenced:
        loaded = yaml.safe_load(fenced.group(1)) or {}
        return loaded if isinstance(loaded, dict) else {}

    json_block = re.search(r"```json\s*\n(.*?)\n```", content, flags=re.DOTALL | re.IGNORECASE)
    if not json_block:
        return {}
    try:
        loaded = json.loads(json_block.group(1)) or {}
    except json.JSONDecodeError:
        return {}
    return loaded if isinstance(loaded, dict) else {}


def _normalize_mode(value: str) -> str:
    return value.strip().casefold().replace(" ", "_")


def _as_bool(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    return str(value).strip().casefold() in {"1", "true", "yes", "y", "review_grade"}


def _positive_provider_count(value: Any) -> int:
    if isinstance(value, dict):
        success_count = value.get("success_count")
        try:
            if success_count is not None:
                return int(success_count)
        except (TypeError, ValueError):
            pass
        hit_counts = value.get("hit_counts")
        if isinstance(hit_counts, dict):
            return _positive_provider_count(hit_counts)
        successful = value.get("successful_providers")
        if isinstance(successful, list):
            return len([item for item in successful if str(item).strip()])
        count = 0
        for provider, hits in value.items():
            if not str(provider).strip():
                continue
            if isinstance(hits, (dict, list)):
                continue
            try:
                if int(hits) > 0:
                    count += 1
            except (TypeError, ValueError):
                if str(hits).strip() and str(hits).strip() not in {"0", "false", "False"}:
                    count += 1
        return count
    if isinstance(value, list):
        return len([item for item in value if str(item).strip()])
    return 0


def _zero_hit_queries(value: Any) -> list[str]:
    if not isinstance(value, dict):
        return []
    zero_hit_ids = value.get("zero_hit_query_ids")
    if isinstance(zero_hit_ids, list):
        return [str(item).strip() for item in zero_hit_ids if str(item).strip()]
    zero_hit: list[str] = []
    for query_id, hits in value.items():
        if isinstance(hits, (dict, list)):
            continue
        try:
            count = int(hits)
        except (TypeError, ValueError):
            continue
        if count <= 0:
            zero_hit.append(str(query_id))
    return zero_hit


def _missing_known_items(metadata: dict[str, Any]) -> list[str]:
    recall = metadata.get("known_item_recall")
    if isinstance(recall, dict):
        missing = recall.get("missing") or recall.get("missing_items") or []
        if isinstance(missing, list):
            return [_known_item_label(item) for item in missing if _known_item_label(item)]
    direct = metadata.get("known_item_missing") or metadata.get("missing_known_items") or []
    if isinstance(direct, list):
        return [str(item).strip() for item in direct if str(item).strip()]
    if isinstance(direct, str) and direct.strip():
        return [direct.strip()]
    return []


def _known_item_label(item: Any) -> str:
    if isinstance(item, dict):
        for key in ("title", "doi", "paper_id"):
            value = str(item.get(key, "")).strip()
            if value:
                return value
        return ""
    return str(item).strip()


def _has_flag(metadata: dict[str, Any], expected: str) -> bool:
    flags = metadata.get("flags") or metadata.get("diagnostic_flags") or []
    if isinstance(flags, str):
        flags = [flags]
    if isinstance(flags, list):
        return expected in {str(flag).strip().casefold() for flag in flags}
    return bool(metadata.get(expected))


def _readiness_unusable(value: Any) -> bool:
    if not isinstance(value, dict):
        return False
    usable = value.get("usable")
    if usable is None:
        usable = value.get("ready")
    if isinstance(usable, bool):
        return not usable
    text = str(usable or "").strip().casefold()
    return text in {"0", "false", "no", "not_ready", "unusable"}


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit literature search_diagnostics.md with mode-aware quality gates."
    )
    parser.add_argument("path", type=Path, help="Project root or search_diagnostics.md file")
    parser.add_argument("--task-id", default="B1", help="Workflow task id, default: B1")
    parser.add_argument("--mode", help="Override diagnostics mode")
    parser.add_argument("--strict", action="store_true", help="Treat warnings as blocking")
    parser.add_argument("--json", action="store_true", dest="json_output", help="Emit JSON summary")
    args = parser.parse_args()

    result = audit_literature_search_quality(
        args.path,
        task_id=args.task_id,
        mode=args.mode,
        strict=args.strict,
    )
    for error in result.errors:
        print(f"[FAIL] {error}")
    for warning in result.warnings:
        print(f"[WARN] {warning}")
    if not result.errors:
        print("[PASS] Literature search diagnostics are valid")
    if args.json_output:
        print(
            json.dumps(
                {
                    "errors": result.errors,
                    "warnings": result.warnings,
                    "passed": result.passed,
                    "verdict": "BLOCK" if result.errors else "PASS",
                },
                indent=2,
            )
        )
    return 1 if result.errors else 0


if __name__ == "__main__":
    sys.exit(main())
