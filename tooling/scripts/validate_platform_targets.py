#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
SCRIPT_DIR = Path(__file__).resolve().parent
for import_root in (SCRIPT_DIR, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from generate_release_downloads import load_companion_targets
from qiongli.platform_targets import validate_platform_target_registry


def validate_release_target_registries(root: Path) -> list[str]:
    failures = [
        f"platform target registry: {failure}"
        for failure in validate_platform_target_registry(root)
    ]
    try:
        load_companion_targets(root)
    except ValueError as exc:
        failures.append(f"release companion target registry: {exc}")
    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate Qiongli release target registry schemas.")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository or materialized distribution root.")
    args = parser.parse_args(argv)

    failures = validate_release_target_registries(args.root)
    if failures:
        for failure in failures:
            print(f"[FAIL] {failure}")
        return 1
    print("[OK] release target registries schema valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
