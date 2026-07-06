# Codex Plugin MCP UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Codex Qiongli installs report plugin-bundled MCP accurately, avoid false `strategy_only` literature-review downgrades, and keep standalone Codex MCP config as an explicit recovery path.

**Architecture:** Keep Codex plugin installs self-contained: `.codex-plugin/plugin.json` points at plugin-local `.mcp.json`, while `~/.codex/config.toml` remains a user-requested standalone fallback. Extend install discovery to distinguish effective MCP, plugin-bundled MCP, and standalone MCP. Update Qiongli workflow guidance so Codex checks `qiongli_literature_status` before declaring that provider-connected literature MCP is unavailable.

**Tech Stack:** Python 3.12, unittest, Codex local plugin manifests, Qiongli universal installer, Markdown skill contracts.

---

### Task 1: Report Codex Plugin-Bundled MCP In `qiongli check`

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/install_discovery.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Write the failing Codex plugin MCP JSON test**

Extend `tests/test_cli.py::CLITests.test_check_json_reports_codex_plugin_surface` so the fixture writes a plugin-local `.mcp.json` next to the Codex plugin manifest:

```python
(plugin_root / ".mcp.json").write_text(
    json.dumps(
        {
            "mcpServers": {
                "qiongli": {
                    "command": "qiongli",
                    "args": ["mcp", "serve", "--transport", "stdio"],
                }
            }
        }
    ),
    encoding="utf-8",
)
```

Add these assertions after the existing plugin assertions:

```python
self.assertTrue(codex["mcp"]["installed"])
self.assertEqual(codex["mcp"]["source"], "plugin")
self.assertEqual(codex["mcp"]["path"], str(plugin_root / ".mcp.json"))
self.assertEqual(codex["mcp"]["server"], "qiongli")
self.assertFalse(codex["standalone_mcp"]["installed"])
self.assertEqual(codex["standalone_mcp"]["source"], "standalone")
self.assertEqual(codex["standalone_mcp"]["path"], str(root / "codex-home" / "config.toml"))
self.assertTrue(codex["plugin_mcp"]["installed"])
self.assertEqual(codex["plugin_mcp"]["source"], "plugin")
```

Update the mocked environment in this test to set `CODEX_HOME` so the standalone path is deterministic:

```python
_isolated_qiongli_env(
    root,
    CODEX_HOME=str(root / "codex-home"),
    QIONGLI_CODEX_MARKETPLACE_PATH=str(marketplace),
    QIONGLI_CLAUDE_PLUGIN_PARENT=str(root / "claude-plugins"),
    CLAUDE_CODE_CONFIG_PATH=str(root / "claude.json"),
    ANTIGRAVITY_CONFIG_PATH=str(root / "antigravity-settings.json"),
    HERMES_CONFIG_PATH=str(root / "hermes-settings.json"),
)
```

- [ ] **Step 2: Run the targeted test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_cli.CLITests.test_check_json_reports_codex_plugin_surface -v
```

Expected: FAIL because `codex["mcp"]["installed"]` is still `False` and `plugin_mcp` / `standalone_mcp` are not present.

- [ ] **Step 3: Add plugin and standalone MCP status helpers**

In `packages/python-qiongli/src/qiongli/install_discovery.py`, add these helpers near `_codex_mcp_status`:

```python
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

    mcp_path = (plugin_root / mcp_ref).resolve(strict=False)
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
```

This helper intentionally accepts plugin-bundled Node MCP servers and CLI-backed MCP servers. Plugin MCP validity is determined by the plugin manifest reference plus a Qiongli server entry, not by requiring `command == "qiongli"`.

- [ ] **Step 4: Rename the existing Codex config check to standalone**

Replace `_codex_mcp_status` with `_codex_standalone_mcp_status`:

```python
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
```

- [ ] **Step 5: Combine effective Codex MCP status without changing install ownership**

In `discover_install_surfaces`, compute Codex plugin and MCP state before building the Codex entry:

```python
codex_plugin = _codex_plugin_status(check_activation=check_activation)
codex_plugin_mcp = _codex_plugin_mcp_status(codex_plugin)
codex_standalone_mcp = _codex_standalone_mcp_status()
codex_effective_mcp = codex_plugin_mcp if codex_plugin_mcp["installed"] else codex_standalone_mcp
```

Then pass both extra diagnostics into `_combine_surface`:

```python
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
```

Change `_combine_surface` to accept and merge extra diagnostics:

```python
def _combine_surface(
    *,
    client: str,
    plugin: dict[str, object],
    skill: dict[str, object],
    mcp: dict[str, object],
    extra: dict[str, object] | None = None,
) -> dict[str, object]:
    ...
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
```

Keep Claude, Antigravity, and Hermes calls using the default `extra=None`.

- [ ] **Step 6: Run the targeted test and verify it passes**

Run:

```bash
python3 -m unittest tests.test_cli.CLITests.test_check_json_reports_codex_plugin_surface -v
```

Expected: PASS.

### Task 2: Preserve Standalone MCP As Explicit Codex Fallback

**Files:**
- Modify: `tests/test_cli.py`
- Modify: `packages/python-qiongli/src/qiongli/install_discovery.py`
- Confirm unchanged: `tests/test_universal_installer.py`

- [ ] **Step 1: Write a standalone Codex MCP check test**

Add this test to `tests/test_cli.py`:

```python
def test_check_json_reports_codex_standalone_mcp_when_no_plugin_mcp(self) -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        codex_home = root / "codex-home"
        config_path = codex_home / "config.toml"
        config_path.parent.mkdir(parents=True)
        config_path.write_text(
            "\n".join(
                [
                    "# >>> qiongli managed mcp >>>",
                    "[mcp_servers.qiongli]",
                    'command = "qiongli"',
                    'args = ["mcp", "serve", "--transport", "stdio"]',
                    "# <<< qiongli managed mcp <<<",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False, offline=True)
        stdout = io.StringIO()

        with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
            cli_module, "_check_system_env", return_value={}
        ), mock.patch.object(
            cli_module.os,
            "environ",
            _isolated_qiongli_env(root, CODEX_HOME=str(codex_home)),
        ), contextlib.redirect_stdout(stdout):
            exit_code = cli_module.cmd_check(args)

    self.assertEqual(exit_code, 0)
    payload = json.loads(stdout.getvalue())
    codex = payload["installed"]["codex"]
    self.assertEqual(codex["surface"], "mcp")
    self.assertTrue(codex["mcp"]["installed"])
    self.assertEqual(codex["mcp"]["source"], "standalone")
    self.assertFalse(codex["plugin_mcp"]["installed"])
    self.assertTrue(codex["standalone_mcp"]["installed"])
```

- [ ] **Step 2: Run the standalone test and verify it fails before implementation**

Run:

```bash
python3 -m unittest tests.test_cli.CLITests.test_check_json_reports_codex_standalone_mcp_when_no_plugin_mcp -v
```

Expected: FAIL until `_codex_standalone_mcp_status` and `plugin_mcp` / `standalone_mcp` diagnostics exist.

- [ ] **Step 3: Run installer ownership tests without changing installer behavior**

Run:

```bash
python3 -m unittest \
  tests.test_universal_installer.UniversalInstallerTests.test_plugin_surface_installs_codex_plugin_without_global_skill_or_codex_mcp_config \
  tests.test_universal_installer.UniversalInstallerTests.test_mcp_part_only_registers_codex_mcp_without_global_skill \
  -v
```

Expected: PASS. The plugin surface still does not create `CODEX_HOME/config.toml`, while `--parts mcp` still creates standalone config.

### Task 3: Make CLI Text Output Explain Effective MCP Source

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Write a text-summary test for Codex plugin MCP**

Add this focused test to `tests/test_cli.py`:

```python
def test_doctor_summary_reports_codex_plugin_mcp_source(self) -> None:
    args = argparse.Namespace(cwd=".")
    completed = mock.Mock(returncode=0, stdout="doctor ok\n")
    installed = {
        client: {
            "installed": False,
            "surface": "none",
            "version": None,
            "path": f"/tmp/{client}/qiongli-workflow",
            "mcp": {"installed": False, "source": "standalone", "path": "", "server": ""},
        }
        for client in ("codex", "claude", "antigravity", "hermes")
    }
    installed["codex"] = {
        "installed": True,
        "surface": "plugin",
        "version": "v9.9.9",
        "path": "/tmp/plugins/qiongli",
        "mcp": {
            "installed": True,
            "source": "plugin",
            "path": "/tmp/plugins/qiongli/.mcp.json",
            "server": "qiongli",
        },
    }
    stdout = io.StringIO()

    with mock.patch.object(cli_module.subprocess, "run", return_value=completed), mock.patch.object(
        cli_module, "discover_install_surfaces", return_value=installed
    ), contextlib.redirect_stdout(stdout):
        exit_code = cli_module.cmd_doctor(args)

    self.assertEqual(exit_code, 0)
    output = stdout.getvalue()
    self.assertIn("- codex: installed, surface=plugin", output)
    self.assertIn("mcp=plugin:qiongli", output)
```

- [ ] **Step 2: Run the text-summary test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_cli.CLITests.test_doctor_summary_reports_codex_plugin_mcp_source -v
```

Expected: FAIL because `_print_client_integration_summary` does not include MCP source.

- [ ] **Step 3: Add compact MCP summary formatting**

In `packages/python-qiongli/src/qiongli/cli.py`, add:

```python
def _client_mcp_summary(item: dict[str, object]) -> str:
    mcp = item.get("mcp")
    if not isinstance(mcp, dict) or not mcp.get("installed"):
        return "mcp=none"
    source = str(mcp.get("source") or "standalone")
    server = str(mcp.get("server") or "qiongli")
    return f"mcp={source}:{server}"
```

Update `_print_client_integration_summary`:

```python
print(
    f"- {client}: {status}, surface={item['surface']}, "
    f"version={version}, {_client_mcp_summary(item)}, path={item['path']}"
)
```

- [ ] **Step 4: Run CLI tests**

Run:

```bash
python3 -m unittest tests.test_cli -v
```

Expected: PASS.

### Task 4: Require Literature MCP Capability Preflight Before `strategy_only`

**Files:**
- Modify: `content/workflow/SKILL.md`
- Modify: `tests/test_mcp_provider_docs.py`
- Modify if generated sync requires it: `packages/python-qiongli/src/qiongli/subject_materializer.py`

- [ ] **Step 1: Add a contract test for Codex provider preflight guidance**

Add this test to `tests/test_mcp_provider_docs.py`:

```python
def test_qiongli_workflow_requires_codex_literature_status_preflight(self) -> None:
    content = (REPO_ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")
    self.assertIn("qiongli_literature_status", content)
    self.assertIn("Codex", content)
    self.assertIn("before declaring `strategy_only`", content)
    self.assertIn("provider_connected", content)
```

- [ ] **Step 2: Run the contract test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_mcp_provider_docs.MCPProviderDocsTests.test_qiongli_workflow_requires_codex_literature_status_preflight -v
```

Expected: FAIL because `content/workflow/SKILL.md` does not yet require Codex to call `qiongli_literature_status` before downgrading.

- [ ] **Step 3: Update the Qiongli workflow contract**

In `content/workflow/SKILL.md`, under `## Literature Provider Configuration`, add this bullet immediately after the provider configuration bullets:

```markdown
- In Codex plugin sessions, before declaring `strategy_only` for literature search, literature review, paper screening, citation snowballing, or evidence synthesis, attempt the visible `qiongli_literature_status` MCP tool. If it returns `capability_mode: provider_connected`, proceed with provider-backed literature workflow. If the tool is not visible in the session, state that Qiongli MCP tools are not visible and recommend restarting Codex or installing the explicit standalone MCP fallback with `qiongli install --target codex --parts mcp`. Only use `strategy_only` after this preflight is unavailable or returns a non-provider-connected mode.
```

- [ ] **Step 4: Run the contract test and generated skill tests**

Run:

```bash
python3 -m unittest tests.test_mcp_provider_docs tests.test_subject_materializer -v
```

Expected: PASS. If a subject materializer assertion checks exact skill text, update that assertion to include the new Codex preflight sentence.

### Task 5: Document The Plugin-First MCP Model

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `docs/guide/troubleshooting.md`
- Modify: `docs/zh/guide/troubleshooting.md`
- Modify: `tests/test_cli_setup_docs.py`
- Modify: `tests/test_mcp_provider_docs.py`

- [ ] **Step 1: Add English docs assertions**

Add assertions to `tests/test_cli_setup_docs.py`:

```python
def test_codex_plugin_install_documents_plugin_bundled_mcp(self) -> None:
    install = (REPO_ROOT / "docs" / "guide" / "install.md").read_text(encoding="utf-8")
    cli = (REPO_ROOT / "docs" / "reference" / "cli.md").read_text(encoding="utf-8")
    troubleshooting = (REPO_ROOT / "docs" / "guide" / "troubleshooting.md").read_text(encoding="utf-8")

    self.assertIn("plugin-bundled MCP", install)
    self.assertIn("does not write `~/.codex/config.toml`", install)
    self.assertIn("standalone MCP fallback", cli)
    self.assertIn("plugin_mcp", cli)
    self.assertIn("standalone_mcp", cli)
    self.assertIn("qiongli_literature_status", troubleshooting)
```

- [ ] **Step 2: Add Chinese docs assertions**

Add assertions to `tests/test_cli_setup_docs.py`:

```python
def test_zh_codex_plugin_install_documents_plugin_bundled_mcp(self) -> None:
    install = (REPO_ROOT / "docs" / "zh" / "guide" / "install.md").read_text(encoding="utf-8")
    cli = (REPO_ROOT / "docs" / "zh" / "reference" / "cli.md").read_text(encoding="utf-8")
    troubleshooting = (REPO_ROOT / "docs" / "zh" / "guide" / "troubleshooting.md").read_text(encoding="utf-8")

    self.assertIn("插件内置 MCP", install)
    self.assertIn("不会写入 `~/.codex/config.toml`", install)
    self.assertIn("standalone MCP fallback", cli)
    self.assertIn("plugin_mcp", cli)
    self.assertIn("standalone_mcp", cli)
    self.assertIn("qiongli_literature_status", troubleshooting)
```

- [ ] **Step 3: Run docs tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs -v
```

Expected: FAIL until the docs include the plugin-first MCP explanation and preflight guidance.

- [ ] **Step 4: Update English docs**

In `docs/guide/install.md`, add a Codex note near plugin install instructions:

```markdown
Codex plugin installs use plugin-bundled MCP by default. The installer writes `.mcp.json` inside the Qiongli plugin and the plugin manifest points to it; it does not write `~/.codex/config.toml` on the plugin path. Use `qiongli install --target codex --parts mcp` only when you need the standalone MCP fallback.
```

In `docs/reference/cli.md`, update the `qiongli check` section:

```markdown
For Codex plugin installs, `mcp` reports the effective MCP source. `plugin_mcp` reports the plugin-local `.mcp.json`, and `standalone_mcp` reports `~/.codex/config.toml`. A false `standalone_mcp.installed` value is expected when `plugin_mcp.installed` is true.
```

In `docs/guide/troubleshooting.md`, add a recovery entry:

```markdown
If a Codex literature task says Qiongli is in `strategy_only` even though the plugin is installed, run `qiongli check --json` and confirm `installed.codex.plugin_mcp.installed` is true. In a Codex session, ask Qiongli to call `qiongli_literature_status`; if the tool is not visible, restart Codex. If plugin MCP remains invisible, install the explicit standalone fallback with `qiongli install --target codex --parts mcp`.
```

- [ ] **Step 5: Update Chinese docs**

In `docs/zh/guide/install.md`, add:

```markdown
Codex 插件安装默认使用插件内置 MCP。安装器会把 `.mcp.json` 写在 Qiongli 插件目录内，并由插件 manifest 指向它；插件路径不会写入 `~/.codex/config.toml`。只有在需要 standalone MCP fallback 时，才运行 `qiongli install --target codex --parts mcp`。
```

In `docs/zh/reference/cli.md`, add:

```markdown
对于 Codex 插件安装，`mcp` 表示当前有效 MCP 来源。`plugin_mcp` 表示插件本地 `.mcp.json`，`standalone_mcp` 表示 `~/.codex/config.toml`。当 `plugin_mcp.installed` 为 true 时，`standalone_mcp.installed` 为 false 是预期状态。
```

In `docs/zh/guide/troubleshooting.md`, add:

```markdown
如果 Codex 文献任务在插件已安装时仍提示 Qiongli 处于 `strategy_only`，先运行 `qiongli check --json`，确认 `installed.codex.plugin_mcp.installed` 为 true。在 Codex 会话里要求 Qiongli 调用 `qiongli_literature_status`；如果该工具不可见，重启 Codex。如果插件 MCP 仍不可见，再使用显式 standalone fallback：`qiongli install --target codex --parts mcp`。
```

- [ ] **Step 6: Run docs tests and verify they pass**

Run:

```bash
python3 -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs -v
```

Expected: PASS.

### Task 6: Run Boundary And Release Verification

**Files:**
- Read: all modified files in this plan

- [ ] **Step 1: Run focused Python verification**

Run:

```bash
python3 -m unittest \
  tests.test_cli \
  tests.test_universal_installer \
  tests.test_mcp_client_config \
  tests.test_local_plugin_installer \
  tests.test_plugin_artifacts \
  tests.test_plugin_distribution_contract \
  tests.test_mcp_provider_docs \
  tests.test_cli_setup_docs \
  tests.test_subject_materializer \
  -v
```

Expected: PASS.

- [ ] **Step 2: Run syntax and whitespace checks**

Run:

```bash
python3 -m py_compile \
  packages/python-qiongli/src/qiongli/install_discovery.py \
  packages/python-qiongli/src/qiongli/cli.py
git diff --check
```

Expected: both commands exit 0.

- [ ] **Step 3: Review repository boundaries**

Inspect changed files:

```bash
git status --short --untracked-files=all
```

Expected findings:
- No provider secrets in `.mcp.json`, docs, tests, or generated artifacts.
- No machine-specific absolute user paths in committed docs or test fixtures.
- No marketplace catalog files copied into source docs.
- No generated plugin payloads committed unless they are already source-owned fixtures.
- `~/.codex/config.toml` remains an explicit standalone MCP path and is not written by plugin-surface tests.

- [ ] **Step 4: Commit in two logical commits**

First commit the existing wrapper-skill work if it is still uncommitted:

```bash
git add \
  packages/python-qiongli/src/qiongli/workflow_wrapper_skills.py \
  packages/python-qiongli/src/qiongli/local_plugin_installer.py \
  tooling/scripts/build_plugin_artifacts.py \
  tests/test_local_plugin_installer.py \
  tests/test_plugin_artifacts.py \
  docs/guide/install.md \
  docs/guide/troubleshooting.md \
  docs/guide/using-agent-skills.md \
  docs/reference/cli.md \
  docs/zh/guide/install.md \
  docs/zh/guide/troubleshooting.md \
  docs/zh/guide/using-agent-skills.md \
  docs/zh/reference/cli.md
git commit -m "feat(codex): add qiongli workflow wrapper skills"
```

Then commit the MCP UX fix from this plan:

```bash
git add \
  packages/python-qiongli/src/qiongli/install_discovery.py \
  packages/python-qiongli/src/qiongli/cli.py \
  content/workflow/SKILL.md \
  docs/guide/install.md \
  docs/guide/troubleshooting.md \
  docs/reference/cli.md \
  docs/zh/guide/install.md \
  docs/zh/guide/troubleshooting.md \
  docs/zh/reference/cli.md \
  tests/test_cli.py \
  tests/test_cli_setup_docs.py \
  tests/test_mcp_provider_docs.py \
  tests/test_subject_materializer.py \
  docs/superpowers/plans/2026-06-28-codex-plugin-mcp-ux.md
git commit -m "fix(codex): report plugin-bundled qiongli mcp"
```

If `packages/python-qiongli/src/qiongli/subject_materializer.py` already contains unrelated local edits, inspect its diff before staging and stage only hunks that belong to the workflow preflight text propagation.

## Self-Review

**Spec coverage:** The plan covers the requested design: Codex does not write standalone MCP by default, `qiongli check` reports plugin-bundled MCP clearly, standalone MCP remains an explicit fallback, and literature workflows call capability status before downgrading.

**Placeholder scan:** The plan contains concrete file paths, commands, expected outcomes, and code snippets for every implementation task.

**Type consistency:** The new JSON keys are consistently named `plugin_mcp` and `standalone_mcp`; the existing `mcp` key remains the effective MCP source for backward compatibility.
