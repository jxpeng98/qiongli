#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_DIR = Path(__file__).resolve().parent
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, SCRIPT_DIR, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from generated_output_paths import is_generated_output_path

EXCLUDED_NAMES = {
    ".env",
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".qiongli",
    ".qiongli-config",
    ".ruff_cache",
    ".venv",
    ".worktrees",
    "__pycache__",
    "build",
    "dist",
    "node_modules",
}
EXCLUDED_NAME_SUFFIXES = (".egg-info",)


def is_materialized_output_path(path: Path | str) -> bool:
    return is_generated_output_path(path)


def copy_source_tree(source: Path, dest: Path) -> None:
    source = source.resolve()
    dest = dest.resolve()

    def ignore(current: str, names: list[str]) -> set[str]:
        ignored: set[str] = set()
        current_path = Path(current).resolve()
        for name in names:
            if name in EXCLUDED_NAMES or name.endswith(EXCLUDED_NAME_SUFFIXES):
                ignored.add(name)
                continue
            candidate = current_path / name
            try:
                rel = candidate.relative_to(source)
            except ValueError:
                continue
            if is_materialized_output_path(rel):
                ignored.add(name)
        return ignored

    shutil.copytree(source, dest, symlinks=False, ignore=ignore)


def prepare_staging_dir(source: Path, out: Path, *, force: bool) -> Path:
    source = source.resolve()
    out = out.resolve()

    if out == source or source in out.parents:
        raise ValueError("--out must be outside the source tree")

    if out.exists() and any(out.iterdir()):
        if not force:
            raise ValueError(f"{out} already exists and is not empty; pass --force to replace it")
        shutil.rmtree(out)
    elif out.exists() and force:
        shutil.rmtree(out)

    out.parent.mkdir(parents=True, exist_ok=True)
    copy_source_tree(source, out)
    return out


def run_command(command: list[str], *, cwd: Path) -> None:
    subprocess.run(command, cwd=cwd, check=True)


def materialize_python_payload(root: Path) -> None:
    python_source_root = root / "packages" / "python-qiongli" / "src"
    for import_root in (python_source_root, root):
        if str(import_root) not in sys.path:
            sys.path.insert(0, str(import_root))
    from scripts.sync_npm_package_payload import build_materialize_source, sync_python_payload

    with tempfile.TemporaryDirectory(prefix="qiongli-python-payload-source-") as tmp:
        materialize_source = build_materialize_source(root, Path(tmp), dry_run=False)
        sync_python_payload(root, materialize_source, dry_run=False)


def materialize_plugin_payload(root: Path) -> None:
    python_source_root = root / "packages" / "python-qiongli" / "src"
    for import_root in (python_source_root, root):
        if str(import_root) not in sys.path:
            sys.path.insert(0, str(import_root))
    from qiongli.source_layout import RepoLayout
    from scripts.build_plugin_artifacts import (
        materialize_agent_platform,
        materialize_plugin_package,
    )
    from scripts.sync_npm_package_payload import build_materialize_source, copy_path, fail_if_symlinks

    layout = RepoLayout(root)
    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-payload-source-") as tmp:
        materialize_source = build_materialize_source(root, Path(tmp), dry_run=False)
        portable_source = materialize_source / "qiongli-workflow"
        portable_dest = root / "qiongli-workflow"
        plugin_root = layout.plugin_artifact_package
        plugin_dest = plugin_root / "skills" / "qiongli-workflow"

        print(f"Syncing skill package: {portable_dest}")
        copy_path(portable_source, portable_dest, dry_run=False)
        fail_if_symlinks(portable_dest)
        print("  [ok] portable package")

        print(f"Syncing plugin package source: {plugin_root}")
        materialize_plugin_package(root, plugin_root, force=True)
        fail_if_symlinks(plugin_root)
        print("  [ok] plugin package source")

        print(f"Syncing agent platform files: {layout.agent_platform_artifact}")
        materialize_agent_platform(root, layout.agent_platform_artifact, force=True)
        fail_if_symlinks(layout.agent_platform_artifact)
        print("  [ok] agent platform files")

        print(f"Syncing skill package: {plugin_dest}")
        copy_path(portable_dest, plugin_dest, dry_run=False)
        fail_if_symlinks(plugin_dest)
        print("  [ok] mirrored portable package")

    print("[done] Skill package is self-contained.")


def materialize_next_plugin_payload(root: Path) -> None:
    python_source_root = root / "packages" / "python-qiongli" / "src"
    for import_root in (python_source_root, root):
        if str(import_root) not in sys.path:
            sys.path.insert(0, str(import_root))
    from qiongli.source_layout import RepoLayout
    from scripts.build_plugin_artifacts import materialize_next_plugin_package
    from scripts.sync_npm_package_payload import fail_if_symlinks

    layout = RepoLayout(root)
    next_plugin_root = layout.next_plugin_package
    print(f"Syncing generated qiongli-next plugin payload: {next_plugin_root}")
    materialize_next_plugin_package(root, next_plugin_root, force=True)
    fail_if_symlinks(next_plugin_root)
    print("  [ok] qiongli-next plugin payload")


def materialize_in_place(root: Path, target: str) -> None:
    root = root.resolve()
    if target in {"plugin", "all"}:
        materialize_plugin_payload(root)
    if target in {"plugin", "next-plugin", "all"}:
        materialize_next_plugin_payload(root)
    if target == "python":
        materialize_python_payload(root)
    if target in {"npm", "all"}:
        run_command([sys.executable, "scripts/sync_npm_package_payload.py"], cwd=root)
    if target == "all":
        run_command([sys.executable, "scripts/audit_distribution_payloads.py", "--root", str(root)], cwd=root)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Materialize Qiongli distribution payloads.")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository source root.")
    parser.add_argument("--target", choices=("python", "npm", "plugin", "next-plugin", "all"), default="all")
    parser.add_argument("--out", type=Path, help="Staging directory to create and materialize into.")
    parser.add_argument("--force", action="store_true", help="Replace an existing non-empty staging directory.")
    parser.add_argument(
        "--in-place",
        action="store_true",
        help="Materialize in the source tree. Use only for release/staging maintenance.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.out and args.in_place:
        parser.error("--out and --in-place are mutually exclusive")
    if not args.out and not args.in_place:
        parser.error("materialization requires either --out or --in-place")

    root = args.root.resolve()
    if args.in_place:
        materialize_root = root
    else:
        try:
            materialize_root = prepare_staging_dir(root, args.out, force=args.force)
        except ValueError as exc:
            parser.error(str(exc))

    materialize_in_place(materialize_root, args.target)
    print(f"[materialize-distribution] {args.target} payloads ready at {materialize_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
