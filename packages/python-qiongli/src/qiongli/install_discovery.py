from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from bridges.mcp_client_config import (
    BEGIN_MARKER,
    QIONGLI_MCP_ARGS,
    default_antigravity_config_path,
    default_claude_code_config_path,
    default_codex_config_path,
    default_hermes_config_path,
)

from .local_plugin_installer import (
    ClaudePluginPaths,
    MANAGED_MARKER_NAME,
    PLUGIN_ID,
    SKILL_DIR_NAME,
    resolve_antigravity_plugin_root,
    resolve_claude_plugin_paths,
    resolve_codex_plugin_paths,
)


CLIENTS = ("codex", "claude", "antigravity", "hermes")


def legacy_skill_dirs() -> dict[str, Path]:
    codex_home = Path(os.environ.get("CODEX_HOME", "~/.codex")).expanduser()
    claude_home = Path(os.environ.get("CLAUDE_CODE_HOME", "~/.claude")).expanduser()
    antigravity_home = Path(os.environ.get("ANTIGRAVITY_HOME", "~/.gemini/antigravity")).expanduser()
    hermes_home = Path(os.environ.get("HERMES_HOME", "~/.hermes")).expanduser()
    return {
        "codex": codex_home / "skills" / SKILL_DIR_NAME,
        "claude": claude_home / "skills" / SKILL_DIR_NAME,
        "antigravity": antigravity_home / "skills" / SKILL_DIR_NAME,
        "hermes": hermes_home / "skills" / SKILL_DIR_NAME,
    }


def discover_install_surfaces(*, check_activation: bool = True) -> dict[str, dict[str, object]]:
    skill_dirs = legacy_skill_dirs()
    codex_plugin = _codex_plugin_status(check_activation=check_activation)
    codex_plugin_mcp = _codex_plugin_mcp_status(codex_plugin)
    codex_standalone_mcp = _codex_standalone_mcp_status()
    codex_effective_mcp = codex_plugin_mcp if codex_plugin_mcp["installed"] else codex_standalone_mcp
    return {
        "codex": _combine_surface(
            client="codex",
            plugin=codex_plugin,
            skill=_skill_status(skill_dirs["codex"]),
            mcp=codex_effective_mcp,
            extra={
                "plugin_mcp": codex_plugin_mcp,
                "standalone_mcp": codex_standalone_mcp,
            },
        ),
        "claude": _combine_surface(
            client="claude",
            plugin=_claude_plugin_status(check_activation=check_activation),
            skill=_skill_status(skill_dirs["claude"]),
            mcp=_json_mcp_status(default_claude_code_config_path()),
        ),
        "antigravity": _combine_surface(
            client="antigravity",
            plugin=_antigravity_plugin_status(check_activation=check_activation),
            skill=_skill_status(skill_dirs["antigravity"]),
            mcp=_antigravity_mcp_status(),
        ),
        "hermes": _combine_surface(
            client="hermes",
            plugin=_empty_plugin_status("hermes"),
            skill=_skill_status(skill_dirs["hermes"]),
            mcp=_json_mcp_status(default_hermes_config_path()),
        ),
    }


def _combine_surface(
    *,
    client: str,
    plugin: dict[str, object],
    skill: dict[str, object],
    mcp: dict[str, object],
    extra: dict[str, object] | None = None,
) -> dict[str, object]:
    surface = "none"
    selected_path = str(skill["path"])
    version: object = None
    subject: object = None
    coverage: object = None

    if plugin["installed"]:
        surface = "plugin"
        selected_path = str(plugin["path"])
        version = plugin.get("version")
        subject = plugin.get("subject")
        coverage = plugin.get("coverage")
    elif mcp["installed"]:
        surface = "mcp"
        selected_path = str(mcp["path"])
        version = skill.get("version")
        subject = skill.get("subject")
        coverage = skill.get("coverage")
    elif skill["installed"]:
        surface = "legacy_skill"
        selected_path = str(skill["path"])
        version = skill.get("version")
        subject = skill.get("subject")
        coverage = skill.get("coverage")

    result = {
        "client": client,
        "surface": surface,
        "installed": surface != "none",
        "path": selected_path,
        "version": version,
        "subject": subject,
        "coverage": coverage,
        "plugin": plugin,
        "skill": skill,
        "mcp": mcp,
    }
    if extra:
        result.update(extra)
    return result


def _codex_plugin_status(*, check_activation: bool = True) -> dict[str, object]:
    paths = resolve_codex_plugin_paths()
    status = _plugin_status(
        plugin_root=paths.plugin_root,
        manifest_path=paths.plugin_root / ".codex-plugin" / "plugin.json",
    )
    plugin_id = f"{PLUGIN_ID}@{_codex_marketplace_name(paths.marketplace_path)}"
    if status["installed"] and check_activation:
        status.update(_codex_plugin_activation_status(paths.marketplace_path))
    elif status["installed"]:
        status.update(_unchecked_activation_status(plugin_id, "activation not checked"))
    else:
        status.update(_unchecked_activation_status(plugin_id))
    return status


def _claude_plugin_status(*, check_activation: bool = True) -> dict[str, object]:
    paths = resolve_claude_plugin_paths()
    status = _plugin_status(
        plugin_root=paths.plugin_root,
        manifest_path=paths.plugin_root / ".claude-plugin" / "plugin.json",
    )
    plugin_id = f"{PLUGIN_ID}@{paths.marketplace_name}"
    if status["installed"] and check_activation:
        status.update(_claude_plugin_activation_status(paths))
    elif status["installed"]:
        status.update(_unchecked_activation_status(plugin_id, "activation not checked"))
    else:
        status.update(_unchecked_activation_status(plugin_id))
    return status


def _antigravity_plugin_status(*, check_activation: bool = True) -> dict[str, object]:
    plugin_root = resolve_antigravity_plugin_root()
    status = _plugin_status(
        plugin_root=plugin_root,
        manifest_path=plugin_root / "plugin.json",
    )
    if status["installed"] and check_activation:
        status.update(_antigravity_plugin_activation_status(plugin_root))
    elif status["installed"]:
        status.update(_unchecked_activation_status(PLUGIN_ID, "activation not checked"))
    else:
        status.update(_unchecked_activation_status(PLUGIN_ID))
    return status


def _antigravity_mcp_status() -> dict[str, object]:
    plugin_root = resolve_antigravity_plugin_root()
    bundled = _json_mcp_status(plugin_root / "mcp_config.json")
    if bundled["installed"]:
        return {
            **bundled,
            "source": "plugin",
        }
    global_config = _json_mcp_status(default_antigravity_config_path())
    return {
        **global_config,
        "source": "global",
    }


def _empty_plugin_status(client: str) -> dict[str, object]:
    return {
        "client": client,
        "installed": False,
        "managed": False,
        "path": "",
        "manifest_path": "",
        "marker_path": "",
        "version": None,
        "subject": None,
        "coverage": None,
    }


def _unchecked_activation_status(plugin_id: str, detail: str = "plugin payload not installed") -> dict[str, object]:
    return {
        "active": None,
        "enabled": None,
        "plugin_id": plugin_id,
        "activation_detail": detail,
    }


def _plugin_status(*, plugin_root: Path, manifest_path: Path) -> dict[str, object]:
    marker_path = plugin_root / MANAGED_MARKER_NAME
    marker = _read_json_object(marker_path)
    manifest = _read_json_object(manifest_path)
    skill = _skill_status(plugin_root / "skills" / SKILL_DIR_NAME)
    version = skill.get("version") or marker.get("version") or manifest.get("version")
    return {
        "installed": manifest_path.is_file(),
        "managed": _is_managed_marker(marker),
        "path": str(plugin_root),
        "manifest_path": str(manifest_path),
        "marker_path": str(marker_path),
        "version": version,
        "subject": skill.get("subject"),
        "coverage": skill.get("coverage"),
    }


def _codex_plugin_activation_status(marketplace_path: Path) -> dict[str, object]:
    marketplace_name = _codex_marketplace_name(marketplace_path)
    plugin_id = f"{PLUGIN_ID}@{marketplace_name}"
    default_marketplace = Path("~/.agents/plugins/marketplace.json").expanduser()
    if marketplace_path.expanduser() != default_marketplace:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": "custom marketplace path; Codex activation not checked",
        }

    codex = shutil.which("codex")
    if not codex:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": "codex CLI not found",
        }

    try:
        result = subprocess.run(
            [codex, "plugin", "list", "--json"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": f"codex plugin list failed: {exc}",
        }

    if result.returncode != 0:
        output = (result.stdout or "").strip()
        detail = f"codex plugin list exited {result.returncode}"
        if output:
            detail = f"{detail}: {output.splitlines()[-1]}"
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": detail,
        }

    payload = _parse_json_from_mixed_output(result.stdout or "")
    installed = payload.get("installed")
    if not isinstance(installed, list):
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": "codex plugin list did not return installed plugins",
        }

    for entry in installed:
        if not isinstance(entry, dict):
            continue
        if entry.get("pluginId") != plugin_id and not (
            entry.get("name") == PLUGIN_ID and entry.get("marketplaceName") == marketplace_name
        ):
            continue
        enabled = bool(entry.get("enabled"))
        installed_flag = bool(entry.get("installed"))
        active = installed_flag and enabled
        detail = "installed and enabled" if active else "installed but disabled"
        return {
            "active": active,
            "enabled": enabled,
            "plugin_id": str(entry.get("pluginId") or plugin_id),
            "activation_detail": detail,
        }

    return {
        "active": False,
        "enabled": False,
        "plugin_id": plugin_id,
        "activation_detail": f"not in Codex active plugin list; run `codex plugin add {plugin_id}`",
    }


def _claude_plugin_activation_status(paths: ClaudePluginPaths) -> dict[str, object]:
    marketplace_name = paths.marketplace_name
    plugin_root = paths.plugin_root
    plugin_id = f"{PLUGIN_ID}@{marketplace_name}"
    claude = shutil.which("claude")
    if not claude:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": "claude CLI not found",
        }

    try:
        result = subprocess.run(
            [claude, "plugin", "list", "--json"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": f"claude plugin list failed: {exc}",
        }

    if result.returncode != 0:
        output = (result.stdout or "").strip()
        detail = f"claude plugin list exited {result.returncode}"
        if output:
            detail = f"{detail}: {output.splitlines()[-1]}"
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": detail,
        }

    entries = _plugin_entries_from_payload(_parse_json_value_from_mixed_output(result.stdout or ""))
    if entries is None:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": plugin_id,
            "activation_detail": "claude plugin list did not return plugins",
        }

    for entry in entries:
        if not _matches_claude_plugin_entry(entry, plugin_id, marketplace_name, plugin_root):
            continue
        enabled = _entry_enabled(entry)
        active = enabled is not False
        detail = "installed and enabled" if active else "installed but disabled"
        return {
            "active": active,
            "enabled": enabled,
            "plugin_id": str(entry.get("id") or entry.get("pluginId") or plugin_id),
            "activation_detail": detail,
        }

    return {
        "active": False,
        "enabled": False,
        "plugin_id": plugin_id,
        "activation_detail": f"not in Claude Code active plugin list; run `claude plugin install {plugin_id} --scope user`",
    }


def _antigravity_plugin_activation_status(plugin_root: Path) -> dict[str, object]:
    antigravity = shutil.which("antigravity")
    if not antigravity:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": PLUGIN_ID,
            "activation_detail": "antigravity CLI not found",
        }

    try:
        result = subprocess.run(
            [antigravity, "plugin", "list"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return {
            "active": None,
            "enabled": None,
            "plugin_id": PLUGIN_ID,
            "activation_detail": f"antigravity plugin list failed: {exc}",
        }

    if result.returncode != 0:
        output = (result.stdout or "").strip()
        detail = f"antigravity plugin list exited {result.returncode}"
        if output:
            detail = f"{detail}: {output.splitlines()[-1]}"
        return {
            "active": None,
            "enabled": None,
            "plugin_id": PLUGIN_ID,
            "activation_detail": detail,
        }

    output = result.stdout or ""
    matching_line = _find_plugin_line(output, PLUGIN_ID, plugin_root)
    if matching_line is None:
        return {
            "active": False,
            "enabled": False,
            "plugin_id": PLUGIN_ID,
            "activation_detail": "not in Antigravity plugin list; run `antigravity plugin install <qiongli plugin path>`",
        }
    enabled = "disabled" not in matching_line.lower()
    detail = "installed and enabled" if enabled else "installed but disabled"
    return {
        "active": enabled,
        "enabled": enabled,
        "plugin_id": PLUGIN_ID,
        "activation_detail": detail,
    }


def _codex_marketplace_name(marketplace_path: Path) -> str:
    payload = _read_json_object(marketplace_path)
    name = payload.get("name")
    if isinstance(name, str) and name.strip():
        return name.strip()
    return "personal"


def _parse_json_value_from_mixed_output(output: str) -> Any:
    object_start = output.find("{")
    array_start = output.find("[")
    starts = [index for index in (object_start, array_start) if index != -1]
    if not starts:
        return None
    start = min(starts)
    try:
        return json.loads(output[start:])
    except json.JSONDecodeError:
        return None


def _parse_json_from_mixed_output(output: str) -> dict[str, Any]:
    payload = _parse_json_value_from_mixed_output(output)
    return payload if isinstance(payload, dict) else {}


def _plugin_entries_from_payload(payload: object) -> list[dict[str, Any]] | None:
    if isinstance(payload, list):
        return [entry for entry in payload if isinstance(entry, dict)]
    if not isinstance(payload, dict):
        return None
    for key in ("installed", "plugins", "items"):
        value = payload.get(key)
        if isinstance(value, list):
            return [entry for entry in value if isinstance(entry, dict)]
    return None


def _matches_claude_plugin_entry(
    entry: dict[str, Any],
    plugin_id: str,
    marketplace_name: str,
    plugin_root: Path,
) -> bool:
    identifiers = [
        entry.get("id"),
        entry.get("pluginId"),
        entry.get("plugin_id"),
        entry.get("name"),
    ]
    if plugin_id in identifiers:
        return True
    if entry.get("name") == PLUGIN_ID and entry.get("marketplaceName") == marketplace_name:
        return True
    if entry.get("name") == PLUGIN_ID and entry.get("marketplace") == marketplace_name:
        return True
    install_path = entry.get("installPath") or entry.get("path")
    if isinstance(install_path, str) and _same_path(Path(install_path), plugin_root):
        return True
    return False


def _entry_enabled(entry: dict[str, Any]) -> bool | None:
    enabled = entry.get("enabled")
    if isinstance(enabled, bool):
        return enabled
    disabled = entry.get("disabled")
    if isinstance(disabled, bool):
        return not disabled
    status = entry.get("status")
    if isinstance(status, str):
        lowered = status.lower()
        if "disabled" in lowered:
            return False
        if "enabled" in lowered or "active" in lowered or "installed" in lowered:
            return True
    return True


def _find_plugin_line(output: str, plugin_id: str, plugin_root: Path) -> str | None:
    for line in output.splitlines():
        lowered = line.lower()
        if plugin_id.lower() in lowered or str(plugin_root) in line:
            return line
    return None


def _same_path(left: Path, right: Path) -> bool:
    try:
        return left.expanduser().resolve(strict=False) == right.expanduser().resolve(strict=False)
    except OSError:
        return left.expanduser() == right.expanduser()


def _skill_status(skill_dir: Path) -> dict[str, object]:
    return {
        "installed": skill_dir.exists(),
        "path": str(skill_dir),
        "version": _read_installed_version(skill_dir),
        "subject": _read_installed_subject(skill_dir),
        "coverage": _read_installed_coverage(skill_dir),
    }


def _empty_mcp_status(path: Path, *, source: str) -> dict[str, object]:
    return {
        "installed": False,
        "managed": False,
        "path": str(path),
        "server": "",
        "source": source,
    }


def _codex_plugin_mcp_status(plugin: dict[str, object]) -> dict[str, object]:
    if not plugin.get("installed"):
        path_text = str(plugin.get("path") or "")
        plugin_root = Path(path_text) if path_text else resolve_codex_plugin_paths().plugin_root
        return _empty_mcp_status(plugin_root / ".mcp.json", source="plugin")

    plugin_root = Path(str(plugin["path"]))
    manifest = _read_json_object(plugin_root / ".codex-plugin" / "plugin.json")
    mcp_ref = manifest.get("mcpServers")
    if not isinstance(mcp_ref, str) or not mcp_ref.strip():
        return _empty_mcp_status(plugin_root / ".mcp.json", source="plugin")

    mcp_path = plugin_root / mcp_ref
    status = _json_plugin_mcp_status(mcp_path)
    return {
        **status,
        "source": "plugin",
    }


def _json_plugin_mcp_status(path: Path) -> dict[str, object]:
    config = _read_json_object(path)
    mcp_servers = config.get("mcpServers")
    server_name = ""
    managed = False
    if isinstance(mcp_servers, dict):
        for candidate in (PLUGIN_ID, "qiongli-next"):
            if isinstance(mcp_servers.get(candidate), dict):
                server_name = candidate
                managed = True
                break
    return {
        "installed": bool(server_name),
        "managed": managed,
        "path": str(path),
        "server": server_name,
    }


def _codex_standalone_mcp_status() -> dict[str, object]:
    path = default_codex_config_path()
    text = _read_text(path)
    installed = BEGIN_MARKER in text or (
        "[mcp_servers.qiongli]" in text and 'command = "qiongli"' in text and "mcp" in text
    )
    return {
        "installed": installed,
        "managed": BEGIN_MARKER in text,
        "path": str(path),
        "server": "qiongli" if installed else "",
        "source": "standalone",
    }


def _json_mcp_status(path: Path) -> dict[str, object]:
    config = _read_json_object(path)
    server: Any = None
    mcp_servers = config.get("mcpServers")
    if isinstance(mcp_servers, dict):
        server = mcp_servers.get(PLUGIN_ID)
    installed = _is_qiongli_json_server(server)
    return {
        "installed": installed,
        "managed": installed,
        "path": str(path),
        "server": PLUGIN_ID if installed else "",
    }


def _is_qiongli_json_server(server: object) -> bool:
    if not isinstance(server, dict):
        return False
    command = server.get("command")
    args = server.get("args")
    return command == "qiongli" and args == QIONGLI_MCP_ARGS


def _read_installed_subject(skill_dir: Path) -> str | None:
    if not skill_dir.exists():
        return None
    manifest = _read_json_object(skill_dir / "SUBJECT_MANIFEST.json")
    if isinstance(manifest.get("subject"), str) and manifest["subject"].strip():
        return str(manifest["subject"]).strip()
    subject_path = skill_dir / "SUBJECT"
    if subject_path.exists():
        subject = _read_text(subject_path)
        return subject or "core"
    if (skill_dir / "SKILL.md").exists():
        return "core"
    return None


def _read_installed_coverage(skill_dir: Path) -> str | None:
    if not skill_dir.exists():
        return None
    manifest = _read_json_object(skill_dir / "SUBJECT_MANIFEST.json")
    if isinstance(manifest.get("coverage"), str) and manifest["coverage"].strip():
        return str(manifest["coverage"]).strip()
    if (skill_dir / "SKILL.md").exists():
        return "complete"
    return None


def _read_installed_version(skill_dir: Path) -> str | None:
    version_path = skill_dir / "VERSION"
    if not version_path.exists():
        return None
    return _read_text(version_path) or None


def _read_json_object(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except OSError:
        return ""


def _is_managed_marker(marker: dict[str, Any]) -> bool:
    return (
        marker.get("managed_by") == "qiongli-cli"
        and marker.get("plugin") == PLUGIN_ID
        and marker.get("surface") == "plugin"
    )
