#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from dataclasses import dataclass, field
from pathlib import Path


REQUIRED_SECTIONS = [
    "Design Summary",
    "Validity Threat Matrix",
    "Method-Specific Checks",
    "Insufficient Input Notes",
]


@dataclass
class MethodDiagnosticAuditResult:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


def audit_method_diagnostic_report(path: Path) -> MethodDiagnosticAuditResult:
    result = MethodDiagnosticAuditResult()
    if not path.exists():
        result.errors.append(f"missing method diagnostic report: {path}")
        return result
    content = path.read_text(encoding="utf-8")
    headings = {
        match.group(1).strip().lower()
        for match in re.finditer(r"^##\s+(.+?)\s*$", content, flags=re.MULTILINE)
    }
    for section in REQUIRED_SECTIONS:
        if section.lower() not in headings:
            result.errors.append(f"Missing section: {section}")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit method diagnostic report markdown.")
    parser.add_argument("report", type=Path)
    args = parser.parse_args()

    result = audit_method_diagnostic_report(args.report)
    for error in result.errors:
        print(f"[FAIL] {error}")
    for warning in result.warnings:
        print(f"[WARN] {warning}")
    if result.errors:
        return 1
    print("[PASS] Method diagnostic report is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
