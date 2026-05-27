#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from qiongli.subject_materializer import (  # noqa: E402
    MaterializeOptions,
    SubjectCatalogError,
    SubjectMaterializationError,
    materialize_subject_package,
)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Materialize a Qiongli subject-specific skill package.")
    parser.add_argument("--subject", default="core", help="Subject id to materialize (default: core).")
    parser.add_argument("--source", type=Path, default=REPO_ROOT, help="Repository or bundled payload root.")
    parser.add_argument("--out", type=Path, required=True, help="Output package directory.")
    parser.add_argument("--flavor", choices=("full", "desktop"), default="full", help="Output package flavor.")
    parser.add_argument(
        "--coverage",
        choices=("complete", "focused"),
        default="complete",
        help="Subject coverage to materialize (default: complete).",
    )
    args = parser.parse_args(argv)

    try:
        materialize_subject_package(
            MaterializeOptions(
                source=args.source,
                out=args.out,
                subject=args.subject,
                flavor=args.flavor,
                coverage=args.coverage,
            )
        )
    except (SubjectCatalogError, SubjectMaterializationError) as exc:
        print(f"[materialize-subject] {exc}", file=sys.stderr)
        return 2

    print(f"[materialize-subject] {args.subject} -> {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
