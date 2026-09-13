# Local Plugin MCP Activation Design

## Goal

Make the bundled Qiongli Lite MCP server reliably usable from local plugin
installs across Codex App, Codex CLI, Claude Code, and Claude Desktop local
plugin paths. Web-only and remote-cloud execution are out of scope.

## Scope

Supported local surfaces:

- Codex App and Codex CLI marketplace plugin installs.
- Claude Code marketplace plugin installs.
- Claude Desktop local/direct plugin installs when the client accepts Claude
  plugin bundles.
- Claude Desktop MCPB as a provider-only fallback.

Out of scope:

- Claude.ai web, Codex Cloud, hosted remote MCP, and any remote shared server.
- Requiring users to install Node, Python, npm, pip, or the full Qiongli CLI for
  Marketplace Lite provider tools.

## Current State

`v1.18.0-beta.1` already packages the Rust Lite executable as
`bin/qiongli-literature-provider`.

Codex artifacts declare MCP through `.codex-plugin/plugin.json` pointing to
plugin-root `.mcp.json`. Local `codex mcp list` confirms that this form is
loaded by Codex CLI.

Claude artifacts declare MCP inline in `.claude-plugin/plugin.json`, which
matches Claude Code documentation for plugin-provided MCP servers. Claude also
supports plugin-root `.mcp.json`, but inline declaration keeps the server visible
in the manifest.

The missing release gate is not structural presence. It is activation evidence:
the release validator must launch the plugin-declared command and prove that the
server answers `initialize` and `tools/list`.

## Design

Add a marketplace activation validator that:

1. Loads the plugin MCP declaration from the same manifest path the client uses.
2. Resolves plugin-local variables such as `${CLAUDE_PLUGIN_ROOT}`.
3. Starts the declared stdio command from the declared working directory.
4. Sends MCP `initialize` and `tools/list` JSON-RPC requests.
5. Requires the Lite provider tools that prove the server is not an empty MCP
   process:
   - `qiongli_literature_status`
   - `qiongli_literature_search`
   - `qiongli_task_plan`

The validator stays local-only. It does not add remote HTTP MCP, web activation,
or cloud startup expectations.

## Packaging Policy

Keep the current platform manifests:

- Codex: `.codex-plugin/plugin.json` plus plugin-root `.mcp.json`.
- Claude: `.claude-plugin/plugin.json` with inline `mcpServers`.
- Claude Desktop MCPB: standalone `manifest.json` plus bundled executable.

Do not move provider secrets into plugin manifests. Credentials remain in shared
provider config or client-sensitive MCPB fields.

## Acceptance Criteria

- Codex and Claude materialized plugin payloads can launch the declared MCP
  server from their plugin roots.
- Marketplace artifact validation reports `MCP startup checked` for Codex,
  Claude Code, and Claude Desktop direct plugin artifacts.
- The validator fails if the binary is missing, not executable, cannot start, or
  returns no required Lite tools.
- Existing full Python CLI behavior remains unchanged.
