from __future__ import annotations

import os
import re
from dataclasses import dataclass
from pathlib import Path


BEGIN_MARKER = "# BEGIN QIONGLI MANAGED MCP"
END_MARKER = "# END QIONGLI MANAGED MCP"
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


def install_mcp_config(
    *,
    target: str = "codex",
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    if target != "codex":
        return MCPConfigResult(
            status="skipped",
            path=_config_path(config_path),
            changed=False,
            detail=f"managed MCP config is only implemented for codex, got {target}",
        )

    path = _config_path(config_path)
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


def remove_mcp_config(
    *,
    target: str = "codex",
    config_path: Path | str | None = None,
    dry_run: bool = False,
) -> MCPConfigResult:
    if target != "codex":
        return MCPConfigResult(
            status="skipped",
            path=_config_path(config_path),
            changed=False,
            detail=f"managed MCP config is only implemented for codex, got {target}",
        )

    path = _config_path(config_path)
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


def _config_path(config_path: Path | str | None) -> Path:
    return Path(config_path).expanduser() if config_path is not None else default_codex_config_path()


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
