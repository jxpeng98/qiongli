# Qiongli Literature Provider MCPB

This package is the Claude Desktop MCPB for Qiongli literature provider access. It contains a zero-dependency Node stdio MCP server, so users do not need to install the `qiongli` CLI or run npm before installing the MCPB.

For Codex-style desktop clients, use the Codex plugin bundle when available. For Claude Code, Cursor-style clients, or any client that can launch a local stdio MCP command and needs the full Python-backed tool set, use the unified CLI server:

```bash
qiongli mcp serve --transport stdio
qiongli mcp config example --target codex --json
```

The bundled MCPB server and the CLI server both read the shared provider config. The MCPB also accepts Claude Desktop user configuration values directly from the extension settings.

## Local Claude Desktop Install

Build or package this directory as a Claude Desktop `.mcpb` extension, then install it through Claude Desktop's extension settings. The manifest declares user configuration fields for:

- OpenAlex email
- Semantic Scholar API key
- Default result limit

Claude Desktop injects these values into the local Node MCP server environment when the extension runs. The server can also save provider values through `qiongli_save_provider_config` into the shared local provider config. Do not store provider credentials in the Qiongli Desktop skill ZIP or commit local secrets into this package.

## Development

Run the syntax check:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Start the stdio server:

```bash
npm --prefix packages/qiongli-literature-mcpb start
```
