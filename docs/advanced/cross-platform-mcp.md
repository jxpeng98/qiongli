# Cross-Platform MCP Server

Qiongli ships multiple local MCP entrypoints. Desktop users can use bundled local Node MCP runtimes for literature-provider search without installing the `qiongli` CLI. CLI users and advanced workflows can use the full Python-backed `qiongli mcp` server.

## Full CLI Stdio Mode

Use the full CLI server when the desktop or agent client can start a local `qiongli` command:

```bash
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target hermes --json
```

The server command is:

```bash
qiongli mcp serve --transport stdio
```

This mode does not require a remote server. The client launches the local process, and Qiongli reads provider credentials from the shared provider configuration. It requires the npm, pipx/pip, or `full` bootstrap runtime so that `qiongli` is on `PATH`.

The full CLI server exposes both provider/configuration tools and orchestrator tools:

- `qiongli_config_status`, `qiongli_configure_provider`, `qiongli_save_provider_config`, and `qiongli_collect_evidence` for MCP/provider readiness.
- `qiongli_orchestrator_doctor` for local runtime preflight checks.
- `qiongli_task_plan` for a no-agent task plan.
- `qiongli_task_run` for a controlled task-run surface. It defaults to preview and does not launch local Codex, Claude, or Gemini processes unless the caller explicitly passes JSON boolean `run_agents: true`.

Use the full CLI server when Codex, Claude Code, or another local client needs to call the Qiongli orchestrator as a tool.

## Codex Bundled Plugin MCP

The Codex plugin package includes `packages/qiongli-plugin/.mcp.json`, references it from `.codex-plugin/plugin.json`, and bundles a zero-dependency Node server under `packages/qiongli-plugin/mcp/qiongli-literature-provider/`. Codex plugin installs can therefore register and launch the literature-provider MCP server from the plugin bundle instead of requiring users to copy a separate `config.toml` snippet or install the `qiongli` CLI.

The bundled server entry is:

```bash
node ./mcp/qiongli-literature-provider/index.mjs
```

Provider keys are not embedded in the plugin manifest. Desktop users can configure keys with the bundled `qiongli_configure_provider` MCP tool, or script explicit writes with `qiongli_save_provider_config`. CLI users can configure the same shared provider file with `qiongli mcp configure` or `qiongli provider setup`.

Because Codex launches this MCP server from the installed plugin bundle, Codex's MCP settings page should be treated as an enable/disable and tool-policy surface for the bundled server, not as the credential configuration UI. The supported key setup loop is:

1. Call `qiongli_config_status` to inspect the redacted status and shared `config_path`.
2. Call `qiongli_configure_provider` and open the returned `127.0.0.1` URL.
3. Enter the OpenAlex API key, optional OpenAlex email, and Semantic Scholar API key in the local browser form.
4. Call `qiongli_literature_status` before claiming `provider_connected`.

Keep provider secrets out of `.mcp.json`, `.codex-plugin/plugin.json`, marketplace metadata, and release artifacts. The bundled Node server reads the shared provider config at runtime.

The bundled Codex runtime focuses on literature-provider tools. Use the full CLI stdio server when you need the Python-backed orchestration MCP tools.

`qiongli_configure_provider` is the platform-neutral setup contract. Codex Desktop, Claude Desktop MCPB, Claude Code, Cursor-style clients, and any local stdio MCP client should prefer it for credentials because it opens a local `127.0.0.1` setup page and returns only redacted status. `qiongli_open_config_wizard` remains available as a compatibility alias for older clients and docs.

## Claude Code Bundled Plugin MCP

The Claude Code plugin package declares a bundled `qiongli` MCP server from `packages/qiongli-plugin/.claude-plugin/plugin.json`. It uses the same zero-dependency Node literature-provider runtime under `packages/qiongli-plugin/mcp/qiongli-literature-provider/` as the Codex plugin.

The bundled server entry is:

```bash
node ${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs
```

This bundled runtime covers literature-provider tools such as provider configuration, status, and search without requiring the `qiongli` CLI. Use the full CLI stdio server when Claude Code needs Python-backed orchestration tools such as `qiongli_orchestrator_doctor`, `qiongli_task_plan`, or `qiongli_task_run`.

## Claude Desktop MCPB

The Claude Desktop `qiongli-literature-provider.mcpb` also contains the zero-dependency Node literature-provider server. It exposes user configuration fields for OpenAlex API key, optional OpenAlex email, Semantic Scholar API key, and default result limit, so a Desktop user can install the MCPB and configure provider keys without installing `qiongli` or running npm.

For manual Claude Desktop installs, treat the Skill ZIP and MCPB as complementary assets:

- The `qiongli-claude-desktop-skill-*.zip` upload provides the agent instructions, workflows, templates, subject overlays, and skill guidance.
- The `qiongli-literature-provider.mcpb` install provides literature MCP tools such as `qiongli_literature_search`.
- The MCPB does not launch orchestrator agents. If the same Desktop or coding client needs `qiongli_task_run`, install the full CLI MCP server separately with `qiongli mcp serve --transport stdio`.

## Provider Keys

CLI users can configure keys directly:

```bash
qiongli mcp configure --provider openalex --field email --value you@example.com
qiongli mcp configure --provider semantic-scholar --field api-key --value "$S2_API_KEY"
qiongli mcp doctor --json
```

Desktop-only users can use the MCP tools exposed by the bundled Node server or full CLI server:

- `qiongli_configure_provider`: starts a local browser form for provider key setup without putting API keys in chat.
- `qiongli_open_config_wizard`: compatibility alias for `qiongli_configure_provider`.
- `qiongli_save_provider_config`: saves one provider field from the desktop client; use it only for explicit scripted writes or when the user deliberately supplied the value in chat.
- `qiongli_config_status`: reports redacted provider status.
- `qiongli_literature_search`: searches OpenAlex and Semantic Scholar when provider access is configured.

The full CLI server exposes the same `qiongli_configure_provider` flow.

Secrets are written to the same provider config used by `qiongli provider setup` and `qiongli provider doctor`. Tool results and doctor output report only configured/missing status, not raw key values.

## HTTP Mode

Use HTTP only when a platform needs an HTTP endpoint:

```bash
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
```

For normal desktop use, this can still be local. You need a remote server only when the MCP client cannot run local commands, when several machines must share one always-on endpoint, or when your deployment policy requires a central hosted service. If you expose HTTP remotely, put it behind your normal authentication, TLS, and secret-management controls.

## Supported Provider Fields

The MCP server derives provider fields from the Qiongli provider registry. Current fields include:

- `openalex.api_key`
- `openalex.email`
- `semantic_scholar.api_key`
- `crossref.email`
- `pubmed.api_key`

When new providers or fields are added to the registry, the config wizard and provider env alias tool pick them up from the same source.
