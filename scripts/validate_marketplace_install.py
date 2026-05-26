#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import tarfile
import tempfile
import zipfile
from dataclasses import dataclass
from pathlib import Path

from build_plugin_artifacts import build_artifacts


PLUGIN_NAME = "qiongli"
SKILL_DIR_NAME = "qiongli-workflow"
SKILL_NAME = "qiongli"


@dataclass(frozen=True)
class ArtifactSpec:
    platform: str
    manifest: Path
    plugin_root: Path
    requires_commands: bool


ARTIFACT_SPECS = {
    "codex": ArtifactSpec(
        platform="codex",
        manifest=Path("plugins") / PLUGIN_NAME / ".codex-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
    ),
    "claude": ArtifactSpec(
        platform="claude",
        manifest=Path("plugins") / PLUGIN_NAME / ".claude-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
    ),
    "gemini": ArtifactSpec(
        platform="gemini",
        manifest=Path("gemini-extension.json"),
        plugin_root=Path("."),
        requires_commands=False,
    ),
}


def _read_json(path: Path) -> dict[str, object]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return data


def _extract_single_root(artifact: Path, dest: Path) -> Path:
    with tarfile.open(artifact, "r:gz") as tar:
        tar.extractall(dest, filter="data")
    roots = [item for item in dest.iterdir() if item.is_dir()]
    if len(roots) != 1:
        raise ValueError(f"{artifact} should extract to one top-level directory, found {len(roots)}")
    return roots[0]


def _extract_single_zip_root(artifact: Path, dest: Path) -> Path:
    with zipfile.ZipFile(artifact) as archive:
        archive.extractall(dest)
    roots = [item for item in dest.iterdir() if item.is_dir()]
    if len(roots) != 1:
        raise ValueError(f"{artifact} should extract to one top-level directory, found {len(roots)}")
    return roots[0]


def _assert_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise ValueError(f"missing {label}: {path}")


def _assert_dir(path: Path, label: str) -> None:
    if not path.is_dir():
        raise ValueError(f"missing {label}: {path}")


def _assert_skill_invocation(skill_root: Path, expected_repo_tag: str) -> list[str]:
    _assert_file(skill_root / "SKILL.md", "skill entrypoint")
    _assert_file(skill_root / "VERSION", "skill version")
    _assert_file(skill_root / "skills" / "registry.yaml", "skill registry")
    _assert_dir(skill_root / "workflows", "workflow directory")

    skill_text = (skill_root / "SKILL.md").read_text(encoding="utf-8")
    if f"name: {SKILL_NAME}" not in skill_text:
        raise ValueError(f"{skill_root / 'SKILL.md'} must declare name: {SKILL_NAME}")

    actual_version = (skill_root / "VERSION").read_text(encoding="utf-8").strip()
    if actual_version != expected_repo_tag:
        raise ValueError(f"{skill_root / 'VERSION'} expected {expected_repo_tag}, found {actual_version}")

    workflow_names = sorted(path.name for path in (skill_root / "workflows").glob("*.md"))
    if not workflow_names:
        raise ValueError(f"{skill_root / 'workflows'} must contain invokable workflows")
    return workflow_names


def _assert_command_invocation(plugin_root: Path, workflow_names: list[str]) -> None:
    commands_dir = plugin_root / "commands"
    _assert_dir(commands_dir, "slash command directory")

    command_names = sorted(path.name for path in commands_dir.glob("*.md"))
    if command_names != workflow_names:
        missing = sorted(set(workflow_names) - set(command_names))
        extra = sorted(set(command_names) - set(workflow_names))
        raise ValueError(f"command/workflow mismatch; missing={missing}, extra={extra}")

    for command_name in command_names:
        command_path = commands_dir / command_name
        command_text = command_path.read_text(encoding="utf-8")
        expected_reference = f"skills/{SKILL_DIR_NAME}/workflows/{command_name}"
        if SKILL_NAME not in command_text or expected_reference not in command_text:
            raise ValueError(f"{command_path} must load {SKILL_NAME} and reference {expected_reference}")


def _assert_manifest(platform: str, manifest_path: Path, expected_version: str) -> None:
    manifest = _read_json(manifest_path)
    for key in ("name", "version", "description"):
        if not manifest.get(key):
            raise ValueError(f"{manifest_path} missing required field: {key}")
    if manifest["name"] != PLUGIN_NAME:
        raise ValueError(f"{manifest_path} expected name {PLUGIN_NAME}, found {manifest['name']}")
    if manifest["version"] != expected_version:
        raise ValueError(f"{manifest_path} expected version {expected_version}, found {manifest['version']}")

    if platform == "codex":
        if manifest.get("skills") != "./skills/":
            raise ValueError(f"{manifest_path} must expose skills via ./skills/")
        interface = manifest.get("interface")
        if not isinstance(interface, dict):
            raise ValueError(f"{manifest_path} missing Codex interface metadata")
        prompts = interface.get("defaultPrompt")
        if not isinstance(prompts, list) or not any(f"${SKILL_NAME}" in str(item) for item in prompts):
            raise ValueError(f"{manifest_path} defaultPrompt must include ${SKILL_NAME}")


def _validate_artifact(artifact: Path, spec: ArtifactSpec, expected_repo_tag: str, expected_version: str) -> str:
    with tempfile.TemporaryDirectory(prefix=f"qiongli-{spec.platform}-artifact-") as tmp:
        bundle_root = _extract_single_root(artifact, Path(tmp))
        plugin_root = (bundle_root / spec.plugin_root).resolve()
        manifest_path = plugin_root / spec.manifest.relative_to(spec.plugin_root)
        skill_root = plugin_root / "skills" / SKILL_DIR_NAME

        _assert_manifest(spec.platform, manifest_path, expected_version)
        workflow_names = _assert_skill_invocation(skill_root, expected_repo_tag)
        if spec.requires_commands:
            _assert_command_invocation(plugin_root, workflow_names)

    return f"[OK] {spec.platform} marketplace artifact: {SKILL_NAME} invocation checked"


def _validate_claude_desktop_artifact(artifact: Path, expected_repo_tag: str) -> str:
    with tempfile.TemporaryDirectory(prefix="qiongli-claude-desktop-artifact-") as tmp:
        skill_root = _extract_single_zip_root(artifact, Path(tmp))
        if skill_root.name != SKILL_NAME:
            raise ValueError(f"{artifact} must contain top-level {SKILL_NAME}/ directory")
        _assert_skill_invocation(skill_root, expected_repo_tag)
        if (skill_root / ".claude-plugin").exists():
            raise ValueError(f"{artifact} must not include Claude Code plugin metadata")
        if (skill_root / "commands").exists():
            raise ValueError(f"{artifact} must not include Claude Code slash command wrappers")

    return f"[OK] claude-desktop skill artifact: {SKILL_NAME} invocation checked"


def validate(root: Path, dist_dir: Path) -> list[str]:
    root = root.resolve()
    dist_dir = dist_dir.resolve()
    expected_repo_tag = (root / SKILL_DIR_NAME / "VERSION").read_text(encoding="utf-8").strip()
    expected_version = expected_repo_tag.removeprefix("v")

    artifacts = build_artifacts(root, expected_repo_tag, dist_dir)
    by_platform = {artifact.name: artifact for artifact in artifacts}
    messages: list[str] = []

    for platform, spec in ARTIFACT_SPECS.items():
        artifact_name = f"{PLUGIN_NAME}-{platform}-plugin-{expected_repo_tag}.tar.gz"
        if platform == "gemini":
            artifact_name = f"{PLUGIN_NAME}-gemini-extension-{expected_repo_tag}.tar.gz"
        artifact = by_platform.get(artifact_name)
        if artifact is None:
            raise ValueError(f"expected {platform} artifact: {artifact_name}")
        messages.append(_validate_artifact(artifact, spec, expected_repo_tag, expected_version))

    desktop_name = f"{PLUGIN_NAME}-claude-desktop-skill-{expected_repo_tag}.zip"
    desktop_artifact = by_platform.get(desktop_name)
    if desktop_artifact is None:
        raise ValueError(f"expected claude-desktop artifact: {desktop_name}")
    messages.append(_validate_claude_desktop_artifact(desktop_artifact, expected_repo_tag))

    return messages


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate marketplace artifacts install Qiongli and expose platform invocation surfaces."
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--dist-dir", type=Path, help="Directory for temporary artifact builds. Defaults to a temp dir.")
    args = parser.parse_args(argv)

    try:
        if args.dist_dir is None:
            with tempfile.TemporaryDirectory(prefix="qiongli-marketplace-validate-") as tmp:
                messages = validate(args.root, Path(tmp))
        else:
            messages = validate(args.root, args.dist_dir)
    except ValueError as exc:
        print(f"[FAIL] marketplace validation: {exc}")
        return 1

    for message in messages:
        print(message)
    print("[OK] marketplace validation completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
