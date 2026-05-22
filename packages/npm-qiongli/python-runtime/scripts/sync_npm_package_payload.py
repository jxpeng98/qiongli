#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
from pathlib import Path


EXCLUDE_DIRS = {"__pycache__", ".pytest_cache", ".mypy_cache", "node_modules", "dist", "build"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo"}


def ignore_patterns(_src: str, names: list[str]) -> set[str]:
    ignored: set[str] = set()
    for name in names:
        if name in EXCLUDE_DIRS or any(name.endswith(suffix) for suffix in EXCLUDE_SUFFIXES):
            ignored.add(name)
    return ignored


def copy_path(src: Path, dest: Path, *, dry_run: bool) -> None:
    if dry_run:
        print(f"[npm-sync] would sync {src} -> {dest}")
        return
    if dest.exists():
        if dest.is_dir():
            shutil.rmtree(dest)
        else:
            dest.unlink()
    if src.is_dir():
        shutil.copytree(src, dest, ignore=ignore_patterns)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)


def fail_if_symlinks(path: Path) -> None:
    if not path.exists():
        return
    if path.is_symlink():
        raise RuntimeError(f"generated npm package path is a symlink: {path}")
    for item in path.rglob("*"):
        if item.is_symlink():
            raise RuntimeError(f"generated npm package path contains symlink: {item}")


def sync_npm_payload(root: Path, *, dry_run: bool = False) -> None:
    npm_root = root / "packages" / "npm-qiongli"
    payload_root = npm_root / "payload"
    runtime_root = npm_root / "python-runtime"

    copy_path(root / "qiongli-workflow", payload_root / "qiongli-workflow", dry_run=dry_run)

    runtime_dirs = (
        "bridges",
        "qiongli",
        "scripts",
        "standards",
        "templates",
        "roles",
        "venue-profiles",
        "skills",
        "pipelines",
        "schemas",
        "evals",
    )
    for item in runtime_dirs:
        src = root / item
        if src.exists():
            copy_path(src, runtime_root / item, dry_run=dry_run)

    for item in ("skills-core.md", "skills-summary.md", "LICENSE"):
        src = root / item
        if src.exists():
            copy_path(src, runtime_root / item, dry_run=dry_run)
            if item == "LICENSE":
                copy_path(src, npm_root / "LICENSE", dry_run=dry_run)

    if not dry_run:
        fail_if_symlinks(payload_root)
        fail_if_symlinks(runtime_root)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Sync generated npm package payload/runtime content.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)

    sync_npm_payload(args.root.resolve(), dry_run=args.dry_run)
    print("[npm-sync] package payload synced" if not args.dry_run else "[npm-sync] dry-run complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
