#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import tarfile
import tempfile
from pathlib import Path


PLUGIN_NAME = "research-skills"
PLUGIN_ROOT = Path("plugins") / PLUGIN_NAME


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


def _make_tarball(source_dir: Path, tar_path: Path) -> None:
    tar_path.parent.mkdir(parents=True, exist_ok=True)
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tar:
        tar.add(source_dir, arcname=source_dir.name)


def _build_codex(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-codex-plugin-{tag}"
    bundle = work_dir / bundle_name
    _copy_path(root / ".agents" / "plugins" / "marketplace.json", bundle / ".agents" / "plugins" / "marketplace.json")
    plugin_dest = bundle / PLUGIN_ROOT
    _copy_path(root / PLUGIN_ROOT / ".codex-plugin", plugin_dest / ".codex-plugin")
    _copy_common_skill(root, plugin_dest)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_claude(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-claude-plugin-{tag}"
    bundle = work_dir / bundle_name
    _copy_path(root / ".claude-plugin" / "marketplace.json", bundle / ".claude-plugin" / "marketplace.json")
    plugin_dest = bundle / PLUGIN_ROOT
    _copy_path(root / PLUGIN_ROOT / ".claude-plugin", plugin_dest / ".claude-plugin")
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


def build_artifacts(root: Path, raw_tag: str, dist_dir: Path) -> list[Path]:
    root = root.resolve()
    dist_dir = dist_dir.resolve()
    repo_tag, skill_version = _normalize_tag(raw_tag)

    workflow_version = (root / "research-paper-workflow" / "VERSION").read_text(encoding="utf-8").strip()
    if workflow_version != repo_tag:
        raise ValueError(f"version mismatch in research-paper-workflow/VERSION: expected {repo_tag}, found {workflow_version}")

    versioned_json = [
        root / PLUGIN_ROOT / ".codex-plugin" / "plugin.json",
        root / ".claude-plugin" / "marketplace.json",
        root / PLUGIN_ROOT / ".claude-plugin" / "plugin.json",
        root / PLUGIN_ROOT / "gemini-extension.json",
    ]
    for path in versioned_json:
        _assert_json_versions(path, skill_version)

    with tempfile.TemporaryDirectory(prefix="research-skills-marketplace-") as tmp:
        work_dir = Path(tmp)
        artifacts = [
            _build_codex(root, repo_tag, dist_dir, work_dir),
            _build_claude(root, repo_tag, dist_dir, work_dir),
            _build_gemini(root, repo_tag, dist_dir, work_dir),
        ]
    return artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Codex, Claude Code, and Gemini marketplace/extension artifacts.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v0.5.0-beta.3")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="Repository root")
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"), help="Output directory")
    args = parser.parse_args(argv)

    artifacts = build_artifacts(args.root, args.tag, args.dist_dir)
    print("[marketplace-artifacts] built")
    for artifact in artifacts:
        print(f"  - {artifact}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as exc:
        print(f"[marketplace-artifacts] {exc}")
        raise SystemExit(2)
