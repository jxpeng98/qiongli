#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path

from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog


PLUGIN_NAME = "qiongli"
PLUGIN_ROOT = Path("plugins") / PLUGIN_NAME
DESKTOP_SKILL_FILE_BUDGET = 180


def _normalize_tag(raw: str) -> tuple[str, str]:
    tag = raw.strip()
    if not tag:
        raise ValueError("tag is required")
    repo_tag = tag if tag.startswith("v") else f"v{tag}"
    skill_version = repo_tag.removeprefix("v")
    return repo_tag, skill_version


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _collect_versions(value: object) -> list[str]:
    versions: list[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            if key == "version" and isinstance(item, str):
                versions.append(item)
            else:
                versions.extend(_collect_versions(item))
    elif isinstance(value, list):
        for item in value:
            versions.extend(_collect_versions(item))
    return versions


def _assert_json_versions(path: Path, expected_version: str) -> None:
    data = _read_json(path)
    versions = _collect_versions(data)
    if not versions:
        raise ValueError(f"missing version in {path}")
    for version in versions:
        if version != expected_version:
            raise ValueError(f"version mismatch in {path}: expected {expected_version}, found {version}")


def _copy_path(src: Path, dest: Path) -> None:
    if src.is_dir():
        shutil.copytree(src, dest)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)


def _copy_common_skill(root: Path, dest_plugin_root: Path) -> None:
    _copy_path(root / PLUGIN_ROOT / "skills", dest_plugin_root / "skills")


def _copy_commands(root: Path, dest_plugin_root: Path) -> None:
    commands = root / PLUGIN_ROOT / "commands"
    if commands.is_dir():
        _copy_path(commands, dest_plugin_root / "commands")


def _make_tarball(source_dir: Path, tar_path: Path) -> None:
    tar_path.parent.mkdir(parents=True, exist_ok=True)
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tar:
        tar.add(source_dir, arcname=source_dir.name)


def _make_zip(source_dir: Path, zip_path: Path) -> None:
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    if zip_path.exists():
        zip_path.unlink()
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for item in sorted(source_dir.rglob("*")):
            if item.is_file():
                archive.write(item, item.relative_to(source_dir.parent).as_posix())


def _copy_claude_desktop_skill(root: Path, skill_dest: Path, subject: str) -> None:
    materialize_subject_package(
        MaterializeOptions(
            source=root,
            out=skill_dest,
            subject=subject,
            flavor="desktop",
        )
    )
    file_count = sum(1 for item in skill_dest.rglob("*") if item.is_file())
    if file_count > DESKTOP_SKILL_FILE_BUDGET:
        raise ValueError(
            f"Claude Desktop {subject} skill package has {file_count} files; "
            f"limit is {DESKTOP_SKILL_FILE_BUDGET}"
        )


def _build_codex(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-codex-plugin-{tag}"
    bundle = work_dir / bundle_name
    plugin_dest = bundle / PLUGIN_ROOT
    _copy_path(root / PLUGIN_ROOT / ".codex-plugin", plugin_dest / ".codex-plugin")
    _copy_commands(root, plugin_dest)
    _copy_common_skill(root, plugin_dest)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_claude(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-claude-plugin-{tag}"
    bundle = work_dir / bundle_name
    plugin_dest = bundle / PLUGIN_ROOT
    _copy_path(root / PLUGIN_ROOT / ".claude-plugin", plugin_dest / ".claude-plugin")
    _copy_commands(root, plugin_dest)
    _copy_common_skill(root, plugin_dest)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_gemini(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-gemini-extension-{tag}"
    bundle = work_dir / bundle_name
    _copy_path(root / PLUGIN_ROOT / "gemini-extension.json", bundle / "gemini-extension.json")
    _copy_common_skill(root, bundle)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_claude_desktop_skill(root: Path, tag: str, dist_dir: Path, work_dir: Path, subject: str) -> Path:
    bundle_name = f"{PLUGIN_NAME}-claude-desktop-skill-{subject}-{tag}"
    skill_dest = work_dir / f"desktop-{subject}" / "qiongli"
    _copy_claude_desktop_skill(root, skill_dest, subject)
    artifact = dist_dir / f"{bundle_name}.zip"
    _make_zip(skill_dest, artifact)
    return artifact


def _desktop_subjects(root: Path) -> list[str]:
    subjects = sorted(validate_subject_catalog(root).subjects)
    if "core" in subjects:
        subjects.remove("core")
        subjects.insert(0, "core")
    return subjects


def build_artifacts(root: Path, raw_tag: str, dist_dir: Path) -> list[Path]:
    root = root.resolve()
    dist_dir = dist_dir.resolve()
    repo_tag, skill_version = _normalize_tag(raw_tag)

    workflow_version = (root / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip()
    if workflow_version != repo_tag:
        raise ValueError(f"version mismatch in qiongli-workflow/VERSION: expected {repo_tag}, found {workflow_version}")

    versioned_json = [
        root / PLUGIN_ROOT / ".codex-plugin" / "plugin.json",
        root / PLUGIN_ROOT / ".claude-plugin" / "plugin.json",
        root / PLUGIN_ROOT / "gemini-extension.json",
    ]
    for path in versioned_json:
        _assert_json_versions(path, skill_version)

    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-") as tmp:
        work_dir = Path(tmp)
        desktop_artifacts = [
            _build_claude_desktop_skill(root, repo_tag, dist_dir, work_dir, subject)
            for subject in _desktop_subjects(root)
        ]
        legacy_desktop_artifact = dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-{repo_tag}.zip"
        shutil.copy2(dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-core-{repo_tag}.zip", legacy_desktop_artifact)
        artifacts = [
            _build_codex(root, repo_tag, dist_dir, work_dir),
            _build_claude(root, repo_tag, dist_dir, work_dir),
            _build_gemini(root, repo_tag, dist_dir, work_dir),
            *desktop_artifacts,
            legacy_desktop_artifact,
        ]
    return artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Codex, Claude Code, and Gemini plugin/extension artifacts.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v0.5.0-beta.3")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="Repository root")
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"), help="Output directory")
    args = parser.parse_args(argv)

    artifacts = build_artifacts(args.root, args.tag, args.dist_dir)
    print("[plugin-artifacts] built")
    for artifact in artifacts:
        print(f"  - {artifact}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as exc:
        print(f"[plugin-artifacts] {exc}")
        raise SystemExit(2)
