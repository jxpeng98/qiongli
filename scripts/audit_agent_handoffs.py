#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def audit_agent_handoffs(root: Path) -> list[str]:
    errors: list[str] = []
    root = root.resolve()
    if not root.exists():
        return [f"missing audit root: {root}"]

    for packet_path in sorted(root.rglob("*.json")):
        packet = _read_json_object(packet_path)
        if not packet or packet.get("execution_mode") != "duo":
            continue

        run_id = str(packet.get("run_id") or packet_path.stem)
        if not _has_conflicting_positions(packet):
            continue

        artifact_paths = _artifact_paths(root, packet)
        if not _has_existing_artifact(artifact_paths, "handoff"):
            errors.append(f"{run_id}: duo conflict missing handoff artifact")
        if not _has_existing_artifact(artifact_paths, "disagreement"):
            errors.append(f"{run_id}: duo conflict missing disagreement artifact")

    return errors


def _read_json_object(path: Path) -> dict[str, Any] | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if isinstance(data, dict):
        return data
    return None


def _has_conflicting_positions(packet: dict[str, Any]) -> bool:
    codex_position = _normalized_position(packet.get("codex_position"))
    claude_position = _normalized_position(packet.get("claude_position"))
    if codex_position and claude_position and codex_position != claude_position:
        return True

    disagreements = packet.get("disagreements", [])
    if not isinstance(disagreements, list):
        return False
    for item in disagreements:
        if not isinstance(item, dict):
            continue
        codex_position = _normalized_position(item.get("codex_position"))
        claude_position = _normalized_position(item.get("claude_position"))
        if codex_position and claude_position and codex_position != claude_position:
            return True
    return False


def _normalized_position(value: Any) -> str:
    if value is None:
        return ""
    return str(value).strip().lower()


def _artifact_paths(root: Path, packet: dict[str, Any]) -> list[Path]:
    paths: list[Path] = []
    for key in ("artifacts_written", "artifacts"):
        value = packet.get(key, [])
        if not isinstance(value, list):
            continue
        for item in value:
            if isinstance(item, str):
                paths.append(root / item)
            elif isinstance(item, dict) and isinstance(item.get("path"), str):
                paths.append(root / item["path"])
    return paths


def _has_existing_artifact(paths: list[Path], marker: str) -> bool:
    return any(marker in path.name.lower() and path.exists() for path in paths)


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit duo agent handoff artifacts.")
    parser.add_argument("root", type=Path)
    args = parser.parse_args()

    errors = audit_agent_handoffs(args.root)
    for error in errors:
        print(f"[FAIL] {error}")
    if errors:
        return 1
    print("[PASS] Agent handoff artifacts are valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
