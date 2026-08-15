#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from evals.runner.run_eval import run_case  # noqa: E402


@dataclass(frozen=True)
class EvalRunResult:
    case_count: int
    passed_cases: int
    failed_cases: int

    @property
    def success(self) -> bool:
        return self.case_count > 0 and self.failed_cases == 0


def run_evals(case_dir: Path, fixture_root: Path | None = None) -> EvalRunResult:
    case_paths = sorted(case_dir.glob("*.yaml"))
    outputs = fixture_root or case_dir.parent / "fixtures"
    passed_cases = sum(
        run_case(case_path, outputs / case_path.stem) for case_path in case_paths
    )
    return EvalRunResult(
        case_count=len(case_paths),
        passed_cases=passed_cases,
        failed_cases=len(case_paths) - passed_cases,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate captured academic-quality artifacts offline."
    )
    parser.add_argument("case_dir", type=Path)
    args = parser.parse_args(argv)

    result = run_evals(args.case_dir)
    status = "PASS" if result.success else "FAIL"
    print(
        f"[{status}] {result.passed_cases} passed, {result.failed_cases} failed "
        f"across {result.case_count} academic-quality cases"
    )
    return 0 if result.success else 1


if __name__ == "__main__":
    raise SystemExit(main())
