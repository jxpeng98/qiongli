# Cross-Platform MCP Server

Qiongli ships one canonical full local MCP server: `qiongli mcp serve --transport stdio`. It exposes literature-provider tools plus orchestrator and task-run tools from one Python-backed CLI process. The bundled Node literature MCP runtimes in marketplace plugins and MCPB packages remain lite/no-CLI fallbacks for environments that cannot run the full CLI.

## Full CLI Stdio Mode

Use the full CLI server when the desktop or agent client can start a local `qiongli` command:

```bash
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target antigravity --json
qiongli mcp config example --target hermes --json
```

The server command is:

```bash
qiongli mcp serve --transport stdio
```

This mode does not require a remote server. The client launches the local process, and Qiongli reads provider credentials from the shared provider configuration. It requires the npm, pipx/pip, or `full` bootstrap runtime so that `qiongli` is on `PATH`.

For Codex and Claude Code, the preferred full local shape is a CLI-generated local plugin bundle:

```bash
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target claude --surface plugin
```

Those local plugins are different from the public marketplace lite plugins. They include a plugin-owned `.mcp.json` or plugin manifest entry that launches the full Python-backed `qiongli mcp serve --transport stdio` command.

For Antigravity and Hermes, Qiongli still writes managed client-level MCP configs:

```bash
qiongli install --profile full --target antigravity
qiongli install --profile full --target hermes
```

Use the combined install when you want the full local surface everywhere:

```bash
qiongli install --profile full --target all --surface plugin
```

With `--target all --surface plugin`, Codex and Claude Code get CLI-managed local plugin bundles, while Antigravity and Hermes receive managed MCP config entries. Set `ANTIGRAVITY_CONFIG_PATH` or `HERMES_CONFIG_PATH` when those clients use a different MCP config file. Existing unmanaged `qiongli` server entries are preserved and reported as skipped.

The full CLI server exposes literature, provider/configuration, and orchestrator tools:

- `qiongli_config_status`, `qiongli_configure_provider`, `qiongli_save_provider_config`, and `qiongli_collect_evidence` for MCP/provider readiness.
- `qiongli_literature_status`, `qiongli_literature_search`, and `qiongli_literature_export_evidence` for full CLI literature search and auditable evidence snapshots.
- `qiongli_orchestrator_route` for deciding whether Codex, Claude Code, Antigravity, or another client should upgrade from skill-only workflow routing to full orchestrator tools.
- `qiongli_orchestrator_doctor` for local runtime preflight checks.
- `qiongli_task_plan` for a no-agent task plan.
- `qiongli_task_run` for a controlled task-run surface. It defaults to preview and does not launch local runtime agents unless the caller explicitly passes JSON boolean `run_agents: true`. It accepts `guidance_mode` (`off`, `read`, `propose`, or `apply`) and echoes that mode in preview arguments. Preview reports whether `.qiongli/` guidance would be bootstrapped, but only actual task execution writes those files.

When task-run agents are launched, formal artifacts are still expected under `RESEARCH/[topic]/...`. The first non-`off` task run initializes `.qiongli/local_guidance.md` and `.qiongli/trace/` if they are missing. The project-local guidance layer writes auditable run traces under `.qiongli/trace/`; this trace location is separate from formal research outputs and from installed skill assets.

Skill-only Qiongli usage also checks `.qiongli/local_guidance.md` and `.qiongli/guidance.d/*.md` when they are present in the current project. Full orchestrator task-runs remain the stronger path because they write trace bundles, guidance proposals, validator output, and source metadata.

Use the full CLI server when Codex, Claude Code, Antigravity, or another local client needs the complete local product surface: literature tools, provider configuration, routing, planning, doctor checks, or task-run as MCP tools.

## Codex Bundled Plugin MCP

The generated Codex plugin package includes `.mcp.json`, references it from `.codex-plugin/plugin.json`, and bundles a zero-dependency Node server under `mcp/qiongli-literature-provider/`. Codex plugin installs can therefore register and launch the literature-provider MCP server from the plugin bundle instead of requiring users to copy a separate `config.toml` snippet or install the `qiongli` CLI. This bundled server is the marketplace lite/no-CLI fallback, not the full local MCP.

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

The bundled Codex runtime focuses on literature-provider tools. Use `qiongli install --profile full --target codex --surface plugin` when you need a Codex-native plugin container backed by the unified full MCP surface with both literature and Python-backed orchestration tools.

`qiongli_configure_provider` is the platform-neutral setup contract. Codex Desktop, Claude Desktop MCPB, Claude Code, Cursor-style clients, and any local stdio MCP client should prefer it for credentials because it opens a local `127.0.0.1` setup page and returns only redacted status. `qiongli_open_config_wizard` remains available as a compatibility alias for older clients and docs.

## Claude Code Bundled Plugin MCP

The generated Claude Code plugin package declares a bundled `qiongli` MCP server from `.claude-plugin/plugin.json`. It uses the same zero-dependency Node literature-provider runtime under `mcp/qiongli-literature-provider/` as the Codex plugin.

The bundled server entry is:

```bash
node ${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs
```

This bundled runtime covers literature-provider tools such as provider configuration, status, and search without requiring the `qiongli` CLI. Use `qiongli install --profile full --target claude --surface plugin` when Claude Code needs the unified full MCP surface, including `qiongli_literature_search`, `qiongli_orchestrator_route`, `qiongli_orchestrator_doctor`, `qiongli_task_plan`, or `qiongli_task_run`. Use `--target antigravity`, `--target hermes`, or `--target all --surface plugin` for local clients that should load the full MCP server instead of a bundled lite provider runtime.

## Claude Desktop MCPB

The Claude Desktop `qiongli-literature-provider.mcpb` also contains the zero-dependency Node literature-provider server. It exposes user configuration fields for OpenAlex API key, optional OpenAlex email, Semantic Scholar API key, and default result limit, so a Desktop user can install the MCPB and configure provider keys without installing `qiongli` or running npm.

For manual Claude Desktop installs, treat the Skill ZIP and MCPB as complementary assets:

- The `qiongli-claude-desktop-skill-*.zip` upload provides the agent instructions, workflows, templates, subject overlays, and skill guidance.
- The `qiongli-literature-provider.mcpb` install provides literature MCP tools such as `qiongli_literature_search`.
- The MCPB does not launch orchestrator agents. If the same Desktop or coding client needs `qiongli_orchestrator_route` or `qiongli_task_run`, install the full CLI MCP server separately. For Codex/Claude Code use `qiongli install --profile full --target all --surface plugin`; for a direct stdio integration use `qiongli mcp serve --transport stdio`.

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
- `qiongli_literature_search`: searches configured OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv providers with query variants, finance/economics deep-search routing, and sanitized diagnostics. arXiv is enabled without credentials.

The full CLI server exposes the same `qiongli_configure_provider` flow.

Finance/economics data APIs such as FRED and SEC EDGAR should be exposed through a separate data MCP surface rather than the literature MCPB. See [Finance/Economics Data MCP Boundary](finance-econ-data-mcp.md).

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
