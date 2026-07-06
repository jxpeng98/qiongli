#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.bridges.experience_runtime import experience_schema_compatibility


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check Qiongli experience record schema compatibility for release readiness."
    )
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="Repository or staged release root")
    parser.add_argument("--json", action="store_true", help="Emit JSON report")
    args = parser.parse_args(argv)

    report = experience_schema_compatibility(args.root)
    if args.json:
        print(json.dumps(report, ensure_ascii=False, sort_keys=True))
    else:
        status = "ok" if report["ok"] else "failed"
        print(
            "[experience-schema] "
            f"{status}; checked_records={report['checked_records']}; "
            f"malformed_count={report['malformed_count']}"
        )
        for error in report["errors"]:
            print(f"[experience-schema] error: {error}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
