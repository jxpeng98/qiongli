# Connect MCP to a Host

Qiongli 2 compiles Lite and Full MCP into the native executable. Neither profile
needs Python or Node. The CLI-exported Plugin configures Full (32 tools); the
native Marketplace platform Plugin configures Lite (14 tools). Both include
literature configuration, search and local Zotero tools. Full also provides the
managed research-project workflow.

## Codex and Claude Code

Use `qiongli install plugin` and choose the Host. The registered Plugin contains
the executable and MCP configuration. Start a new session after installation.
See [first use](../guide/cli-2x.md#first-use) for registration and tool checks.

## Another local MCP client

If the client accepts a stdio command, configure the absolute path to your
extracted executable with these arguments:

```text
mcp serve --profile full --transport stdio
```

Choose `--profile lite` for the smaller tool set. Let the client start the process;
a command running in a separate terminal is not connected to that client.
The native CLI supports stdio here; the old Python HTTP server instructions do
not apply. A client that cannot launch a local process needs a separate supported
integration, not an invented HTTP URL.

## Check each layer

Run `qiongli mcp check --profile full` to check this executable's initialization,
tool discovery and a read-only call. Then inspect actual session tools in your
Host and call `qiongli_config_status`. Use `qiongli_literature_status` for provider
configuration, and an authorized search to establish that the service responds.
A local MCP check does not verify the Host connection or online availability.

`qiongli_search_plan` returns a plan, including `search_execution_mode` and
suggested native queries. Modes include `provider_connected`, `native_only`,
`hybrid_search` and `strategy_only`. The separate `provider_capability_mode`
reports `provider_connected` or `strategy_only`. MCP servers must not call Codex or Claude
native search directly: the Host runs its own tools and keeps their source records
separate from provider results and the `user_corpus`.

See [provider setup](mcp-providers-setup.md) and
[Plugin contents](plugin-first-architecture.md). The Host still owns model choice
and permissions; connecting MCP does not change them.
