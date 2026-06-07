#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
from dataclasses import dataclass, field
from pathlib import Path


REQUIRED_COLUMNS = [
    "claim_id",
    "claim_text",
    "claim_type",
    "evidence_type",
    "source_id",
    "source_location",
    "artifact_path",
    "confidence",
    "limitations",
    "status",
]
ALLOWED_CLAIM_TYPES = {
    "finding",
    "interpretation",
    "implication",
    "method_assumption",
    "limitation",
    "speculation",
}
ALLOWED_EVIDENCE_TYPES = {
    "paper",
    "dataset",
    "analysis_result",
    "theory",
    "artifact",
    "gap_note",
}


@dataclass
class AuditResult:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


def audit_evidence_ledger(path: Path) -> AuditResult:
    result = AuditResult()
    if not path.exists():
        result.errors.append(f"missing evidence ledger: {path}")
        return result

    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        fieldnames = reader.fieldnames or []
        missing_columns = [column for column in REQUIRED_COLUMNS if column not in fieldnames]
        if missing_columns:
            result.errors.append("missing required columns: " + ", ".join(missing_columns))
            return result

        seen_claim_ids: set[str] = set()
        for row_number, row in enumerate(reader, start=2):
            claim_id = (row.get("claim_id") or "").strip()
            claim_type = (row.get("claim_type") or "").strip()
            evidence_type = (row.get("evidence_type") or "").strip()
            source_id = (row.get("source_id") or "").strip()
            artifact_path = (row.get("artifact_path") or "").strip()
            status = (row.get("status") or "").strip()

            if not claim_id:
                result.errors.append(f"row {row_number}: missing claim_id")
            elif claim_id in seen_claim_ids:
                result.errors.append(f"row {row_number}: duplicate claim_id: {claim_id}")
            seen_claim_ids.add(claim_id)

            if claim_type not in ALLOWED_CLAIM_TYPES:
                result.errors.append(f"row {row_number}: invalid claim_type: {claim_type}")
            if evidence_type not in ALLOWED_EVIDENCE_TYPES:
                result.errors.append(f"row {row_number}: invalid evidence_type: {evidence_type}")

            unsupported = status in {"unsupported", "needs_evidence"} or not source_id
            if unsupported and evidence_type != "gap_note":
                result.errors.append(
                    f"row {row_number}: unsupported claims must use evidence_type=gap_note"
                )
            if evidence_type != "gap_note" and not source_id:
                result.errors.append(f"row {row_number}: supported evidence requires source_id")
            if not artifact_path:
                result.errors.append(f"row {row_number}: missing artifact_path")

    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit claim evidence ledger CSV files.")
    parser.add_argument("ledger", type=Path)
    args = parser.parse_args()

    result = audit_evidence_ledger(args.ledger)
    for error in result.errors:
        print(f"[FAIL] {error}")
    for warning in result.warnings:
        print(f"[WARN] {warning}")
    if result.errors:
        return 1
    print("[PASS] Evidence ledger is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
