#!/usr/bin/env python3
from __future__ import annotations

import argparse
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

REQUIRED_METHOD_FIELDS = {
    "assumptions",
    "required_diagnostics",
    "required_artifacts",
    "failure_modes",
    "minimum_report_fields",
}
ENHANCED_METHOD_FIELDS = {
    "canonical_references",
    "gate_relevance",
    "diagnostic_artifacts",
    "failure_triggers",
}
QUALITY_GATES = {"Q1", "Q2", "Q3", "Q4"}
DEFAULT_PROFILE_NAMES = ("economics", "finance")


@dataclass
class DomainMethodPackAuditResult:
    errors: list[str] = field(default_factory=list)


def audit_domain_profile(path: Path) -> DomainMethodPackAuditResult:
    errors: list[str] = []
    profile = _load_profile(path, errors)
    if profile is None:
        return DomainMethodPackAuditResult(errors=errors)

    methods = profile.get("method_templates")
    if not isinstance(methods, list) or not methods:
        errors.append(f"{path}: method_templates must be a non-empty list")
        return DomainMethodPackAuditResult(errors=errors)

    for index, method in enumerate(methods, start=1):
        if not isinstance(method, dict):
            errors.append(f"{path}: method_templates[{index}] must be an object")
            continue
        method_name = str(method.get("name") or f"method_templates[{index}]").strip()
        for field_name in sorted(REQUIRED_METHOD_FIELDS):
            if not _is_non_empty_string_list(method.get(field_name)):
                errors.append(
                    f"{path}: {method_name} missing or empty required method field: "
                    f"{field_name}"
                )
        for field_name in sorted(ENHANCED_METHOD_FIELDS):
            if field_name in {"canonical_references", "diagnostic_artifacts"}:
                if not _is_non_empty_object_list(method.get(field_name)):
                    errors.append(
                        f"{path}: {method_name} missing or empty required method field: "
                        f"{field_name}"
                    )
            elif not _is_non_empty_string_list(method.get(field_name)):
                errors.append(
                    f"{path}: {method_name} missing or empty required method field: "
                    f"{field_name}"
                )
        _validate_gate_relevance(path, method_name, method.get("gate_relevance"), errors)
        _validate_canonical_references(path, method_name, method.get("canonical_references"), errors)
        _validate_diagnostic_artifacts(path, method_name, method.get("diagnostic_artifacts"), errors)

    return DomainMethodPackAuditResult(errors=errors)


def _load_profile(path: Path, errors: list[str]) -> dict[str, Any] | None:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        errors.append(f"Failed to read profile {path}: {exc}")
        return None

    try:
        payload = yaml.safe_load(content)
    except yaml.YAMLError as exc:
        errors.append(f"Malformed YAML in profile {path}: {exc}")
        return None

    if not isinstance(payload, dict):
        errors.append(f"{path}: profile must be a YAML object")
        return None
    return payload


def _is_non_empty_string_list(value: Any) -> bool:
    return isinstance(value, list) and any(isinstance(item, str) and item.strip() for item in value)


def _is_non_empty_object_list(value: Any) -> bool:
    return isinstance(value, list) and any(isinstance(item, dict) and item for item in value)


def _validate_gate_relevance(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for gate in value:
        if gate not in QUALITY_GATES:
            errors.append(f"{path}: {method_name} gate_relevance contains unsupported gate: {gate}")


def _validate_canonical_references(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for index, reference in enumerate(value, start=1):
        if not isinstance(reference, dict):
            errors.append(f"{path}: {method_name} canonical_references[{index}] must be an object")
            continue
        if not isinstance(reference.get("citation_key"), str) or not reference["citation_key"].strip():
            errors.append(f"{path}: {method_name} canonical_references[{index}] missing citation_key")
        if not isinstance(reference.get("role"), str) or not reference["role"].strip():
            errors.append(f"{path}: {method_name} canonical_references[{index}] missing role")


def _validate_diagnostic_artifacts(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for index, artifact in enumerate(value, start=1):
        if not isinstance(artifact, dict):
            errors.append(f"{path}: {method_name} diagnostic_artifacts[{index}] must be an object")
            continue
        artifact_path = artifact.get("artifact")
        if not isinstance(artifact_path, str) or "RESEARCH/[topic]/" not in artifact_path:
            errors.append(
                f"{path}: {method_name} diagnostic_artifacts[{index}] must name a RESEARCH/[topic]/ artifact"
            )
        if not isinstance(artifact.get("required_for"), str) or not artifact["required_for"].strip():
            errors.append(f"{path}: {method_name} diagnostic_artifacts[{index}] missing required_for")


def _default_profile_paths() -> list[Path]:
    skills_root = RepoLayout(REPO_ROOT).skills
    return [
        skills_root / "domain-profiles" / f"{name}.yaml"
        for name in DEFAULT_PROFILE_NAMES
    ]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit executable economics and finance method-pack fields."
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Return nonzero when audit errors exist.",
    )
    parser.add_argument(
        "profiles",
        nargs="*",
        type=Path,
        help="Domain profile paths. Defaults to economics and finance.",
    )
    args = parser.parse_args()

    profile_paths = args.profiles or _default_profile_paths()
    all_errors: list[str] = []
    for profile_path in profile_paths:
        result = audit_domain_profile(profile_path)
        all_errors.extend(result.errors)

    for error in all_errors:
        print(f"[FAIL] {error}")
    if not all_errors:
        print("[PASS] Economics and finance method packs satisfy required fields")
    return 1 if args.strict and all_errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
