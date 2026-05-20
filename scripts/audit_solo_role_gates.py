#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Callable


EXECUTION_MODES = {"solo", "solo_codex", "solo_claude", "solo_gemini", "duo", "triad"}
RUNTIME_AGENTS = {"codex", "claude", "gemini"}
WRITING_TASK_TYPES = {"writing", "drafting", "manuscript", "paper_write"}
CODE_TASK_TYPES = {"code", "coding", "implementation", "software"}


def audit_solo_role_gates(root: Path) -> list[str]:
    """Audit controller-mode JSON run and review artifacts under a project root."""
    root = root.resolve()
    if not root.exists():
        return [f"missing audit root: {root}"]

    errors: list[str] = []
    runs: dict[str, dict[str, Any]] = {}
    reviews: list[dict[str, Any]] = []

    for json_path in sorted(root.rglob("*.json")):
        payload = _read_json_object(json_path)
        if payload is None:
            continue
        if _is_run_packet(payload):
            run_id = _run_id(payload, json_path)
            runs[run_id] = payload
            errors.extend(_audit_run_packet(root, payload, run_id))
        elif _is_review_packet(payload):
            reviews.append(payload)

    errors.extend(_audit_review_blockers(runs, reviews))
    return sorted(errors)


def _read_json_object(path: Path) -> dict[str, Any] | None:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if isinstance(payload, dict):
        return payload
    return None


def _is_run_packet(payload: dict[str, Any]) -> bool:
    return bool(str(payload.get("run_id") or "").strip()) and (
        _normalized(payload.get("execution_mode")) in EXECUTION_MODES
    )


def _is_review_packet(payload: dict[str, Any]) -> bool:
    return bool(payload.get("reviewed_run_id")) and (
        "review_status" in payload or "blocking_issues" in payload
    )


def _audit_run_packet(
    root: Path,
    packet: dict[str, Any],
    run_id: str,
) -> list[str]:
    errors: list[str] = []
    execution_mode = _effective_execution_mode(packet)
    role_gate_level = _normalized(packet.get("solo_role_gates")) or "standard"
    verification_status = packet.get("verification_status")
    artifact_paths, artifact_errors = _artifact_paths(root, packet)
    errors.extend(f"{run_id}: {error}" for error in artifact_errors)

    if not _normalized(verification_status):
        errors.append(f"{run_id}: missing verification_status")

    if role_gate_level == "off":
        return errors

    if execution_mode == "solo_codex" and _is_writing_packet(packet, artifact_paths):
        if not _has_existing_artifact(artifact_paths, _is_claim_map_path):
            errors.append(f"{run_id}: solo Codex writing missing claim map artifact")

    if execution_mode == "solo_claude" and _is_code_packet(packet, artifact_paths):
        if not _has_existing_artifact(artifact_paths, _is_implementation_intent_path):
            errors.append(
                f"{run_id}: solo Claude code missing implementation intent artifact"
            )

    if execution_mode == "duo":
        if not _has_existing_artifact(artifact_paths, _is_handoff_path):
            errors.append(f"{run_id}: duo run missing handoff artifact")

    return errors


def _audit_review_blockers(
    runs: dict[str, dict[str, Any]],
    reviews: list[dict[str, Any]],
) -> list[str]:
    errors: list[str] = []
    for review in reviews:
        reviewed_run_id = str(review.get("reviewed_run_id") or "").strip()
        if not reviewed_run_id:
            continue
        run = runs.get(reviewed_run_id)
        if not run or _normalized(run.get("verification_status")) != "passed":
            continue
        if _review_has_blocker(review):
            errors.append(
                f"{reviewed_run_id}: reviewer blocker conflicts with final status passed"
            )
    return errors


def _effective_execution_mode(packet: dict[str, Any]) -> str:
    execution_mode = _normalized(packet.get("execution_mode"))
    if execution_mode != "solo":
        return execution_mode
    solo_agent = _normalized(packet.get("primary_agent")) or _normalized(packet.get("controller"))
    if solo_agent in RUNTIME_AGENTS:
        return f"solo_{solo_agent}"
    return execution_mode


def _review_has_blocker(review: dict[str, Any]) -> bool:
    review_status = _normalized(review.get("review_status"))
    if review_status in {"blocked", "failed", "fail", "changes_requested"}:
        return True
    return bool(_meaningful_items(review.get("blocking_issues")))


def _is_writing_packet(packet: dict[str, Any], artifact_paths: list[Path]) -> bool:
    task_type = _normalized(packet.get("task_type") or packet.get("task_domain"))
    if task_type in WRITING_TASK_TYPES:
        return True
    task_id = str(packet.get("task_id") or "").strip().upper()
    if task_id.startswith(("F", "J")):
        return True
    return any(
        _path_has_token(path, ("manuscript", "draft", "writing"))
        for path in artifact_paths
    )


def _is_code_packet(packet: dict[str, Any], artifact_paths: list[Path]) -> bool:
    task_type = _normalized(packet.get("task_type") or packet.get("task_domain"))
    if task_type in CODE_TASK_TYPES:
        return True
    task_id = str(packet.get("task_id") or "").strip().upper()
    if task_id.startswith("I"):
        return True
    return any(
        _path_has_token(path, ("code", "patch", "implementation"))
        for path in artifact_paths
    )


def _artifact_paths(root: Path, packet: dict[str, Any]) -> tuple[list[Path], list[str]]:
    paths: list[Path] = []
    errors: list[str] = []
    resolved_root = root.resolve()
    for key in ("artifacts_written", "artifacts"):
        value = packet.get(key, [])
        if not isinstance(value, list):
            continue
        for item in value:
            path_value = ""
            if isinstance(item, str):
                path_value = item
            elif isinstance(item, dict) and isinstance(item.get("path"), str):
                path_value = item["path"]
            if path_value:
                raw_path = Path(path_value)
                candidate = raw_path if raw_path.is_absolute() else root / raw_path
                resolved_candidate = candidate.resolve(strict=False)
                if not resolved_candidate.is_relative_to(resolved_root):
                    errors.append(f"artifact path outside audit root: {path_value}")
                    continue
                paths.append(resolved_candidate)
    return paths, errors


def _has_existing_artifact(
    paths: list[Path],
    predicate: Callable[[Path], bool],
) -> bool:
    return any(predicate(path) and path.exists() for path in paths)


def _is_claim_map_path(path: Path) -> bool:
    name = path.name.lower()
    return "claim" in name and "map" in name


def _is_implementation_intent_path(path: Path) -> bool:
    name = path.name.lower()
    return "implementation" in name and "intent" in name


def _is_handoff_path(path: Path) -> bool:
    return "handoff" in path.name.lower()


def _path_has_token(path: Path, tokens: tuple[str, ...]) -> bool:
    text = str(path).lower()
    return any(token in text for token in tokens)


def _meaningful_items(value: Any) -> list[Any]:
    if not isinstance(value, list):
        return []
    meaningful = []
    for item in value:
        normalized = _normalized(item)
        if normalized and normalized not in {"none", "none.", "n/a", "na"}:
            meaningful.append(item)
    return meaningful


def _run_id(packet: dict[str, Any], path: Path) -> str:
    return str(packet.get("run_id") or path.stem).strip()


def _normalized(value: Any) -> str:
    if value is None:
        return ""
    return str(value).strip().lower()


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit solo role and controller gates.")
    parser.add_argument("root", type=Path)
    args = parser.parse_args()

    errors = audit_solo_role_gates(args.root)
    for error in errors:
        print(f"[FAIL] {error}")
    if errors:
        return 1
    print("[PASS] Solo role gates are valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
