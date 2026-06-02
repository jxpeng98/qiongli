#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

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
        gate_report = _as_mapping(report_gates.get(gate_id))
        if not gate_report:
            errors.append(f"{gate_id} missing from report gates")
            continue

        report_fields = [str(field) for field in _as_list(_as_mapping(gate_contract).get("report_fields"))]
        for field_name in report_fields:
            if field_name not in gate_report:
                errors.append(f"{gate_id} missing report field: {field_name}")

        status = str(gate_report.get("status", "")).strip()
        if status not in status_values:
            errors.append(f"{gate_id} status {status or '<missing>'} not in contract status_values")

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
