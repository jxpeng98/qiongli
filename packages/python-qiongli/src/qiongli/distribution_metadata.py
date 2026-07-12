from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

from qiongli.source_layout import RepoLayout


class PluginDistributionError(ValueError):
    """Raised when plugin distribution metadata is missing or malformed."""


@dataclass(frozen=True)
class PluginDefinition:
    id: str
    display_name: str
    skill_name: str
    mcp_server_name: str
    description: str
    author: dict[str, str]
    category: str
    homepage: str
    repository: str
    license: str
    keywords: tuple[str, ...]
    release_lines: tuple[str, ...]
    release_channels: tuple[str, ...]
    planned_release_lines: tuple[str, ...]
    planned_release_channels: tuple[str, ...]
    default_prompts: tuple[str, ...]
    codex_short_description: str
    brand_color: str
    claude_enabled: bool


@dataclass(frozen=True)
class PluginDistribution:
    plugins: dict[str, PluginDefinition]


def load_plugin_distribution(root: Path | str) -> PluginDistribution:
    path = RepoLayout(Path(root)).content / "distribution" / "plugins.yaml"
    if not path.is_file():
        raise PluginDistributionError(f"missing plugin distribution metadata: {path}")

    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as exc:
        raise PluginDistributionError(f"malformed plugin distribution metadata: {path}: {exc}") from exc

    if not isinstance(payload, dict) or not isinstance(payload.get("plugins"), dict):
        raise PluginDistributionError(f"{path} must contain a plugins object")

    plugins: dict[str, PluginDefinition] = {}
    for plugin_id, raw_plugin in payload["plugins"].items():
        if not isinstance(plugin_id, str) or not isinstance(raw_plugin, dict):
            raise PluginDistributionError(f"{path} plugins entries must be objects")
        plugins[plugin_id] = _parse_plugin_definition(path, plugin_id, raw_plugin)

    return PluginDistribution(plugins=plugins)


def _parse_plugin_definition(path: Path, plugin_id: str, raw_plugin: dict[str, Any]) -> PluginDefinition:
    codex = _required_mapping(path, plugin_id, raw_plugin, "codex")
    claude = _required_mapping(path, plugin_id, raw_plugin, "claude")
    author = _required_mapping(path, plugin_id, raw_plugin, "author")

    return PluginDefinition(
        id=plugin_id,
        display_name=_required_string(path, plugin_id, raw_plugin, "display_name"),
        skill_name=_required_string(path, plugin_id, raw_plugin, "skill_name"),
        mcp_server_name=_required_string(path, plugin_id, raw_plugin, "mcp_server_name"),
        description=_required_string(path, plugin_id, raw_plugin, "description"),
        author={
            "name": _required_string(path, plugin_id, author, "author.name"),
            "url": _required_string(path, plugin_id, author, "author.url"),
        },
        category=_required_string(path, plugin_id, raw_plugin, "category"),
        homepage=_required_string(path, plugin_id, raw_plugin, "homepage"),
        repository=_required_string(path, plugin_id, raw_plugin, "repository"),
        license=_required_string(path, plugin_id, raw_plugin, "license"),
        keywords=_required_string_tuple(path, plugin_id, raw_plugin, "keywords"),
        release_lines=_required_choice_tuple(
            path,
            plugin_id,
            raw_plugin,
            "release_lines",
            {"legacy-1x", "native-2x"},
        ),
        release_channels=_required_choice_tuple(
            path,
            plugin_id,
            raw_plugin,
            "release_channels",
            {"alpha", "beta", "stable"},
        ),
        planned_release_lines=_optional_choice_tuple(
            path,
            plugin_id,
            raw_plugin,
            "planned_release_lines",
            {"legacy-1x", "native-2x"},
        ),
        planned_release_channels=_optional_choice_tuple(
            path,
            plugin_id,
            raw_plugin,
            "planned_release_channels",
            {"alpha", "beta", "stable"},
        ),
        default_prompts=_required_string_tuple(path, plugin_id, codex, "codex.default_prompts"),
        codex_short_description=_required_string(path, plugin_id, codex, "codex.short_description"),
        brand_color=_required_string(path, plugin_id, codex, "codex.brand_color"),
        claude_enabled=_required_bool(path, plugin_id, claude, "claude.enabled"),
    )


def _required_mapping(path: Path, plugin_id: str, container: dict[str, Any], field: str) -> dict[str, Any]:
    value = _required_value(path, plugin_id, container, field)
    if not isinstance(value, dict):
        raise PluginDistributionError(f"{path} plugins.{plugin_id}.{field} must be an object")
    return value


def _required_string(path: Path, plugin_id: str, container: dict[str, Any], field: str) -> str:
    key = field.rsplit(".", 1)[-1]
    value = _required_value(path, plugin_id, container, key)
    if not isinstance(value, str) or not value.strip():
        raise PluginDistributionError(f"{path} plugins.{plugin_id}.{field} must be a non-empty string")
    return value


def _required_bool(path: Path, plugin_id: str, container: dict[str, Any], field: str) -> bool:
    key = field.rsplit(".", 1)[-1]
    value = _required_value(path, plugin_id, container, key)
    if not isinstance(value, bool):
        raise PluginDistributionError(f"{path} plugins.{plugin_id}.{field} must be a boolean")
    return value


def _required_string_tuple(path: Path, plugin_id: str, container: dict[str, Any], field: str) -> tuple[str, ...]:
    key = field.rsplit(".", 1)[-1]
    value = _required_value(path, plugin_id, container, key)
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(item, str) or not item.strip() for item in value)
    ):
        raise PluginDistributionError(f"{path} plugins.{plugin_id}.{field} must be a non-empty string list")
    return tuple(value)


def _required_choice_tuple(
    path: Path,
    plugin_id: str,
    container: dict[str, Any],
    field: str,
    allowed: set[str],
) -> tuple[str, ...]:
    values = _required_string_tuple(path, plugin_id, container, field)
    if len(set(values)) != len(values) or any(value not in allowed for value in values):
        raise PluginDistributionError(
            f"{path} plugins.{plugin_id}.{field} must contain unique values from {sorted(allowed)}"
        )
    return values


def _optional_choice_tuple(
    path: Path,
    plugin_id: str,
    container: dict[str, Any],
    field: str,
    allowed: set[str],
) -> tuple[str, ...]:
    if field not in container:
        return ()
    return _required_choice_tuple(path, plugin_id, container, field, allowed)


def _required_value(path: Path, plugin_id: str, container: dict[str, Any], field: str) -> Any:
    if field not in container:
        raise PluginDistributionError(f"{path} plugins.{plugin_id}.{field} is required")
    return container[field]
