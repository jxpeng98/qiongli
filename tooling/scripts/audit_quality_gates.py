#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout


@dataclass
class GateAuditResult:
    passed: bool
    errors: list[str] = field(default_factory=list)


class QualityGateContractError(Exception):
    """Raised when the quality gate contract cannot be loaded."""


def load_gate_contract(path: Path) -> dict[str, object]:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise QualityGateContractError(f"{path}: {exc}") from exc
    try:
        payload = yaml.safe_load(content) or {}
    except yaml.YAMLError as exc:
        raise QualityGateContractError(f"{path}: {exc}") from exc
    return payload if isinstance(payload, dict) else {}


def audit_gate_report(path: Path, contract: dict[str, object]) -> GateAuditResult:
    errors: list[str] = []
    report = _load_report_yaml(path, errors)
    gates = _as_mapping(contract.get("gates"))
    report_gates = _as_mapping(report.get("gates"))
    status_values = {str(value) for value in _as_list(contract.get("status_values"))}

    if not gates:
        errors.append("Contract missing gates")
    if not status_values:
        errors.append("Contract missing status_values")
    if not report_gates:
        errors.append("Report missing gates")

    for gate_id, gate_contract in gates.items():
        gate_contract = _as_mapping(gate_contract)
        gate_report = _as_mapping(report_gates.get(gate_id))
        if not gate_report:
            errors.append(f"{gate_id} missing from report gates")
            continue

        report_fields = [str(field) for field in _as_list(gate_contract.get("report_fields"))]
        for field_name in report_fields:
            if field_name not in gate_report:
                errors.append(f"{gate_id} missing report field: {field_name}")

        status = str(gate_report.get("status", "")).strip()
        if status not in status_values:
            errors.append(f"{gate_id} status {status or '<missing>'} not in contract status_values")

        _validate_semantic_checks(gate_id, gate_report, gate_contract, status_values, errors)
        _validate_structured_evidence(gate_id, gate_report, errors)
        _validate_structured_blocking_issues(gate_id, gate_report, errors)

        if status in {"PASS", "WARN"} and not _has_non_empty_items(gate_report.get("evidence")):
            errors.append(f"{gate_id} status {status} requires non-empty evidence")

        if status in {"FAIL", "BLOCKED"} and not _has_non_empty_items(gate_report.get("blocking_issues")):
            errors.append(f"{gate_id} status {status} requires non-empty blocking_issues")

    return GateAuditResult(passed=not errors, errors=errors)


def _load_report_yaml(path: Path, errors: list[str]) -> dict[str, Any]:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        errors.append(f"Failed to read quality gate report: {exc}")
        return {}

    match = re.search(r"```yaml\s*\n(.*?)\n```", content, flags=re.DOTALL | re.IGNORECASE)
    if not match:
        errors.append("Report missing fenced yaml block")
        return {}

    try:
        payload = yaml.safe_load(match.group(1)) or {}
    except yaml.YAMLError as exc:
        errors.append(f"Report yaml block is invalid: {exc}")
        return {}
    return payload if isinstance(payload, dict) else {}


def _as_mapping(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def _as_list(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def _has_non_empty_items(value: Any) -> bool:
    if isinstance(value, list):
        return any(str(item).strip() for item in value)
    return bool(str(value or "").strip())


def _validate_semantic_checks(
    gate_id: str,
    gate_report: dict[str, Any],
    gate_contract: dict[str, Any],
    status_values: set[str],
    errors: list[str],
) -> None:
    semantic_checks = gate_report.get("semantic_checks")
    if not isinstance(semantic_checks, list) or not semantic_checks:
        if "semantic_checks" in gate_report:
            errors.append(f"{gate_id} semantic_checks must be a non-empty list")
        return

    expected_ids = {
        str(item.get("check_id")).strip()
        for item in _as_list(gate_contract.get("semantic_checks"))
        if isinstance(item, dict) and str(item.get("check_id", "")).strip()
    }
    found_ids: set[str] = set()
    required_fields = ("check_id", "status", "finding", "evidence_refs")

    for index, check in enumerate(semantic_checks, start=1):
        if not isinstance(check, dict):
            errors.append(f"{gate_id} semantic_checks[{index}] must be an object")
            continue

        for field_name in required_fields:
            if field_name not in check:
                errors.append(
                    f"{gate_id} semantic_checks[{index}] missing required field: {field_name}"
                )

        check_id = str(check.get("check_id", "")).strip()
        if check_id:
            found_ids.add(check_id)

        status = str(check.get("status", "")).strip()
        if status not in status_values:
            errors.append(
                f"{gate_id} semantic_checks[{index}] status {status or '<missing>'} "
                "not in contract status_values"
            )

        finding = str(check.get("finding", "")).strip()
        if not finding:
            errors.append(f"{gate_id} semantic_checks[{index}] finding is empty")

        evidence_refs = check.get("evidence_refs")
        if not isinstance(evidence_refs, list):
            errors.append(f"{gate_id} semantic_checks[{index}] evidence_refs must be a list")

    for check_id in sorted(expected_ids - found_ids):
        errors.append(f"{gate_id} missing semantic check id: {check_id}")


def _validate_structured_evidence(
    gate_id: str,
    gate_report: dict[str, Any],
    errors: list[str],
) -> None:
    evidence = gate_report.get("evidence")
    if "evidence" in gate_report and not isinstance(evidence, list):
        errors.append(f"{gate_id} evidence must be a list")
        return

    for index, item in enumerate(_as_list(evidence), start=1):
        if isinstance(item, str):
            continue
        if not isinstance(item, dict):
            errors.append(f"{gate_id} evidence[{index}] must be a string or object")
            continue
        for field_name in ("artifact", "anchor", "supports"):
            if not str(item.get(field_name, "")).strip():
                errors.append(f"{gate_id} evidence[{index}] missing field: {field_name}")


def _validate_structured_blocking_issues(
    gate_id: str,
    gate_report: dict[str, Any],
    errors: list[str],
) -> None:
    blocking_issues = gate_report.get("blocking_issues")
    if "blocking_issues" in gate_report and not isinstance(blocking_issues, list):
        errors.append(f"{gate_id} blocking_issues must be a list")
        return

    for index, item in enumerate(_as_list(blocking_issues), start=1):
        if isinstance(item, str):
            continue
        if not isinstance(item, dict):
            errors.append(f"{gate_id} blocking_issues[{index}] must be a string or object")
            continue
        for field_name in ("issue", "required_action"):
            if not str(item.get(field_name, "")).strip():
                errors.append(f"{gate_id} blocking_issues[{index}] missing field: {field_name}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit quality gate report contract compliance.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Return nonzero when audit errors exist.",
    )
    parser.add_argument(
        "--contract",
        type=Path,
        default=RepoLayout(REPO_ROOT).standards / "quality-gate-contract.yaml",
        help="Quality gate contract path.",
    )
    parser.add_argument(
        "--report",
        type=Path,
        default=RepoLayout(REPO_ROOT).templates / "quality-gate-report.md",
        help="Quality gate report path.",
    )
    args = parser.parse_args()

    try:
        contract = load_gate_contract(args.contract)
    except QualityGateContractError as exc:
        print(f"[FAIL] Failed to load quality gate contract: {exc}")
        return 1

    result = audit_gate_report(args.report, contract)
    for error in result.errors:
        print(f"[FAIL] {error}")
    if result.passed:
        print("[PASS] Quality gate report satisfies contract")
    return 1 if args.strict and result.errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
