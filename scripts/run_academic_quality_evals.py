#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from pathlib import Path

import yaml


REQUIRED_DIMENSIONS = [
    "artifact_completeness",
    "evidence_traceability",
    "no_fabricated_sources",
    "claim_calibration",
    "venue_fit",
    "method_validity",
    "scholarly_voice",
    "quality_gate_compliance",
    "domain_method_fit",
]


@dataclass
class EvalRunResult:
    case_count: int
    dimension_scores: dict[str, float] = field(default_factory=dict)
    errors: list[str] = field(default_factory=list)


def run_evals(case_dir: Path) -> EvalRunResult:
    errors: list[str] = []
    totals = {dimension: 0.0 for dimension in REQUIRED_DIMENSIONS}
    case_count = 0
    for case_path in sorted(case_dir.glob("*.yaml")):
        payload = yaml.safe_load(case_path.read_text(encoding="utf-8")) or {}
        case_count += 1
        dimensions = payload.get("expected_dimensions", {})
        if not isinstance(dimensions, dict):
            errors.append(f"{case_path.name}: expected_dimensions must be an object")
            continue
        for dimension in REQUIRED_DIMENSIONS:
            value = dimensions.get(dimension)
            if not isinstance(value, (int, float)):
                errors.append(f"{case_path.name}: missing numeric dimension {dimension}")
                continue
            totals[dimension] += float(value)
    scores = {
        dimension: (totals[dimension] / case_count if case_count else 0.0)
        for dimension in REQUIRED_DIMENSIONS
    }
    if case_count == 0:
        errors.append(f"no eval cases found in {case_dir}")
    return EvalRunResult(case_count=case_count, dimension_scores=scores, errors=errors)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run offline academic quality eval fixtures.")
    parser.add_argument("case_dir", type=Path)
    args = parser.parse_args()

    result = run_evals(args.case_dir)
    for error in result.errors:
        print(f"[FAIL] {error}")
    for dimension, score in sorted(result.dimension_scores.items()):
        print(f"{dimension}: {score:.2f}")
    if result.errors:
        return 1
    print(f"[PASS] Scored {result.case_count} academic quality cases")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
