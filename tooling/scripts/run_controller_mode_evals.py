#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import yaml


REQUIRED_DIMENSIONS = [
    "artifact_completeness",
    "role_gate_compliance",
    "evidence_traceability",
    "command_verification_honesty",
    "handoff_quality",
    "disagreement_resolution",
]

EXECUTION_MODES = {"solo", "duo", "triad"}
RUNTIME_AGENTS = {"codex", "claude"}
VERIFICATION_STATUSES = {"passed", "failed", "blocked", "skipped"}
RESOLUTION_STATUSES = {"resolved", "accepted", "deferred", "escalated"}


def run_evals(case_dir: Path) -> dict[str, Any]:
    """Run deterministic offline controller-mode evals from YAML fixtures."""
    failures: list[str] = []
    totals = {dimension: 0.0 for dimension in REQUIRED_DIMENSIONS}
    cases: list[dict[str, Any]] = []

    case_paths = _case_paths(case_dir)
    for case_path in case_paths:
        payload = _load_case(case_path, failures)
        if payload is None:
            continue

        case_id = _string(payload.get("id")) or case_path.stem
        case_failures = _validate_case(case_id, payload)
        failures.extend(case_failures)

        scores = payload.get("scores")
        if isinstance(scores, dict):
            for dimension in REQUIRED_DIMENSIONS:
                value = scores.get(dimension)
                if _is_score_number(value):
                    totals[dimension] += float(value)

        verification_outcome = _verification_outcome(payload)
        cases.append(
            {
                "id": case_id,
                "execution_mode": _string(payload.get("execution_mode")),
                "controller": _string(payload.get("controller")),
                "primary_agent": _string(payload.get("primary_agent")),
                "reviewer_agent": _string(payload.get("reviewer_agent")),
                "verifier_agent": _string(payload.get("verifier_agent")),
                "verification_outcome": verification_outcome,
                "valid": not case_failures,
            }
        )

    case_count = len(case_paths)
    if case_count == 0:
        failures.append(f"no eval cases found in {case_dir}")

    scores_summary = {
        dimension: (totals[dimension] / case_count if case_count else 0.0)
        for dimension in REQUIRED_DIMENSIONS
    }
    return {
        "status": "failed" if failures else "passed",
        "scores": scores_summary,
        "failures": sorted(failures),
        "case_count": case_count,
        "blocked_verification_count": sum(
            1 for case in cases if case["verification_outcome"] == "expected_blocked"
        ),
        "cases": cases,
    }


def _load_case(path: Path, failures: list[str]) -> dict[str, Any] | None:
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except (OSError, yaml.YAMLError) as exc:
        failures.append(f"{path.stem}: failed to read fixture: {exc}")
        return None
    if not isinstance(payload, dict):
        failures.append(f"{path.stem}: fixture must be a YAML object")
        return None
    return payload


def _case_paths(case_dir: Path) -> list[Path]:
    return sorted(
        [
            *case_dir.glob("*.json"),
            *case_dir.glob("*.yaml"),
            *case_dir.glob("*.yml"),
        ]
    )


def _validate_case(case_id: str, payload: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    _validate_scores(case_id, payload, failures)
    _validate_controller_metadata(case_id, payload, failures)
    _validate_artifacts(case_id, payload, failures)
    _validate_verification(case_id, payload, failures)
    _validate_disagreement(case_id, payload, failures)
    return failures


def _validate_scores(
    case_id: str,
    payload: dict[str, Any],
    failures: list[str],
) -> None:
    scores = payload.get("scores")
    if not isinstance(scores, dict):
        failures.append(f"{case_id}: scores must be an object")
        return
    for dimension in REQUIRED_DIMENSIONS:
        value = scores.get(dimension)
        if not _is_score_number(value):
            failures.append(f"{case_id}: missing numeric score dimension {dimension}")
        elif not 0.0 <= float(value) <= 1.0:
            failures.append(f"{case_id}: score {dimension} must be between 0 and 1")


def _validate_controller_metadata(
    case_id: str,
    payload: dict[str, Any],
    failures: list[str],
) -> None:
    execution_mode = _string(payload.get("execution_mode"))
    if execution_mode not in EXECUTION_MODES:
        failures.append(f"{case_id}: invalid execution_mode {execution_mode!r}")

    for key in ("controller", "primary_agent", "reviewer_agent", "verifier_agent"):
        value = _string(payload.get(key))
        if value not in RUNTIME_AGENTS:
            failures.append(f"{case_id}: invalid {key} {value!r}")

    if execution_mode == "solo":
        controller = _string(payload.get("controller"))
        primary = _string(payload.get("primary_agent"))
        if controller and primary and controller != primary:
            failures.append(f"{case_id}: solo controller must match primary_agent")

    if execution_mode == "duo":
        primary = _string(payload.get("primary_agent"))
        reviewer = _string(payload.get("reviewer_agent"))
        if primary and reviewer and primary == reviewer:
            failures.append(f"{case_id}: duo primary_agent and reviewer_agent must differ")


def _validate_artifacts(
    case_id: str,
    payload: dict[str, Any],
    failures: list[str],
) -> None:
    expected = _string_list(payload.get("expected_artifacts"))
    written = set(_string_list(payload.get("artifacts_written")))
    if not expected:
        failures.append(f"{case_id}: expected_artifacts must list required artifacts")
        return
    missing = [artifact for artifact in expected if artifact not in written]
    for artifact in missing:
        failures.append(f"{case_id}: expected artifact not written: {artifact}")


def _validate_verification(
    case_id: str,
    payload: dict[str, Any],
    failures: list[str],
) -> None:
    verification = payload.get("verification")
    if not isinstance(verification, dict):
        failures.append(f"{case_id}: verification must be an object")
        return

    status = _string(verification.get("status"))
    expected_blocked = bool(verification.get("expected_blocked"))
    evidence = verification.get("evidence")
    if status not in VERIFICATION_STATUSES:
        failures.append(f"{case_id}: invalid verification status {status!r}")
        return

    if expected_blocked:
        if status != "blocked":
            failures.append(
                f"{case_id}: expected_blocked verification must use status blocked"
            )
        if not _string(verification.get("blocker_reason")):
            failures.append(f"{case_id}: blocked verification needs blocker_reason")
        return

    if status == "blocked":
        failures.append(f"{case_id}: blocked verification must be marked expected_blocked")
    evidence_errors = _validate_command_evidence(case_id, evidence)
    failures.extend(evidence_errors)
    if status == "passed" and not _has_passed_command_evidence(evidence):
        failures.append(f"{case_id}: passed verification needs passed command evidence")


def _validate_disagreement(
    case_id: str,
    payload: dict[str, Any],
    failures: list[str],
) -> None:
    disagreements = payload.get("disagreements")
    if not isinstance(disagreements, list) or not disagreements:
        return

    resolution = payload.get("resolution")
    if not isinstance(resolution, dict):
        failures.append(f"{case_id}: disagreements require a resolution object")
        return
    status = _string(resolution.get("status"))
    if status not in RESOLUTION_STATUSES:
        failures.append(f"{case_id}: invalid resolution status {status!r}")
    if not _string(resolution.get("decision")):
        failures.append(f"{case_id}: resolution requires a decision")


def _verification_outcome(payload: dict[str, Any]) -> str:
    verification = payload.get("verification")
    if not isinstance(verification, dict):
        return "invalid"
    status = _string(verification.get("status"))
    if status == "blocked" and bool(verification.get("expected_blocked")):
        return "expected_blocked"
    return status or "invalid"


def _is_score_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _validate_command_evidence(case_id: str, value: Any) -> list[str]:
    failures: list[str] = []
    if value is None:
        return failures
    if not isinstance(value, list):
        return [f"{case_id}: verification evidence must be a list"]
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            failures.append(f"{case_id}: verification evidence item {index} must be an object")
            continue
        evidence_status = _string(item.get("status"))
        if evidence_status and evidence_status not in VERIFICATION_STATUSES:
            failures.append(
                f"{case_id}: invalid verification evidence status {evidence_status!r}"
            )
    return failures


def _has_passed_command_evidence(value: Any) -> bool:
    if not isinstance(value, list):
        return False
    for item in value:
        if not isinstance(item, dict):
            continue
        if _string(item.get("command")) and _string(item.get("status")) == "passed":
            return True
    return False


def _string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [_string(item) for item in value if _string(item)]


def _string(value: Any) -> str:
    if value is None:
        return ""
    return str(value).strip().lower()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run offline controller-mode evals.")
    parser.add_argument(
        "case_dir",
        type=Path,
        nargs="?",
        default=Path("evals/controller_modes"),
    )
    args = parser.parse_args()

    summary = run_evals(args.case_dir)
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 1 if summary["status"] == "failed" else 0


if __name__ == "__main__":
    raise SystemExit(main())
