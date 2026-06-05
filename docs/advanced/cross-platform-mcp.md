# Cross-Platform MCP Server

Qiongli ships multiple local MCP entrypoints. Desktop users can use bundled local Node MCP runtimes for literature-provider search without installing the `qiongli` CLI. CLI users and advanced workflows can use the full Python-backed `qiongli mcp` server.

## Full CLI Stdio Mode

Use the full CLI server when the desktop or agent client can start a local `qiongli` command:

```bash
qiongli mcp config example --target codex --json
```

The server command is:

```bash
qiongli mcp serve --transport stdio
```

This mode does not require a remote server. The client launches the local process, and Qiongli reads provider credentials from the shared provider configuration. It requires the npm, pipx/pip, or `full` bootstrap runtime so that `qiongli` is on `PATH`.

## Codex Bundled Plugin MCP

The Codex plugin package includes `packages/qiongli-plugin/.mcp.json`, references it from `.codex-plugin/plugin.json`, and bundles a zero-dependency Node server under `packages/qiongli-plugin/mcp/qiongli-literature-provider/`. Codex plugin installs can therefore register and launch the literature-provider MCP server from the plugin bundle instead of requiring users to copy a separate `config.toml` snippet or install the `qiongli` CLI.

The bundled server entry is:

```bash
node ./mcp/qiongli-literature-provider/index.mjs
```

Provider keys are not embedded in the plugin manifest. Desktop users can configure keys with the bundled `qiongli_save_provider_config` MCP tool. CLI users can configure the same shared provider file with `qiongli mcp configure` or `qiongli provider setup`.

The bundled Codex runtime focuses on literature-provider tools. Use the full CLI stdio server when you need the Python-backed orchestration MCP tools.

## Claude Desktop MCPB

The Claude Desktop `qiongli-literature-provider.mcpb` also contains the zero-dependency Node literature-provider server. It exposes user configuration fields for OpenAlex email, Semantic Scholar API key, and default result limit, so a Desktop user can install the MCPB and configure provider keys without installing `qiongli` or running npm.

## Provider Keys

CLI users can configure keys directly:

```bash
qiongli mcp configure --provider openalex --field email --value you@example.com
qiongli mcp configure --provider semantic-scholar --field api-key --value "$S2_API_KEY"
qiongli mcp doctor --json
```

Desktop-only users can use the MCP tools exposed by the bundled Node server or full CLI server:

- `qiongli_save_provider_config`: saves one provider field from the desktop client.
- `qiongli_config_status`: reports redacted provider status.
- `qiongli_literature_search`: searches OpenAlex and Semantic Scholar when provider access is configured.

The full CLI server also exposes `qiongli_open_config_wizard` for a local browser form.

Secrets are written to the same provider config used by `qiongli provider setup` and `qiongli provider doctor`. Tool results and doctor output report only configured/missing status, not raw key values.

## HTTP Mode

Use HTTP only when a platform needs an HTTP endpoint:

```bash
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
```

For normal desktop use, this can still be local. You need a remote server only when the MCP client cannot run local commands, when several machines must share one always-on endpoint, or when your deployment policy requires a central hosted service. If you expose HTTP remotely, put it behind your normal authentication, TLS, and secret-management controls.

## Supported Provider Fields

The MCP server derives provider fields from the Qiongli provider registry. Current fields include:

- `openalex.email`
- `semantic_scholar.api_key`
- `crossref.email`
- `pubmed.api_key`

When new providers or fields are added to the registry, the config wizard and provider env alias tool pick them up from the same source.
