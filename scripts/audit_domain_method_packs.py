#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
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
