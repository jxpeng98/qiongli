from __future__ import annotations

import json
import os
import re
import shutil
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from qiongli.distribution_metadata import PluginDefinition, load_plugin_distribution
from qiongli.source_layout import RepoLayout
from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package


PLUGIN_ID = "qiongli"
SKILL_DIR_NAME = "qiongli-workflow"
DEFAULT_CODEX_MARKETPLACE_PATH = Path("~/.agents/plugins/marketplace.json")
DEFAULT_CLAUDE_PLUGIN_PARENT = Path("~/.qiongli/plugins/claude-code")


@dataclass(frozen=True)
class CodexPluginPaths:
    marketplace_path: Path
    plugin_root: Path
    marketplace_source_path: str


@dataclass(frozen=True)
class LocalPluginOptions:
    repo_root: Path
    subject: str = "core"
    coverage: str = "complete"
    target: str = "all"
    mode: str = "copy"
    overwrite: bool = False
    dry_run: bool = False
    codex_marketplace_path: Path | None = None
    claude_plugin_parent: Path | None = None


@dataclass(frozen=True)
class LocalPluginResult:
    installed_roots: dict[str, Path] = field(default_factory=dict)
    changed: bool = False


def resolve_codex_plugin_paths(plugin_id: str = PLUGIN_ID, marketplace_path: Path | None = None) -> CodexPluginPaths:
    marketplace = (
        Path(marketplace_path)
        if marketplace_path is not None
        else Path(os.environ.get("QIONGLI_CODEX_MARKETPLACE_PATH", DEFAULT_CODEX_MARKETPLACE_PATH)).expanduser()
    )
    return CodexPluginPaths(
        marketplace_path=marketplace,
        plugin_root=marketplace.parent / "plugins" / plugin_id,
        marketplace_source_path=f"./plugins/{plugin_id}",
    )


def install_local_plugin(options: LocalPluginOptions) -> LocalPluginResult:
    targets = _selected_targets(options.target)
    if not targets:
        return LocalPluginResult(installed_roots={}, changed=False)

    installed_roots: dict[str, Path] = {}
    if "codex" in targets:
        codex_paths = resolve_codex_plugin_paths(marketplace_path=options.codex_marketplace_path)
        installed_roots["codex"] = codex_paths.plugin_root
    if "claude" in targets:
        installed_roots["claude"] = _claude_plugin_root(options.claude_plugin_parent)

    if options.dry_run:
        return LocalPluginResult(installed_roots=installed_roots, changed=False)

    repo_root = Path(options.repo_root).resolve()
    plugin = load_plugin_distribution(repo_root).plugins[PLUGIN_ID]
    version = _skill_version(repo_root)
    changed = False

    if "codex" in targets:
        codex_paths = resolve_codex_plugin_paths(marketplace_path=options.codex_marketplace_path)
        _prepare_destination(codex_paths.plugin_root, options.overwrite)
        _materialize_plugin_root(
            repo_root=repo_root,
            plugin_root=codex_paths.plugin_root,
            plugin=plugin,
            version=version,
            platform="codex",
            subject=options.subject,
            coverage=options.coverage,
        )
        _write_codex_marketplace_entry(codex_paths, plugin)
        changed = True

    if "claude" in targets:
        claude_root = _claude_plugin_root(options.claude_plugin_parent)
        _prepare_destination(claude_root, options.overwrite)
        _materialize_plugin_root(
            repo_root=repo_root,
            plugin_root=claude_root,
            plugin=plugin,
            version=version,
            platform="claude",
            subject=options.subject,
            coverage=options.coverage,
        )
        changed = True

    return LocalPluginResult(installed_roots=installed_roots, changed=changed)


def remove_local_plugin(
    target: str = "all",
    dry_run: bool = False,
    codex_marketplace_path: Path | None = None,
    claude_plugin_parent: Path | None = None,
) -> int:
    targets = _selected_targets(target)
    if not targets:
        return 0

    removed = 0
    if "codex" in targets:
        codex_paths = resolve_codex_plugin_paths(marketplace_path=codex_marketplace_path)
        if _remove_managed_root(codex_paths.plugin_root, dry_run=dry_run):
            removed += 1
        if not dry_run:
            _remove_codex_marketplace_entry(codex_paths.marketplace_path)

    if "claude" in targets:
        claude_root = _claude_plugin_root(claude_plugin_parent)
        if _remove_managed_root(claude_root, dry_run=dry_run):
            removed += 1

    return removed


def _selected_targets(target: str) -> tuple[str, ...]:
    if target == "all":
        return ("codex", "claude")
    if target in {"codex", "claude"}:
        return (target,)
    return ()


def _claude_plugin_root(parent: Path | None) -> Path:
    parent_path = parent
    if parent_path is None:
        parent_path = Path(os.environ.get("QIONGLI_CLAUDE_PLUGIN_PARENT", DEFAULT_CLAUDE_PLUGIN_PARENT))
    return Path(parent_path).expanduser() / PLUGIN_ID


def _skill_version(repo_root: Path) -> str:
    return (RepoLayout(repo_root).workflow / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")


def _materialize_plugin_root(
    *,
    repo_root: Path,
    plugin_root: Path,
    plugin: PluginDefinition,
    version: str,
    platform: str,
    subject: str,
    coverage: str,
) -> None:
    if platform == "codex":
        _write_json(plugin_root / ".codex-plugin" / "plugin.json", _codex_manifest(plugin, version))
        _write_json(plugin_root / ".mcp.json", _codex_mcp_manifest(plugin))
    elif platform == "claude":
        _write_json(plugin_root / ".claude-plugin" / "plugin.json", _claude_manifest(plugin, version))
    else:
        raise ValueError(f"unsupported local plugin platform: {platform}")

    _generate_commands(repo_root, plugin_root / "commands", plugin.skill_name)
    _materialize_skill(repo_root, plugin_root / "skills" / SKILL_DIR_NAME, subject=subject, coverage=coverage)


def _codex_manifest(plugin: PluginDefinition, version: str) -> dict[str, Any]:
    return {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "codex-skills"),
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": plugin.display_name,
            "shortDescription": plugin.codex_short_description,
            "longDescription": plugin.description,
            "developerName": plugin.author["name"],
            "category": plugin.category,
            "capabilities": ["Write"],
            "websiteURL": plugin.repository,
            "defaultPrompt": list(plugin.default_prompts),
            "brandColor": plugin.brand_color,
        },
    }


def _claude_manifest(plugin: PluginDefinition, version: str) -> dict[str, Any]:
    return {
        "name": plugin.id,
        "description": plugin.description,
        "version": version,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "claude-code-plugins"),
        "mcpServers": {
            plugin.mcp_server_name: {
                "type": "stdio",
                "command": "qiongli",
                "args": ["mcp", "serve", "--transport", "stdio"],
            }
        },
    }


def _codex_mcp_manifest(plugin: PluginDefinition) -> dict[str, Any]:
    return {
        "mcpServers": {
            plugin.mcp_server_name: {
                "command": "qiongli",
                "args": ["mcp", "serve", "--transport", "stdio"],
                "startup_timeout_sec": 20,
                "tool_timeout_sec": 120,
            }
        }
    }


def _keywords(plugin: PluginDefinition, platform_keyword: str) -> list[str]:
    return [*plugin.keywords, *[item for item in (platform_keyword,) if item not in plugin.keywords]]


def _workflow_description(workflow_path: Path) -> str:
    text = workflow_path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)^---\n(.*?)\n---", text)
    if not match:
        return f"Run the {workflow_path.stem} research workflow."
    desc = re.search(r"(?m)^description:\s*(.+)$", match.group(1))
    return desc.group(1).strip() if desc else f"Run the {workflow_path.stem} research workflow."


def _generate_commands(repo_root: Path, commands_root: Path, skill_name: str) -> None:
    workflow_root = RepoLayout(repo_root).workflow / "workflows"
    commands_root.mkdir(parents=True, exist_ok=True)
    for workflow_path in sorted(workflow_root.glob("*.md")):
        text = "\n".join(
            [
                "---",
                f"description: {_workflow_description(workflow_path)}",
                "---",
                "",
                (
                    f"Load the `{skill_name}` skill from this plugin, then follow "
                    f"`skills/{SKILL_DIR_NAME}/workflows/{workflow_path.name}`."
                ),
                "",
                "Use that workflow as the source of truth for task order, artifacts, and quality gates.",
                "",
            ]
        )
        (commands_root / workflow_path.name).write_text(text, encoding="utf-8")


def _materialize_skill(repo_root: Path, skill_dest: Path, *, subject: str, coverage: str) -> None:
    with tempfile.TemporaryDirectory(prefix="qiongli-local-plugin-source-") as tmp:
        materialize_root = _build_materialize_source(repo_root, Path(tmp))
        materialize_subject_package(
            MaterializeOptions(
                source=materialize_root,
                out=skill_dest,
                subject=subject,
                flavor="full",
                coverage=coverage,
            )
        )


def _build_materialize_source(repo_root: Path, work_dir: Path) -> Path:
    layout = RepoLayout(repo_root)
    source = work_dir / "materialize-source"
    _copy_path(layout.workflow, source / SKILL_DIR_NAME)
    _copy_path(layout.skills, source / "skills")
    _copy_path(layout.subjects, source / "subjects")
    for name, path in {
        "skills": layout.skills,
        "templates": layout.templates,
        "standards": layout.standards,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
    }.items():
        if path.exists():
            _copy_path(path, source / SKILL_DIR_NAME / name)
    for name, path in {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }.items():
        if path.exists():
            _copy_path(path, source / SKILL_DIR_NAME / name)
    return source


def _prepare_destination(plugin_root: Path, overwrite: bool) -> None:
    if not plugin_root.exists():
        plugin_root.parent.mkdir(parents=True, exist_ok=True)
        return
    if not _is_managed_plugin_root(plugin_root) and not overwrite:
        raise FileExistsError(f"{plugin_root} exists and is not a managed qiongli plugin root")
    if plugin_root.is_dir():
        shutil.rmtree(plugin_root)
    else:
        plugin_root.unlink()
    plugin_root.parent.mkdir(parents=True, exist_ok=True)


def _is_managed_plugin_root(plugin_root: Path) -> bool:
    if not plugin_root.exists() or not plugin_root.is_dir():
        return False
    for manifest_path in (
        plugin_root / ".codex-plugin" / "plugin.json",
        plugin_root / ".claude-plugin" / "plugin.json",
    ):
        if _manifest_name(manifest_path) == PLUGIN_ID:
            return True
    return False


def _manifest_name(manifest_path: Path) -> str | None:
    if not manifest_path.is_file():
        return None
    try:
        payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    name = payload.get("name") if isinstance(payload, dict) else None
    return name if isinstance(name, str) else None


def _remove_managed_root(plugin_root: Path, *, dry_run: bool) -> bool:
    if not _is_managed_plugin_root(plugin_root):
        return False
    if not dry_run:
        shutil.rmtree(plugin_root)
    return True


def _write_codex_marketplace_entry(paths: CodexPluginPaths, plugin: PluginDefinition) -> None:
    marketplace = _read_marketplace(paths.marketplace_path)
    plugins = _marketplace_plugins_list(marketplace)
    entry = {
        "name": PLUGIN_ID,
        "source": {
            "source": "local",
            "path": paths.marketplace_source_path,
        },
        "policy": {
            "installation": "AVAILABLE",
            "authentication": "ON_INSTALL",
        },
        "category": plugin.category,
    }
    _upsert_marketplace_plugin(plugins, entry)
    marketplace["plugins"] = plugins
    _write_json(paths.marketplace_path, marketplace)


def _remove_codex_marketplace_entry(marketplace_path: Path) -> None:
    if not marketplace_path.is_file():
        return
    marketplace = _read_marketplace(marketplace_path)
    plugins = _marketplace_plugins_list(marketplace)
    marketplace["plugins"] = [
        entry for entry in plugins if not (isinstance(entry, dict) and entry.get("name") == PLUGIN_ID)
    ]
    _write_json(marketplace_path, marketplace)


def _read_marketplace(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return _default_marketplace()
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"malformed Codex marketplace JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must contain a JSON object")
    payload.setdefault("name", "personal")
    payload.setdefault("interface", {"displayName": "Personal"})
    if "plugins" not in payload:
        payload["plugins"] = []
    return payload


def _default_marketplace() -> dict[str, Any]:
    return {
        "name": "personal",
        "interface": {"displayName": "Personal"},
        "plugins": [],
    }


def _marketplace_plugins_list(marketplace: dict[str, Any]) -> list[Any]:
    plugins = marketplace.get("plugins")
    if isinstance(plugins, list):
        return list(plugins)
    if isinstance(plugins, dict):
        normalized: list[Any] = []
        for name, entry in plugins.items():
            if isinstance(entry, dict):
                normalized.append({**entry, "name": name})
        return normalized
    raise ValueError("Codex marketplace plugins must be a list or object")


def _upsert_marketplace_plugin(plugins: list[Any], entry: dict[str, Any]) -> None:
    for index, candidate in enumerate(plugins):
        if isinstance(candidate, dict) and candidate.get("name") == PLUGIN_ID:
            plugins[index] = entry
            return
    plugins.append(entry)


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _copy_path(src: Path, dest: Path) -> None:
    if src.is_dir():
        shutil.copytree(src, dest, symlinks=False)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
