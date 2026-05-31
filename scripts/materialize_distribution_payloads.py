#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

EXCLUDED_NAMES = {
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".venv",
    ".worktrees",
    "__pycache__",
    "node_modules",
}

MATERIALIZED_OUTPUT_PREFIXES = (
    "qiongli/payload",
    "packages/npm-qiongli/payload",
    "packages/npm-qiongli/python-runtime",
    "plugins/qiongli/skills/qiongli-workflow",
    "qiongli-workflow/skills",
    "qiongli-workflow/templates",
    "qiongli-workflow/standards",
    "qiongli-workflow/roles",
    "qiongli-workflow/venue-profiles",
)

MATERIALIZED_OUTPUT_FILES = {
    "qiongli-workflow/skills-core.md",
    "qiongli-workflow/skills-summary.md",
}


def is_materialized_output_path(path: Path | str) -> bool:
    rel = Path(path).as_posix().lstrip("./")
    return rel in MATERIALIZED_OUTPUT_FILES or any(
        rel == prefix or rel.startswith(f"{prefix}/") for prefix in MATERIALIZED_OUTPUT_PREFIXES
    )


def copy_source_tree(source: Path, dest: Path) -> None:
    source = source.resolve()
    dest = dest.resolve()

    def ignore(current: str, names: list[str]) -> set[str]:
        ignored: set[str] = set()
        current_path = Path(current).resolve()
        for name in names:
            if name in EXCLUDED_NAMES:
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
    if str(root) not in sys.path:
        sys.path.insert(0, str(root))
    from scripts.sync_npm_package_payload import build_materialize_source, sync_python_payload

    with tempfile.TemporaryDirectory(prefix="qiongli-python-payload-source-") as tmp:
        materialize_source = build_materialize_source(root, Path(tmp), dry_run=False)
        sync_python_payload(root, materialize_source, dry_run=False)


def materialize_plugin_payload(root: Path) -> None:
    if str(root) not in sys.path:
        sys.path.insert(0, str(root))
    from scripts.sync_npm_package_payload import build_materialize_source, copy_path, fail_if_symlinks

    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-payload-source-") as tmp:
        materialize_source = build_materialize_source(root, Path(tmp), dry_run=False)
        portable_source = materialize_source / "qiongli-workflow"
        portable_dest = root / "qiongli-workflow"
        plugin_dest = root / "plugins" / "qiongli" / "skills" / "qiongli-workflow"

        print(f"Syncing skill package: {portable_dest}")
        copy_path(portable_source, portable_dest, dry_run=False)
        fail_if_symlinks(portable_dest)
        print("  [ok] portable package")

        print(f"Syncing skill package: {plugin_dest}")
        copy_path(portable_dest, plugin_dest, dry_run=False)
        fail_if_symlinks(plugin_dest)
        print("  [ok] mirrored portable package")

    print("[done] Skill package is self-contained.")


def materialize_in_place(root: Path, target: str) -> None:
    root = root.resolve()
    if target in {"plugin", "all"}:
        materialize_plugin_payload(root)
    if target == "python":
        materialize_python_payload(root)
    if target in {"npm", "all"}:
        run_command([sys.executable, "scripts/sync_npm_package_payload.py"], cwd=root)
    if target == "all":
        run_command([sys.executable, "scripts/audit_distribution_payloads.py", "--root", str(root)], cwd=root)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Materialize Qiongli distribution payloads.")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository source root.")
    parser.add_argument("--target", choices=("python", "npm", "plugin", "all"), default="all")
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
