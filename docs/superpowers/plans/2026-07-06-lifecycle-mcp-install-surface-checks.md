# Lifecycle MCP Install Surface Checks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release local install acceptance prove that packaged full-runtime MCP install surfaces expose the adaptive subject lifecycle tools.

**Architecture:** Extend `release_local_install_check.py` after the isolated plugin+MCP install and structural tree validation. The check starts the full Python MCP stdio server from the staged release root, sends `initialize` and `tools/list`, and fails release readiness if `qiongli_subject_status` or `qiongli_subject_update` is missing.

**Tech Stack:** Python stdlib `subprocess`, JSON-RPC over stdio, existing `unittest` release-local-install tests.

---

## Files

- Modify: `tooling/scripts/release_local_install_check.py`
  - Add required lifecycle MCP tool constants.
  - Add a lifecycle tools-list validator.
  - Call the validator from `run_install_check`.
- Modify: `tests/test_release_local_install_check.py`
  - Add failing unit coverage for invocation from `run_install_check`.
  - Add parser-level coverage for missing lifecycle tools.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Stage 6 install-surface lifecycle MCP checks as implemented after verification.

## Task 1: Add Failing Tests

- [x] **Step 1: Add run-install invocation test**

Add this test to `tests/test_release_local_install_check.py`:

```python
    def test_run_install_check_verifies_lifecycle_mcp_tools_after_tree_validation(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            payload = {
                "installed": {
                    "codex": {"installed": True, "surface": "plugin"},
                    "claude": {"installed": True, "surface": "plugin"},
                    "antigravity": {"installed": True, "surface": "plugin"},
                    "hermes": {"installed": True, "surface": "mcp"},
                }
            }

            with mock.patch.object(module, "run_cli", side_effect=["", json.dumps(payload)]):
                with mock.patch.object(module, "validate_install_tree") as validate_tree:
                    with mock.patch.object(module, "validate_lifecycle_mcp_tools", create=True) as validate_mcp:
                        result = module.run_install_check(repo_root, sandbox, python="python")

        self.assertEqual(result, payload)
        validate_tree.assert_called_once()
        validate_mcp.assert_called_once()
        self.assertEqual(validate_mcp.call_args.kwargs["python"], "python")
```

- [x] **Step 2: Add missing-tool validator test**

Add this test to `tests/test_release_local_install_check.py`:

```python
    def test_lifecycle_mcp_tool_name_validator_requires_subject_tools(self) -> None:
        module = load_release_local_install_check()

        module.validate_lifecycle_mcp_tool_names(
            ["qiongli_config_status", "qiongli_subject_status", "qiongli_subject_update"],
        )

        with self.assertRaisesRegex(
            module.LocalInstallCheckError,
            "missing lifecycle MCP tools: qiongli_subject_status, qiongli_subject_update",
        ):
            module.validate_lifecycle_mcp_tool_names(["qiongli_config_status"])
```

- [x] **Step 3: Run tests and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check -q
```

Expected: FAIL because `validate_lifecycle_mcp_tool_names` is missing and
`run_install_check` does not call `validate_lifecycle_mcp_tools`.

## Task 2: Implement Lifecycle MCP Tools-List Check

- [x] **Step 1: Add constants and validator helpers**

In `tooling/scripts/release_local_install_check.py`, add:

```python
REQUIRED_LIFECYCLE_MCP_TOOLS = ("qiongli_subject_status", "qiongli_subject_update")
```

Add:

```python
def validate_lifecycle_mcp_tool_names(tool_names: list[str]) -> None:
    names = set(tool_names)
    missing = [name for name in REQUIRED_LIFECYCLE_MCP_TOOLS if name not in names]
    if missing:
        raise LocalInstallCheckError(
            "missing lifecycle MCP tools: " + ", ".join(missing)
        )
```

- [x] **Step 2: Add stdio tools-list smoke**

Add:

```python
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
    result = subprocess.run(
        [python, "-m", "bridges.mcp_server_stdio"],
        cwd=str(repo_root),
        env=env,
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
    responses = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    tools_response = next(
        (response for response in responses if response.get("id") == 2),
        None,
    )
    if not isinstance(tools_response, dict):
        raise LocalInstallCheckError("lifecycle MCP tools/list returned no response")
    tools = tools_response.get("result", {}).get("tools", [])
    if not isinstance(tools, list):
        raise LocalInstallCheckError("lifecycle MCP tools/list returned invalid tools payload")
    validate_lifecycle_mcp_tool_names(
        [tool.get("name", "") for tool in tools if isinstance(tool, dict)]
    )
```

- [x] **Step 3: Call the helper after structural install validation**

Update `run_install_check`:

```python
    validate_install_tree(repo_root, sandbox, payload)
    validate_lifecycle_mcp_tools(repo_root, sandbox, env, python=python)
    return payload
```

- [x] **Step 4: Run tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check -q
```

Expected: PASS.

## Task 3: Verify Actual MCP Smoke And Roadmap

- [x] **Step 1: Run focused MCP/release tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check tests.test_mcp_stdio_server tests.test_mcp_tool_handlers -q
```

Expected: PASS.

- [x] **Step 2: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 3: Update roadmap Stage 6**

Change Stage 6 status in
`docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md` so
it states that isolated local install acceptance now starts the full MCP stdio
server and checks lifecycle tools in `tools/list`.

- [x] **Step 4: Commit by content**

Implementation:

```bash
git add tooling/scripts/release_local_install_check.py tests/test_release_local_install_check.py
git commit -m "test(release): verify lifecycle MCP tools in local installs"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-lifecycle-mcp-install-surface-checks.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record lifecycle MCP install checks"
```

## Self-Review

- Spec coverage: Covers Stage 6 package/install-surface checks for lifecycle
  MCP inclusion by using the staged full runtime, isolated install env, and MCP
  `tools/list`.
- Placeholder scan: No placeholders remain.
- Type consistency: Tests and implementation use `validate_lifecycle_mcp_tools`,
  `validate_lifecycle_mcp_tool_names`, `LocalInstallCheckError`, and
  `InstallSandbox` consistently.
