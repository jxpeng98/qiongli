#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bridges.providers.literature_artifacts import materialize_search_bundle


def materialize_literature_search_bundle(payload: dict[str, Any], project_root: Path) -> list[Path]:
    written = materialize_search_bundle(project_root, payload)
    return list(written.values())


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Materialize provider literature search output into canonical project artifacts."
    )
    parser.add_argument("--input", type=Path, help="JSON payload file; defaults to stdin")
    parser.add_argument("--project-root", required=True, type=Path, help="RESEARCH/[topic] project root")
    args = parser.parse_args()

    raw = args.input.read_text(encoding="utf-8") if args.input else sys.stdin.read()
    payload = json.loads(raw)
    written = materialize_literature_search_bundle(payload, args.project_root)
    for path in written:
        print(f"[WRITE] {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
