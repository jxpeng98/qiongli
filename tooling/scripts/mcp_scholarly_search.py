#!/usr/bin/env python3
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from bridges.providers.literature_search import run_scholarly_search
from bridges.providers.s2_client import search_paper
from bridges.provider_config import (
    provider_capability_mode,
    provider_config_summary,
    resolve_provider_config,
)


def _attach_provider_config(output: dict[str, object]) -> None:
    data = output.get("data")
    if not isinstance(data, dict):
        return
    summary = provider_config_summary(resolve_provider_config(cwd=Path.cwd()))
    data["provider_config"] = summary
    data["capability_mode"] = provider_capability_mode(summary)


def main() -> None:
    try:
        input_data = sys.stdin.read()
        if not input_data.strip():
            print(json.dumps({"status": "error", "summary": "No input provided"}))
            return

        payload = json.loads(input_data)
        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}

        output = run_scholarly_search(task_packet, search_paper)
        _attach_provider_config(output)
        print(json.dumps(output))
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "summary": f"Scholarly search provider exception: {str(e)}",
            "data": {"error": str(e)}
        }))

if __name__ == "__main__":
    main()
