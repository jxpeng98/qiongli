#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from bridges.providers.screening_tracker import run_screening_tracker


def main() -> None:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            print(json.dumps({"status": "error", "summary": "No input provided", "data": {}}))
            return
        payload = json.loads(raw)
        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}
        print(json.dumps(run_screening_tracker(task_packet, Path.cwd())))
    except Exception as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "summary": f"Screening tracker provider exception: {exc}",
                    "data": {"error": str(exc)},
                },
            )
        )


if __name__ == "__main__":
    main()
