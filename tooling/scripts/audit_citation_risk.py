#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import re
from dataclasses import dataclass, field
from pathlib import Path


CITABLE_EVIDENCE_TYPES = {"paper", "theory"}
BIB_KEY_PATTERN = re.compile(r"@\w+\s*\{\s*([^,\s]+)")


@dataclass
class CitationAuditResult:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


def _split_source_ids(raw: str) -> list[str]:
    return [item.strip() for item in re.split(r"[;|]", raw) if item.strip()]


def _read_bib_keys(path: Path) -> set[str]:
    if not path.exists():
        return set()
    return set(BIB_KEY_PATTERN.findall(path.read_text(encoding="utf-8")))


def audit_citation_integrity(ledger_path: Path, bibliography_path: Path) -> CitationAuditResult:
    result = CitationAuditResult()
    bib_keys = _read_bib_keys(bibliography_path)
    if not bib_keys:
        result.errors.append(f"missing or empty bibliography: {bibliography_path}")
        return result

    with ledger_path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row_number, row in enumerate(reader, start=2):
            evidence_type = (row.get("evidence_type") or "").strip()
            if evidence_type not in CITABLE_EVIDENCE_TYPES:
                continue
            for source_id in _split_source_ids(row.get("source_id") or ""):
                if source_id not in bib_keys:
                    result.errors.append(f"source_id not found in bibliography: {source_id}")
            if evidence_type in CITABLE_EVIDENCE_TYPES and not (row.get("source_id") or "").strip():
                result.errors.append(f"row {row_number}: citable evidence is missing source_id")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit source IDs in an evidence ledger against a BibTeX bibliography."
    )
    parser.add_argument("ledger", type=Path)
    parser.add_argument("bibliography", type=Path)
    args = parser.parse_args()

    result = audit_citation_integrity(args.ledger, args.bibliography)
    for error in result.errors:
        print(f"[FAIL] {error}")
    for warning in result.warnings:
        print(f"[WARN] {warning}")
    if result.errors:
        return 1
    print("[PASS] Citation integrity is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
