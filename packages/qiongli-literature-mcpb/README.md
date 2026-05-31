# Qiongli Literature Provider MCPB

This package is the Claude Desktop MCPB skeleton for Qiongli literature provider access. It is intentionally minimal: provider clients and full tool behavior will be added in a later task.

## Local Claude Desktop Install

Build or package this directory as a Claude Desktop `.mcpb` extension, then install it through Claude Desktop's extension settings. The manifest declares user configuration fields for:

- OpenAlex email
- Semantic Scholar API key
- Default result limit

Claude Desktop injects these values into the local Node MCP server environment when the extension runs. Do not store provider credentials in the Qiongli Desktop skill ZIP or commit local secrets into this package.

## Development

Run the syntax check:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Start the stdio server after installing package dependencies:

```bash
npm --prefix packages/qiongli-literature-mcpb start
```
