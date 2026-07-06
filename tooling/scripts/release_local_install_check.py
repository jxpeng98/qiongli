#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.platform_targets import PlatformTarget, load_platform_targets


PLUGIN_ID = "qiongli"
SKILL_DIR_NAME = "qiongli-workflow"
QIONGLI_MCP_ARGS = ["mcp", "serve", "--transport", "stdio"]
REQUIRED_LIFECYCLE_MCP_TOOLS = ("qiongli_subject_status", "qiongli_subject_update")
LOCAL_INSTALL_RECOMMENDED_KEY_CLIENTS = {
    "codex": ("codex", "Codex"),
    "claude_code": ("claude", "Claude"),
    "antigravity": ("antigravity", "Antigravity"),
}


@dataclass(frozen=True)
class InstallSandbox:
    root: Path
    home: Path
    project_dir: Path
    codex_marketplace_path: Path
    codex_plugin_root: Path
    claude_plugin_parent: Path
    claude_plugin_root: Path
    antigravity_plugin_parent: Path
    antigravity_plugin_root: Path
    codex_home: Path
    claude_home: Path
    claude_config_path: Path
    antigravity_home: Path
    antigravity_config_path: Path
    hermes_home: Path
    hermes_config_path: Path


class LocalInstallCheckError(RuntimeError):
    pass


def build_sandbox(root: Path) -> InstallSandbox:
    sandbox_root = root.resolve()
    home = sandbox_root / "home"
    codex_marketplace_path = home / ".agents" / "plugins" / "marketplace.json"
    claude_plugin_parent = sandbox_root / "qiongli" / "plugins" / "claude-code"
    antigravity_plugin_parent = sandbox_root / "qiongli" / "plugins" / "antigravity"
    codex_plugin_root = home / "plugins" / PLUGIN_ID
    return InstallSandbox(
        root=sandbox_root,
        home=home,
        project_dir=sandbox_root / "project",
        codex_marketplace_path=codex_marketplace_path,
        codex_plugin_root=codex_plugin_root,
        claude_plugin_parent=claude_plugin_parent,
        claude_plugin_root=claude_plugin_parent / "plugins" / PLUGIN_ID,
        antigravity_plugin_parent=antigravity_plugin_parent,
        antigravity_plugin_root=antigravity_plugin_parent / PLUGIN_ID,
        codex_home=sandbox_root / "codex-home",
        claude_home=sandbox_root / "claude-home",
        claude_config_path=sandbox_root / "claude.json",
        antigravity_home=sandbox_root / "antigravity-home",
        antigravity_config_path=home / ".gemini" / "config" / "mcp_config.json",
        hermes_home=sandbox_root / "hermes-home",
        hermes_config_path=sandbox_root / "hermes-home" / "settings.json",
    )


def build_env(repo_root: Path, sandbox: InstallSandbox) -> dict[str, str]:
    python_source_root = repo_root / "packages" / "python-qiongli" / "src"
    env = os.environ.copy()
    existing_pythonpath = env.get("PYTHONPATH", "")
    import_roots = [str(python_source_root), str(repo_root)]
    env["PYTHONPATH"] = os.pathsep.join(import_roots + ([existing_pythonpath] if existing_pythonpath else []))
    env["HOME"] = str(sandbox.home)
    env["XDG_CONFIG_HOME"] = str(sandbox.root / "xdg-config")
    env["NO_COLOR"] = "1"
    env["QIONGLI_CODEX_MARKETPLACE_PATH"] = str(sandbox.codex_marketplace_path)
    env["QIONGLI_CLAUDE_MARKETPLACE_ROOT"] = str(sandbox.claude_plugin_parent)
    env["QIONGLI_ANTIGRAVITY_PLUGIN_PARENT"] = str(sandbox.antigravity_plugin_parent)
    env["CODEX_HOME"] = str(sandbox.codex_home)
    env["CLAUDE_CODE_HOME"] = str(sandbox.claude_home)
    env["CLAUDE_CODE_CONFIG_PATH"] = str(sandbox.claude_config_path)
    env["ANTIGRAVITY_HOME"] = str(sandbox.antigravity_home)
    env["ANTIGRAVITY_CONFIG_PATH"] = str(sandbox.antigravity_config_path)
    env["HERMES_HOME"] = str(sandbox.hermes_home)
    env["HERMES_CONFIG_PATH"] = str(sandbox.hermes_config_path)
    return env


def run_cli(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    label: str,
) -> str:
    result = subprocess.run(
        command,
        cwd=str(cwd),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    output = result.stdout or ""
    if result.returncode != 0:
        raise LocalInstallCheckError(
            f"{label} failed with exit code {result.returncode}\n"
            f"command: {' '.join(command)}\n"
            f"{output.rstrip()}"
        )
    return output


def run_install_check(repo_root: Path, sandbox: InstallSandbox, *, python: str = sys.executable) -> dict[str, Any]:
    sandbox.project_dir.mkdir(parents=True, exist_ok=True)
    env = build_env(repo_root, sandbox)
    install_cmd = [
        python,
        "-m",
        "qiongli.cli",
        "install",
        "--target",
        "all",
        "--surface",
        "plugin",
        "--parts",
        "plugin,mcp",
        "--project-dir",
        str(sandbox.project_dir),
        "--overwrite",
    ]
    run_cli(install_cmd, cwd=repo_root, env=env, label="qiongli install")
    check_cmd = [python, "-m", "qiongli.cli", "check", "--json", "--offline"]
    check_output = run_cli(check_cmd, cwd=repo_root, env=env, label="qiongli check --offline")
    try:
        payload = json.loads(check_output)
    except json.JSONDecodeError as exc:
        raise LocalInstallCheckError(f"qiongli check --json returned invalid JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise LocalInstallCheckError("qiongli check --json returned a non-object payload")
    validate_install_tree(repo_root, sandbox, payload)
    validate_lifecycle_mcp_tools(repo_root, sandbox, env, python=python)
    return payload


def validate_install_tree(repo_root: Path, sandbox: InstallSandbox, check_payload: dict[str, Any]) -> None:
    expected_version = (repo_root / "content" / "workflow" / "VERSION").read_text(encoding="utf-8").strip()
    expected_manifest_version = expected_version.lstrip("v")
    targets_by_client = _local_acceptance_targets_by_client(repo_root)

    codex_manifest = _read_json_object(sandbox.codex_plugin_root / ".codex-plugin" / "plugin.json")
    _expect(codex_manifest.get("name") == PLUGIN_ID, "Codex plugin manifest name is qiongli")
    _expect(codex_manifest.get("version") == expected_manifest_version, "Codex plugin manifest version matches release")
    _expect("category" not in codex_manifest, "Codex plugin manifest does not use top-level category")
    _expect(codex_manifest.get("skills") == "./skills/", "Codex plugin manifest points at skills directory")
    _expect(codex_manifest.get("mcpServers") == "./.mcp.json", "Codex plugin manifest points at bundled MCP manifest")
    interface = codex_manifest.get("interface")
    _expect(isinstance(interface, dict), "Codex plugin manifest has interface metadata")
    _expect(interface.get("category") == "Education", "Codex plugin interface category is Education")

    codex_mcp = _read_json_object(sandbox.codex_plugin_root / ".mcp.json")
    _expect_json_server(_mcp_server(codex_mcp, "Codex bundled MCP"), "Codex bundled MCP")

    marketplace = _read_json_object(sandbox.codex_marketplace_path)
    entries = marketplace.get("plugins")
    _expect(isinstance(entries, list), "Codex marketplace plugins is a list")
    _expect(
        any(
            isinstance(entry, dict)
            and entry.get("name") == PLUGIN_ID
            and entry.get("source") == {"source": "local", "path": "./plugins/qiongli"}
            for entry in entries
        ),
        "Codex marketplace registers qiongli local plugin",
    )

    claude_manifest = _read_json_object(sandbox.claude_plugin_root / ".claude-plugin" / "plugin.json")
    _expect(claude_manifest.get("name") == PLUGIN_ID, "Claude plugin manifest name is qiongli")
    _expect(claude_manifest.get("version") == expected_manifest_version, "Claude plugin manifest version matches release")
    _expect_json_server(_mcp_server(claude_manifest, "Claude plugin MCP"), "Claude plugin MCP")

    antigravity_manifest = _read_json_object(sandbox.antigravity_plugin_root / "plugin.json")
    _expect(antigravity_manifest.get("name") == PLUGIN_ID, "Antigravity plugin manifest name is qiongli")
    _expect(
        antigravity_manifest.get("version") == expected_manifest_version,
        "Antigravity plugin manifest version matches release",
    )
    antigravity_bundled_mcp = _read_json_object(sandbox.antigravity_plugin_root / "mcp_config.json")
    _expect_json_server(_mcp_server(antigravity_bundled_mcp, "Antigravity bundled MCP"), "Antigravity bundled MCP")

    for plugin_root, label, client in (
        (sandbox.codex_plugin_root, "Codex", "codex"),
        (sandbox.claude_plugin_root, "Claude", "claude"),
        (sandbox.antigravity_plugin_root, "Antigravity", "antigravity"),
    ):
        _validate_plugin_skill(plugin_root / "skills" / SKILL_DIR_NAME, expected_version, label)
        marker = _read_json_object(plugin_root / ".qiongli-managed.json")
        _expect(marker.get("managed_by") == "qiongli-cli", f"{label} plugin has managed marker")
        _expect(marker.get("surface") == "plugin", f"{label} plugin marker surface is plugin")
        target = targets_by_client.get(client)
        if target is not None:
            _validate_marker_platform_target(marker, target, label)

    if sandbox.antigravity_config_path.exists():
        _expect_no_json_server(_read_json_object(sandbox.antigravity_config_path), "Antigravity global MCP")
    _expect_json_server(_mcp_server(_read_json_object(sandbox.hermes_config_path), "Hermes MCP"), "Hermes MCP")

    installed = check_payload.get("installed")
    _expect(isinstance(installed, dict), "qiongli check payload has installed object")
    for client, expected_surface in {
        "codex": "plugin",
        "claude": "plugin",
        "antigravity": "plugin",
        "hermes": "mcp",
    }.items():
        item = installed.get(client) if isinstance(installed, dict) else None
        _expect(isinstance(item, dict), f"qiongli check reports {client}")
        _expect(item.get("installed") is True, f"qiongli check reports {client} installed")
        _expect(item.get("surface") == expected_surface, f"qiongli check reports {client} surface={expected_surface}")
        if client == "codex":
            plugin = item.get("plugin")
            if isinstance(plugin, dict) and "active" in plugin:
                _expect(plugin.get("active") is not False, "qiongli check reports Codex plugin is active or unverifiable")
        if client == "antigravity":
            mcp = item.get("mcp")
            _expect(isinstance(mcp, dict), "qiongli check reports Antigravity MCP details")
            _expect(mcp.get("path") == str(sandbox.antigravity_plugin_root / "mcp_config.json"), "qiongli check reports Antigravity bundled MCP path")
            _expect(mcp.get("source") == "plugin", "qiongli check reports Antigravity MCP source=plugin")


def local_install_acceptance_targets(repo_root: Path) -> dict[str, PlatformTarget]:
    return {
        target_id: target
        for target_id, target in load_platform_targets(repo_root).items()
        if target.smoke.get("client_activation_check") == "local_install_acceptance"
    }


def _local_acceptance_targets_by_client(repo_root: Path) -> dict[str, PlatformTarget]:
    targets_by_client: dict[str, PlatformTarget] = {}
    for target_id, target in local_install_acceptance_targets(repo_root).items():
        recommended_key = target.release_download.get("recommended_key")
        if not isinstance(recommended_key, str) or not recommended_key:
            raise LocalInstallCheckError(
                f"local-install acceptance target {target_id} has no release_download.recommended_key"
            )
        client = LOCAL_INSTALL_RECOMMENDED_KEY_CLIENTS.get(recommended_key)
        if client is None:
            raise LocalInstallCheckError(
                "local-install acceptance target "
                f"{target_id} has no client validation mapping for "
                f"release_download.recommended_key={recommended_key!r}"
            )
        client_id, _label = client
        targets_by_client[client_id] = target
    return targets_by_client


def _validate_marker_platform_target(marker: dict[str, Any], target: PlatformTarget, label: str) -> None:
    payload = marker.get("platform_target")
    if not isinstance(payload, dict):
        payload = {}
    expected = {
        "target_id": target.target_id,
        "artifact_kind": target.artifact_kind,
        "archive_format": target.archive_format,
        "bundled_mcp_mode": target.bundled_mcp_mode,
        "command_surface": target.command_surface,
        "validator": target.validator,
    }
    for key, value in expected.items():
        _expect(
            payload.get(key) == value,
            f"{label} plugin marker platform_target.{key} expected {value}",
        )


def _validate_plugin_skill(skill_dir: Path, expected_version: str, label: str) -> None:
    _expect(skill_dir.is_dir(), f"{label} plugin skill directory exists")
    _expect((skill_dir / "VERSION").read_text(encoding="utf-8").strip() == expected_version, f"{label} skill version matches release")
    subject_manifest = _read_json_object(skill_dir / "SUBJECT_MANIFEST.json")
    _expect(subject_manifest.get("subject") == "core", f"{label} skill subject is core")
    _expect(subject_manifest.get("coverage") == "complete", f"{label} skill coverage is complete")
    frontmatter = _read_skill_frontmatter(skill_dir / "SKILL.md")
    _expect(frontmatter.get("name") in {PLUGIN_ID, SKILL_DIR_NAME}, f"{label} skill frontmatter has expected name")
    _expect(isinstance(frontmatter.get("description"), str), f"{label} skill frontmatter description is valid YAML")


def _expect_json_server(server: object, label: str) -> None:
    _expect(isinstance(server, dict), f"{label} server is present")
    _expect(server.get("command") == "qiongli", f"{label} command is qiongli")
    _expect(server.get("args") == QIONGLI_MCP_ARGS, f"{label} args use qiongli mcp serve stdio")


def validate_lifecycle_mcp_tools(
    repo_root: Path,
    sandbox: InstallSandbox,
    env: dict[str, str],
    *,
    python: str = sys.executable,
) -> None:
    messages = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "release-local-install-check", "version": "0"},
            },
        },
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    ]
    stdin = "\n".join(json.dumps(message) for message in messages) + "\n"
    mcp_env = dict(env)
    mcp_env.setdefault("QIONGLI_CONFIG_HOME", str(sandbox.root / "qiongli-config"))
    result = subprocess.run(
        [python, "-m", "bridges.mcp_server_stdio"],
        cwd=str(repo_root),
        env=mcp_env,
        input=stdin,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        timeout=15,
        check=False,
    )
    if result.returncode != 0:
        raise LocalInstallCheckError(
            "lifecycle MCP tools/list failed with exit code "
            f"{result.returncode}\n{result.stderr.rstrip()}"
        )
    try:
        responses = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    except json.JSONDecodeError as exc:
        raise LocalInstallCheckError(
            f"lifecycle MCP tools/list returned invalid JSON: {exc}"
        ) from exc
    tools_response = next(
        (response for response in responses if isinstance(response, dict) and response.get("id") == 2),
        None,
    )
    if not isinstance(tools_response, dict):
        raise LocalInstallCheckError("lifecycle MCP tools/list returned no response")
    result_payload = tools_response.get("result")
    if not isinstance(result_payload, dict):
        raise LocalInstallCheckError("lifecycle MCP tools/list returned no result object")
    tools = result_payload.get("tools")
    if not isinstance(tools, list):
        raise LocalInstallCheckError("lifecycle MCP tools/list returned invalid tools payload")
    validate_lifecycle_mcp_tool_names(
        [str(tool.get("name", "")) for tool in tools if isinstance(tool, dict)]
    )


def validate_lifecycle_mcp_tool_names(tool_names: list[str]) -> None:
    names = set(tool_names)
    missing = [name for name in REQUIRED_LIFECYCLE_MCP_TOOLS if name not in names]
    if missing:
        raise LocalInstallCheckError(
            "missing lifecycle MCP tools: " + ", ".join(missing)
        )


def _mcp_server(config: dict[str, Any], label: str) -> object:
    servers = config.get("mcpServers")
    _expect(isinstance(servers, dict), f"{label} config has mcpServers object")
    return servers.get(PLUGIN_ID)


def _expect_no_json_server(config: dict[str, Any], label: str) -> None:
    servers = config.get("mcpServers")
    if servers is None:
        return
    _expect(isinstance(servers, dict), f"{label} config has mcpServers object")
    _expect(PLUGIN_ID not in servers, f"{label} does not duplicate qiongli")


def _read_skill_frontmatter(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    _expect(text.startswith("---\n"), f"{path} starts with YAML frontmatter")
    marker = "\n---"
    end = text.find(marker, 4)
    _expect(end != -1, f"{path} closes YAML frontmatter")
    try:
        payload = yaml.safe_load(text[4:end]) or {}
    except yaml.YAMLError as exc:
        raise LocalInstallCheckError(f"{path} has invalid YAML frontmatter: {exc}") from exc
    _expect(isinstance(payload, dict), f"{path} YAML frontmatter is an object")
    return payload


def _read_json_object(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise LocalInstallCheckError(f"missing file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise LocalInstallCheckError(f"invalid JSON in {path}: {exc}") from exc
    _expect(isinstance(payload, dict), f"{path} contains a JSON object")
    return payload


def _expect(condition: bool, message: str) -> None:
    if not condition:
        raise LocalInstallCheckError(message)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run isolated local plugin install acceptance before release.")
    parser.add_argument("--root", required=True, help="Materialized release root to validate")
    parser.add_argument("--sandbox-root", help="Optional sandbox root for diagnostics; default is a temporary directory")
    parser.add_argument("--python", default=sys.executable, help="Python executable used to invoke qiongli.cli")
    parser.add_argument("--keep-sandbox", action="store_true", help="Keep the temporary sandbox for debugging")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    repo_root = Path(args.root).expanduser().resolve()
    if not (repo_root / "packages" / "python-qiongli" / "src" / "qiongli" / "cli.py").is_file():
        print(f"[local-install-check] release root is missing qiongli CLI source: {repo_root}", file=sys.stderr)
        return 2

    temp_dir: tempfile.TemporaryDirectory[str] | None = None
    if args.sandbox_root:
        sandbox_root = Path(args.sandbox_root).expanduser().resolve()
        sandbox_root.mkdir(parents=True, exist_ok=True)
    else:
        temp_dir = tempfile.TemporaryDirectory(prefix="qiongli-release-local-install-")
        sandbox_root = Path(temp_dir.name)

    sandbox = build_sandbox(sandbox_root)
    try:
        print("[local-install-check] installing plugin+mcp surfaces in isolated sandbox")
        payload = run_install_check(repo_root, sandbox, python=args.python)
        installed = payload.get("installed", {})
        surfaces = ", ".join(
            f"{client}={installed.get(client, {}).get('surface')}" for client in ("codex", "claude", "antigravity", "hermes")
        )
        print(f"[local-install-check] ok: {surfaces}")
        return 0
    except LocalInstallCheckError as exc:
        print(f"[local-install-check] FAIL: {exc}", file=sys.stderr)
        print(f"[local-install-check] sandbox: {sandbox.root}", file=sys.stderr)
        return 1
    finally:
        if temp_dir is not None and args.keep_sandbox:
            temp_dir._finalizer.detach()  # type: ignore[attr-defined]  # noqa: SLF001
            print(f"[local-install-check] kept sandbox: {sandbox.root}")
        elif temp_dir is not None:
            temp_dir.cleanup()


if __name__ == "__main__":
    raise SystemExit(main())
