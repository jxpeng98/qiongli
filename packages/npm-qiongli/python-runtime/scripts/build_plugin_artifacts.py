#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path


PLUGIN_NAME = "qiongli"
PLUGIN_ROOT = Path("plugins") / PLUGIN_NAME
DESKTOP_SKILL_FILE_BUDGET = 180


DESKTOP_SKILL_TEXT = """---
name: qiongli
description: Qiongli academic workflow for Codex, Claude Code, and Gemini. Use when a user needs to plan papers, run literature reviews, choose a paper type (empirical, qualitative, systematic review, methods, theory), select a workflow stage, and produce consistent artifacts under RESEARCH/[topic]/ with explicit task IDs, quality gates, and submission-ready outputs.
---

# Qiongli Academic Workflow

Run a model-agnostic paper workflow using shared Task IDs and artifact contracts.

This is the Claude Desktop / Claude.ai slim package. It stays under Claude's uploaded skill ZIP file-count limit while preserving the executable workflow surface: workflows, consolidated skill references, output templates, standards, roles, venue profiles, and contracts.

## Quick Start

1. Ask for `paper_type`: `empirical`, `qualitative`, `systematic-review`, `methods`, or `theory`.
2. Ask for `task_id` from the contract, for example `F3` or `G1`.
3. Read `workflows/paper.md` as the master router, then follow the matching workflow file.
4. Use `skills-summary.md` to identify the relevant capability and `skills-core.md` for the consolidated process details.
5. Write outputs to `RESEARCH/[topic]/` using the paths and templates declared by the workflow and `references/workflow-contract.md`.
6. Apply quality gates before submission-facing, proofread, final writing, code, or presentation outputs.

## Bundled Workflows

Full workflow definitions are included in `workflows/`. The `workflows/paper.md` file is the master router and maps Task IDs A1-K4 to the correct workflow and output contract.

Available workflow entry points include:

```
/paper
/lit-review
/paper-read
/find-gap
/build-framework
/academic-write
/synthesize
/paper-write
/study-design
/ethics-check
/submission-prep
/rebuttal
/code-build
/proofread
/academic-present
```

## Skill Loading Strategy

This Desktop/Web package uses two consolidated references:

1. `skills-summary.md` for quick lookup of skill names and one-line descriptions.
2. `skills-core.md` for process instructions, output formats, and common templates.

Detailed per-skill markdown files are intentionally omitted from this ZIP to keep the package installable in Claude Desktop and Claude.ai. The full Codex, Claude Code, Gemini, and source-repo distributions keep those detailed files for advanced local workflows.

## Required Behavior

- Use canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- When a workflow references `templates/<name>.md`, load the template from `templates/`.
- Use `references/academic-output-rubric.md` for scholarly prose, synthesis, design, review, and submission artifacts.
- Use `references/citation-risk-policy.md` when citation support is material.
- Track central claims with `templates/claim-evidence-ledger.csv` and the evidence rules in `references/evidence-ledger-contract.md`.
- Write stage handoffs from `templates/stage-handoff.md` at high-risk transitions.
- Use `venue-profiles/` when a target venue profile is available; otherwise create a venue gap note.

## Bundled Assets

| Directory / File | Contents |
|------------------|----------|
| `workflows/` | 16 workflow definitions |
| `references/` | Stage playbooks and workflow contracts |
| `skills-summary.md` | Quick-reference skill index |
| `skills-core.md` | Consolidated skill reference |
| `skills/registry.yaml` | Skill metadata registry |
| `templates/` | Output templates for manuscripts, submissions, ethics, evidence, and handoffs |
| `standards/` | Canonical contract YAML and capability map |
| `roles/` | Agent role definitions |
| `venue-profiles/` | Venue expectation profiles |
"""


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


def _copy_claude_desktop_skill(root: Path, skill_dest: Path) -> None:
    source = root / "qiongli-workflow"
    skill_dest.mkdir(parents=True)

    (skill_dest / "SKILL.md").write_text(DESKTOP_SKILL_TEXT, encoding="utf-8")
    for filename in ("VERSION", "skills-core.md", "skills-summary.md"):
        _copy_path(source / filename, skill_dest / filename)

    for dirname in ("workflows", "references", "templates", "standards", "roles", "venue-profiles", "agents"):
        source_path = source / dirname
        if source_path.exists():
            _copy_path(source_path, skill_dest / dirname)

    registry_source = source / "skills" / "registry.yaml"
    _copy_path(registry_source, skill_dest / "skills" / "registry.yaml")

    file_count = sum(1 for item in skill_dest.rglob("*") if item.is_file())
    if file_count > DESKTOP_SKILL_FILE_BUDGET:
        raise ValueError(
            f"Claude Desktop skill package has {file_count} files; "
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


def _build_claude_desktop_skill(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-claude-desktop-skill-{tag}"
    skill_dest = work_dir / "qiongli"
    _copy_claude_desktop_skill(root, skill_dest)
    artifact = dist_dir / f"{bundle_name}.zip"
    _make_zip(skill_dest, artifact)
    return artifact


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
        artifacts = [
            _build_codex(root, repo_tag, dist_dir, work_dir),
            _build_claude(root, repo_tag, dist_dir, work_dir),
            _build_gemini(root, repo_tag, dist_dir, work_dir),
            _build_claude_desktop_skill(root, repo_tag, dist_dir, work_dir),
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
