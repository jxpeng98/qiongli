# Qiongli Cross-Platform MCP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standard Qiongli MCP server that works in Codex, Claude, Cursor, Gemini, and other MCP hosts, with both CLI-friendly and desktop-friendly provider key configuration.

**Architecture:** Keep the existing provider bridge contract (`MCPConnector`, `scripts/mcp_*.py`, and `MCPEvidence`) as the business layer. Add a platform-neutral MCP tool layer above it, plus a stdio MCP adapter for local desktop/CLI hosts and a Streamable HTTP-compatible local HTTP adapter for shared deployments. Secrets are resolved by a Qiongli-owned config layer so keys are not tied to any single host's config format.

**Tech Stack:** Python 3.12 stdlib for the stdio adapter, local HTTP adapter, and desktop config wizard; existing `PyYAML` dependency; `tomllib`/`json` config parsing; existing npm wrapper for packaged desktop installs.

---

## Source Notes

- Codex supports MCP stdio servers, Streamable HTTP servers, `env` / `env_vars`, bearer token auth, OAuth, and plugin-provided MCP servers in `config.toml`.
- MCP's official SDK documentation describes MCP servers exposing tools, resources, and prompts over stdio and Streamable HTTP. First release should implement the stable subset needed by desktop hosts: `initialize`, `tools/list`, `tools/call`, and `ping`.
- The current repo does not have a `dev/` directory. Existing MCP-like behavior is in `bridges/mcp_connectors.py` and `scripts/mcp_*.py`; these are command providers, not a standard MCP server.

## Target User Paths

1. **CLI user:** installs with PyPI/npm/source, runs `qiongli mcp configure`, `qiongli mcp doctor`, and `qiongli mcp config example --target codex`.
2. **Desktop user, direct MCP settings:** adds Qiongli MCP in Codex/Claude/Cursor settings and enters keys as MCP `env` values.
3. **Desktop user, safer wizard:** installs/adds the MCP once, then asks the agent to configure Qiongli. The MCP opens a local `localhost` wizard so the user enters keys outside chat.
4. **Team/remote user:** runs the HTTP MCP adapter on a server; provider keys live on the server, clients configure only URL and optional MCP auth token.

## File Structure

- Modify: `bridges/mcp_connectors.py`
  - Add config-derived environment injection and redacted provider resolution metadata.
- Create: `bridges/mcp_config.py`
  - Provider registry, env aliases, dotenv loading, user secrets loading/saving, redaction, status checks.
- Create: `bridges/mcp_tool_handlers.py`
  - Host-independent MCP tool handlers that call `MCPConnector` and config functions.
- Create: `bridges/mcp_server_stdio.py`
  - Minimal stdio JSON-RPC MCP adapter for local hosts.
- Create: `bridges/mcp_config_wizard.py`
  - Local one-shot HTTP configuration wizard for desktop users.
- Create: `bridges/mcp_cli.py`
  - CLI entrypoint for `serve`, `doctor`, `configure`, `config example`, and tool smoke tests.
- Modify: `qiongli/cli.py`
  - Add `qiongli mcp ...` subcommands that delegate to `bridges.mcp_cli`.
- Modify: `packages/npm-qiongli/lib/cli.mjs`
  - Add npm `qiongli mcp ...` routing into the bundled Python runtime.
- Modify: `packages/npm-qiongli/lib/python-runtime.mjs`
  - Allow invoking Python modules beyond `bridges.orchestrator`.
- Modify: `pyproject.toml`
  - Ensure `bridges` is packaged if PyPI users are expected to run `qiongli mcp`.
- Modify: `plugins/qiongli/.codex-plugin/plugin.json`
  - Add interface copy that says the plugin pairs with the Qiongli MCP server and supports desktop key setup.
- Modify: `plugins/qiongli/skills/qiongli-workflow/agents/openai.yaml`
  - Declare the Qiongli MCP dependency in Codex skill metadata.
- Modify: `.env.example`
  - Add desktop MCP env key examples and provider aliases.
- Create: `docs/advanced/qiongli-mcp.md`
  - English setup guide.
- Create: `docs/zh/advanced/qiongli-mcp.md`
  - Chinese setup guide.
- Modify: `docs/advanced/mcp-providers-setup.md`
  - Link to the new standard MCP guide.
- Modify: `docs/zh/advanced/mcp-providers-setup.md`
  - Link to the Chinese standard MCP guide.
- Create: `tests/test_mcp_config.py`
  - Config, redaction, and secret storage tests.
- Create: `tests/test_mcp_tool_handlers.py`
  - Tool handler tests without starting an MCP client.
- Create: `tests/test_mcp_stdio_server.py`
  - Protocol-level stdio tests.
- Create: `tests/test_mcp_cli.py`
  - CLI command and config example tests.
- Modify: `tests/test_npm_package_contract.py`
  - Ensure npm package exposes `qiongli mcp`.
- Modify: `tests/test_plugin_manifests.py`
  - Ensure plugin metadata references the Qiongli MCP dependency.

---

### Task 1: Add Provider Config Registry

**Files:**
- Create: `bridges/mcp_config.py`
- Test: `tests/test_mcp_config.py`

- [ ] **Step 1: Write failing tests for provider status and redaction**

```python
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.mcp_config import (
    PROVIDER_CONFIGS,
    build_provider_env,
    provider_config_status,
    redact_secret,
)


class MCPConfigTests(unittest.TestCase):
    def test_registry_includes_known_literature_providers(self) -> None:
        self.assertIn("semantic-scholar", PROVIDER_CONFIGS)
        self.assertIn("openalex", PROVIDER_CONFIGS)
        self.assertIn("zotero", PROVIDER_CONFIGS)

    def test_status_uses_env_aliases_without_revealing_values(self) -> None:
        with mock.patch.dict(os.environ, {"SEMANTIC_SCHOLAR_API_KEY": "s2-secret-value"}, clear=True):
            status = provider_config_status()

        semantic = status["semantic-scholar"]
        self.assertEqual(semantic["status"], "configured")
        self.assertEqual(semantic["configured_keys"], ["SEMANTIC_SCHOLAR_API_KEY"])
        self.assertNotIn("s2-secret-value", str(semantic))

    def test_redact_secret_masks_middle_characters(self) -> None:
        self.assertEqual(redact_secret("abcdef123456"), "abc...456")
        self.assertEqual(redact_secret("short"), "***")

    def test_build_provider_env_prefers_process_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            home = Path(tmp_dir)
            with mock.patch.dict(os.environ, {"S2_API_KEY": "from-env"}, clear=True):
                env = build_provider_env(home=home)

        self.assertEqual(env["S2_API_KEY"], "from-env")
        self.assertEqual(env["SEMANTIC_SCHOLAR_API_KEY"], "from-env")
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_config.py -q`

Expected: FAIL because `bridges.mcp_config` does not exist.

- [ ] **Step 3: Implement provider registry and redaction**

Create `bridges/mcp_config.py`:

```python
from __future__ import annotations

import json
import os
import stat
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ProviderConfigSpec:
    provider: str
    display_name: str
    required_any: tuple[str, ...] = ()
    optional: tuple[str, ...] = ()
    aliases: dict[str, tuple[str, ...]] | None = None
    help_text: str = ""


PROVIDER_CONFIGS: dict[str, ProviderConfigSpec] = {
    "semantic-scholar": ProviderConfigSpec(
        provider="semantic-scholar",
        display_name="Semantic Scholar",
        required_any=("SEMANTIC_SCHOLAR_API_KEY", "S2_API_KEY"),
        aliases={"SEMANTIC_SCHOLAR_API_KEY": ("S2_API_KEY",)},
        help_text="Recommended for repeated scholarly-search and citation-graph calls.",
    ),
    "openalex": ProviderConfigSpec(
        provider="openalex",
        display_name="OpenAlex",
        required_any=(),
        optional=("OPENALEX_EMAIL", "OPENALEX_API_KEY"),
        help_text="Optional metadata enrichment identity and provider-specific key support.",
    ),
    "zotero": ProviderConfigSpec(
        provider="zotero",
        display_name="Zotero",
        required_any=(),
        optional=("ZOTERO_API_KEY", "ZOTERO_LIBRARY_ID", "ZOTERO_LIBRARY_TYPE"),
        help_text="Optional full-text and library resolver configuration.",
    ),
}

SECRET_ENV_NAMES = {
    "SEMANTIC_SCHOLAR_API_KEY",
    "S2_API_KEY",
    "OPENALEX_API_KEY",
    "ZOTERO_API_KEY",
}


def qiongli_home(home: Path | None = None) -> Path:
    return (home or Path.home()) / ".qiongli"


def user_secrets_path(home: Path | None = None) -> Path:
    return qiongli_home(home) / "secrets.toml"


def project_env_path(cwd: Path | None = None) -> Path:
    return (cwd or Path.cwd()) / ".env"


def redact_secret(value: str) -> str:
    text = str(value)
    if len(text) < 10:
        return "***"
    return f"{text[:3]}...{text[-3:]}"


def redact_mapping(mapping: dict[str, str]) -> dict[str, str]:
    return {
        key: redact_secret(value) if key in SECRET_ENV_NAMES or key.endswith(("KEY", "TOKEN")) else value
        for key, value in mapping.items()
    }


def _parse_dotenv(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        if key:
            values[key] = value
    return values


def _load_user_secrets(home: Path | None = None) -> dict[str, str]:
    path = user_secrets_path(home)
    if not path.exists():
        return {}
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError:
        return {}
    secrets = data.get("secrets", {})
    if not isinstance(secrets, dict):
        return {}
    return {str(key): str(value) for key, value in secrets.items() if str(value).strip()}


def _apply_aliases(env: dict[str, str]) -> dict[str, str]:
    resolved = dict(env)
    for spec in PROVIDER_CONFIGS.values():
        for canonical, aliases in (spec.aliases or {}).items():
            if canonical in resolved:
                for alias in aliases:
                    resolved.setdefault(alias, resolved[canonical])
                continue
            for alias in aliases:
                if alias in resolved:
                    resolved[canonical] = resolved[alias]
                    break
    return resolved


def build_provider_env(
    *,
    cwd: Path | None = None,
    home: Path | None = None,
    base_env: dict[str, str] | None = None,
) -> dict[str, str]:
    merged: dict[str, str] = {}
    merged.update(_load_user_secrets(home))
    merged.update(_parse_dotenv(project_env_path(cwd)))
    merged.update(base_env if base_env is not None else os.environ)
    return _apply_aliases({key: str(value) for key, value in merged.items() if str(value).strip()})


def provider_config_status(
    *,
    cwd: Path | None = None,
    home: Path | None = None,
    base_env: dict[str, str] | None = None,
) -> dict[str, dict[str, Any]]:
    env = build_provider_env(cwd=cwd, home=home, base_env=base_env)
    out: dict[str, dict[str, Any]] = {}
    for provider, spec in PROVIDER_CONFIGS.items():
        configured_required = [name for name in spec.required_any if env.get(name)]
        configured_optional = [name for name in spec.optional if env.get(name)]
        if spec.required_any:
            state = "configured" if configured_required else "missing"
        else:
            state = "configured" if configured_optional else "optional"
        out[provider] = {
            "provider": provider,
            "display_name": spec.display_name,
            "status": state,
            "configured_keys": configured_required + configured_optional,
            "required_any": list(spec.required_any),
            "optional": list(spec.optional),
            "help": spec.help_text,
        }
    return out


def save_user_secrets(values: dict[str, str], *, home: Path | None = None) -> Path:
    allowed = {name for spec in PROVIDER_CONFIGS.values() for name in (*spec.required_any, *spec.optional)}
    clean = {key: value for key, value in values.items() if key in allowed and str(value).strip()}
    root = qiongli_home(home)
    root.mkdir(parents=True, exist_ok=True)
    path = user_secrets_path(home)
    existing = _load_user_secrets(home)
    existing.update(clean)
    body = "[secrets]\n" + "".join(
        f'{key} = {json.dumps(value)}\n'
        for key, value in sorted(existing.items())
    )
    path.write_text(body, encoding="utf-8")
    path.chmod(stat.S_IRUSR | stat.S_IWUSR)
    return path
```

- [ ] **Step 4: Run tests**

Run: `python3 -m pytest tests/test_mcp_config.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bridges/mcp_config.py tests/test_mcp_config.py
git commit -m "feat(mcp): add provider configuration registry"
```

---

### Task 2: Add Secret Storage Tests and Save Path Hardening

**Files:**
- Modify: `tests/test_mcp_config.py`
- Modify: `bridges/mcp_config.py`

- [ ] **Step 1: Add failing tests for desktop-safe secret save**

Append to `tests/test_mcp_config.py`:

```python
    def test_save_user_secrets_writes_user_only_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            home = Path(tmp_dir)
            path = save_user_secrets(
                {"SEMANTIC_SCHOLAR_API_KEY": "saved-secret", "UNKNOWN": "ignored"},
                home=home,
            )
            mode = path.stat().st_mode & 0o777
            env = build_provider_env(home=home, base_env={})

        self.assertEqual(mode, 0o600)
        self.assertEqual(env["SEMANTIC_SCHOLAR_API_KEY"], "saved-secret")
        self.assertNotIn("UNKNOWN", env)
```

Also update the import:

```python
from bridges.mcp_config import (
    PROVIDER_CONFIGS,
    build_provider_env,
    provider_config_status,
    redact_secret,
    save_user_secrets,
)
```

- [ ] **Step 2: Run test to verify current behavior**

Run: `python3 -m pytest tests/test_mcp_config.py::MCPConfigTests::test_save_user_secrets_writes_user_only_file -q`

Expected: PASS if Task 1 implementation included `save_user_secrets`; otherwise FAIL and fix the missing function exactly as shown in Task 1.

- [ ] **Step 3: Commit**

```bash
git add bridges/mcp_config.py tests/test_mcp_config.py
git commit -m "feat(mcp): persist provider secrets safely"
```

---

### Task 3: Inject Saved Provider Config Into Existing MCPConnector

**Files:**
- Modify: `bridges/mcp_connectors.py`
- Modify: `tests/test_mcp_connectors.py`

- [ ] **Step 1: Add failing test for connector env injection**

Append to `tests/test_mcp_connectors.py`:

```python
    def test_collect_external_provider_receives_saved_provider_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            home = root / "home"
            workspace = root / "workspace"
            workspace.mkdir()
            script = workspace / "env_echo.py"
            script.write_text(
                "import json, os, sys\n"
                "json.loads(sys.stdin.read())\n"
                "print(json.dumps({\n"
                "  'status': 'ok',\n"
                "  'summary': 'env ok',\n"
                "  'data': {'semantic': os.environ.get('SEMANTIC_SCHOLAR_API_KEY', '')}\n"
                "}))\n",
                encoding="utf-8",
            )
            from bridges.mcp_config import save_user_secrets

            save_user_secrets({"SEMANTIC_SCHOLAR_API_KEY": "saved-key"}, home=home)
            connector = MCPConnector(config_home=home)
            with mock.patch.dict(
                os.environ,
                {"RESEARCH_MCP_SCREENING_TRACKER_CMD": current_python_command(str(script))},
                clear=True,
            ):
                evidence = connector.collect("screening-tracker", {"topic": "demo"}, workspace)

        self.assertEqual(evidence.status, "ok")
        self.assertEqual(evidence.data["semantic"], "saved-key")
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_connectors.py::MCPConnectorTests::test_collect_external_provider_receives_saved_provider_env -q`

Expected: FAIL because `MCPConnector` does not accept `config_home`.

- [ ] **Step 3: Modify connector constructor and subprocess env**

In `bridges/mcp_connectors.py`, add the import:

```python
from bridges.mcp_config import build_provider_env
```

Update `MCPConnector.__init__`:

```python
    def __init__(
        self,
        timeout_seconds: int = 20,
        env_prefix: str = "RESEARCH_MCP_",
        config_home: Path | None = None,
    ):
        self.timeout_seconds = timeout_seconds
        self.env_prefix = env_prefix
        self.config_home = config_home
```

Before `subprocess.run(...)`, build the child env:

```python
            child_env = build_provider_env(cwd=cwd, home=self.config_home)
            run_result = subprocess.run(
                parsed_cmd,
                input=json.dumps(payload, ensure_ascii=False),
                capture_output=True,
                text=True,
                cwd=str(cwd),
                env=child_env,
                timeout=self.timeout_seconds,
                check=False,
            )
```

- [ ] **Step 4: Run connector tests**

Run: `python3 -m pytest tests/test_mcp_connectors.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bridges/mcp_connectors.py tests/test_mcp_connectors.py
git commit -m "feat(mcp): inject provider config into connector commands"
```

---

### Task 4: Add Host-Independent MCP Tool Handlers

**Files:**
- Create: `bridges/mcp_tool_handlers.py`
- Test: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Write failing tool handler tests**

Create `tests/test_mcp_tool_handlers.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.mcp_config import save_user_secrets
from bridges.mcp_tool_handlers import (
    MCP_TOOL_DEFINITIONS,
    call_qiongli_tool,
)


class MCPToolHandlerTests(unittest.TestCase):
    def test_tool_definitions_include_config_and_evidence_tools(self) -> None:
        names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertIn("qiongli_config_status", names)
        self.assertIn("qiongli_save_provider_config", names)
        self.assertIn("qiongli_collect_evidence", names)

    def test_config_status_tool_redacts_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            home = Path(tmp_dir)
            save_user_secrets({"SEMANTIC_SCHOLAR_API_KEY": "secret-value"}, home=home)
            result = call_qiongli_tool(
                "qiongli_config_status",
                {"home": str(home)},
                cwd=home,
            )

        self.assertFalse(result["isError"])
        self.assertIn("semantic-scholar", result["structuredContent"]["providers"])
        self.assertNotIn("secret-value", str(result))

    def test_save_provider_config_tool_persists_secret(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            home = Path(tmp_dir)
            result = call_qiongli_tool(
                "qiongli_save_provider_config",
                {
                    "home": str(home),
                    "values": {"SEMANTIC_SCHOLAR_API_KEY": "saved-secret"},
                },
                cwd=home,
            )

        self.assertFalse(result["isError"])
        self.assertEqual(result["structuredContent"]["saved_keys"], ["SEMANTIC_SCHOLAR_API_KEY"])
        self.assertNotIn("saved-secret", str(result))
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py -q`

Expected: FAIL because `bridges.mcp_tool_handlers` does not exist.

- [ ] **Step 3: Implement tool definitions and handlers**

Create `bridges/mcp_tool_handlers.py`:

```python
from __future__ import annotations

from pathlib import Path
from typing import Any

from bridges.mcp_config import (
    build_provider_env,
    provider_config_status,
    redact_mapping,
    save_user_secrets,
)
from bridges.mcp_connectors import MCPConnector


MCP_TOOL_DEFINITIONS: list[dict[str, Any]] = [
    {
        "name": "qiongli_config_status",
        "description": "Check Qiongli provider configuration without exposing secret values.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string"},
                "home": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_save_provider_config",
        "description": "Save provider API keys to the local Qiongli user secret store.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "home": {"type": "string"},
                "values": {
                    "type": "object",
                    "additionalProperties": {"type": "string"},
                },
            },
            "required": ["values"],
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_collect_evidence",
        "description": "Collect Qiongli research evidence from a configured provider.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {"type": "string"},
                "task_packet": {"type": "object"},
                "cwd": {"type": "string"},
                "home": {"type": "string"},
            },
            "required": ["provider", "task_packet"],
            "additionalProperties": False,
        },
    },
    {
        "name": "qiongli_list_provider_env",
        "description": "List recognized provider environment variable names and aliases.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": {"type": "string"},
                "home": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
]


def _path_arg(arguments: dict[str, Any], key: str, fallback: Path | None = None) -> Path | None:
    raw = arguments.get(key)
    if raw is None or str(raw).strip() == "":
        return fallback
    return Path(str(raw)).expanduser().resolve()


def _tool_result(text: str, structured: dict[str, Any], *, is_error: bool = False) -> dict[str, Any]:
    return {
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": is_error,
    }


def call_qiongli_tool(
    name: str,
    arguments: dict[str, Any] | None = None,
    *,
    cwd: Path | None = None,
) -> dict[str, Any]:
    args = arguments or {}
    run_cwd = _path_arg(args, "cwd", cwd) or Path.cwd()
    home = _path_arg(args, "home")

    if name == "qiongli_config_status":
        providers = provider_config_status(cwd=run_cwd, home=home)
        return _tool_result(
            "Qiongli provider configuration status collected.",
            {"providers": providers},
        )

    if name == "qiongli_save_provider_config":
        values = args.get("values", {})
        if not isinstance(values, dict):
            return _tool_result("values must be an object.", {"error": "invalid_values"}, is_error=True)
        saved_path = save_user_secrets({str(k): str(v) for k, v in values.items()}, home=home)
        saved_keys = sorted(k for k, v in values.items() if str(v).strip())
        return _tool_result(
            f"Saved {len(saved_keys)} Qiongli provider setting(s).",
            {"path": str(saved_path), "saved_keys": saved_keys},
        )

    if name == "qiongli_collect_evidence":
        provider = str(args.get("provider", "")).strip()
        task_packet = args.get("task_packet", {})
        if not provider or not isinstance(task_packet, dict):
            return _tool_result(
                "provider and task_packet are required.",
                {"error": "missing_provider_or_task_packet"},
                is_error=True,
            )
        evidence = MCPConnector(config_home=home).collect(provider, task_packet, run_cwd)
        return _tool_result(
            evidence.summary,
            {"evidence": evidence.to_dict()},
            is_error=evidence.status in {"error", "not_configured"},
        )

    if name == "qiongli_list_provider_env":
        env = redact_mapping(build_provider_env(cwd=run_cwd, home=home))
        return _tool_result(
            "Qiongli provider environment resolved with secret redaction.",
            {"env": env},
        )

    return _tool_result(f"Unknown Qiongli tool: {name}", {"error": "unknown_tool"}, is_error=True)
```

- [ ] **Step 4: Run tests**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): add qiongli tool handlers"
```

---

### Task 5: Implement Stdio MCP Adapter

**Files:**
- Create: `bridges/mcp_server_stdio.py`
- Test: `tests/test_mcp_stdio_server.py`

- [ ] **Step 1: Write failing protocol tests**

Create `tests/test_mcp_stdio_server.py`:

```python
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPStdioServerTests(unittest.TestCase):
    def _run_server(self, messages: list[dict]) -> list[dict]:
        payload = "\n".join(json.dumps(item) for item in messages) + "\n"
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_server_stdio"],
            input=payload,
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
            check=False,
            timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

    def test_initialize_and_list_tools(self) -> None:
        responses = self._run_server(
            [
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-03-26",
                        "capabilities": {},
                        "clientInfo": {"name": "test", "version": "0"},
                    },
                },
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
            ]
        )

        self.assertEqual(responses[0]["result"]["serverInfo"]["name"], "qiongli")
        tool_names = {tool["name"] for tool in responses[1]["result"]["tools"]}
        self.assertIn("qiongli_config_status", tool_names)

    def test_call_config_status_tool(self) -> None:
        responses = self._run_server(
            [
                {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "qiongli_config_status",
                        "arguments": {"cwd": str(REPO_ROOT)},
                    },
                },
            ]
        )

        self.assertFalse(responses[1]["result"]["isError"])
        self.assertIn("structuredContent", responses[1]["result"])
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_stdio_server.py -q`

Expected: FAIL because `bridges.mcp_server_stdio` does not exist.

- [ ] **Step 3: Implement minimal stdio MCP server**

Create `bridges/mcp_server_stdio.py`:

```python
from __future__ import annotations

import json
import sys
from typing import Any

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool


PROTOCOL_VERSION = "2025-03-26"


def _response(request_id: Any, result: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def _error(request_id: Any, code: int, message: str) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}}


def handle_message(message: dict[str, Any]) -> dict[str, Any] | None:
    request_id = message.get("id")
    method = str(message.get("method", ""))
    params = message.get("params", {})
    if not isinstance(params, dict):
        params = {}

    if method == "initialize":
        return _response(
            request_id,
            {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "qiongli", "version": "0.14.0"},
                "instructions": (
                    "Use Qiongli tools for academic research evidence collection and provider "
                    "configuration. Never ask users to paste API keys into chat; prefer the "
                    "configuration wizard or MCP env settings. Tool outputs redact secrets."
                ),
            },
        )
    if method == "notifications/initialized":
        return None
    if method == "ping":
        return _response(request_id, {})
    if method == "tools/list":
        return _response(request_id, {"tools": MCP_TOOL_DEFINITIONS})
    if method == "tools/call":
        name = str(params.get("name", ""))
        arguments = params.get("arguments", {})
        if not isinstance(arguments, dict):
            arguments = {}
        return _response(request_id, call_qiongli_tool(name, arguments))
    return _error(request_id, -32601, f"Method not found: {method}")


def main() -> int:
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            message = json.loads(line)
            if not isinstance(message, dict):
                raise ValueError("JSON-RPC message must be an object")
            response = handle_message(message)
        except Exception as exc:
            response = _error(None, -32603, f"Internal error: {exc}")
        if response is not None:
            sys.stdout.write(json.dumps(response, ensure_ascii=False) + "\n")
            sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run protocol tests**

Run: `python3 -m pytest tests/test_mcp_stdio_server.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bridges/mcp_server_stdio.py tests/test_mcp_stdio_server.py
git commit -m "feat(mcp): add stdio server adapter"
```

---

### Task 6: Add Desktop Local Configuration Wizard

**Files:**
- Create: `bridges/mcp_config_wizard.py`
- Modify: `bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Add failing test for wizard tool availability**

Append to `tests/test_mcp_tool_handlers.py`:

```python
    def test_open_config_wizard_tool_returns_local_url(self) -> None:
        result = call_qiongli_tool(
            "qiongli_open_config_wizard",
            {"host": "127.0.0.1", "port": 0},
        )

        self.assertFalse(result["isError"])
        self.assertTrue(result["structuredContent"]["url"].startswith("http://127.0.0.1:"))
        self.assertIn("token", result["structuredContent"])
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py::MCPToolHandlerTests::test_open_config_wizard_tool_returns_local_url -q`

Expected: FAIL because the tool is not registered.

- [ ] **Step 3: Implement wizard server**

Create `bridges/mcp_config_wizard.py`:

```python
from __future__ import annotations

import html
import json
import secrets
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse

from bridges.mcp_config import PROVIDER_CONFIGS, provider_config_status, save_user_secrets


class WizardState:
    def __init__(self, home: Path | None = None):
        self.token = secrets.token_urlsafe(24)
        self.home = home
        self.saved = False


def _html(state: WizardState) -> bytes:
    fields: list[str] = []
    for spec in PROVIDER_CONFIGS.values():
        for key in (*spec.required_any, *spec.optional):
            fields.append(
                f"<label>{html.escape(spec.display_name)} {html.escape(key)}"
                f"<input name='{html.escape(key)}' type='password' autocomplete='off'></label>"
            )
    body = f"""
<!doctype html>
<html>
<head><meta charset="utf-8"><title>Qiongli MCP Configuration</title></head>
<body>
  <h1>Qiongli MCP Configuration</h1>
  <p>Enter provider keys locally. Values are saved outside chat and redacted from MCP output.</p>
  <form method="post" action="/save?token={state.token}">
    {"<br>".join(fields)}
    <button type="submit">Save and test status</button>
  </form>
</body>
</html>
"""
    return body.encode("utf-8")


def create_wizard_server(host: str = "127.0.0.1", port: int = 0, home: Path | None = None) -> tuple[ThreadingHTTPServer, WizardState]:
    state = WizardState(home=home)

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, format: str, *args: Any) -> None:
            return

        def do_GET(self) -> None:
            parsed = urlparse(self.path)
            if parsed.path != "/":
                self.send_error(404)
                return
            body = _html(state)
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self) -> None:
            parsed = urlparse(self.path)
            if parsed.path != "/save" or parse_qs(parsed.query).get("token", [""])[0] != state.token:
                self.send_error(403)
                return
            length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(length).decode("utf-8")
            data = {key: value[-1] for key, value in parse_qs(raw).items() if value and value[-1].strip()}
            save_user_secrets(data, home=state.home)
            state.saved = True
            status = provider_config_status(home=state.home)
            body = json.dumps({"saved": True, "providers": status}, ensure_ascii=False, indent=2).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer((host, port), Handler)
    return server, state


def start_wizard(host: str = "127.0.0.1", port: int = 0, home: Path | None = None) -> dict[str, Any]:
    server, state = create_wizard_server(host=host, port=port, home=home)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    actual_host, actual_port = server.server_address
    return {
        "url": f"http://{actual_host}:{actual_port}/",
        "token": state.token,
        "host": actual_host,
        "port": actual_port,
    }
```

- [ ] **Step 4: Register wizard tool**

In `bridges/mcp_tool_handlers.py`, add import:

```python
from bridges.mcp_config_wizard import start_wizard
```

Add tool definition:

```python
    {
        "name": "qiongli_open_config_wizard",
        "description": "Open a local browser-based configuration wizard for desktop users.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "host": {"type": "string", "default": "127.0.0.1"},
                "port": {"type": "integer", "default": 0},
                "home": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
```

Add handler before the unknown-tool fallback:

```python
    if name == "qiongli_open_config_wizard":
        host = str(args.get("host") or "127.0.0.1")
        port = int(args.get("port") or 0)
        wizard = start_wizard(host=host, port=port, home=home)
        return _tool_result(
            f"Open the local Qiongli configuration wizard: {wizard['url']}",
            wizard,
        )
```

- [ ] **Step 5: Run tool tests**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py -q`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add bridges/mcp_config_wizard.py bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): add desktop provider config wizard"
```

---

### Task 7: Add `qiongli mcp` CLI Entrypoint

**Files:**
- Create: `bridges/mcp_cli.py`
- Modify: `qiongli/cli.py`
- Test: `tests/test_mcp_cli.py`

- [ ] **Step 1: Write failing CLI tests**

Create `tests/test_mcp_cli.py`:

```python
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPCLITests(unittest.TestCase):
    def test_mcp_doctor_json(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "doctor", "--json"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertIn("providers", payload)

    def test_mcp_config_example_codex(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "codex"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("[mcp_servers.qiongli]", result.stdout)
        self.assertIn("SEMANTIC_SCHOLAR_API_KEY", result.stdout)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m pytest tests/test_mcp_cli.py -q`

Expected: FAIL because `bridges.mcp_cli` does not exist.

- [ ] **Step 3: Implement CLI module**

Create `bridges/mcp_cli.py`:

```python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from bridges.mcp_config import provider_config_status, save_user_secrets
from bridges.mcp_server_stdio import main as stdio_main


def codex_example() -> str:
    return """[mcp_servers.qiongli]
enabled = true
command = "python3"
args = ["-m", "bridges.mcp_server_stdio"]

[mcp_servers.qiongli.env]
SEMANTIC_SCHOLAR_API_KEY = "paste-key-here"
OPENALEX_EMAIL = "you@example.com"
"""


def claude_example() -> str:
    return """{
  "mcpServers": {
    "qiongli": {
      "command": "python3",
      "args": ["-m", "bridges.mcp_server_stdio"],
      "env": {
        "SEMANTIC_SCHOLAR_API_KEY": "paste-key-here",
        "OPENALEX_EMAIL": "you@example.com"
      }
    }
  }
}
"""


def cmd_doctor(args: argparse.Namespace) -> int:
    payload = {"providers": provider_config_status(cwd=Path(args.cwd).resolve())}
    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    else:
        for provider, status in payload["providers"].items():
            print(f"{provider}: {status['status']} ({', '.join(status['configured_keys']) or 'no keys'})")
    return 0


def cmd_config_example(args: argparse.Namespace) -> int:
    if args.target == "codex":
        print(codex_example())
    elif args.target in {"claude", "cursor", "gemini"}:
        print(claude_example())
    else:
        raise ValueError(f"Unsupported target: {args.target}")
    return 0


def cmd_configure(args: argparse.Namespace) -> int:
    values: dict[str, str] = {}
    for item in args.set or []:
        if "=" not in item:
            raise ValueError(f"Expected KEY=VALUE, got {item}")
        key, value = item.split("=", 1)
        values[key.strip()] = value.strip()
    path = save_user_secrets(values)
    print(f"Saved {len(values)} setting(s) to {path}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="qiongli mcp")
    sub = parser.add_subparsers(dest="cmd", required=True)

    serve = sub.add_parser("serve")
    serve.add_argument("--transport", choices=("stdio",), default="stdio")

    doctor = sub.add_parser("doctor")
    doctor.add_argument("--cwd", default=".")
    doctor.add_argument("--json", action="store_true")

    configure = sub.add_parser("configure")
    configure.add_argument("--set", action="append", default=[])

    config = sub.add_parser("config")
    config_sub = config.add_subparsers(dest="config_cmd", required=True)
    example = config_sub.add_parser("example")
    example.add_argument("--target", choices=("codex", "claude", "cursor", "gemini"), required=True)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.cmd == "serve":
        return stdio_main()
    if args.cmd == "doctor":
        return cmd_doctor(args)
    if args.cmd == "configure":
        return cmd_configure(args)
    if args.cmd == "config" and args.config_cmd == "example":
        return cmd_config_example(args)
    parser.error("unsupported mcp command")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Wire Python CLI**

In `qiongli/cli.py`, add an MCP subparser near other subcommands:

```python
    mcp = subparsers.add_parser("mcp", help="Run and configure the Qiongli MCP server")
    mcp.add_argument("mcp_args", nargs=argparse.REMAINDER)
```

In `main`, before the final error path:

```python
    if args.cmd == "mcp":
        from bridges.mcp_cli import main as mcp_main
        return mcp_main(args.mcp_args)
```

- [ ] **Step 5: Run CLI tests**

Run: `python3 -m pytest tests/test_mcp_cli.py -q`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add bridges/mcp_cli.py qiongli/cli.py tests/test_mcp_cli.py
git commit -m "feat(mcp): add qiongli mcp cli"
```

---

### Task 8: Wire npm Package Runtime

**Files:**
- Modify: `packages/npm-qiongli/lib/python-runtime.mjs`
- Modify: `packages/npm-qiongli/lib/cli.mjs`
- Test: `packages/npm-qiongli/test/*.test.mjs`
- Modify: `tests/test_npm_package_contract.py`

- [ ] **Step 1: Add failing npm CLI test**

Add to `packages/npm-qiongli/test/cli.test.mjs` or create it if missing:

```javascript
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { main } from '../lib/cli.mjs';

test('mcp config example routes to python runtime', async () => {
  const writes = [];
  const errors = [];
  const code = await main(['mcp', 'config', 'example', '--target', 'codex'], {
    stdout: { write: (text) => writes.push(String(text)) },
    stderr: { write: (text) => errors.push(String(text)) },
  });
  assert.equal(code, 0);
  assert.match(writes.join(''), /mcp_servers\.qiongli|Python bridge/);
});
```

- [ ] **Step 2: Run npm tests to verify failure**

Run: `npm --prefix packages/npm-qiongli test`

Expected: FAIL because `mcp` is unknown.

- [ ] **Step 3: Generalize Python runtime module invocation**

In `packages/npm-qiongli/lib/python-runtime.mjs`, add:

```javascript
export function runPythonModule({ packageRoot, module, args, cwd = process.cwd(), env = process.env, stdio = 'inherit' }) {
  const runtime = checkPythonRuntime();
  if (!runtime.ok) {
    console.error(`[qiongli] ${runtime.message}`);
    console.error(`Hint: ${runtime.hint}`);
    return 1;
  }
  const pythonPath = path.join(packageRoot, 'python-runtime');
  const childEnv = {
    ...env,
    PYTHONPATH: env.PYTHONPATH ? `${pythonPath}${path.delimiter}${env.PYTHONPATH}` : pythonPath,
  };
  const result = nodeSpawnSync(runtime.python, ['-m', module, ...args], {
    cwd,
    env: childEnv,
    stdio,
  });
  return typeof result.status === 'number' ? result.status : 1;
}
```

Then make `runBridgeCommand` call `runPythonModule`:

```javascript
export function runBridgeCommand(options) {
  return runPythonModule({ ...options, module: 'bridges.orchestrator' });
}
```

- [ ] **Step 4: Route npm `mcp` command**

In `packages/npm-qiongli/lib/cli.mjs`, import `runPythonModule`:

```javascript
import { checkPythonRuntime, runBridgeCommand, runPythonModule } from './python-runtime.mjs';
```

Before bridge commands:

```javascript
  if (parsed.command === 'mcp') {
    return runPythonModule({ packageRoot: root, module: 'bridges.mcp_cli', args: parsed.rest });
  }
```

- [ ] **Step 5: Run npm tests**

Run: `npm --prefix packages/npm-qiongli test`

Expected: PASS.

- [ ] **Step 6: Run package contract tests**

Run: `python3 -m pytest tests/test_npm_package_contract.py -q`

Expected: PASS or update the contract to include `mcp` in the npm command surface.

- [ ] **Step 7: Commit**

```bash
git add packages/npm-qiongli/lib/python-runtime.mjs packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/test tests/test_npm_package_contract.py
git commit -m "feat(mcp): expose qiongli mcp through npm runtime"
```

---

### Task 9: Package `bridges` for PyPI and npm Runtime

**Files:**
- Modify: `pyproject.toml`
- Modify: `scripts/sync_npm_package_payload.py` only if the existing runtime directory sync does not copy the new files
- Test: `tests/test_distribution_payloads.py`
- Test: `tests/test_npm_package_contract.py`

- [ ] **Step 1: Add failing distribution assertion**

In `tests/test_distribution_payloads.py`, add:

```python
def test_mcp_runtime_files_are_in_npm_python_runtime() -> None:
    root = Path(__file__).resolve().parents[1]
    runtime = root / "packages" / "npm-qiongli" / "python-runtime" / "bridges"
    expected = [
        "mcp_cli.py",
        "mcp_config.py",
        "mcp_config_wizard.py",
        "mcp_server_stdio.py",
        "mcp_tool_handlers.py",
    ]
    for name in expected:
        assert (runtime / name).is_file(), f"missing npm runtime MCP file: {name}"
```

- [ ] **Step 2: Run test to verify failure**

Run: `python3 -m pytest tests/test_distribution_payloads.py::test_mcp_runtime_files_are_in_npm_python_runtime -q`

Expected: FAIL until the runtime copy path is updated.

- [ ] **Step 3: Include `bridges` in PyPI package**

In `pyproject.toml`, change:

```toml
[tool.setuptools]
packages = ["qiongli", "research_skills"]
```

to:

```toml
[tool.setuptools]
packages = ["qiongli", "research_skills", "bridges", "bridges.providers"]
```

- [ ] **Step 4: Sync runtime files into npm package**

Run the repo's existing sync command:

```bash
python3 scripts/sync_npm_package_payload.py
```

The script already copies the root `bridges/` directory into `packages/npm-qiongli/python-runtime/bridges/`. If this command does not copy the MCP files, modify `scripts/sync_npm_package_payload.py` by keeping `"bridges"` in the `runtime_dirs` tuple and ensuring `ignore_patterns()` excludes only `__pycache__`, pytest/mypy caches, node/build artifacts, and `.pyc`/`.pyo` files.

Expected copied files:

```text
packages/npm-qiongli/python-runtime/bridges/mcp_cli.py
packages/npm-qiongli/python-runtime/bridges/mcp_config.py
packages/npm-qiongli/python-runtime/bridges/mcp_config_wizard.py
packages/npm-qiongli/python-runtime/bridges/mcp_server_stdio.py
packages/npm-qiongli/python-runtime/bridges/mcp_tool_handlers.py
```

- [ ] **Step 5: Run distribution tests**

Run: `python3 -m pytest tests/test_distribution_payloads.py tests/test_npm_package_contract.py -q`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add pyproject.toml packages/npm-qiongli/python-runtime tests/test_distribution_payloads.py
git commit -m "build(mcp): package qiongli mcp runtime"
```

---

### Task 10: Add Desktop MCP Config Examples

**Files:**
- Modify: `bridges/mcp_cli.py`
- Modify: `.env.example`
- Create: `docs/advanced/qiongli-mcp.md`
- Create: `docs/zh/advanced/qiongli-mcp.md`
- Modify: `docs/advanced/mcp-providers-setup.md`
- Modify: `docs/zh/advanced/mcp-providers-setup.md`
- Test: `tests/test_mcp_cli.py`
- Test: `tests/test_mcp_provider_docs.py`

- [ ] **Step 1: Add tests for all target examples**

Append to `tests/test_mcp_cli.py`:

```python
    def test_mcp_config_examples_cover_all_desktop_targets(self) -> None:
        for target in ("codex", "claude", "cursor", "gemini"):
            result = subprocess.run(
                [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", target],
                cwd=str(REPO_ROOT),
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, target)
            self.assertIn("qiongli", result.stdout)
            self.assertIn("SEMANTIC_SCHOLAR_API_KEY", result.stdout)
```

- [ ] **Step 2: Run test**

Run: `python3 -m pytest tests/test_mcp_cli.py -q`

Expected: PASS if Task 7 examples covered all targets; otherwise update `bridges/mcp_cli.py`.

- [ ] **Step 3: Update `.env.example`**

Add:

```env
# ==========================================
# 0. Desktop MCP direct-key setup
# ==========================================
# Desktop users can enter these values in Codex/Claude/Cursor MCP settings
# under the qiongli MCP server's env block. Prefer the local Qiongli config
# wizard when possible so keys do not live in platform config files.
#
# SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
# S2_API_KEY="alias-for-semantic-scholar-key"
# OPENALEX_EMAIL="you@example.com"
# OPENALEX_API_KEY="your-openalex-key-if-your-provider-requires-one"
# ZOTERO_API_KEY="your-zotero-key"
```

- [ ] **Step 4: Add English docs**

Create `docs/advanced/qiongli-mcp.md` with:

~~~markdown
# Qiongli MCP Server

Qiongli exposes its research provider layer as a standard MCP server.

## Recommended setup

Use stdio for local desktop apps:

```toml
[mcp_servers.qiongli]
enabled = true
command = "python3"
args = ["-m", "bridges.mcp_server_stdio"]

[mcp_servers.qiongli.env]
SEMANTIC_SCHOLAR_API_KEY = "paste-key-here"
OPENALEX_EMAIL = "you@example.com"
```

## Desktop key setup

You can either enter keys directly in MCP settings or ask the agent to open the
Qiongli configuration wizard. The wizard stores keys in `~/.qiongli/secrets.toml`
with user-only file permissions and redacts keys from tool output.

## Tools

- `qiongli_config_status`
- `qiongli_open_config_wizard`
- `qiongli_save_provider_config`
- `qiongli_collect_evidence`
- `qiongli_list_provider_env`

## Secret handling

Never paste provider keys into ordinary chat. Use MCP environment settings or
the local wizard. `qiongli mcp doctor --json` reports only key names and
configured/missing state.
```
~~~

- [ ] **Step 5: Add Chinese docs**

Create `docs/zh/advanced/qiongli-mcp.md` with:

~~~markdown
# Qiongli MCP Server

Qiongli 可以把研究 provider 层暴露为标准 MCP server。

## 推荐配置

本地桌面应用优先使用 stdio：

```toml
[mcp_servers.qiongli]
enabled = true
command = "python3"
args = ["-m", "bridges.mcp_server_stdio"]

[mcp_servers.qiongli.env]
SEMANTIC_SCHOLAR_API_KEY = "在这里填写 key"
OPENALEX_EMAIL = "you@example.com"
```

## 桌面版 key 配置

你可以在 MCP 设置里直接填写 key，也可以让 agent 打开 Qiongli 本地配置向导。
配置向导会把 key 保存到 `~/.qiongli/secrets.toml`，并设置为仅当前用户可读写。
MCP 输出、doctor 结果和日志都会隐藏 key 的真实值。

## 工具

- `qiongli_config_status`
- `qiongli_open_config_wizard`
- `qiongli_save_provider_config`
- `qiongli_collect_evidence`
- `qiongli_list_provider_env`

## Secret 处理

不要把 provider key 粘贴到普通聊天消息里。优先使用 MCP 环境变量设置或本地配置向导。
`qiongli mcp doctor --json` 只报告 key 名称和 configured/missing 状态，不显示 key 值。
```
~~~

- [ ] **Step 6: Link provider setup docs**

At the top of both MCP provider setup guides, add:

```markdown
> For standard MCP server setup across Codex, Claude, Cursor, and Gemini, see
> [Qiongli MCP Server](./qiongli-mcp.md).
```

Use the Chinese equivalent in `docs/zh/advanced/mcp-providers-setup.md`.

- [ ] **Step 7: Run docs tests**

Run: `python3 -m pytest tests/test_mcp_provider_docs.py tests/test_mcp_cli.py -q`

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add .env.example docs/advanced/qiongli-mcp.md docs/zh/advanced/qiongli-mcp.md docs/advanced/mcp-providers-setup.md docs/zh/advanced/mcp-providers-setup.md bridges/mcp_cli.py tests/test_mcp_cli.py tests/test_mcp_provider_docs.py
git commit -m "docs(mcp): add cross-platform qiongli setup guide"
```

---

### Task 11: Update Codex Plugin Metadata

**Files:**
- Modify: `plugins/qiongli/.codex-plugin/plugin.json`
- Modify: `plugins/qiongli/skills/qiongli-workflow/agents/openai.yaml`
- Test: `tests/test_plugin_manifests.py`
- Test: `tests/test_plugin_artifacts.py`

- [ ] **Step 1: Add manifest test**

In `tests/test_plugin_manifests.py`, add:

```python
def test_codex_plugin_mentions_qiongli_mcp_dependency() -> None:
    root = Path(__file__).resolve().parents[1]
    manifest = json.loads((root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").read_text())
    long_description = manifest["interface"]["longDescription"]
    assert "MCP" in long_description
    assert "provider key" in long_description.lower()
```

- [ ] **Step 2: Run manifest test to verify failure**

Run: `python3 -m pytest tests/test_plugin_manifests.py::test_codex_plugin_mentions_qiongli_mcp_dependency -q`

Expected: FAIL because current description does not mention MCP provider key setup.

- [ ] **Step 3: Update plugin manifest copy**

In `plugins/qiongli/.codex-plugin/plugin.json`, change `interface.longDescription` to:

```json
"longDescription": "Install the Qiongli skill through the Codex plugin marketplace. It provides standardized paper planning, literature review, manuscript writing, compliance, submission, presentation, and research-code workflows. Pair it with the Qiongli MCP server to collect provider-backed research evidence and configure provider keys such as Semantic Scholar and OpenAlex from Codex desktop settings or the local Qiongli configuration wizard."
```

- [ ] **Step 4: Update `agents/openai.yaml` dependency metadata**

In `plugins/qiongli/skills/qiongli-workflow/agents/openai.yaml`, add or update:

```yaml
dependencies:
  tools:
    - type: "mcp"
      value: "qiongli"
      description: "Qiongli MCP server for research evidence collection and provider key configuration."
      transport: "stdio"
```

- [ ] **Step 5: Run plugin tests**

Run: `python3 -m pytest tests/test_plugin_manifests.py tests/test_plugin_artifacts.py -q`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add plugins/qiongli/.codex-plugin/plugin.json plugins/qiongli/skills/qiongli-workflow/agents/openai.yaml tests/test_plugin_manifests.py
git commit -m "feat(plugin): declare qiongli mcp setup path"
```

---

### Task 12: Add Provider Smoke Tests

**Files:**
- Modify: `bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Add failing test for provider smoke tool**

Append to `tests/test_mcp_tool_handlers.py`:

```python
    def test_test_provider_tool_handles_missing_semantic_scholar_key(self) -> None:
        result = call_qiongli_tool(
            "qiongli_test_provider",
            {"provider": "semantic-scholar", "home": "/tmp/qiongli-missing-home-for-test"},
        )

        self.assertIn("status", result["structuredContent"])
        self.assertNotIn("secret", str(result).lower())
```

- [ ] **Step 2: Run test to verify failure**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py::MCPToolHandlerTests::test_test_provider_tool_handles_missing_semantic_scholar_key -q`

Expected: FAIL because the tool does not exist.

- [ ] **Step 3: Add tool definition**

In `bridges/mcp_tool_handlers.py`, add:

```python
    {
        "name": "qiongli_test_provider",
        "description": "Run a lightweight provider readiness test without exposing secrets.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {"type": "string"},
                "cwd": {"type": "string"},
                "home": {"type": "string"},
            },
            "required": ["provider"],
            "additionalProperties": False,
        },
    },
```

Add handler:

```python
    if name == "qiongli_test_provider":
        provider = str(args.get("provider", "")).strip()
        status = provider_config_status(cwd=run_cwd, home=home)
        if provider in status:
            provider_status = status[provider]
            return _tool_result(
                f"{provider} status: {provider_status['status']}",
                {"provider": provider, "status": provider_status["status"], "configured_keys": provider_status["configured_keys"]},
            )
        return _tool_result(
            f"Unknown provider: {provider}",
            {"provider": provider, "status": "unknown"},
            is_error=True,
        )
```

- [ ] **Step 4: Run tests**

Run: `python3 -m pytest tests/test_mcp_tool_handlers.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): add provider readiness tool"
```

---

### Task 13: Add Optional Streamable HTTP Transport

**Files:**
- Create: `bridges/mcp_server_http.py`
- Modify: `bridges/mcp_cli.py`
- Create: `tests/test_mcp_http_server.py`

- [ ] **Step 1: Write failing HTTP smoke test**

Create `tests/test_mcp_http_server.py`:

```python
from __future__ import annotations

import json
import subprocess
import sys
import time
import unittest
import urllib.request
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPHTTPServerTests(unittest.TestCase):
    def test_http_server_initialize(self) -> None:
        proc = subprocess.Popen(
            [sys.executable, "-m", "bridges.mcp_server_http", "--host", "127.0.0.1", "--port", "18765"],
            cwd=str(REPO_ROOT),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            time.sleep(1.0)
            body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}).encode()
            req = urllib.request.Request(
                "http://127.0.0.1:18765/mcp",
                data=body,
                headers={"Content-Type": "application/json"},
            )
            with urllib.request.urlopen(req, timeout=5) as response:
                payload = json.loads(response.read())
            self.assertEqual(payload["result"]["serverInfo"]["name"], "qiongli")
        finally:
            proc.terminate()
            proc.wait(timeout=5)
```

- [ ] **Step 2: Run test to verify failure**

Run: `python3 -m pytest tests/test_mcp_http_server.py -q`

Expected: FAIL because HTTP server is missing.

- [ ] **Step 3: Implement HTTP adapter**

Create `bridges/mcp_server_http.py`:

```python
from __future__ import annotations

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

from bridges.mcp_server_stdio import handle_message


class MCPHTTPHandler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: Any) -> None:
        return

    def do_POST(self) -> None:
        if self.path != "/mcp":
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        try:
            message = json.loads(self.rfile.read(length).decode("utf-8"))
            response = handle_message(message)
            body = json.dumps(response or {}, ensure_ascii=False).encode("utf-8")
        except Exception as exc:
            body = json.dumps(
                {"jsonrpc": "2.0", "id": None, "error": {"code": -32603, "message": str(exc)}},
                ensure_ascii=False,
            ).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    args = parser.parse_args(argv)
    server = ThreadingHTTPServer((args.host, args.port), MCPHTTPHandler)
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Wire CLI HTTP transport**

In `bridges/mcp_cli.py`, add `http` to transport choices:

```python
    serve.add_argument("--transport", choices=("stdio", "http"), default="stdio")
    serve.add_argument("--host", default="127.0.0.1")
    serve.add_argument("--port", type=int, default=8765)
```

Update `serve` handling:

```python
    if args.cmd == "serve":
        if args.transport == "stdio":
            return stdio_main()
        from bridges.mcp_server_http import main as http_main
        return http_main(["--host", args.host, "--port", str(args.port)])
```

- [ ] **Step 5: Run HTTP tests**

Run: `python3 -m pytest tests/test_mcp_http_server.py tests/test_mcp_cli.py -q`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add bridges/mcp_server_http.py bridges/mcp_cli.py tests/test_mcp_http_server.py
git commit -m "feat(mcp): add local http transport"
```

---

### Task 14: Add Verification and Safety Tests

**Files:**
- Modify: `tests/test_mcp_stdio_server.py`
- Modify: `tests/test_mcp_config.py`

- [ ] **Step 1: Add no-secret-output regression tests**

Append to `tests/test_mcp_stdio_server.py`:

```python
    def test_stdio_does_not_emit_secret_values(self) -> None:
        responses = self._run_server(
            [
                {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "qiongli_list_provider_env",
                        "arguments": {"cwd": str(REPO_ROOT)},
                    },
                },
            ]
        )

        self.assertNotIn("sk-", json.dumps(responses))
        self.assertNotIn("secret-value", json.dumps(responses))
```

- [ ] **Step 2: Run safety tests**

Run: `python3 -m pytest tests/test_mcp_config.py tests/test_mcp_stdio_server.py tests/test_mcp_tool_handlers.py -q`

Expected: PASS.

- [ ] **Step 3: Run existing MCP and orchestration tests**

Run: `python3 -m pytest tests/test_mcp_connectors.py tests/test_orchestrator_workflows.py tests/test_literature_pipeline_integration.py -q`

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/test_mcp_stdio_server.py tests/test_mcp_config.py
git commit -m "test(mcp): guard against secret leakage"
```

---

### Task 15: End-to-End Verification

**Files:**
- No source edits unless verification exposes a bug.

- [ ] **Step 1: Run targeted MCP tests**

Run:

```bash
python3 -m pytest \
  tests/test_mcp_config.py \
  tests/test_mcp_tool_handlers.py \
  tests/test_mcp_stdio_server.py \
  tests/test_mcp_cli.py \
  tests/test_mcp_connectors.py \
  -q
```

Expected: PASS.

- [ ] **Step 2: Run package tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
python3 -m pytest tests/test_npm_package_contract.py tests/test_distribution_payloads.py -q
```

Expected: PASS.

- [ ] **Step 3: Run full validation**

Run:

```bash
npm run validate
```

Expected: PASS.

- [ ] **Step 4: Manual MCP stdio smoke**

Run:

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"manual","version":"0"}}}' \
'{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
| python3 -m bridges.mcp_server_stdio
```

Expected:
- First response includes `serverInfo.name = "qiongli"`.
- Second response includes `qiongli_config_status`.
- No logs or warnings appear on stdout outside JSON-RPC responses.

- [ ] **Step 5: Manual desktop config example smoke**

Run:

```bash
python3 -m bridges.mcp_cli config example --target codex
python3 -m bridges.mcp_cli doctor --json
```

Expected:
- Config example includes `[mcp_servers.qiongli]`.
- Doctor output shows provider statuses without secret values.

- [ ] **Step 6: Final commit if verification fixes were needed**

```bash
git add bridges/mcp_config.py bridges/mcp_connectors.py bridges/mcp_tool_handlers.py bridges/mcp_server_stdio.py bridges/mcp_server_http.py bridges/mcp_config_wizard.py bridges/mcp_cli.py qiongli/cli.py packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/lib/python-runtime.mjs tests
git commit -m "fix(mcp): address verification findings"
```

Only run this commit step if Step 1-5 required fixes.

---

## Acceptance Criteria

- Codex Desktop/CLI can add Qiongli as a stdio MCP server and pass provider keys through MCP `env`.
- Claude Desktop/Claude Code can add Qiongli as a stdio MCP server with equivalent env configuration.
- Desktop users can call `qiongli_open_config_wizard` and enter provider keys in a local browser page instead of chat.
- CLI users can run `qiongli mcp configure`, `qiongli mcp doctor`, and `qiongli mcp config example`.
- `qiongli_collect_evidence` reuses existing `MCPConnector` provider behavior.
- Existing `RESEARCH_MCP_*_CMD` command bridge behavior remains backward compatible.
- Secret values are never returned by doctor, tool output, logs, or config status.
- npm package can route `qiongli mcp` through bundled Python runtime.
- PyPI package includes the modules needed to run `qiongli mcp`.
- Documentation covers direct desktop env-key configuration and the safer wizard flow.

## Deferred Work

- OAuth for providers that support OAuth directly.
- OS keychain backend beyond the user-only `~/.qiongli/secrets.toml` fallback.
- Fully spec-complete Streamable HTTP with resumable sessions and server-sent events.
- Plugin manifest-native MCP server bundling if Codex exposes a stable manifest schema for commands plus secret UI.
- Cloud-hosted team deployment with HTTPS, bearer token auth, rate limiting, and audit logs.
