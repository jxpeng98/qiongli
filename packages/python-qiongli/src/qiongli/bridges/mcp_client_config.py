from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


BEGIN_MARKER = "# BEGIN QIONGLI MANAGED MCP"
END_MARKER = "# END QIONGLI MANAGED MCP"
QIONGLI_MCP_ARGS = ["mcp", "serve", "--transport", "stdio"]
QIONGLI_CODEX_SERVER_HEADER = "[mcp_servers.qiongli]"
QIONGLI_CODEX_SERVER_BLOCK = "\n".join(
    [
        BEGIN_MARKER,
        QIONGLI_CODEX_SERVER_HEADER,
        'command = "qiongli"',
        'args = ["mcp", "serve", "--transport", "stdio"]',
        END_MARKER,
        "",
    ]
)
QIONGLI_JSON_SERVER = {
    "command": "qiongli",
    "args": QIONGLI_MCP_ARGS,
}
QIONGLI_CLAUDE_CODE_SERVER = {
    **QIONGLI_JSON_SERVER,
    "type": "stdio",
}
MANAGED_BLOCK_RE = re.compile(
    rf"\n*{re.escape(BEGIN_MARKER)}\n.*?{re.escape(END_MARKER)}\n*",
    re.DOTALL,
)


@dataclass(frozen=True)
class MCPConfigResult:
    status: str
    path: Path
    changed: bool
    detail: str = ""
    preview: str = ""


def default_codex_config_path() -> Path:
    codex_home = os.environ.get("CODEX_HOME", "").strip()
    root = Path(codex_home).expanduser() if codex_home else Path.home() / ".codex"
    return root / "config.toml"


def default_claude_code_config_path() -> Path:
    explicit = os.environ.get("CLAUDE_CODE_CONFIG_PATH", "").strip()
    if explicit:
        return Path(explicit).expanduser()
    claude_home = os.environ.get("CLAUDE_CODE_HOME", "").strip()
    if claude_home:
        return Path(claude_home).expanduser().parent / ".claude.json"
    return Path.home() / ".claude.json"


def default_antigravity_config_path() -> Path:
    explicit = os.environ.get("ANTIGRAVITY_CONFIG_PATH", "").strip()
    if explicit:
        return Path(explicit).expanduser()
    antigravity_home = os.environ.get("ANTIGRAVITY_HOME", "").strip()
    root = Path(antigravity_home).expanduser() if antigravity_home else Path.home() / ".gemini" / "antigravity"
    return root / "settings.json"


def default_hermes_config_path() -> Path:
    explicit = os.environ.get("HERMES_CONFIG_PATH", "").strip()
    if explicit:
        return Path(explicit).expanduser()
    hermes_home = os.environ.get("HERMES_HOME", "").strip()
    root = Path(hermes_home).expanduser() if hermes_home else Path.home() / ".hermes"
    return root / "settings.json"


def install_mcp_config(
    *,
    target: str = "codex",
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    normalized_target = _normalize_target(target)
    if normalized_target == "codex":
        return _install_codex_mcp_config(config_path=config_path, dry_run=dry_run)
    if normalized_target == "claude-code":
        return _install_claude_code_mcp_config(config_path=config_path, dry_run=dry_run)
    if normalized_target == "antigravity":
        return _install_json_mcp_config(
            target=normalized_target,
            client_label="Antigravity",
            server=QIONGLI_JSON_SERVER,
            config_path=config_path,
            dry_run=dry_run,
        )
    if normalized_target == "hermes":
        return _install_json_mcp_config(
            target=normalized_target,
            client_label="Hermes",
            server=QIONGLI_JSON_SERVER,
            config_path=config_path,
            dry_run=dry_run,
        )

    return MCPConfigResult(
        status="skipped",
        path=_config_path(normalized_target, config_path),
        changed=False,
        detail=f"managed MCP config is only implemented for codex, claude-code, antigravity, and hermes, got {target}",
    )


def remove_mcp_config(
    *,
    target: str = "codex",
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    normalized_target = _normalize_target(target)
    if normalized_target == "codex":
        return _remove_codex_mcp_config(config_path=config_path, dry_run=dry_run)
    if normalized_target == "claude-code":
        return _remove_claude_code_mcp_config(config_path=config_path, dry_run=dry_run)
    if normalized_target == "antigravity":
        return _remove_json_mcp_config(
            target=normalized_target,
            client_label="Antigravity",
            config_path=config_path,
            dry_run=dry_run,
        )
    if normalized_target == "hermes":
        return _remove_json_mcp_config(
            target=normalized_target,
            client_label="Hermes",
            config_path=config_path,
            dry_run=dry_run,
        )

    return MCPConfigResult(
        status="skipped",
        path=_config_path(normalized_target, config_path),
        changed=False,
        detail=f"managed MCP config is only implemented for codex, claude-code, antigravity, and hermes, got {target}",
    )


def _install_codex_mcp_config(
    *,
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    path = _config_path("codex", config_path)
    original = _read_text(path)
    if not _has_managed_block(original) and _has_unmanaged_qiongli_server(original):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail="unmanaged qiongli MCP server already exists",
            preview=original,
        )

    rendered = _replace_or_append_managed_block(original)
    changed = rendered != original
    if dry_run:
        return MCPConfigResult(
            status="dry-run",
            path=path,
            changed=changed,
            detail="would install managed qiongli MCP server",
            preview=rendered,
        )

    if changed:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(rendered, encoding="utf-8")
    return MCPConfigResult(
        status="installed" if changed else "current",
        path=path,
        changed=changed,
        detail="managed qiongli MCP server installed" if changed else "managed qiongli MCP server already current",
        preview=rendered,
    )


def _install_claude_code_mcp_config(
    *,
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    return _install_json_mcp_config(
        target="claude-code",
        client_label="Claude Code",
        server=QIONGLI_CLAUDE_CODE_SERVER,
        config_path=config_path,
        dry_run=dry_run,
    )


def _install_json_mcp_config(
    *,
    target: str,
    client_label: str,
    server: dict[str, object],
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    path = _config_path(target, config_path)
    original = _read_text(path)
    parsed = _parse_json_config(original, client_label)
    if isinstance(parsed, str):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail=parsed,
            preview=original,
        )
    data = parsed
    mcp_servers = data.get("mcpServers")
    if mcp_servers is None:
        mcp_servers = {}
    if not isinstance(mcp_servers, dict):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail=f"{client_label} config mcpServers must be an object",
            preview=original,
        )
    existing = mcp_servers.get("qiongli")
    if existing is not None and not _is_managed_json_server(existing):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail="unmanaged qiongli MCP server already exists",
            preview=original,
        )

    rendered_data = dict(data)
    rendered_servers = dict(mcp_servers)
    rendered_servers["qiongli"] = dict(server)
    rendered_data["mcpServers"] = rendered_servers
    rendered = _dump_json_config(rendered_data)
    changed = rendered != original
    if existing == server and original:
        rendered = original
        changed = False
    if dry_run:
        return MCPConfigResult(
            status="dry-run",
            path=path,
            changed=changed,
            detail="would install managed qiongli MCP server",
            preview=rendered,
        )

    if changed:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(rendered, encoding="utf-8")
    return MCPConfigResult(
        status="installed" if changed else "current",
        path=path,
        changed=changed,
        detail="managed qiongli MCP server installed" if changed else "managed qiongli MCP server already current",
        preview=rendered,
    )


def _remove_codex_mcp_config(
    *,
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    path = _config_path("codex", config_path)
    original = _read_text(path)
    if not _has_managed_block(original):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail="no managed qiongli MCP server block found",
            preview=original,
        )

    rendered = _remove_managed_block(original)
    changed = rendered != original
    if dry_run:
        return MCPConfigResult(
            status="dry-run",
            path=path,
            changed=changed,
            detail="would remove managed qiongli MCP server",
            preview=rendered,
        )

    path.write_text(rendered, encoding="utf-8")
    return MCPConfigResult(
        status="removed" if changed else "skipped",
        path=path,
        changed=changed,
        detail="managed qiongli MCP server removed",
        preview=rendered,
    )


def _remove_claude_code_mcp_config(
    *,
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    return _remove_json_mcp_config(
        target="claude-code",
        client_label="Claude Code",
        config_path=config_path,
        dry_run=dry_run,
    )


def _remove_json_mcp_config(
    *,
    target: str,
    client_label: str,
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    path = _config_path(target, config_path)
    original = _read_text(path)
    parsed = _parse_json_config(original, client_label)
    if isinstance(parsed, str):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail=parsed,
            preview=original,
        )
    data = parsed
    mcp_servers = data.get("mcpServers")
    if not isinstance(mcp_servers, dict):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail="no managed qiongli MCP server found",
            preview=original,
        )
    existing = mcp_servers.get("qiongli")
    if not _is_managed_json_server(existing):
        return MCPConfigResult(
            status="skipped",
            path=path,
            changed=False,
            detail="no managed qiongli MCP server found",
            preview=original,
        )

    rendered_data = dict(data)
    rendered_servers = dict(mcp_servers)
    del rendered_servers["qiongli"]
    rendered_data["mcpServers"] = rendered_servers
    rendered = _dump_json_config(rendered_data)
    changed = rendered != original
    if dry_run:
        return MCPConfigResult(
            status="dry-run",
            path=path,
            changed=changed,
            detail="would remove managed qiongli MCP server",
            preview=rendered,
        )

    path.write_text(rendered, encoding="utf-8")
    return MCPConfigResult(
        status="removed" if changed else "skipped",
        path=path,
        changed=changed,
        detail="managed qiongli MCP server removed",
        preview=rendered,
    )


def _normalize_target(target: str) -> str:
    normalized = target.strip().lower().replace("_", "-")
    if normalized == "claude":
        return "claude-code"
    return normalized


def _config_path(target: str, config_path: Path | str | None) -> Path:
    if config_path is not None:
        return Path(config_path).expanduser()
    if target == "claude-code":
        return default_claude_code_config_path()
    if target == "antigravity":
        return default_antigravity_config_path()
    if target == "hermes":
        return default_hermes_config_path()
    return default_codex_config_path()


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def _has_managed_block(text: str) -> bool:
    return BEGIN_MARKER in text and END_MARKER in text


def _has_unmanaged_qiongli_server(text: str) -> bool:
    if not text.strip():
        return False
    unmanaged_text = MANAGED_BLOCK_RE.sub("\n", text)
    return QIONGLI_CODEX_SERVER_HEADER in unmanaged_text


def _replace_or_append_managed_block(text: str) -> str:
    if _has_managed_block(text):
        return _normalize_trailing_newline(MANAGED_BLOCK_RE.sub(f"\n{QIONGLI_CODEX_SERVER_BLOCK}", text))
    if not text.strip():
        return QIONGLI_CODEX_SERVER_BLOCK
    return _normalize_trailing_newline(text) + "\n" + QIONGLI_CODEX_SERVER_BLOCK


def _remove_managed_block(text: str) -> str:
    return _normalize_trailing_newline(MANAGED_BLOCK_RE.sub("\n", text).strip())


def _normalize_trailing_newline(text: str) -> str:
    return text.rstrip() + "\n" if text.strip() else ""


def _parse_json_config(text: str, client_label: str) -> dict[str, Any] | str:
    if not text.strip():
        return {}
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        return f"invalid {client_label} JSON config: {exc.msg}"
    if not isinstance(data, dict):
        return f"{client_label} config root must be an object"
    return data


def _dump_json_config(data: dict[str, Any]) -> str:
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def _is_managed_json_server(server: object) -> bool:
    if not isinstance(server, dict):
        return False
    return (
        server.get("command") == "qiongli"
        and server.get("args") == QIONGLI_MCP_ARGS
        and server.get("type", "stdio") == "stdio"
    )
