#!/usr/bin/env python3
"""Internal compatibility helper for materializing legacy package mirrors.

Do not use this as the normal feature-development entrypoint.
Use scripts/materialize_distribution_payloads.py for local checks, CI, release
staging, and package publishing.
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout
from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog
from qiongli.bridges.subject_contracts import load_runtime_subject_contracts


EXCLUDE_DIRS = {"__pycache__", ".pytest_cache", ".mypy_cache", "node_modules", "dist", "build"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo"}
SHELL_CLI_SOURCE_FILES = ("qiongli_cli.sh", "bootstrap_qiongli.sh")
RUNTIME_RESOURCE_PATH_FIELDS = (
    "domain_profile",
    "overlay",
    "subject_skill",
    "evaluation_pack",
)


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


def sync_shell_cli_sources(layout: RepoLayout, payload_scripts: Path, *, dry_run: bool) -> None:
    if dry_run:
        for file_name in SHELL_CLI_SOURCE_FILES:
            print(f"[npm-sync] would sync {layout.scripts / file_name} -> {payload_scripts / file_name}")
        return
    if payload_scripts.exists():
        if payload_scripts.is_dir():
            shutil.rmtree(payload_scripts)
        else:
            payload_scripts.unlink()
    payload_scripts.mkdir(parents=True, exist_ok=True)
    for file_name in SHELL_CLI_SOURCE_FILES:
        src = layout.scripts / file_name
        if not src.is_file():
            raise FileNotFoundError(f"missing shell CLI source: {src}")
        shutil.copy2(src, payload_scripts / file_name)


def sync_npm_payload(root: Path, *, dry_run: bool = False) -> None:
    layout = RepoLayout(root)
    npm_root = root / "packages" / "npm-qiongli"
    payload_root = npm_root / "payload"
    runtime_root = npm_root / "python-runtime"

    with tempfile.TemporaryDirectory(prefix="qiongli-payload-source-") as tmp:
        materialize_source = build_materialize_source(root, Path(tmp), dry_run=dry_run)

        sync_python_payload(root, materialize_source, dry_run=dry_run)

        copy_path(materialize_source / "qiongli-workflow", payload_root / "qiongli-workflow", dry_run=dry_run)
        sync_plugin_lite_payloads(root, payload_root, dry_run=dry_run)
        sync_shell_cli_sources(layout, payload_root / "scripts", dry_run=dry_run)
        sync_subject_payloads(root, payload_root, dry_run=dry_run, materialize_source=materialize_source)

    runtime_dirs = {
        "bridges": layout.bridges_compat_package,
        "qiongli": layout.python_package,
        "scripts": layout.scripts,
        "standards": layout.standards,
        "templates": layout.templates,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
        "skills": layout.skills,
        "subjects": layout.subjects,
        "pipelines": layout.pipelines,
        "schemas": layout.schemas,
        "evals": root / "evals",
    }
    for item, src in runtime_dirs.items():
        if src.exists():
            extra_excludes = {"payload"} if item == "qiongli" else None
            copy_path(src, runtime_root / item, dry_run=dry_run, extra_exclude_dirs=extra_excludes)

    runtime_files = {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
        "LICENSE": root / "LICENSE",
    }
    for item, src in runtime_files.items():
        if src.exists():
            copy_path(src, runtime_root / item, dry_run=dry_run)
            if item == "LICENSE":
                copy_path(src, npm_root / "LICENSE", dry_run=dry_run)

    sync_runtime_subject_resources(root, runtime_root, dry_run=dry_run)

    if not dry_run:
        fail_if_symlinks(payload_root)
        fail_if_symlinks(runtime_root)


def sync_plugin_lite_payloads(root: Path, payload_root: Path, *, dry_run: bool) -> None:
    plugins_root = payload_root / "plugins"
    target_roots = {
        "fallback": plugins_root / "qiongli",
        "codex": plugins_root / "codex" / "qiongli",
        "claude": plugins_root / "claude" / "qiongli",
    }

    if dry_run:
        for target, dest in target_roots.items():
            print(f"[npm-sync] would materialize plugin-lite payload {target} -> {dest}")
        return

    if plugins_root.exists():
        shutil.rmtree(plugins_root)

    from scripts.build_plugin_artifacts import materialize_plugin_package

    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-lite-payload-") as tmp:
        materialized_plugin_root = Path(tmp) / "qiongli"
        materialize_plugin_package(root, materialized_plugin_root, force=True)
        for dest in target_roots.values():
            copy_path(materialized_plugin_root, dest, dry_run=False)

    fail_if_symlinks(plugins_root)


def build_materialize_source(root: Path, tmp_root: Path, *, dry_run: bool) -> Path:
    if dry_run:
        return root
    layout = RepoLayout(root)
    source = tmp_root / "source"
    copy_path(layout.workflow, source / "qiongli-workflow", dry_run=False)
    copy_path(layout.skills, source / "skills", dry_run=False)
    copy_path(layout.subjects, source / "subjects", dry_run=False)
    package_mirrors = {
        "skills": layout.skills,
        "templates": layout.templates,
        "standards": layout.standards,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
    }
    for item, src in package_mirrors.items():
        if src.exists():
            extra_excludes = {"CLAUDE.project.md"} if item == "templates" else None
            copy_path(src, source / "qiongli-workflow" / item, dry_run=False, extra_exclude_dirs=extra_excludes)
    package_files = {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }
    for item, src in package_files.items():
        if src.exists():
            copy_path(src, source / "qiongli-workflow" / item, dry_run=False)
    return source


def sync_python_payload(root: Path, materialize_source: Path, *, dry_run: bool) -> None:
    layout = RepoLayout(root)
    payload_root = layout.python_package / "payload"
    payload_sources = {
        "qiongli-workflow": materialize_source / "qiongli-workflow",
        "skills": layout.skills,
        "subjects": layout.subjects,
        "content/distribution": layout.content / "distribution",
        "content/subjects": layout.content / "subjects",
    }
    for item, src in payload_sources.items():
        copy_path(src, payload_root / item, dry_run=dry_run)
    sync_shell_cli_sources(layout, payload_root / "scripts", dry_run=dry_run)
    sync_subject_payloads(root, payload_root, dry_run=dry_run, materialize_source=payload_root, clear_subjects_root=False)
    sync_runtime_subject_resources(root, payload_root, dry_run=dry_run)
    if not dry_run:
        fail_if_symlinks(payload_root)


def sync_subject_payloads(
    root: Path,
    payload_root: Path,
    *,
    dry_run: bool,
    materialize_source: Path | None = None,
    clear_subjects_root: bool = True,
) -> None:
    catalog = validate_subject_catalog(root)
    subjects_root = payload_root / "subjects"
    if dry_run:
        for subject in sorted(catalog.subjects):
            for coverage in ("complete", "focused"):
                print(
                    "[npm-sync] would materialize "
                    f"subject {subject}/{coverage} -> {subjects_root / subject / coverage / 'qiongli-workflow'}"
                )
        return
    if clear_subjects_root and subjects_root.exists():
        shutil.rmtree(subjects_root)
    for subject in sorted(catalog.subjects):
        for coverage in ("complete", "focused"):
            materialize_subject_package(
                MaterializeOptions(
                    source=materialize_source or root,
                    out=subjects_root / subject / coverage / "qiongli-workflow",
                    subject=subject,
                    flavor="full",
                    coverage=coverage,
                )
            )


def runtime_subject_resource_paths(root: Path) -> list[Path]:
    contracts = load_runtime_subject_contracts(
        RepoLayout(root).subjects,
        recursive=False,
    )
    resources: set[Path] = set()
    for contract in contracts.values():
        if contract.activation_status != "runtime_enabled":
            continue
        for field in RUNTIME_RESOURCE_PATH_FIELDS:
            resource = getattr(contract, field, "")
            if resource:
                resources.add(Path(resource))
        for config in contract.method_lenses.values():
            resource = config.get("resource")
            if isinstance(resource, str) and resource:
                resources.add(Path(resource))
    return sorted(resources)


def sync_runtime_subject_resources(root: Path, target_root: Path, *, dry_run: bool) -> None:
    for rel_path in runtime_subject_resource_paths(root):
        src = root / rel_path
        if not src.exists():
            raise FileNotFoundError(f"missing runtime subject resource: {src}")
        copy_path(src, target_root / rel_path, dry_run=dry_run)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Internal compatibility helper for generated npm payload/runtime content. "
            "Use scripts/materialize_distribution_payloads.py for normal workflows."
        )
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)

    print("[npm-sync] internal compatibility helper; prefer scripts/materialize_distribution_payloads.py")
    sync_npm_payload(args.root.resolve(), dry_run=args.dry_run)
    print("[npm-sync] package payload synced" if not args.dry_run else "[npm-sync] dry-run complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
