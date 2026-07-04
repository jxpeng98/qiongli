from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
from typing import Any, Callable

from bridges.mcp_config_wizard import start_config_wizard
from bridges.mcp_connectors import MCPConnector
from bridges.guidance_runtime import GUIDANCE_MODES, effective_guidance, guidance_bootstrap_status
from bridges.project_manifest import OFFICIAL_SUBJECTS, ProjectManifestError, load_project_manifest
from bridges.subject_lifecycle import ACTIONS, apply_subject_action, subject_status
from bridges.subject_refinement import infer_subject_refinement
from bridges.subject_runtime import implicit_project_manifest_state, resolve_project_subject
from bridges.literature_mcp_tools import (
    LITERATURE_TOOL_DEFINITIONS,
    handle_literature_export_evidence,
    handle_literature_search,
    handle_literature_status,
    handle_search_plan,
)
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
RUNTIME_AGENT_ENUM = ["codex", "claude", "antigravity"]
SUBJECT_LIFECYCLE_ACTION_ORDER = ("confirm", "dismiss", "reset", "lock", "unlock")
SUBJECT_LIFECYCLE_ACTION_ENUM = [
    action for action in SUBJECT_LIFECYCLE_ACTION_ORDER if action in ACTIONS
]
SUBJECT_LIFECYCLE_SUBJECT_ENUM = [
    subject for subject in OFFICIAL_SUBJECTS if subject not in {"auto", "core"}
]


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
        "description": (
            "Save explicit Qiongli provider config values from chat or scripts. "
            "Prefer qiongli_configure_provider for API keys."
        ),
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
        "name": "qiongli_configure_provider",
        "description": (
            "Open a local browser-based setup page for Qiongli provider credentials. "
            "Prefer this for API keys so secrets do not enter chat history."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string",
                    "enum": ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"],
                },
                "host": {"type": "string", "default": "127.0.0.1"},
                "port": {"type": "integer", "default": 0},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_collect_evidence",
        "description": (
            "Collect evidence from filesystem, built-in workflow adapters, or external "
            "command adapters configured outside Qiongli. Do not use this to judge "
            "built-in literature provider config; use qiongli_literature_status or "
            "qiongli_literature_search for OpenAlex, Semantic Scholar, Crossref, PubMed, "
            "and arXiv provider status/search."
        ),
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
        "name": "qiongli_subject_status",
        "description": (
            "Inspect adaptive subject state, project manifest, evidence memory, "
            "and managed subject guidance for a project."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string", "description": "Project directory to inspect."},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_subject_update",
        "description": (
            "Confirm, dismiss, reset, lock, or unlock adaptive subject guidance "
            "and managed project subject guidance."
        ),
        "inputSchema": {
            "type": "object",
            "required": ["action"],
            "properties": {
                "cwd": {"type": "string", "description": "Project directory to update."},
                "action": {"type": "string", "enum": SUBJECT_LIFECYCLE_ACTION_ENUM},
                "subject": {"type": "string", "enum": SUBJECT_LIFECYCLE_SUBJECT_ENUM},
                "run_id": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_open_config_wizard",
        "description": (
            "Compatibility alias for qiongli_configure_provider. "
            "Starts a local browser-based provider configuration wizard."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string",
                    "enum": ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"],
                },
                "host": {"type": "string", "default": "127.0.0.1"},
                "port": {"type": "integer", "default": 0},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_orchestrator_route",
        "description": (
            "Decide whether a Codex, Claude Code, or other MCP client should use "
            "the full Qiongli orchestrator instead of skill-only workflow routing."
        ),
        "inputSchema": {
            "type": "object",
            "required": ["request"],
            "properties": {
                "request": {
                    "type": "string",
                    "description": "Natural-language user request or current task summary.",
                },
                "platform": {
                    "type": "string",
                    "enum": ["codex", "claude_code", "claude", "antigravity", "cli", "unknown"],
                    "default": "unknown",
                },
                "cwd": {"type": "string"},
                "task_id": {"type": "string"},
                "paper_type": {"type": "string"},
                "topic": {"type": "string"},
                "execution_mode": {"type": "string", "enum": ["solo", "duo", "triad"]},
                "controller": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "primary": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "reviewer": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "verifier": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "solo_role_gates": {"type": "string", "enum": ["strict", "standard", "off"]},
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
        "name": "qiongli_lifecycle_plan",
        "description": "Build a preview full-cycle paper lifecycle gate report without launching agents.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string"},
                "topic": {"type": "string"},
                "paper_type": {"type": "string"},
                "mode": {"type": "string", "enum": ["preview"]},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_journal_fit_recommend",
        "description": "Recommend journals from an existing manuscript using local venue profiles.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string"},
                "venue_roots": {
                    "type": "array",
                    "items": {"type": "string"},
                },
                "limit": {"type": "integer"},
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
                "controller": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "primary": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "reviewer": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "verifier": {"type": "string", "enum": RUNTIME_AGENT_ENUM},
                "solo_role_gates": {"type": "string", "enum": ["strict", "standard", "off"]},
                "profile": {"type": "string"},
                "mcp_strict": {"type": "boolean"},
                "skills_strict": {"type": "boolean"},
                "guidance_mode": {"type": "string", "enum": list(GUIDANCE_MODES), "default": "propose"},
                "run_agents": {"type": "boolean", "default": False},
                "max_revision_rounds": {"type": "integer", "minimum": 0},
                "output_budget": {"type": "integer", "minimum": 1},
                "skip_validation": {"type": "boolean"},
            },
            "additionalProperties": False,
        },
    },
]

MCP_TOOL_DEFINITIONS = [*LITERATURE_TOOL_DEFINITIONS, *MCP_TOOL_DEFINITIONS]


def call_qiongli_tool(name: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
    args = arguments or {}
    handlers: dict[str, Callable[[dict[str, Any]], dict[str, Any]]] = {
        "qiongli_literature_status": handle_literature_status,
        "qiongli_search_plan": handle_search_plan,
        "qiongli_literature_search": handle_literature_search,
        "qiongli_literature_export_evidence": handle_literature_export_evidence,
        "qiongli_config_status": _tool_config_status,
        "qiongli_save_provider_config": _tool_save_provider_config,
        "qiongli_collect_evidence": _tool_collect_evidence,
        "qiongli_list_provider_env": _tool_list_provider_env,
        "qiongli_test_provider": _tool_test_provider,
        "qiongli_subject_status": _tool_subject_status,
        "qiongli_subject_update": _tool_subject_update,
        "qiongli_configure_provider": _tool_configure_provider,
        "qiongli_open_config_wizard": _tool_open_config_wizard,
        "qiongli_orchestrator_route": _tool_orchestrator_route,
        "qiongli_orchestrator_doctor": _tool_orchestrator_doctor,
        "qiongli_lifecycle_plan": _tool_lifecycle_plan,
        "qiongli_journal_fit_recommend": _tool_journal_fit_recommend,
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
    missing = _missing_provider_fields(summary)
    payload = {
        "server": {"name": SERVER_NAME},
        "config_path": str(global_provider_config_path()),
        "providers": summary,
        "capability_mode": provider_capability_mode(summary),
        "missing": missing,
        "redacted_config": redact_provider_config(config),
    }
    next_action = _provider_setup_next_action(missing)
    if next_action is not None:
        payload["next_action"] = next_action
    return payload


def _tool_save_provider_config(args: dict[str, Any]) -> dict[str, Any]:
    provider = _required_str(args, "provider")
    field = _required_str(args, "field")
    value = _required_str(args, "value")
    path = set_provider_value(provider, field, value)
    field_id = _normalize_label(field)
    payload = {
        "status": "saved",
        "provider": _normalize_label(provider),
        "field": field_id,
        "config_path": str(path),
    }
    if field_id == "api_key":
        payload["warning"] = (
            "api_key was saved from chat input. Prefer qiongli_configure_provider "
            "so provider secrets do not enter chat history."
        )
    return payload


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


def _tool_subject_status(args: dict[str, Any]) -> dict[str, Any]:
    return subject_status(_cwd_from_args(args))


def _tool_subject_update(args: dict[str, Any]) -> dict[str, Any]:
    action = _required_str(args, "action")
    subject = args.get("subject")
    run_id = args.get("run_id")
    return apply_subject_action(
        _cwd_from_args(args),
        action,
        str(subject) if subject else None,
        source="mcp",
        run_id=str(run_id) if run_id else None,
    )


def _tool_open_config_wizard(args: dict[str, Any]) -> dict[str, Any]:
    return _tool_configure_provider(args)


def _tool_configure_provider(args: dict[str, Any]) -> dict[str, Any]:
    host = str(args.get("host", "127.0.0.1") or "127.0.0.1")
    port = int(args.get("port", 0) or 0)
    provider = str(args.get("provider", "") or "").strip()
    provider_id = _normalize_provider(provider) if provider else None
    wizard = start_config_wizard(host=host, port=port, provider=provider_id)
    payload = {
        "url": wizard.url,
        "host": wizard.host,
        "port": wizard.port,
        "config_path": wizard.config_path,
    }
    if provider_id:
        payload["provider"] = provider_id
    return payload


def _tool_orchestrator_route(args: dict[str, Any]) -> dict[str, Any]:
    request = _required_str(args, "request")
    platform = _normalize_platform(str(args.get("platform", "unknown") or "unknown"))
    task_id = _optional_str(args, "task_id")
    paper_type = _optional_str(args, "paper_type")
    topic = _optional_str(args, "topic")
    execution_mode = _optional_str(args, "execution_mode")
    controller = _optional_str(args, "controller")
    primary = _optional_str(args, "primary")
    reviewer = _optional_str(args, "reviewer")
    verifier = _optional_str(args, "verifier")
    solo_role_gates = _optional_str(args, "solo_role_gates", "standard") or "standard"
    cwd = str(_cwd_from_args(args))

    signals = _orchestrator_route_signals(request, task_id=task_id, execution_mode=execution_mode)
    use_orchestrator = bool(task_id and paper_type and topic and signals["orchestrator_recommended"])
    base_args = {
        "cwd": cwd,
        "task_id": task_id or "<task_id>",
        "paper_type": paper_type or "<paper_type>",
        "topic": topic or "<topic>",
    }
    task_run_args: dict[str, Any] = dict(base_args)
    if execution_mode:
        task_run_args["execution_mode"] = execution_mode
    if controller:
        task_run_args["controller"] = controller
    if primary:
        task_run_args["primary"] = primary
    if reviewer:
        task_run_args["reviewer"] = reviewer
    if verifier:
        task_run_args["verifier"] = verifier
    if solo_role_gates:
        task_run_args["solo_role_gates"] = solo_role_gates
    task_run_args["run_agents"] = False

    missing = [
        key
        for key, value in (
            ("task_id", task_id),
            ("paper_type", paper_type),
            ("topic", topic),
        )
        if not value
    ]
    if use_orchestrator:
        route = "orchestrator_mcp"
        recommended_tool = "qiongli_task_run"
        requires_full_runtime = True
        sequence = [
            {"tool": "qiongli_orchestrator_doctor", "args": {"cwd": cwd}},
            {"tool": "qiongli_task_plan", "args": base_args},
            {"tool": "qiongli_task_run", "args": task_run_args},
        ]
        why = signals["why"]
    else:
        route = "skill_workflow"
        recommended_tool = "qiongli_task_plan"
        requires_full_runtime = False
        sequence = [{"tool": "qiongli_task_plan", "args": base_args}]
        why = [
            "skill workflow is enough unless the task needs runtime agent handoff, independent review, strict gates, or auditable task-run artifacts"
        ]
        if missing:
            why.append("missing canonical task fields: " + ", ".join(missing))

    return {
        "route": route,
        "recommended_tool": recommended_tool,
        "requires_full_runtime": requires_full_runtime,
        "platform": platform,
        "platform_note": _orchestrator_platform_note(platform),
        "why": why,
        "sequence": sequence,
        "missing": missing,
        "safety": (
            "qiongli_task_run is preview-first through MCP. It launches local "
            "runtime processes only when run_agents is the JSON boolean true "
            "and qiongli_orchestrator_doctor passes."
        ),
    }


def _tool_orchestrator_doctor(args: dict[str, Any]) -> dict[str, Any]:
    result = _model_orchestrator().doctor(_cwd_from_args(args))
    return _collaboration_payload(result)


def _tool_lifecycle_plan(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.lifecycle_harness import build_lifecycle_report

    cwd = _cwd_from_args(args).resolve()
    return build_lifecycle_report(
        cwd,
        topic=str(args.get("topic") or cwd.name),
        paper_type=str(args.get("paper_type") or "empirical"),
        mode="preview",
    )


def _tool_journal_fit_recommend(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.journal_fit import recommend_journals

    cwd = _cwd_from_args(args).resolve()
    payload = recommend_journals(
        cwd,
        venue_roots=_journal_fit_venue_roots(args, cwd),
        limit=_optional_int(args, "limit", 5, minimum=0) or 0,
    )
    return _normalize_journal_fit_sources(payload, cwd)


def _journal_fit_venue_roots(args: dict[str, Any], cwd: Path) -> list[Path]:
    raw_roots = args.get("venue_roots")
    if raw_roots is None or raw_roots == []:
        return [cwd / "venues"]
    if not isinstance(raw_roots, list):
        raise ValueError("venue_roots must be an array of strings")

    roots: list[Path] = []
    for index, raw_root in enumerate(raw_roots):
        if not isinstance(raw_root, str):
            raise ValueError(f"venue_roots[{index}] must be a string")
        root = Path(raw_root).expanduser()
        if not root.is_absolute():
            root = cwd / root
        roots.append(root.resolve())
    return roots or [cwd / "venues"]


def _normalize_journal_fit_sources(payload: dict[str, Any], cwd: Path) -> dict[str, Any]:
    ranked_venues = payload.get("ranked_venues")
    if not isinstance(ranked_venues, list):
        return payload

    cwd = cwd.resolve()
    for venue in ranked_venues:
        if not isinstance(venue, dict):
            continue
        raw_source = venue.get("source")
        if not isinstance(raw_source, str) or not raw_source:
            continue
        source = Path(raw_source).expanduser()
        if not source.is_absolute():
            source = cwd / source
        try:
            venue["source"] = source.resolve().relative_to(cwd).as_posix()
        except ValueError:
            pass
    return payload


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
            project_subject = preview["project_subject"]
            subject_refinement = preview["subject_refinement"]
            task_packet.update(
                _task_run_preview_domain_fields(
                    orchestrator,
                    task_run_kwargs,
                    project_subject=project_subject,
                    subject_refinement=subject_refinement,
                )
            )
            task_packet["project_subject"] = project_subject
            task_packet["subject_refinement"] = subject_refinement
            task_packet["runtime_plan"] = preview["effective_runtime_plan"]
            task_packet["local_guidance"] = _task_run_preview_local_guidance(task_run_kwargs)
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
    guidance_mode = _optional_str(args, "guidance_mode", "propose") or "propose"
    if guidance_mode not in GUIDANCE_MODES:
        raise ValueError(
            "guidance_mode must be one of: " + ", ".join(GUIDANCE_MODES)
        )
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
        "guidance_mode": guidance_mode,
        "profile": _optional_str(args, "profile", "default") or "default",
        "execution_mode": execution_mode,
        "triad": _optional_bool(args, "triad", default=triad_default),
        "controller": _optional_str(args, "controller"),
        "primary_agent": _optional_str(args, "primary"),
        "review_agent": _optional_str(args, "reviewer"),
        "verifier_agent": _optional_str(args, "verifier"),
        "solo_role_gates": _optional_str(args, "solo_role_gates", "standard"),
        "max_revision_rounds": _optional_int(args, "max_revision_rounds", 2, minimum=0),
        "output_budget": _optional_int(args, "output_budget", None, minimum=1),
        "skip_validation": _optional_bool(args, "skip_validation", default=False),
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
    project_manifest_state = _task_run_preview_project_manifest_state(task_run_kwargs)
    project_subject = _task_run_preview_project_subject(
        task_run_kwargs,
        manifest_state=project_manifest_state,
    )
    subject_refinement = _task_run_preview_subject_refinement(
        plan_data,
        task_run_kwargs,
        manifest_state=project_manifest_state,
    )
    effective_domain = _task_run_preview_effective_domain(
        task_run_kwargs,
        project_subject,
        subject_refinement,
    )
    return {
        "will_launch_agents": False,
        "enable_with": {"run_agents": True},
        "controller_metadata": controller_metadata,
        "effective_runtime_plan": effective_runtime_plan,
        "project_manifest": project_manifest_state.to_packet(),
        "project_subject": project_subject,
        "subject_refinement": subject_refinement,
        "effective_domain": effective_domain,
        "guidance_bootstrap": guidance_bootstrap_status(
            task_run_kwargs["cwd"],
            mode=str(task_run_kwargs.get("guidance_mode", "propose")),
        ),
        "task_run_arguments": _serializable_task_run_arguments(task_run_kwargs),
    }


def _task_run_preview_local_guidance(task_run_kwargs: dict[str, Any]) -> dict[str, Any]:
    cwd = task_run_kwargs["cwd"]
    guidance_mode = str(task_run_kwargs.get("guidance_mode", "propose") or "propose")
    try:
        return effective_guidance(cwd, mode=guidance_mode).to_packet()
    except ProjectManifestError as exc:
        packet = effective_guidance(cwd, mode="off").to_packet()
        packet["warnings"] = [
            *list(packet.get("warnings", []) or []),
            f"Project guidance ignored: {exc}",
        ]
        return packet


def _task_run_preview_domain_fields(
    orchestrator: Any,
    task_run_kwargs: dict[str, Any],
    *,
    project_subject: dict[str, Any] | None = None,
    subject_refinement: dict[str, Any] | None = None,
) -> dict[str, Any]:
    requested_domain = str(task_run_kwargs.get("domain") or "auto").strip() or "auto"
    effective_domain = (
        _task_run_preview_effective_domain(
            task_run_kwargs,
            project_subject,
            subject_refinement,
        )
        if project_subject or subject_refinement
        else requested_domain
    )
    load_context = getattr(orchestrator, "_load_domain_profile_context", None)
    build_fields = getattr(orchestrator, "_build_domain_packet_fields", None)
    if not callable(load_context) or not callable(build_fields):
        return {"domain": effective_domain, "requested_domain": requested_domain}

    domain_context = load_context(effective_domain)
    raw_fields = build_fields(domain_context)
    fields = dict(raw_fields) if isinstance(raw_fields, dict) else {}
    if not fields.get("domain"):
        context_domain = domain_context.get("domain") if isinstance(domain_context, dict) else None
        fields["domain"] = str(context_domain or effective_domain).strip() or effective_domain
    if not fields.get("requested_domain"):
        context_requested = domain_context.get("requested_domain") if isinstance(domain_context, dict) else None
        fields["requested_domain"] = str(context_requested or requested_domain).strip() or requested_domain
    fields["requested_domain"] = requested_domain
    return fields


def _task_run_preview_project_manifest_state(task_run_kwargs: dict[str, Any]) -> Any:
    cwd = task_run_kwargs["cwd"]
    guidance_mode = str(task_run_kwargs.get("guidance_mode", "propose") or "propose")
    if guidance_mode == "off":
        return implicit_project_manifest_state(cwd)
    try:
        return load_project_manifest(cwd)
    except ProjectManifestError as exc:
        return replace(
            implicit_project_manifest_state(cwd),
            warnings=[f"Project manifest ignored: {exc}"],
        )


def _task_run_preview_project_subject(
    task_run_kwargs: dict[str, Any],
    *,
    manifest_state: Any | None = None,
) -> dict[str, Any]:
    manifest_state = manifest_state or _task_run_preview_project_manifest_state(task_run_kwargs)
    return resolve_project_subject(
        manifest_state,
        requested_domain=task_run_kwargs.get("domain"),
    ).to_packet()


def _task_run_preview_subject_refinement(
    plan_data: dict[str, Any],
    task_run_kwargs: dict[str, Any],
    *,
    manifest_state: Any,
) -> dict[str, Any]:
    task_packet = _task_run_preview_subject_refinement_task_packet(plan_data, task_run_kwargs)
    return infer_subject_refinement(
        task_packet,
        manifest_state=manifest_state,
        merged_analysis=str(plan_data.get("merged_analysis") or ""),
    ).to_packet()


def _task_run_preview_subject_refinement_task_packet(
    plan_data: dict[str, Any],
    task_run_kwargs: dict[str, Any],
) -> dict[str, Any]:
    task_packet: dict[str, Any] = {}
    raw_task_packet = plan_data.get("task_packet")
    if isinstance(raw_task_packet, dict):
        task_packet.update(raw_task_packet)
    for key in ("task_id", "paper_type", "topic"):
        if key not in task_packet and plan_data.get(key) is not None:
            task_packet[key] = plan_data[key]
    for key in ("task_id", "paper_type", "topic", "domain", "venue", "context", "profile"):
        value = task_run_kwargs.get(key)
        if value not in {None, ""}:
            task_packet[key] = value
    return task_packet


def _task_run_preview_effective_domain(
    task_run_kwargs: dict[str, Any],
    project_subject: dict[str, Any] | None,
    subject_refinement: dict[str, Any] | None = None,
) -> str:
    requested_domain = str(task_run_kwargs.get("domain") or "auto").strip() or "auto"
    if requested_domain.lower() != "auto":
        return requested_domain
    refinement_domain = _subject_refinement_effective_domain(subject_refinement)
    if refinement_domain:
        return refinement_domain
    if isinstance(project_subject, dict):
        subject_domain = str(project_subject.get("domain", "")).strip()
        if subject_domain:
            return subject_domain
    return "auto"


def _subject_refinement_effective_domain(subject_refinement: dict[str, Any] | None) -> str:
    if not isinstance(subject_refinement, dict):
        return ""
    decision = str(subject_refinement.get("decision") or "").strip()
    if decision not in {"suggest_subject", "confirm_subject", "lock_subject"}:
        return ""
    domain = str(subject_refinement.get("domain") or "").strip()
    if not domain or domain.lower() == "auto":
        return ""
    return domain


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


def _missing_provider_fields(summary: dict[str, str]) -> list[str]:
    missing: list[str] = []
    if summary.get("openalex") != "configured":
        missing.append("openalex.api_key")
    if summary.get("semantic_scholar") != "configured":
        missing.append("semantic_scholar.api_key")
    return missing


def _provider_setup_next_action(missing: list[str]) -> dict[str, Any] | None:
    if "openalex.api_key" in missing:
        return {
            "tool": "qiongli_configure_provider",
            "args": {"provider": "openalex"},
            "message": (
                "Run qiongli_configure_provider to open a local setup page. "
                "Do not paste API keys in chat."
            ),
        }
    if "semantic_scholar.api_key" not in missing:
        return None
    return {
        "tool": "qiongli_configure_provider",
        "args": {"provider": "semantic_scholar"},
        "message": (
            "Run qiongli_configure_provider to open a local setup page. "
            "Do not paste API keys in chat."
        ),
    }


def _normalize_platform(value: str) -> str:
    normalized = _normalize_label(value)
    aliases = {
        "claudecode": "claude_code",
        "claude-code": "claude_code",
        "claude_code": "claude_code",
        "claude": "claude_code",
        "antigravity": "antigravity",
        "ag": "antigravity",
    }
    platform = aliases.get(normalized, normalized)
    return platform if platform in {"codex", "claude_code", "antigravity", "cli"} else "unknown"


def _orchestrator_platform_note(platform: str) -> str:
    if platform == "codex":
        return (
            "Codex can use Qiongli skills directly for light work; use the full MCP "
            "orchestrator when Codex should coordinate with Claude Code or auditable "
            "task-run gates."
        )
    if platform == "claude_code":
        return (
            "Claude Code can use slash workflows and skills directly; use the full "
            "MCP orchestrator when Claude Code should coordinate with Codex or "
            "auditable task-run gates."
        )
    if platform == "antigravity":
        return (
            "Antigravity can use Qiongli skills directly for light work; use the full "
            "MCP orchestrator when Antigravity should coordinate with Codex, Claude "
            "Code, or auditable task-run gates."
        )
    if platform == "cli":
        return "CLI users can call qiongli task-plan/task-run directly or through this MCP server."
    return (
        "Skill-only routing is fine for simple work. Use the full MCP orchestrator "
        "for multi-agent handoff, independent review, strict gates, or task-run artifacts."
    )


def _orchestrator_route_signals(
    request: str,
    *,
    task_id: str | None,
    execution_mode: str | None,
) -> dict[str, Any]:
    normalized = request.lower()
    strong_terms = {
        "orchestrator": "request names the orchestrator",
        "task-run": "request names task-run",
        "multi-agent": "request asks for multi-agent work",
        "multi model": "request asks for multi-model work",
        "codex and claude": "request names Codex and Claude Code together",
        "claude and codex": "request names Claude Code and Codex together",
        "antigravity": "request names Antigravity runtime collaboration",
        "codex and antigravity": "request names Codex and Antigravity together",
        "claude and antigravity": "request names Claude Code and Antigravity together",
        "triad": "request asks for triad execution",
        "duo": "request asks for duo execution",
        "handoff": "request needs agent handoff",
        "independent review": "request needs independent review",
        "quality gate": "request needs quality gates",
        "audit": "request needs auditability",
        "reviewer": "request includes reviewer-style checking",
    }
    why = [reason for term, reason in strong_terms.items() if term in normalized]
    if execution_mode in {"duo", "triad"}:
        why.append(f"execution_mode={execution_mode} requires orchestrated runtime routing")
    if task_id:
        why.append("canonical task_id is available for task-run")
    return {
        "orchestrator_recommended": bool(why and (execution_mode in {"duo", "triad"} or len(why) >= 2)),
        "why": why or ["no orchestrator-specific signal found"],
    }


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


def _optional_int(
    args: dict[str, Any],
    key: str,
    default: int | None = None,
    *,
    minimum: int | None = None,
) -> int | None:
    if key not in args or args[key] is None:
        return default
    raw = args[key]
    if isinstance(raw, bool) or not isinstance(raw, int):
        raise ValueError(f"{key} must be an integer")
    if minimum is not None and raw < minimum:
        raise ValueError(f"{key} must be >= {minimum}")
    return raw


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
