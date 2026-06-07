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
ModelOrchestrator: Any | None = None


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
    {
        "name": "qiongli_orchestrator_doctor",
        "description": "Run Qiongli orchestrator preflight checks for a project directory.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string", "description": "Project directory to check."},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_task_plan",
        "description": "Render a Qiongli task execution plan without launching runtime agents.",
        "inputSchema": {
            "type": "object",
            "required": ["task_id", "paper_type", "topic"],
            "properties": {
                "task_id": {"type": "string"},
                "paper_type": {"type": "string"},
                "topic": {"type": "string"},
                "cwd": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_task_run",
        "description": (
            "Preview or run a Qiongli task through the local orchestrator. "
            "Defaults to preview; set run_agents=true to launch local runtime agents."
        ),
        "inputSchema": {
            "type": "object",
            "required": ["task_id", "paper_type", "topic"],
            "properties": {
                "task_id": {"type": "string"},
                "paper_type": {"type": "string"},
                "topic": {"type": "string"},
                "cwd": {"type": "string"},
                "domain": {"type": "string"},
                "venue": {"type": "string"},
                "context": {"type": "string"},
                "execution_mode": {"type": "string", "enum": ["solo", "duo", "triad"]},
                "triad": {"type": "boolean"},
                "controller": {"type": "string", "enum": ["codex", "claude", "gemini"]},
                "primary": {"type": "string", "enum": ["codex", "claude", "gemini"]},
                "reviewer": {"type": "string", "enum": ["codex", "claude", "gemini"]},
                "verifier": {"type": "string", "enum": ["codex", "claude", "gemini"]},
                "solo_role_gates": {"type": "string", "enum": ["strict", "standard", "off"]},
                "profile": {"type": "string"},
                "mcp_strict": {"type": "boolean"},
                "skills_strict": {"type": "boolean"},
                "run_agents": {"type": "boolean", "default": False},
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
        "qiongli_orchestrator_doctor": _tool_orchestrator_doctor,
        "qiongli_task_plan": _tool_task_plan,
        "qiongli_task_run": _tool_task_run,
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


def _tool_orchestrator_doctor(args: dict[str, Any]) -> dict[str, Any]:
    result = _model_orchestrator().doctor(_cwd_from_args(args))
    return _collaboration_payload(result)


def _tool_task_plan(args: dict[str, Any]) -> dict[str, Any]:
    result = _model_orchestrator().task_plan(
        task_id=_required_str(args, "task_id"),
        paper_type=_required_str(args, "paper_type"),
        topic=_required_str(args, "topic"),
        cwd=_cwd_from_args(args),
    )
    return _collaboration_payload(result)


def _tool_task_run(args: dict[str, Any]) -> dict[str, Any]:
    run_agents = _run_agents_enabled(args)
    orchestrator = _model_orchestrator()
    task_run_kwargs = _task_run_kwargs(args)
    if not run_agents:
        result = orchestrator.task_plan(
            task_id=task_run_kwargs["task_id"],
            paper_type=task_run_kwargs["paper_type"],
            topic=task_run_kwargs["topic"],
            cwd=task_run_kwargs["cwd"],
        )
        payload = _collaboration_payload(result)
        payload["mode"] = "task-run-preview"
        payload["run_agents"] = False
        payload["safety_note"] = "Preview only. Set run_agents=true to launch local runtime agents."
        data = payload.setdefault("data", {})
        if isinstance(data, dict):
            preview = _task_run_preview(orchestrator, data, task_run_kwargs)
            data["task_run_preview"] = preview
            task_packet = data.get("task_packet")
            if not isinstance(task_packet, dict):
                task_packet = {}
                data["task_packet"] = task_packet
            task_packet.setdefault(
                "task_id",
                data.get("task_id", str(task_run_kwargs["task_id"]).strip().upper()),
            )
            task_packet.setdefault("paper_type", data.get("paper_type", task_run_kwargs["paper_type"]))
            task_packet.setdefault("topic", data.get("topic", task_run_kwargs["topic"]))
            task_packet.setdefault("artifact_root", data.get("artifact_root"))
            task_packet.update(_task_run_preview_domain_fields(orchestrator, task_run_kwargs))
            task_packet["runtime_plan"] = preview["effective_runtime_plan"]
        return payload

    result = orchestrator.task_run(**task_run_kwargs)
    payload = _collaboration_payload(result)
    payload["run_agents"] = True
    return payload


def _tool_result(structured: dict[str, Any], *, is_error: bool = False) -> dict[str, Any]:
    text = json.dumps(structured, ensure_ascii=False, indent=2, sort_keys=True)
    return {
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": is_error,
    }


def _collaboration_payload(result: Any) -> dict[str, Any]:
    if hasattr(result, "to_json"):
        return json.loads(result.to_json())
    payload = {
        "mode": getattr(result, "mode", "unknown"),
        "confidence": getattr(result, "confidence", 0.0),
        "merged_analysis": getattr(result, "merged_analysis", ""),
        "recommendations": getattr(result, "recommendations", []),
    }
    data = getattr(result, "data", None)
    if data:
        payload["data"] = data
    task_description = getattr(result, "task_description", None)
    if task_description is not None:
        payload["task_description"] = task_description
    return payload


def _model_orchestrator() -> Any:
    global ModelOrchestrator
    if ModelOrchestrator is None:
        from bridges.orchestrator import ModelOrchestrator as LoadedModelOrchestrator

        ModelOrchestrator = LoadedModelOrchestrator
    return ModelOrchestrator()


def _task_run_kwargs(args: dict[str, Any]) -> dict[str, Any]:
    execution_mode = _optional_str(args, "execution_mode")
    triad_default = execution_mode == "triad"
    return {
        "task_id": _required_str(args, "task_id"),
        "paper_type": _required_str(args, "paper_type"),
        "topic": _required_str(args, "topic"),
        "cwd": _cwd_from_args(args),
        "domain": _optional_str(args, "domain", "auto"),
        "venue": _optional_str(args, "venue"),
        "context": _optional_str(args, "context"),
        "mcp_strict": _optional_bool(args, "mcp_strict", default=False),
        "skills_strict": _optional_bool(args, "skills_strict", default=False),
        "profile": _optional_str(args, "profile", "default") or "default",
        "execution_mode": execution_mode,
        "triad": _optional_bool(args, "triad", default=triad_default),
        "controller": _optional_str(args, "controller"),
        "primary_agent": _optional_str(args, "primary"),
        "review_agent": _optional_str(args, "reviewer"),
        "verifier_agent": _optional_str(args, "verifier"),
        "solo_role_gates": _optional_str(args, "solo_role_gates", "standard"),
    }


def _task_run_preview(
    orchestrator: Any,
    plan_data: dict[str, Any],
    task_run_kwargs: dict[str, Any],
) -> dict[str, Any]:
    controller_metadata = orchestrator._build_controller_metadata(
        execution_mode=task_run_kwargs["execution_mode"],
        controller=task_run_kwargs["controller"],
        primary_agent=task_run_kwargs["primary_agent"],
        review_agent=task_run_kwargs["review_agent"],
        verifier_agent=task_run_kwargs["verifier_agent"],
        solo_role_gates=task_run_kwargs["solo_role_gates"],
        triad=bool(task_run_kwargs.get("triad")),
    )
    runtime_plan = plan_data.get("runtime_plan", {})
    effective_runtime_plan = dict(runtime_plan) if isinstance(runtime_plan, dict) else {}
    effective_runtime_plan.update(orchestrator._controller_runtime_overrides(controller_metadata))
    return {
        "will_launch_agents": False,
        "enable_with": {"run_agents": True},
        "controller_metadata": controller_metadata,
        "effective_runtime_plan": effective_runtime_plan,
        "task_run_arguments": _serializable_task_run_arguments(task_run_kwargs),
    }


def _task_run_preview_domain_fields(
    orchestrator: Any,
    task_run_kwargs: dict[str, Any],
) -> dict[str, str]:
    requested_domain = str(task_run_kwargs.get("domain") or "auto").strip() or "auto"
    load_context = getattr(orchestrator, "_load_domain_profile_context", None)
    build_fields = getattr(orchestrator, "_build_domain_packet_fields", None)
    if not callable(load_context) or not callable(build_fields):
        return {"domain": requested_domain, "requested_domain": requested_domain}

    domain_context = load_context(requested_domain)
    raw_fields = build_fields(domain_context)
    fields = dict(raw_fields) if isinstance(raw_fields, dict) else {}
    if not fields.get("domain"):
        context_domain = domain_context.get("domain") if isinstance(domain_context, dict) else None
        fields["domain"] = str(context_domain or requested_domain).strip() or requested_domain
    if not fields.get("requested_domain"):
        context_requested = domain_context.get("requested_domain") if isinstance(domain_context, dict) else None
        fields["requested_domain"] = str(context_requested or requested_domain).strip() or requested_domain
    return fields


def _serializable_task_run_arguments(task_run_kwargs: dict[str, Any]) -> dict[str, Any]:
    payload: dict[str, Any] = {}
    for key, value in task_run_kwargs.items():
        if value is None:
            continue
        payload[key] = str(value) if isinstance(value, Path) else value
    return payload


def _cwd_from_args(args: dict[str, Any]) -> Path:
    raw = str(args.get("cwd", "") or "").strip()
    return Path(raw).expanduser().resolve() if raw else Path.cwd()


def _required_str(args: dict[str, Any], key: str) -> str:
    value = str(args.get(key, "") or "").strip()
    if not value:
        raise ValueError(f"{key} is required")
    return value


def _optional_str(args: dict[str, Any], key: str, default: str | None = None) -> str | None:
    raw = args.get(key, default)
    value = str(raw or "").strip()
    return value or default


def _optional_bool(args: dict[str, Any], key: str, *, default: bool) -> bool:
    raw = args.get(key, default)
    if raw is None:
        return default
    if isinstance(raw, bool):
        return raw
    if isinstance(raw, str):
        normalized = raw.strip().lower()
        if normalized in {"1", "true", "yes", "on"}:
            return True
        if normalized in {"0", "false", "no", "off"}:
            return False
    raise ValueError(f"{key} must be a boolean.")


def _run_agents_enabled(args: dict[str, Any]) -> bool:
    if "run_agents" not in args:
        return False
    raw = args["run_agents"]
    if isinstance(raw, bool):
        return raw
    raise ValueError(
        "run_agents must be the JSON boolean true to launch local runtime agents; "
        "omit it or set false for preview."
    )


def _normalize_label(value: str) -> str:
    return value.strip().lower().replace("-", "_")


def _normalize_provider(value: str) -> str:
    normalized = _normalize_label(value)
    aliases = {"s2": "semantic_scholar", "semanticscholar": "semantic_scholar", "ncbi": "pubmed"}
    return aliases.get(normalized, normalized)
