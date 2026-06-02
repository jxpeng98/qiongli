#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from generated_output_paths import is_generated_output_path, normalize_generated_path


ALLOW_ENV = "QIONGLI_ALLOW_GENERATED_PAYLOAD_CHANGES"


def normalize_path(path: str) -> str:
    return normalize_generated_path(path)


def is_generated_payload_path(path: str) -> bool:
    return is_generated_output_path(path)


def changed_files_from_git(base_ref: str) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-status", f"{base_ref}...HEAD"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        print(result.stderr.strip(), file=sys.stderr)
        raise SystemExit(result.returncode)
    return changed_non_deleted_paths_from_name_status(result.stdout.splitlines())


def changed_non_deleted_paths_from_name_status(lines: list[str]) -> list[str]:
    changed: list[str] = []
    for line in lines:
        if not line.strip():
            continue
        parts = line.split("\t")
        status = parts[0]
        if status == "D":
            continue
        if status.startswith(("R", "C")) and len(parts) >= 3:
            changed.append(parts[2])
            continue
        if len(parts) >= 2:
            changed.append(parts[1])
    return changed


def check_changed_files(changed_files: list[str]) -> list[str]:
    return sorted({normalize_path(path) for path in changed_files if is_generated_payload_path(path)})


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Reject ordinary feature changes that edit generated distribution payloads."
    )
    parser.add_argument(
        "--base-ref",
        default="origin/dev",
        help="Base ref used for git diff mode. Defaults to origin/dev.",
    )
    parser.add_argument(
        "--changed-file",
        action="append",
        default=[],
        help="Explicit changed file path. May be passed more than once; bypasses git diff mode.",
    )
    args = parser.parse_args(argv)

    changed_files = list(args.changed_file) if args.changed_file else changed_files_from_git(args.base_ref)
    generated_changes = check_changed_files(changed_files)

    if not generated_changes:
        print("[generated-payload-guard] no generated distribution payload edits detected")
        return 0

    if os.environ.get(ALLOW_ENV) == "1":
        print(
            f"[generated-payload-guard] override enabled via {ALLOW_ENV}=1; "
            f"allowing {len(generated_changes)} generated path change(s)"
        )
        return 0

    print(
        "Generated distribution payload files changed in a non-release context.\n"
        "Edit the canonical source instead, then let CI or release staging materialize payloads.\n"
        "Set QIONGLI_ALLOW_GENERATED_PAYLOAD_CHANGES=1 only for explicit release/staging maintenance.\n",
        file=sys.stderr,
    )
    for path in generated_changes:
        print(f"  - {path}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
