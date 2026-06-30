# Claude Code MCP Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Claude Code marketplace installs match Codex daily-use capability by bundling the zero-dependency Qiongli literature MCP while preserving the current source, payload, and runtime boundaries.

**Architecture:** Keep `packages/qiongli-plugin/` as the only native plugin source and keep generated release payloads out of the development checkout. Claude Code gets the same bundled Node literature-provider runtime as Codex, while the Python-backed full CLI MCP remains a separate advanced runtime installed through npm/pipx/bootstrap `full`.

**Tech Stack:** Python 3.12+, `unittest`, JSON plugin manifests, tar/zip release artifacts, Node stdio MCP server under `packages/qiongli-plugin/mcp/qiongli-literature-provider/`.

---

## Scope Guardrails

- Do not move canonical content from `content/`.
- Do not edit generated payload directories such as `plugins/qiongli/` or `qiongli-workflow/` in the source checkout.
- Do not add a second Claude-specific MCP runtime directory.
- Do not bundle Python, model CLIs, or `qiongli mcp serve` into the native plugin.
- Keep Desktop Skill ZIP and MCPB behavior unchanged.
- Build staged artifacts only in temp directories or explicit `dist` directories used by tests.

## File Structure

- `packages/qiongli-plugin/.claude-plugin/plugin.json`: declare Claude Code's bundled literature MCP server inline.
- `packages/qiongli-plugin/.codex-plugin/plugin.json`: no functional change expected; keep Codex `mcpServers` reference stable.
- `packages/qiongli-plugin/.mcp.json`: no functional change expected; remains Codex plugin MCP config.
- `packages/qiongli-plugin/mcp/qiongli-literature-provider/`: reused by Codex and Claude Code.
- `tooling/scripts/build_plugin_artifacts.py`: copy the bundled MCP runtime into Claude marketplace artifacts in the same path as Codex artifacts.
- `tooling/scripts/validate_marketplace_install.py`: verify Codex and Claude marketplace artifacts expose bundled literature MCP.
- `tests/test_plugin_manifests.py`: source manifest contract tests.
- `tests/test_plugin_distribution_contract.py`: staged plugin materialization contract tests.
- `tests/test_plugin_artifacts.py`: release tarball content tests.
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`: full CLI MCP preview and triad fixes.
- `tests/test_mcp_tool_handlers.py`: full CLI MCP behavior tests.
- `tests/test_mcp_stdio_server.py`: stdio smoke coverage for full CLI MCP preview.
- `README.md`, `docs/guide/install.md`, `docs/advanced/plugin-first-architecture.md`, `docs/advanced/cross-platform-mcp.md`: user-facing support matrix and boundary updates.

## Task 0: Preserve Current README Decision Guide

**Files:**
- Modify: `README.md`
- Verify: `tests/test_cli_setup_docs.py`
- Verify: `tests/test_mcp_provider_docs.py`
- Verify: `tests/test_distribution_materialization_docs.py`

- [ ] **Step 1: Review existing README diff**

Run:

```bash
git diff -- README.md
```

Expected: the diff contains `## Installation Decision Guide` and a table separating Codex, Claude Code, Desktop, npm/npx, bootstrap, and Python CLI paths.

- [ ] **Step 2: Run documentation checks**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs tests.test_distribution_materialization_docs -v
```

Expected: all tests pass.

- [ ] **Step 3: Commit the documentation baseline**

Run:

```bash
git add README.md
git commit -m "docs: clarify qiongli installation surfaces"
```

Expected: commit succeeds and `git status --short` shows no README change.

## Task 1: Add Failing Tests For Claude Code Bundled Literature MCP

**Files:**
- Modify: `tests/test_plugin_manifests.py`
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tests/test_plugin_artifacts.py`

- [ ] **Step 1: Add Claude source manifest MCP test**

In `tests/test_plugin_manifests.py`, add this method inside `PluginManifestTests` after `test_codex_plugin_bundles_qiongli_mcp_server`:

```python
    def test_claude_plugin_bundles_qiongli_literature_mcp_server(self) -> None:
        manifest = json.loads(CLAUDE_PLUGIN_MANIFEST.read_text(encoding="utf-8"))

        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
        self.assertTrue((PLUGIN_ROOT / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file())

        rendered = json.dumps(manifest, sort_keys=True)
        self.assertNotIn("QIONGLI_OPENALEX_EMAIL", rendered)
        self.assertNotIn("SEMANTIC_SCHOLAR_API_KEY", rendered)
        self.assertNotIn("qiongli mcp", rendered)
```

- [ ] **Step 2: Add staged plugin materialization MCP test**

In `tests/test_plugin_distribution_contract.py`, add this method inside `PluginDistributionContractTests` after `test_codex_plugin_materializes_bundled_mcp_manifest`:

```python
    def test_claude_plugin_materializes_bundled_mcp_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8")
            )

        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
        self.assertTrue(
            (materialized_plugin / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file()
        )
```

- [ ] **Step 3: Add Claude release artifact MCP expectations**

In `tests/test_plugin_artifacts.py`, extend the expected files for `qiongli-claude-plugin`, `qiongli-economics-claude-plugin`, and one additional subject artifact such as `qiongli-finance-claude-plugin`.

For `qiongli-claude-plugin`, add:

```python
                    f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/mcp/qiongli-literature-provider/index.mjs",
```

For `qiongli-economics-claude-plugin`, add:

```python
                    f"qiongli-economics-claude-plugin-{current_tag}/plugins/qiongli-economics/mcp/qiongli-literature-provider/index.mjs",
```

For `qiongli-finance-claude-plugin`, add:

```python
                    f"qiongli-finance-claude-plugin-{current_tag}/plugins/qiongli-finance/mcp/qiongli-literature-provider/index.mjs",
```

After reading `claude_manifest`, add:

```python
            self.assertEqual(
                claude_manifest["mcpServers"]["qiongli"]["args"],
                ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
            )
```

- [ ] **Step 4: Run tests and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_plugin_manifests tests.test_plugin_distribution_contract tests.test_plugin_artifacts -v
```

Expected: failures mention missing `mcpServers` in `.claude-plugin/plugin.json` and missing Claude artifact `mcp/qiongli-literature-provider/index.mjs`.

## Task 2: Implement Claude Code Bundled Literature MCP

**Files:**
- Modify: `packages/qiongli-plugin/.claude-plugin/plugin.json`
- Modify: `tooling/scripts/build_plugin_artifacts.py`

- [ ] **Step 1: Add inline Claude Code MCP declaration**

In `packages/qiongli-plugin/.claude-plugin/plugin.json`, replace the closing keywords block:

```json
  "keywords": [
    "research",
    "academic-writing",
    "literature-review",
    "claude-code-plugins"
  ]
}
```

with:

```json
  "keywords": [
    "research",
    "academic-writing",
    "literature-review",
    "claude-code-plugins"
  ],
  "mcpServers": {
    "qiongli": {
      "command": "node",
      "args": [
        "${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"
      ],
      "cwd": "${CLAUDE_PLUGIN_ROOT}"
    }
  }
}
```

- [ ] **Step 2: Rename the runtime copy helper**

In `tooling/scripts/build_plugin_artifacts.py`, replace:

```python
def _copy_codex_mcp_runtime(root: Path, dest_plugin_root: Path) -> None:
    mcp_runtime = RepoLayout(root).plugin_package / "mcp"
    if mcp_runtime.is_dir():
        _copy_path(mcp_runtime, dest_plugin_root / "mcp")
```

with:

```python
def _copy_literature_mcp_runtime(root: Path, dest_plugin_root: Path) -> None:
    mcp_runtime = RepoLayout(root).plugin_package / "mcp"
    if mcp_runtime.is_dir():
        _copy_path(mcp_runtime, dest_plugin_root / "mcp")
```

- [ ] **Step 3: Copy MCP runtime for Codex and Claude artifacts**

In `_build_marketplace_plugin`, replace:

```python
    if platform == "codex":
        _copy_codex_mcp_manifest(root, plugin_dest)
        _copy_codex_mcp_runtime(root, plugin_dest)
```

with:

```python
    if platform == "codex":
        _copy_codex_mcp_manifest(root, plugin_dest)
        _copy_literature_mcp_runtime(root, plugin_dest)
    elif platform == "claude":
        _copy_literature_mcp_runtime(root, plugin_dest)
```

- [ ] **Step 4: Run targeted tests**

Run:

```bash
uv run python -m unittest tests.test_plugin_manifests tests.test_plugin_distribution_contract tests.test_plugin_artifacts -v
```

Expected: all three test modules pass.

- [ ] **Step 5: Commit Claude Code MCP parity**

Run:

```bash
git add packages/qiongli-plugin/.claude-plugin/plugin.json tooling/scripts/build_plugin_artifacts.py tests/test_plugin_manifests.py tests/test_plugin_distribution_contract.py tests/test_plugin_artifacts.py
git commit -m "feat(plugin): bundle literature mcp for claude code"
```

Expected: commit succeeds.

## Task 3: Extend Marketplace Artifact Validator

**Files:**
- Modify: `tooling/scripts/validate_marketplace_install.py`
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Add artifact spec field**

In `tooling/scripts/validate_marketplace_install.py`, replace:

```python
@dataclass(frozen=True)
class ArtifactSpec:
    platform: str
    manifest: Path
    plugin_root: Path
    requires_commands: bool
```

with:

```python
@dataclass(frozen=True)
class ArtifactSpec:
    platform: str
    manifest: Path
    plugin_root: Path
    requires_commands: bool
    expects_bundled_mcp: bool = False
```

- [ ] **Step 2: Mark Codex and Claude as bundled-MCP artifacts**

In `ARTIFACT_SPECS`, add `expects_bundled_mcp=True` to `codex` and `claude`:

```python
    "codex": ArtifactSpec(
        platform="codex",
        manifest=Path("plugins") / PLUGIN_NAME / ".codex-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
        expects_bundled_mcp=True,
    ),
    "claude": ArtifactSpec(
        platform="claude",
        manifest=Path("plugins") / PLUGIN_NAME / ".claude-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
        expects_bundled_mcp=True,
    ),
```

- [ ] **Step 3: Add bundled MCP validator helper**

In `tooling/scripts/validate_marketplace_install.py`, add this helper after `_assert_manifest`:

```python
def _assert_bundled_literature_mcp(plugin_root: Path, platform: str) -> None:
    runtime_entry = plugin_root / "mcp" / "qiongli-literature-provider" / "index.mjs"
    _assert_file(runtime_entry, f"{platform} bundled literature MCP runtime")

    if platform == "codex":
        mcp_manifest = _read_json(plugin_root / ".mcp.json")
        server = mcp_manifest["mcpServers"]["qiongli"]
        expected_args = ["./mcp/qiongli-literature-provider/index.mjs"]
    elif platform == "claude":
        manifest = _read_json(plugin_root / ".claude-plugin" / "plugin.json")
        server = manifest["mcpServers"]["qiongli"]
        expected_args = ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"]
    else:
        raise ValueError(f"{platform} should not declare bundled Qiongli literature MCP")

    if server["command"] != "node":
        raise ValueError(f"{platform} bundled MCP must run with node")
    if server["args"] != expected_args:
        raise ValueError(f"{platform} bundled MCP args mismatch: {server['args']}")

    rendered = json.dumps(server, sort_keys=True)
    for forbidden in ("QIONGLI_OPENALEX_EMAIL", "SEMANTIC_SCHOLAR_API_KEY", "qiongli mcp"):
        if forbidden in rendered:
            raise ValueError(f"{platform} bundled MCP config must not contain {forbidden}")
```

- [ ] **Step 4: Call bundled MCP validator during artifact validation**

In `_validate_artifact`, after command validation:

```python
        if spec.requires_commands:
            _assert_command_invocation(plugin_root, workflow_names)
```

add:

```python
        if spec.expects_bundled_mcp:
            _assert_bundled_literature_mcp(plugin_root, spec.platform)
```

- [ ] **Step 5: Make validation output prove MCP coverage**

In `_validate_artifact`, replace:

```python
    return f"[OK] {spec.platform} marketplace artifact{subject_suffix}: {SKILL_NAME} invocation checked"
```

with:

```python
    mcp_suffix = "; bundled literature MCP checked" if spec.expects_bundled_mcp else ""
    return f"[OK] {spec.platform} marketplace artifact{subject_suffix}: {SKILL_NAME} invocation checked{mcp_suffix}"
```

- [ ] **Step 6: Extend distribution contract assertion**

In `tests/test_plugin_distribution_contract.py`, inside `test_marketplace_validator_builds_platform_artifacts_and_checks_invocation`, add:

```python
        self.assertIn("bundled literature MCP checked", result.stdout)
```

- [ ] **Step 7: Run marketplace validation tests**

Run:

```bash
uv run python -m unittest tests.test_plugin_distribution_contract -v
```

Expected: all tests pass and validator output includes `bundled literature MCP checked`.

- [ ] **Step 8: Commit validator coverage**

Run:

```bash
git add tooling/scripts/validate_marketplace_install.py tests/test_plugin_distribution_contract.py
git commit -m "test(plugin): validate bundled literature mcp artifacts"
```

Expected: commit succeeds.

## Task 4: Update Documentation To Match New Support Matrix

**Files:**
- Modify: `README.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/advanced/plugin-first-architecture.md`
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `packages/npm-qiongli/README.md`

- [ ] **Step 1: Update README Claude Code row**

In `README.md`, replace the Claude Code trade-off sentence:

```markdown
Unlike the Codex plugin, the Claude Code marketplace artifact does not bundle MCP registration/runtime. Use CLI MCP when Claude Code needs provider tools or `qiongli_task_run` as MCP.
```

with:

```markdown
Bundles the same zero-dependency literature MCP runtime as Codex for provider search/status tools. Use full CLI MCP only when Claude Code needs Python-backed tools such as `qiongli_task_run`.
```

- [ ] **Step 2: Update README marketplace status paragraph**

Replace:

```markdown
Claude Code marketplace status: yes, Claude Code can install the full Qiongli methodology through Skillsplace for core and subject `complete` packages. The gap versus Codex is MCP bundling: Codex currently installs both skills and the bundled literature MCP registration/runtime, while Claude Code marketplace installs the full skill/command package but still needs the separate CLI MCP setup for MCP tools.
```

with:

```markdown
Claude Code marketplace status: yes, Claude Code can install the full Qiongli methodology through Skillsplace for core and subject `complete` packages. After the bundled MCP parity update, Codex and Claude Code both install the skill/command package plus the Node literature MCP runtime. Full Python-backed orchestration MCP remains a separate CLI runtime.
```

- [ ] **Step 3: Update install guide native plugin section**

In `docs/guide/install.md`, after the Claude Code marketplace commands, add:

```markdown
The Claude Code plugin also bundles the zero-dependency Node literature-provider MCP runtime. It exposes literature/provider tools from the plugin package without requiring users to hand-write MCP configuration. The full Python-backed `qiongli mcp serve` server still requires npm, pipx/pip, or bootstrap `full`.
```

- [ ] **Step 4: Update plugin architecture platform table**

In `docs/advanced/plugin-first-architecture.md`, replace the Claude Code runtime entry:

```markdown
| Claude Code | `packages/qiongli-plugin/.claude-plugin/plugin.json`; public catalog entry in `jxpeng98/skillsplace` | `commands/*.md` plus `skills/qiongli-workflow/` |
```

with:

```markdown
| Claude Code | `packages/qiongli-plugin/.claude-plugin/plugin.json`; public catalog entry in `jxpeng98/skillsplace` | `commands/*.md` plus `skills/qiongli-workflow/`; bundled Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/` |
```

- [ ] **Step 5: Update cross-platform MCP doc**

In `docs/advanced/cross-platform-mcp.md`, add a short Claude Code subsection after the Codex bundled plugin MCP subsection:

```markdown
## Claude Code Bundled Plugin MCP

The Claude Code plugin package declares a bundled `qiongli` MCP server in `.claude-plugin/plugin.json` and uses the same zero-dependency Node literature-provider runtime under `mcp/qiongli-literature-provider/`. This gives Claude Code marketplace installs local literature/provider tools without requiring a separate MCP config for the bundled literature runtime.

The bundled Claude Code runtime is still literature-provider only. Use the full CLI stdio server when Claude Code needs Python-backed orchestration tools such as `qiongli_task_plan` or `qiongli_task_run`.
```

- [ ] **Step 6: Update npm README boundary wording**

In `packages/npm-qiongli/README.md`, ensure the MCP section contains this sentence:

```markdown
Native Codex and Claude Code plugins can bundle the Node literature-provider MCP runtime; npm/pipx/bootstrap `full` remains the path for the Python-backed full CLI MCP server.
```

- [ ] **Step 7: Run documentation tests**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs tests.test_distribution_materialization_docs tests.test_package_readmes -v
```

Expected: all tests pass.

- [ ] **Step 8: Commit documentation updates**

Run:

```bash
git add README.md docs/guide/install.md docs/advanced/plugin-first-architecture.md docs/advanced/cross-platform-mcp.md packages/npm-qiongli/README.md
git commit -m "docs: describe bundled literature mcp support"
```

Expected: commit succeeds.

## Task 5: Fix Full CLI MCP Preview Domain Packet

**Files:**
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`

- [ ] **Step 1: Add failing domain preview test**

In `tests/test_mcp_tool_handlers.py`, add this method inside `MCPToolHandlerTests` after `test_task_run_preview_exposes_effective_runtime_options`:

```python
    def test_task_run_preview_includes_domain_context_in_task_packet(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "cwd": str(root),
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "finance-preview",
                    "domain": "finance",
                },
            )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        task_packet = payload["data"]["task_packet"]
        self.assertEqual(task_packet["domain"], "finance")
        self.assertEqual(task_packet["requested_domain"], "finance")
        self.assertIn("domain_profile_status", task_packet)
        self.assertIn("domain_profile_display_name", task_packet)
```

- [ ] **Step 2: Run test and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_task_run_preview_includes_domain_context_in_task_packet -v
```

Expected: failure because `task_packet` lacks `domain` or domain profile fields.

- [ ] **Step 3: Add domain preview helper**

In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, add this helper after `_task_run_preview`:

```python
def _task_run_preview_domain_fields(orchestrator: Any, task_run_kwargs: dict[str, Any]) -> dict[str, Any]:
    domain = str(task_run_kwargs.get("domain") or "auto").strip() or "auto"
    if domain == "auto":
        return {"domain": "auto", "requested_domain": "auto"}
    domain_context = orchestrator._load_domain_profile_context(domain)
    fields = orchestrator._build_domain_packet_fields(domain_context)
    return dict(fields) if isinstance(fields, dict) else {"domain": domain, "requested_domain": domain}
```

- [ ] **Step 4: Merge domain fields into preview task packet**

In `_tool_task_run`, after:

```python
            task_packet.setdefault("artifact_root", data.get("artifact_root"))
```

add:

```python
            task_packet.update(_task_run_preview_domain_fields(orchestrator, task_run_kwargs))
```

The block should end as:

```python
            task_packet.setdefault("artifact_root", data.get("artifact_root"))
            task_packet.update(_task_run_preview_domain_fields(orchestrator, task_run_kwargs))
            task_packet["runtime_plan"] = preview["effective_runtime_plan"]
```

- [ ] **Step 5: Run domain preview test**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_task_run_preview_includes_domain_context_in_task_packet -v
```

Expected: test passes.

## Task 6: Fix Full CLI MCP Triad Mapping

**Files:**
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`

- [ ] **Step 1: Add failing triad mapping test**

In `tests/test_mcp_tool_handlers.py`, add this method inside `MCPToolHandlerTests` after `test_task_run_tool_can_launch_agents_when_explicitly_enabled`:

```python
    def test_task_run_tool_maps_triad_execution_mode_to_triad_flag(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "claude"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "triad-topic",
                        "execution_mode": "triad",
                        "run_agents": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertTrue(stub.kwargs["triad"])
```

- [ ] **Step 2: Run test and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_task_run_tool_maps_triad_execution_mode_to_triad_flag -v
```

Expected: failure because `triad` is missing from `stub.kwargs`.

- [ ] **Step 3: Add triad schema field**

In `MCP_TOOL_DEFINITIONS`, inside `qiongli_task_run` properties, add:

```python
                "triad": {"type": "boolean"},
```

Place it near `execution_mode`.

- [ ] **Step 4: Compute execution mode once in `_task_run_kwargs`**

Replace `_task_run_kwargs` with:

```python
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
```

- [ ] **Step 5: Use triad flag in preview metadata**

In `_task_run_preview`, replace:

```python
        triad=False,
```

with:

```python
        triad=bool(task_run_kwargs.get("triad")),
```

- [ ] **Step 6: Run MCP handler tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers -v
```

Expected: all MCP handler tests pass.

- [ ] **Step 7: Commit full CLI MCP fixes**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "fix(mcp): align task run preview and triad routing"
```

Expected: commit succeeds.

## Task 7: Final Verification Pass

**Files:**
- Verify: `README.md`
- Verify: `packages/qiongli-plugin/.claude-plugin/plugin.json`
- Verify: `tooling/scripts/build_plugin_artifacts.py`
- Verify: `tooling/scripts/validate_marketplace_install.py`
- Verify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`

- [ ] **Step 1: Run plugin and artifact tests**

Run:

```bash
uv run python -m unittest tests.test_plugin_manifests tests.test_plugin_distribution_contract tests.test_plugin_artifacts -v
```

Expected: all tests pass.

- [ ] **Step 2: Run MCP server and handler tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_cli tests.test_mcp_tool_handlers tests.test_mcp_stdio_server -v
```

Expected: all tests pass.

- [ ] **Step 3: Run documentation boundary tests**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs tests.test_distribution_materialization_docs tests.test_package_readmes -v
```

Expected: all tests pass.

- [ ] **Step 4: Run npm tests**

Run:

```bash
npm test --prefix packages/npm-qiongli
```

Expected: all Node tests pass.

- [ ] **Step 5: Run marketplace validator**

Run:

```bash
uv run python scripts/validate_marketplace_install.py --dist-dir /tmp/qiongli-marketplace-check
```

Expected: output includes:

```text
[OK] codex marketplace artifact
[OK] claude marketplace artifact
bundled literature MCP checked
[OK] marketplace validation completed
```

- [ ] **Step 6: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 7: Review changed files**

Run:

```bash
git status --short
git diff --stat HEAD~4..HEAD
```

Expected: changed files are limited to README/docs, Claude plugin manifest, plugin artifact tooling/tests, validator tooling/tests, and MCP handler/tests.

## Self-Review

- Spec coverage: The plan covers README baseline, Claude Code bundled literature MCP parity, artifact validation, documentation updates, full CLI MCP preview domain context, full CLI MCP triad mapping, and final verification.
- Placeholder scan: No placeholder markers are used.
- Type consistency: The plan consistently uses `mcpServers`, `qiongli`, `command`, `args`, `cwd`, `expects_bundled_mcp`, `execution_mode`, and `triad`.
- Structure stability: The plan keeps canonical source in `content/`, plugin source in `packages/qiongli-plugin/`, runtime MCP source in `packages/qiongli-plugin/mcp/qiongli-literature-provider/`, and generated artifacts in temporary build locations.
