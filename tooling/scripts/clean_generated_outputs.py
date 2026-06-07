#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from generated_output_paths import GENERATED_OUTPUT_PATHS, normalize_generated_path


REPO_ROOT = Path(__file__).resolve().parents[2]


def existing_generated_outputs(root: Path) -> list[tuple[str, Path]]:
    found: list[tuple[str, Path]] = []
    for rel in GENERATED_OUTPUT_PATHS:
        path = root / rel
        if path.exists() or path.is_symlink():
            found.append((rel, path))
    return found


def git_check_ignored(root: Path, rel: str) -> bool:
    result = subprocess.run(
        ["git", "-C", str(root), "check-ignore", "-q", "--", rel],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def validate_ignored(root: Path, targets: list[tuple[str, Path]]) -> list[str]:
    return [rel for rel, _path in targets if not git_check_ignored(root, rel)]


def remove_path(path: Path) -> None:
    if path.is_symlink() or path.is_file():
        path.unlink()
        return
    if path.is_dir():
        shutil.rmtree(path)


def clean_generated_outputs(root: Path, *, apply: bool) -> int:
    root = root.resolve()
    targets = existing_generated_outputs(root)
    if not targets:
        print("[clean-generated-outputs] no generated outputs found")
        return 0

    not_ignored = validate_ignored(root, targets)
    if not_ignored:
        print(
            "Refusing to remove generated output targets that are not ignored by git.",
            file=sys.stderr,
        )
        for rel in not_ignored:
            print(f"  - {rel}", file=sys.stderr)
        return 1

    for rel, path in targets:
        normalized = normalize_generated_path(rel)
        if apply:
            remove_path(path)
            print(f"Removed {normalized}")
        else:
            print(f"Would remove {normalized}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Remove ignored generated Qiongli distribution outputs from a local checkout."
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository checkout root.")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="Preview removals without deleting anything.")
    mode.add_argument("--apply", action="store_true", help="Delete the ignored generated outputs.")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    root = args.root.resolve()
    if not root.exists():
        parser.error(f"{root} does not exist")

    return clean_generated_outputs(root, apply=args.apply)


if __name__ == "__main__":
    raise SystemExit(main())
