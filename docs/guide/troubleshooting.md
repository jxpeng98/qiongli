# Troubleshooting

Start with `qiongli --version`, `qiongli doctor` and `qiongli install list`.
They help distinguish a CLI problem from a Plugin registration or Host-session
problem. Use `--json` if you need structured diagnostics.

## The terminal still runs an older version

Check `type -a qiongli ql` on macOS/Linux, or `Get-Command qiongli,ql -All` in
PowerShell. Different package managers can install separate copies; PATH decides
which one runs. Use the new executable's absolute path, then run `qiongli setup`
to review the other installations. The review provides manual cleanup guidance;
it does not delete, archive or move packages or research files.

The native npm and PyPI packages already include the CLI. Installing a separate
Python Full runtime is not a remedy for a 2.x problem. See
[package-manager installation](cli-2x.md#package-managers) for PATH and npm script warnings.

## Plugin files were exported, but registration failed

Keep the exported directory and rerun `qiongli install plugin` for the same Host.
The installer checks its receipt before updating it. If another Qiongli Plugin is
enabled, follow the listed migration or manual-disable guidance. Known Codex entries support confirmed migration; review other conflicts manually. A changed file or unexpected registration needs review, not a
blanket cache deletion. See [first use](cli-2x.md#first-use).

## Skills or MCP tools are missing

Confirm that the intended Plugin is enabled, then start a new Host session.
The Host loads Skills from its Plugin cache; no `~/.agents/skills` copy is needed.
In Codex, select `$qiongli` or a visible shortcut such as `$qiongli-paper-read`.
You can also state the research task naturally. A Skills-only export does not
register the Host or connect MCP.

Run `qiongli mcp check --profile full` for a local protocol check. In the Host,
ask for the actual tools and a `qiongli_config_status` call. A successful local
check alone does not prove the session has connected.

## Literature search returns `strategy_only`

Call `qiongli_literature_status` before deciding which search route is available.
Provider setup and Host native search are separate. `qiongli_search_plan` reports
`provider_capability_mode` separately from `search_execution_mode`: the latter
can be `native_only`, `hybrid_search`, `provider_connected` or `strategy_only`.
MCP servers do not execute Host native search; the active agent does that with
its own available tools. Follow [provider setup](../advanced/mcp-providers-setup.md)
and report service errors or missing access rather than inventing search results.

## The Graph page is empty or out of date

The Graph projects saved research records; it does not scan arbitrary PDFs into
verified claims. Read the [Graph example](../examples/research-graph.md), review
and save structured records, then refresh the project and export a new snapshot.
An older HTML page stays unchanged. If `--open` does not show a window, open the
printed file path manually. Source commands from an old revision need a fresh export.

## Hooks or Zotero do not respond

For hooks, verify the Plugin setting, Host trust and an actual supported event;
installation checks do not prove delivery. See [Hook verification](../advanced/agent-skill-collaboration.md#optional-context-hooks).
For Zotero, check that the application is running with a compatible Companion,
then call `qiongli_zotero_status`. See [Zotero setup](../advanced/mcp-zotero-integration.md).

When reporting a problem, include the CLI version, platform, installation channel,
command and relevant error. Remove private paths, credentials and research text
before sharing diagnostics.
