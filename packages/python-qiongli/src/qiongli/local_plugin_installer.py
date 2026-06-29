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
from qiongli.workflow_wrapper_skills import write_codex_workflow_wrapper_skills


PLUGIN_ID = "qiongli"
SKILL_DIR_NAME = "qiongli-workflow"
QIONGLI_MCP_ARGS = ["mcp", "serve", "--transport", "stdio"]
DEFAULT_CODEX_MARKETPLACE_PATH = Path("~/.agents/plugins/marketplace.json")
DEFAULT_CLAUDE_MARKETPLACE_ROOT = Path("~/.qiongli/plugins/claude-code")
DEFAULT_ANTIGRAVITY_PLUGIN_PARENT = Path("~/.qiongli/plugins/antigravity")
CLAUDE_MARKETPLACE_NAME = "qiongli-local"
MANAGED_MARKER_NAME = ".qiongli-managed.json"


@dataclass(frozen=True)
class CodexPluginPaths:
    marketplace_path: Path
    plugin_root: Path
    marketplace_source_path: str


@dataclass(frozen=True)
class ClaudePluginPaths:
    marketplace_root: Path
    marketplace_path: Path
    plugin_root: Path
    marketplace_name: str
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
    antigravity_plugin_parent: Path | None = None


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
    marketplace_root = _codex_marketplace_root(marketplace)
    return CodexPluginPaths(
        marketplace_path=marketplace,
        plugin_root=marketplace_root / "plugins" / plugin_id,
        marketplace_source_path=f"./plugins/{plugin_id}",
    )


def resolve_claude_plugin_paths(
    plugin_id: str = PLUGIN_ID,
    marketplace_root: Path | None = None,
) -> ClaudePluginPaths:
    root = (
        Path(marketplace_root)
        if marketplace_root is not None
        else Path(
            os.environ.get(
                "QIONGLI_CLAUDE_MARKETPLACE_ROOT",
                os.environ.get("QIONGLI_CLAUDE_PLUGIN_PARENT", DEFAULT_CLAUDE_MARKETPLACE_ROOT),
            )
        ).expanduser()
    )
    return ClaudePluginPaths(
        marketplace_root=root,
        marketplace_path=root / ".claude-plugin" / "marketplace.json",
        plugin_root=root / "plugins" / plugin_id,
        marketplace_name=CLAUDE_MARKETPLACE_NAME,
        marketplace_source_path=f"./plugins/{plugin_id}",
    )


def resolve_antigravity_plugin_root(parent: Path | None = None, plugin_id: str = PLUGIN_ID) -> Path:
    parent_path = parent
    if parent_path is None:
        parent_path = Path(os.environ.get("QIONGLI_ANTIGRAVITY_PLUGIN_PARENT", DEFAULT_ANTIGRAVITY_PLUGIN_PARENT))
    return Path(parent_path).expanduser() / plugin_id


def install_local_plugin(options: LocalPluginOptions) -> LocalPluginResult:
    targets = _selected_targets(options.target)
    if not targets:
        return LocalPluginResult(installed_roots={}, changed=False)

    installed_roots: dict[str, Path] = {}
    if "codex" in targets:
        codex_paths = resolve_codex_plugin_paths(marketplace_path=options.codex_marketplace_path)
        installed_roots["codex"] = codex_paths.plugin_root
    if "claude" in targets:
        installed_roots["claude"] = resolve_claude_plugin_paths(marketplace_root=options.claude_plugin_parent).plugin_root
    if "antigravity" in targets:
        installed_roots["antigravity"] = resolve_antigravity_plugin_root(options.antigravity_plugin_parent)

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
        claude_paths = resolve_claude_plugin_paths(marketplace_root=options.claude_plugin_parent)
        _prepare_destination(claude_paths.plugin_root, options.overwrite)
        _materialize_plugin_root(
            repo_root=repo_root,
            plugin_root=claude_paths.plugin_root,
            plugin=plugin,
            version=version,
            platform="claude",
            subject=options.subject,
            coverage=options.coverage,
        )
        _write_claude_marketplace_entry(claude_paths, plugin, version)
        changed = True

    if "antigravity" in targets:
        antigravity_root = resolve_antigravity_plugin_root(options.antigravity_plugin_parent)
        _prepare_destination(antigravity_root, options.overwrite)
        _materialize_plugin_root(
            repo_root=repo_root,
            plugin_root=antigravity_root,
            plugin=plugin,
            version=version,
            platform="antigravity",
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
        codex_root_exists = codex_paths.plugin_root.exists() or codex_paths.plugin_root.is_symlink()
        removed_codex_root = _remove_managed_root(codex_paths.plugin_root, dry_run=dry_run)
        removed_codex_marketplace = (
            _remove_codex_marketplace_entry(codex_paths, dry_run=dry_run)
            if removed_codex_root or not codex_root_exists
            else False
        )
        if removed_codex_root:
            removed += 1
        elif removed_codex_marketplace:
            removed += 1

    if "claude" in targets:
        claude_paths = resolve_claude_plugin_paths(marketplace_root=claude_plugin_parent)
        claude_root_exists = claude_paths.plugin_root.exists() or claude_paths.plugin_root.is_symlink()
        removed_claude_root = _remove_managed_root(claude_paths.plugin_root, dry_run=dry_run)
        removed_claude_marketplace = (
            _remove_claude_marketplace_entry(claude_paths, dry_run=dry_run)
            if removed_claude_root or not claude_root_exists
            else False
        )
        if removed_claude_root:
            removed += 1
        elif removed_claude_marketplace:
            removed += 1

    if "antigravity" in targets:
        antigravity_root = resolve_antigravity_plugin_root()
        if _remove_managed_root(antigravity_root, dry_run=dry_run):
            removed += 1

    return removed


def _selected_targets(target: str) -> tuple[str, ...]:
    if target == "all":
        return ("codex", "claude", "antigravity")
    if target in {"codex", "claude", "antigravity"}:
        return (target,)
    return ()


def _codex_marketplace_root(marketplace_path: Path) -> Path:
    marketplace = Path(marketplace_path).expanduser()
    if (
        marketplace.name == "marketplace.json"
        and marketplace.parent.name == "plugins"
        and marketplace.parent.parent.name == ".agents"
    ):
        return marketplace.parent.parent.parent
    return marketplace.parent


def resolve_claude_plugin_root(parent: Path | None = None) -> Path:
    return resolve_claude_plugin_paths(marketplace_root=parent).plugin_root


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
    elif platform == "antigravity":
        _write_json(plugin_root / "plugin.json", _antigravity_manifest(plugin, version))
        _write_json(plugin_root / "mcp_config.json", _antigravity_mcp_manifest(plugin))
    else:
        raise ValueError(f"unsupported local plugin platform: {platform}")

    _write_json(plugin_root / MANAGED_MARKER_NAME, _managed_marker(platform=platform, version=version))
    _generate_commands(repo_root, plugin_root / "commands", plugin.skill_name)
    _materialize_skill(repo_root, plugin_root / "skills" / SKILL_DIR_NAME, subject=subject, coverage=coverage)
    if platform == "codex":
        _generate_codex_workflow_wrapper_skills(repo_root, plugin_root / "skills", plugin.skill_name)


def _codex_manifest(plugin: PluginDefinition, version: str) -> dict[str, Any]:
    return {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
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
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "claude-code-plugins"),
        "mcpServers": {
            plugin.mcp_server_name: {
                "type": "stdio",
                "command": "qiongli",
                "args": QIONGLI_MCP_ARGS,
            }
        },
    }


def _antigravity_manifest(plugin: PluginDefinition, version: str) -> dict[str, Any]:
    return {
        "name": plugin.id,
        "description": plugin.description,
        "version": version,
    }


def _codex_mcp_manifest(plugin: PluginDefinition) -> dict[str, Any]:
    return {
        "mcpServers": {
            plugin.mcp_server_name: {
                "command": "qiongli",
                "args": QIONGLI_MCP_ARGS,
                "startup_timeout_sec": 20,
                "tool_timeout_sec": 120,
            }
        }
    }


def _antigravity_mcp_manifest(plugin: PluginDefinition) -> dict[str, Any]:
    return {
        "mcpServers": {
            plugin.mcp_server_name: {
                "command": "qiongli",
                "args": QIONGLI_MCP_ARGS,
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


def _generate_codex_workflow_wrapper_skills(repo_root: Path, skills_root: Path, skill_name: str) -> None:
    write_codex_workflow_wrapper_skills(
        RepoLayout(repo_root).workflow / "workflows",
        skills_root,
        skill_name=skill_name,
        canonical_skill_dir=SKILL_DIR_NAME,
    )


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
        dest = source / SKILL_DIR_NAME / name
        if path.exists() and not dest.exists():
            _copy_path(path, dest)
    for name, path in {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }.items():
        dest = source / SKILL_DIR_NAME / name
        if path.exists() and not dest.exists():
            _copy_path(path, dest)
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
    marker_path = plugin_root / MANAGED_MARKER_NAME
    if not marker_path.is_file():
        return False
    try:
        marker = json.loads(marker_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    if not isinstance(marker, dict):
        return False
    return (
        marker.get("managed_by") == "qiongli-cli"
        and marker.get("plugin") == PLUGIN_ID
        and marker.get("surface") == "plugin"
    )


def _managed_marker(*, platform: str, version: str) -> dict[str, Any]:
    return {
        "managed_by": "qiongli-cli",
        "plugin": PLUGIN_ID,
        "surface": "plugin",
        "platform": platform,
        "version": version,
        "mcp": {
            "command": "qiongli",
            "args": QIONGLI_MCP_ARGS,
        },
    }


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
        "metadata": {
            "managedBy": "qiongli-cli",
            "surface": "plugin",
        },
    }
    _upsert_marketplace_plugin(plugins, entry)
    marketplace["plugins"] = plugins
    _write_json(paths.marketplace_path, marketplace)


def _write_claude_marketplace_entry(paths: ClaudePluginPaths, plugin: PluginDefinition, version: str) -> None:
    marketplace = _read_claude_marketplace(paths.marketplace_path, paths.marketplace_name, plugin)
    plugins = _marketplace_plugins_list(marketplace)
    entry = {
        "name": PLUGIN_ID,
        "description": plugin.description,
        "version": version,
        "author": plugin.author,
        "source": paths.marketplace_source_path,
        "category": plugin.category,
        "strict": False,
    }
    _upsert_marketplace_plugin(plugins, entry)
    marketplace["plugins"] = plugins
    _write_json(paths.marketplace_path, marketplace)


def _remove_codex_marketplace_entry(paths: CodexPluginPaths, *, dry_run: bool = False) -> bool:
    if not paths.marketplace_path.is_file():
        return False
    marketplace = _read_marketplace(paths.marketplace_path)
    plugins = _marketplace_plugins_list(marketplace)
    filtered_plugins = [
        entry for entry in plugins if not _is_managed_codex_marketplace_entry(entry, paths.marketplace_source_path)
    ]
    if len(filtered_plugins) == len(plugins):
        return False
    if not dry_run:
        marketplace["plugins"] = filtered_plugins
        _write_json(paths.marketplace_path, marketplace)
    return True


def _remove_claude_marketplace_entry(paths: ClaudePluginPaths, *, dry_run: bool = False) -> bool:
    if not paths.marketplace_path.is_file():
        return False
    marketplace = _read_claude_marketplace(paths.marketplace_path, paths.marketplace_name, None)
    plugins = _marketplace_plugins_list(marketplace)
    filtered_plugins = [
        entry for entry in plugins if not _is_managed_claude_marketplace_entry(entry, paths.marketplace_source_path)
    ]
    if len(filtered_plugins) == len(plugins):
        return False
    if not dry_run:
        marketplace["plugins"] = filtered_plugins
        _write_json(paths.marketplace_path, marketplace)
    return True


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


def _read_claude_marketplace(path: Path, name: str, plugin: PluginDefinition | None) -> dict[str, Any]:
    if not path.is_file():
        owner = plugin.author if plugin is not None else {"name": "Qiongli"}
        return {
            "$schema": "https://anthropic.com/claude-code/marketplace.schema.json",
            "name": name,
            "description": "Local Qiongli CLI-managed Claude Code plugins.",
            "owner": owner,
            "plugins": [],
        }
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"malformed Claude marketplace JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"Claude marketplace must be a JSON object: {path}")
    payload.setdefault("name", name)
    payload.setdefault("plugins", [])
    return payload


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


def _is_managed_codex_marketplace_entry(entry: object, marketplace_source_path: str) -> bool:
    if not isinstance(entry, dict) or entry.get("name") != PLUGIN_ID:
        return False
    source = entry.get("source")
    if not isinstance(source, dict):
        return False
    metadata = entry.get("metadata")
    if not isinstance(metadata, dict):
        return False
    return (
        source.get("source") == "local"
        and source.get("path") == marketplace_source_path
        and metadata.get("managedBy") == "qiongli-cli"
        and metadata.get("surface") == "plugin"
    )


def _is_managed_claude_marketplace_entry(entry: object, marketplace_source_path: str) -> bool:
    return isinstance(entry, dict) and entry.get("name") == PLUGIN_ID and entry.get("source") == marketplace_source_path


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _copy_path(src: Path, dest: Path) -> None:
    if src.is_dir():
        shutil.copytree(src, dest, symlinks=False)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
