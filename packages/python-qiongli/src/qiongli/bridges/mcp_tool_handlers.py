from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable

from bridges.mcp_config_wizard import start_config_wizard
from bridges.mcp_connectors import MCPConnector
from bridges.provider_config import (
    PROVIDER_FIELDS,
    global_provider_config_path,
    provider_capability_mode,
    provider_config_summary,
    redact_provider_config,
    resolve_provider_config,
    set_provider_value,
)


SERVER_NAME = "qiongli-mcp"


MCP_TOOL_DEFINITIONS: list[dict[str, Any]] = [
    {
        "name": "qiongli_config_status",
        "description": "Return redacted Qiongli provider configuration status.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string", "description": "Project directory for project-local config lookup."}
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_save_provider_config",
        "description": "Save a Qiongli provider key or email into the shared provider config.",
        "inputSchema": {
            "type": "object",
            "required": ["provider", "field", "value"],
            "properties": {
                "provider": {"type": "string"},
                "field": {"type": "string"},
                "value": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_collect_evidence",
        "description": "Collect evidence from a Qiongli MCP provider for a task packet.",
        "inputSchema": {
            "type": "object",
            "required": ["provider", "task_packet"],
            "properties": {
                "provider": {"type": "string"},
                "task_packet": {"type": "object"},
                "cwd": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_list_provider_env",
        "description": "List supported provider environment variable aliases without values.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": False},
    },
    {
        "name": "qiongli_test_provider",
        "description": "Check whether a provider is configured without making network calls.",
        "inputSchema": {
            "type": "object",
            "required": ["provider"],
            "properties": {
                "provider": {"type": "string"},
                "cwd": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_open_config_wizard",
        "description": "Start a local browser-based provider configuration wizard.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "host": {"type": "string", "default": "127.0.0.1"},
                "port": {"type": "integer", "default": 0},
            },
            "additionalProperties": False,
        },
    },
]


def call_qiongli_tool(name: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
    args = arguments or {}
    handlers: dict[str, Callable[[dict[str, Any]], dict[str, Any]]] = {
        "qiongli_config_status": _tool_config_status,
        "qiongli_save_provider_config": _tool_save_provider_config,
        "qiongli_collect_evidence": _tool_collect_evidence,
        "qiongli_list_provider_env": _tool_list_provider_env,
        "qiongli_test_provider": _tool_test_provider,
        "qiongli_open_config_wizard": _tool_open_config_wizard,
    }
    handler = handlers.get(name)
    if handler is None:
        return _tool_result({"error": f"unknown tool: {name}"}, is_error=True)
    try:
        return _tool_result(handler(args))
    except Exception as exc:  # noqa: BLE001 - MCP tools must convert failures into tool results.
        return _tool_result({"error": str(exc), "tool": name}, is_error=True)


def _tool_config_status(args: dict[str, Any]) -> dict[str, Any]:
    cwd = _cwd_from_args(args)
    config = resolve_provider_config(cwd=cwd)
    summary = provider_config_summary(config)
    return {
        "server": {"name": SERVER_NAME},
        "config_path": str(global_provider_config_path()),
        "providers": summary,
        "capability_mode": provider_capability_mode(summary),
        "redacted_config": redact_provider_config(config),
    }


def _tool_save_provider_config(args: dict[str, Any]) -> dict[str, Any]:
    provider = _required_str(args, "provider")
    field = _required_str(args, "field")
    value = _required_str(args, "value")
    path = set_provider_value(provider, field, value)
    return {
        "status": "saved",
        "provider": _normalize_label(provider),
        "field": _normalize_label(field),
        "config_path": str(path),
    }


def _tool_collect_evidence(args: dict[str, Any]) -> dict[str, Any]:
    provider = _required_str(args, "provider")
    raw_packet = args.get("task_packet", {})
    if not isinstance(raw_packet, dict):
        raise ValueError("task_packet must be an object")
    cwd = _cwd_from_args(args)
    evidence = MCPConnector().collect(provider, raw_packet, cwd)
    return {"evidence": evidence.to_dict()}


def _tool_list_provider_env(_args: dict[str, Any]) -> dict[str, Any]:
    providers: dict[str, dict[str, list[str]]] = {}
    for provider, fields in PROVIDER_FIELDS.items():
        providers[provider] = {field: list(aliases) for field, aliases in fields.items()}
    return {"providers": providers}


def _tool_test_provider(args: dict[str, Any]) -> dict[str, Any]:
    provider = _normalize_provider(_required_str(args, "provider"))
    cwd = _cwd_from_args(args)
    config = resolve_provider_config(cwd=cwd)
    redacted = redact_provider_config(config)
    providers = redacted.get("providers", {})
    raw = providers.get(provider, {}) if isinstance(providers, dict) else {}
    configured = bool(raw.get("configured")) if isinstance(raw, dict) else False
    return {
        "provider": provider,
        "status": "configured" if configured else "missing",
        "configured": configured,
        "fields": raw.get("fields", {}) if isinstance(raw, dict) else {},
    }


def _tool_open_config_wizard(args: dict[str, Any]) -> dict[str, Any]:
    host = str(args.get("host", "127.0.0.1") or "127.0.0.1")
    port = int(args.get("port", 0) or 0)
    wizard = start_config_wizard(host=host, port=port)
    return {
        "url": wizard.url,
        "host": wizard.host,
        "port": wizard.port,
        "config_path": wizard.config_path,
    }


def _tool_result(structured: dict[str, Any], *, is_error: bool = False) -> dict[str, Any]:
    text = json.dumps(structured, ensure_ascii=False, indent=2, sort_keys=True)
    return {
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": is_error,
    }


def _cwd_from_args(args: dict[str, Any]) -> Path:
    raw = str(args.get("cwd", "") or "").strip()
    return Path(raw).expanduser().resolve() if raw else Path.cwd()


def _required_str(args: dict[str, Any], key: str) -> str:
    value = str(args.get(key, "") or "").strip()
    if not value:
        raise ValueError(f"{key} is required")
    return value


def _normalize_label(value: str) -> str:
    return value.strip().lower().replace("-", "_")


def _normalize_provider(value: str) -> str:
    normalized = _normalize_label(value)
    aliases = {"s2": "semantic_scholar", "semanticscholar": "semantic_scholar", "ncbi": "pubmed"}
    return aliases.get(normalized, normalized)
