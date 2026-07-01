from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from bridges.mcp_config_wizard import start_config_wizard
from bridges.mcp_tool_handlers import SERVER_NAME
from bridges.provider_config import (
    PROVIDER_FIELDS,
    global_provider_config_path,
    provider_capability_mode,
    provider_config_summary,
    redact_provider_config,
    resolve_provider_config,
    set_provider_value,
)
from qiongli import __version__
from qiongli.universal_installer import PART_CHOICES, TARGET_CHOICES


LITERATURE_TOOLS = [
    "qiongli_literature_status",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
]
ORCHESTRATOR_TOOLS = [
    "qiongli_orchestrator_route",
    "qiongli_orchestrator_doctor",
    "qiongli_task_plan",
    "qiongli_task_run",
]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run and configure the Qiongli cross-platform MCP server.")
    subparsers = parser.add_subparsers(dest="cmd", required=True)

    serve = subparsers.add_parser("serve", help="Run the MCP server")
    serve.add_argument("--transport", choices=["stdio", "http"], default="stdio")
    serve.add_argument("--host", default="127.0.0.1")
    serve.add_argument("--port", type=int, default=8765)

    doctor = subparsers.add_parser("doctor", help="Check MCP provider configuration")
    doctor.add_argument("--cwd", default=str(Path.cwd()))
    doctor.add_argument("--json", action="store_true")

    upgrade = subparsers.add_parser(
        "upgrade",
        help="Upgrade the Qiongli CLI runtime and installed assets used by the MCP server",
    )
    upgrade.add_argument(
        "--repo",
        help="Upstream repo in owner/repo form or Git URL. Defaults to configured Qiongli upstream.",
    )
    upgrade.add_argument("--ref", help="Tag or branch name. Defaults to the latest release tag.")
    upgrade.add_argument(
        "--ref-type",
        choices=["tag", "branch"],
        default="tag",
        help="How to interpret --ref (default: tag; latest uses tag).",
    )
    upgrade.add_argument(
        "--target",
        default="all",
        choices=TARGET_CHOICES,
        help="Install target to refresh after upgrade (default: all).",
    )
    upgrade.add_argument("--beta", action="store_true", help="Include beta/pre-release tags for upgrade.")
    upgrade.add_argument(
        "--subject",
        default="core",
        help=(
            "Advanced override for pre-materialized subject packages. "
            "Default core keeps runtime subject refinement adaptive."
        ),
    )
    upgrade.add_argument(
        "--coverage",
        default="complete",
        choices=["complete", "focused"],
        help="Subject coverage to install (default: complete).",
    )
    upgrade.add_argument(
        "--mode",
        default="copy",
        choices=["copy", "link"],
        help="Install mode (default: copy).",
    )
    upgrade.add_argument(
        "--project-dir",
        default=str(Path.cwd()),
        help="Project directory used when project surfaces are enabled (default: current dir).",
    )
    upgrade.add_argument("--install-cli", action="store_true", help="Install or refresh shell CLI wrappers.")
    upgrade.add_argument("--no-cli", action="store_true", help="Skip shell CLI installation during upgrade.")
    upgrade.add_argument("--cli-dir", help="Directory for shell CLI wrappers.")
    upgrade.add_argument(
        "--overwrite",
        action="store_true",
        default=True,
        help="Overwrite existing installs (default: on).",
    )
    upgrade.add_argument(
        "--no-overwrite",
        action="store_false",
        dest="overwrite",
        help="Do not overwrite existing installs.",
    )
    upgrade.add_argument("--doctor", action="store_true", help="Run orchestrator doctor after install.")
    upgrade.add_argument("--dry-run", action="store_true", help="Show install actions only.")
    upgrade.add_argument(
        "--parts",
        help=f"Comma-separated install surfaces to apply: {', '.join(PART_CHOICES)}.",
    )

    configure = subparsers.add_parser("configure", help="Save a provider config value")
    configure.add_argument("--provider", required=True)
    configure.add_argument("--field", required=True)
    configure.add_argument("--value", required=True)
    configure.add_argument("--json", action="store_true")

    wizard = subparsers.add_parser("wizard", help="Start a local provider configuration wizard")
    wizard.add_argument("--host", default="127.0.0.1")
    wizard.add_argument("--port", type=int, default=0)
    wizard.add_argument("--no-block", action="store_true")
    wizard.add_argument("--json", action="store_true")

    config = subparsers.add_parser("config", help="Print MCP client configuration examples")
    config_subparsers = config.add_subparsers(dest="config_cmd", required=True)
    example = config_subparsers.add_parser("example", help="Print a client config fragment")
    example.add_argument(
        "--target",
        choices=["codex", "claude", "claude-code", "antigravity", "cursor", "hermes"],
        default="codex",
    )
    example.add_argument("--json", action="store_true")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.cmd == "serve":
        return _cmd_serve(args)
    if args.cmd == "doctor":
        return _cmd_doctor(args)
    if args.cmd == "upgrade":
        return _cmd_upgrade(args)
    if args.cmd == "configure":
        return _cmd_configure(args)
    if args.cmd == "wizard":
        return _cmd_wizard(args)
    if args.cmd == "config":
        return _cmd_config(args)
    raise RuntimeError(f"Unhandled MCP command: {args.cmd}")


def _cmd_serve(args: argparse.Namespace) -> int:
    if args.transport == "stdio":
        from bridges.mcp_server_stdio import run_stdio

        return run_stdio()
    from bridges.mcp_server_http import run_http_server

    return run_http_server(host=args.host, port=args.port)


def _cmd_doctor(args: argparse.Namespace) -> int:
    payload = _doctor_payload(Path(args.cwd))
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    print("Qiongli MCP Doctor")
    print("==================")
    print(f"- server: {payload['server']['name']} {payload['server']['version']}")
    print(f"- config_path: {payload['config_path']}")
    for provider, status in payload["providers"].items():
        print(f"- {provider}: {status}")
    print(f"- capability_mode: {payload['capability_mode']}")
    print(f"- literature_tools_available: {payload['literature_tools_available']}")
    print(f"- orchestrator_tools_available: {payload['orchestrator_tools_available']}")
    return 0


def _cmd_upgrade(args: argparse.Namespace) -> int:
    from qiongli.cli import cmd_upgrade

    return cmd_upgrade(args)


def _cmd_configure(args: argparse.Namespace) -> int:
    path = set_provider_value(args.provider, args.field, args.value)
    payload = {
        "status": "saved",
        "provider": _normalize_label(args.provider),
        "field": _normalize_label(args.field),
        "config_path": str(path),
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(f"Saved {payload['provider']} {payload['field']} in {path}")
    return 0


def _cmd_wizard(args: argparse.Namespace) -> int:
    wizard = start_config_wizard(host=args.host, port=args.port)
    payload = {
        "url": wizard.url,
        "host": wizard.host,
        "port": wizard.port,
        "config_path": wizard.config_path,
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True), flush=True)
    else:
        print(f"Qiongli MCP config wizard: {wizard.url}", flush=True)
        print(f"Config path: {wizard.config_path}", flush=True)
    if args.no_block:
        return 0
    try:
        wizard.completed.wait()
        return 0
    except KeyboardInterrupt:
        wizard.stop()
        return 0


def _cmd_config(args: argparse.Namespace) -> int:
    if args.config_cmd != "example":
        raise RuntimeError(f"Unhandled MCP config command: {args.config_cmd}")
    payload = config_example(args.target)
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(json.dumps(payload["config"], indent=2, sort_keys=True))
    return 0


def _doctor_payload(cwd: Path) -> dict[str, Any]:
    config = resolve_provider_config(cwd=cwd)
    summary = provider_config_summary(config)
    missing = _missing_provider_fields(summary)
    payload: dict[str, Any] = {
        "server": {"name": SERVER_NAME, "version": __version__},
        "config_path": str(global_provider_config_path()),
        "providers": summary,
        "capability_mode": provider_capability_mode(summary),
        "literature_tools_available": True,
        "orchestrator_tools_available": True,
        "literature_tools": LITERATURE_TOOLS,
        "orchestrator_tools": ORCHESTRATOR_TOOLS,
        "missing": missing,
        "redacted_config": redact_provider_config(config),
        "provider_env_aliases": {
            provider: {field: list(aliases) for field, aliases in fields.items()}
            for provider, fields in PROVIDER_FIELDS.items()
        },
    }
    next_action = _provider_setup_next_action(missing)
    if next_action is not None:
        payload["next_action"] = next_action
    return payload


def config_example(target: str) -> dict[str, Any]:
    server = {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}
    if target in {"claude", "claude-code"}:
        config: dict[str, Any] = {"mcpServers": {"qiongli": server}}
    elif target == "codex":
        config = {"mcp_servers": {"qiongli": server}}
    elif target == "cursor":
        config = {"mcpServers": {"qiongli": server}}
    else:
        config = {"mcpServers": {"qiongli": server}}
    return {
        "target": target,
        "server": server,
        "config": config,
        "configuration_tools": [
            "qiongli_config_status",
            "qiongli_configure_provider",
            "qiongli_open_config_wizard",
            "qiongli_save_provider_config",
        ],
        "literature_tools": LITERATURE_TOOLS,
        "orchestrator_tools": ORCHESTRATOR_TOOLS,
        "orchestration_tools": ORCHESTRATOR_TOOLS,
        "safety": {
            "task_run_default": "preview",
            "run_agents_required": True,
        },
    }


def _normalize_label(value: str) -> str:
    return value.strip().lower().replace("-", "_")


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


if __name__ == "__main__":
    raise SystemExit(main())
