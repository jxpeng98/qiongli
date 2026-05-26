#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
from pathlib import Path

from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog


EXCLUDE_DIRS = {"__pycache__", ".pytest_cache", ".mypy_cache", "node_modules", "dist", "build"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo"}


def ignore_patterns(_src: str, names: list[str], extra_exclude_dirs: set[str] | None = None) -> set[str]:
    ignored: set[str] = set()
    excluded_dirs = EXCLUDE_DIRS | (extra_exclude_dirs or set())
    for name in names:
        if name in excluded_dirs or any(name.endswith(suffix) for suffix in EXCLUDE_SUFFIXES):
            ignored.add(name)
    return ignored


def copy_path(src: Path, dest: Path, *, dry_run: bool, extra_exclude_dirs: set[str] | None = None) -> None:
    if dry_run:
        print(f"[npm-sync] would sync {src} -> {dest}")
        return
    if dest.exists():
        if dest.is_dir():
            shutil.rmtree(dest)
        else:
            dest.unlink()
    if src.is_dir():
        shutil.copytree(src, dest, ignore=lambda copy_src, names: ignore_patterns(copy_src, names, extra_exclude_dirs))
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
    sync_subject_payloads(root, payload_root, dry_run=dry_run)

    runtime_dirs = (
        "bridges",
        "qiongli",
        "scripts",
        "standards",
        "templates",
        "roles",
        "venue-profiles",
        "skills",
        "subjects",
        "pipelines",
        "schemas",
        "evals",
    )
    for item in runtime_dirs:
        src = root / item
        if src.exists():
            extra_excludes = {"payload"} if item == "qiongli" else None
            copy_path(src, runtime_root / item, dry_run=dry_run, extra_exclude_dirs=extra_excludes)

    for item in ("skills-core.md", "skills-summary.md", "LICENSE"):
        src = root / item
        if src.exists():
            copy_path(src, runtime_root / item, dry_run=dry_run)
            if item == "LICENSE":
                copy_path(src, npm_root / "LICENSE", dry_run=dry_run)

    if not dry_run:
        fail_if_symlinks(payload_root)
        fail_if_symlinks(runtime_root)


def sync_subject_payloads(root: Path, payload_root: Path, *, dry_run: bool) -> None:
    catalog = validate_subject_catalog(root)
    subjects_root = payload_root / "subjects"
    if dry_run:
        for subject in sorted(catalog.subjects):
            print(f"[npm-sync] would materialize subject {subject} -> {subjects_root / subject / 'qiongli-workflow'}")
        return
    if subjects_root.exists():
        shutil.rmtree(subjects_root)
    for subject in sorted(catalog.subjects):
        materialize_subject_package(
            MaterializeOptions(
                source=root,
                out=subjects_root / subject / "qiongli-workflow",
                subject=subject,
                flavor="full",
            )
        )


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
