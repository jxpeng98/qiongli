# Qiongli Literature Provider MCPB

This package is the Claude Desktop MCPB for Qiongli literature provider access. It contains a zero-dependency Node stdio MCP server, so users do not need to install the `qiongli` CLI or run npm before installing the MCPB.

Pair it with a manual Desktop skill ZIP when you are installing Qiongli without Claude Code or Codex plugin marketplaces. Upload a `qiongli-claude-desktop-skill-*.zip` skill first, then install this MCPB when the same Desktop workspace needs literature MCP tools such as `qiongli_literature_search`, `qiongli_config_status`, `qiongli_configure_provider`, and `qiongli_save_provider_config`.

This MCPB does not launch orchestrator agents. If the Desktop or coding client also needs the full CLI MCP server, local agent runtime, or orchestration tools such as `qiongli_task_run`, install the Python or npm Qiongli CLI and configure the full CLI MCP server separately:

For Codex-style desktop clients, use the Codex plugin bundle when available. For Claude Code, Cursor-style clients, or any client that can launch a local stdio MCP command and needs the full Python-backed tool set, use the unified CLI server:

```bash
qiongli mcp serve --transport stdio
qiongli mcp config example --target codex --json
```

The bundled MCPB server and the full CLI MCP server both read the shared provider config. The MCPB also accepts Claude Desktop user configuration values directly from the extension settings.

## Local Claude Desktop Install

Build or package this directory as a Claude Desktop `.mcpb` extension, then install it through Claude Desktop's extension settings. The manifest declares user configuration fields for:

- OpenAlex API key
- OpenAlex email
- Semantic Scholar API key
- Default result limit

Claude Desktop injects these values into the local Node MCP server environment when the extension runs. The server can also open a local setup page through `qiongli_configure_provider` or save explicit provider values through `qiongli_save_provider_config` into the shared local provider config. `qiongli_open_config_wizard` remains as a compatibility alias for older instructions. Do not store provider credentials in the Qiongli Desktop skill ZIP or commit local secrets into this package.

## Development

Run the syntax check:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Start the stdio server:

```bash
npm --prefix packages/qiongli-literature-mcpb start
```
