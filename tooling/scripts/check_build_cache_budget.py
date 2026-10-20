#!/usr/bin/env python3
"""Read-only budget check for regenerable Cargo caches; never delete artifacts."""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[2]
GIB = 1024 ** 3
CACHES = (
    "packages/qiongli-native/target/debug/deps",
    "packages/qiongli-native/target/debug/incremental",
    "packages/qiongli-lite-mcp/target/debug/deps",
    "packages/qiongli-lite-mcp/target/debug/incremental",
    "packages/qiongli-lite-mcp/target/release/deps",
)


def cache_bytes(root: Path, relative: str) -> int:
    path = root / relative
    if any(p.is_symlink() for p in (path, *path.parents) if p.is_relative_to(root)):
        raise ValueError(f"linked cache path: {relative}")
    if not path.exists():
        return 0
    def measure(entry: Path) -> int:
        if entry.is_symlink():
            raise ValueError(f"linked cache entry in {relative}")
        if entry.is_dir():
            return sum(measure(child) for child in entry.iterdir())
        return entry.stat().st_size
    return measure(path)


def check_budget(root: Path, max_cache_bytes: int, min_free_bytes: int) -> dict:
    root = root.resolve(strict=True)
    sizes = {relative: cache_bytes(root, relative) for relative in CACHES}
    total, free = sum(sizes.values()), shutil.disk_usage(root).free
    return {"cache_bytes": sizes, "total_cache_bytes": total, "free_bytes": free,
            "max_cache_bytes": max_cache_bytes, "min_free_bytes": min_free_bytes,
            "within_budget": total <= max_cache_bytes and free >= min_free_bytes}


def positive_gib(value: str) -> float:
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("budget must be a finite positive GiB value")
    return number


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--max-cache-gib", type=positive_gib, default=16)
    parser.add_argument("--min-free-gib", type=positive_gib, default=8)
    args = parser.parse_args()
    try:
        result = check_budget(args.root, int(args.max_cache_gib * GIB), int(args.min_free_gib * GIB))
    except (OSError, ValueError) as error:
        parser.exit(2, f"Cannot measure build budget: {error}\n")
    print(json.dumps(result, indent=2))
    return 0 if result["within_budget"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
