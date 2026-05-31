#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


ALLOW_ENV = "QIONGLI_ALLOW_GENERATED_PAYLOAD_CHANGES"

GENERATED_PREFIXES = (
    "qiongli/payload/",
    "packages/npm-qiongli/payload/",
    "packages/npm-qiongli/python-runtime/",
    "plugins/qiongli/skills/qiongli-workflow/",
    "qiongli-workflow/skills/",
    "qiongli-workflow/templates/",
    "qiongli-workflow/standards/",
    "qiongli-workflow/roles/",
    "qiongli-workflow/venue-profiles/",
)

GENERATED_FILES = {
    "qiongli-workflow/skills-core.md",
    "qiongli-workflow/skills-summary.md",
}


def normalize_path(path: str) -> str:
    return Path(path.strip()).as_posix().lstrip("./")


def is_generated_payload_path(path: str) -> bool:
    normalized = normalize_path(path)
    return normalized in GENERATED_FILES or any(normalized.startswith(prefix) for prefix in GENERATED_PREFIXES)


def changed_files_from_git(base_ref: str) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_ref}...HEAD"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        print(result.stderr.strip(), file=sys.stderr)
        raise SystemExit(result.returncode)
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


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
