#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from pathlib import Path

import yaml


REQUIRED_FIELDS = [
    "venue_id",
    "community",
    "article_types",
    "contribution_expectations",
    "methods_expectations",
    "evidence_standards",
    "writing_style",
    "common_reviewer_objections",
    "formatting_constraints",
    "required_reporting_standards",
]


@dataclass
class VenueProfileAuditResult:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


def audit_venue_profile(path: Path) -> VenueProfileAuditResult:
    result = VenueProfileAuditResult()
    if not path.exists():
        result.errors.append(f"missing venue profile: {path}")
        return result
    payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    if not isinstance(payload, dict):
        result.errors.append("venue profile must be a YAML object")
        return result
    for field in REQUIRED_FIELDS:
        value = payload.get(field)
        if value in (None, "", []):
            result.errors.append(f"missing required field: {field}")
    venue_id = str(payload.get("venue_id", "")).strip()
    if venue_id and path.stem != venue_id:
        result.errors.append(f"venue_id must match filename stem: {path.stem}")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit qiongli venue profile YAML files.")
    parser.add_argument("profile", type=Path)
    args = parser.parse_args()

    result = audit_venue_profile(args.profile)
    for error in result.errors:
        print(f"[FAIL] {error}")
    for warning in result.warnings:
        print(f"[WARN] {warning}")
    if result.errors:
        return 1
    print("[PASS] Venue profile is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
