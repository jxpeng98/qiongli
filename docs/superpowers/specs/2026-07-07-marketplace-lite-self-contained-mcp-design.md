# Marketplace Lite Self-Contained MCP Design

## Problem

Qiongli marketplace plugins currently bundle the literature-provider MCP source
under `mcp/qiongli-literature-provider/`, but the server entry still depends on
a user-installed runtime:

```json
{
  "command": "node",
  "args": ["./mcp/qiongli-literature-provider/index.mjs"]
}
```

That is not a true no-dependency marketplace experience. Codex, Claude Code,
and Claude Desktop can launch local MCP subprocesses, but they do not provide
Node or Python for bundled server code. If a user's machine does not have
`node` on `PATH`, the current Marketplace Lite MCP can fail before the first
`initialize` request.

The desired product shape is a local MCP server that is installed as part of the
marketplace plugin itself. The user installs the plugin once, the MCP server
starts locally from the plugin payload, and no Node, Python, npm, pip, or full
Qiongli CLI install is required for literature-provider tools.

## Product Direction

Qiongli should have two user-facing runtime profiles:

| Profile | Entry | Runtime | Purpose |
|---|---|---|---|
| Marketplace Lite | Codex, Claude Code, or Claude Desktop marketplace/direct plugin | Rust self-contained local MCP executable bundled in the plugin | Literature provider tools, provider setup, search planning, evidence export, Zotero import-file support, optional Zotero companion bridge |
| Full CLI | `qiongli install --profile full` or direct `qiongli mcp serve --transport stdio` | Python Qiongli CLI | Complete local product: literature tools plus orchestrator, task planning, task execution, doctor checks, project guidance, and local agent coordination |

This is one Qiongli product with two runtime profiles, not two separate
projects. Marketplace Lite is the no-dependency local plugin runtime. Full CLI
remains the complete Python runtime.

## Core Decision

Build a new self-contained Literature Provider MCP runtime in **Rust** and make
marketplace plugins launch it instead of `node`.

Rationale:

- Rust produces native executables for macOS, Linux, and Windows and has strong
  compile-time guarantees around data shapes, error handling, and ownership.
- The Lite MCP server mainly needs JSON-RPC stdio, HTTP API clients, XML/JSON
  parsing, filesystem config, and a small local HTTP setup wizard. Rust handles
  this well with a small, explicit dependency set.
- Rust avoids shipping an embedded Python interpreter or Node runtime in every
  marketplace package.
- Rust is a better long-term choice than starting in another compiled language
  and migrating later if maintainers want a stricter systems-runtime foundation.
- PyInstaller, Nuitka, or Node single-file packagers would reduce user setup but
  keep the runtime tied to Python or Node packaging behavior, larger runtime
  bundles, and platform-specific bundler edge cases.

The MCP protocol remains standard. The change is the server implementation and
packaging, not a vendor-specific transport.

## Why Full CLI Should Stay Python

The Python Full CLI should **not** be migrated to Rust as part of this project.

Full CLI owns broad workflow behavior:

- install and update flows
- project materialization
- subject/runtime guidance
- local agent orchestration
- task-run preview and execution
- doctor checks
- release and validation tooling
- Python provider modules already used by tests and scripts

Those are higher-level product workflows, not the narrow no-dependency
marketplace runtime problem. Rewriting Full CLI in Rust would create a large
product rewrite with little direct benefit to the current marketplace issue.

Rust should be used only for the Marketplace Lite MCP runtime in this phase.
Full CLI remains the canonical complete Python runtime.

## Marketplace Runtime Architecture

The self-contained runtime is a local stdio MCP server:

```text
Codex / Claude / other local MCP host
  -> launches plugin-owned executable
  -> exchanges JSON-RPC MCP messages over stdio
  -> receives tools/list and tools/call responses
```

The plugin manifest points to a plugin-local executable path:

```json
{
  "command": "./bin/qiongli-literature-provider",
  "args": ["--transport", "stdio"],
  "cwd": "."
}
```

Claude manifests use the platform plugin root variable:

```json
{
  "command": "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
  "args": ["--transport", "stdio"],
  "cwd": "${CLAUDE_PLUGIN_ROOT}"
}
```

Because an executable cannot be universal across macOS, Linux, and Windows, the
release system must produce platform-specific plugin artifacts or
platform-aware package entries. Each artifact should keep the executable at the
same logical path inside the plugin payload, but the file content is built for
the target platform.

Minimum target set:

- `darwin-aarch64`
- `darwin-x86_64`
- `linux-x86_64`
- `linux-aarch64`
- `windows-x86_64`

If a marketplace does not support platform-specific asset selection, the release
guide must publish distinct platform artifacts or keep a clearly marked legacy
Node fallback for that marketplace until platform-aware delivery exists. A
single binary artifact cannot be no-dependency across all operating systems.

## Rust Runtime Scope

The Rust runtime should live under:

```text
packages/qiongli-lite-mcp/
  Cargo.toml
  src/
    main.rs
    mcp/
    tools/
    config/
    providers/
    searchplan/
    zotero/
    orchestrator/
  tests/
  testdata/
```

Recommended dependency policy:

- Use `serde` and `serde_json` for JSON contracts.
- Use `thiserror` for focused error types.
- Use `reqwest` with `rustls-tls` and blocking client for provider HTTP calls,
  unless binary-size tests force a smaller client later.
- Use `quick-xml` for arXiv/PubMed XML parsing.
- Use a small local HTTP implementation for the provider config wizard. If a
  crate is needed, use one with a narrow surface such as `tiny_http`.
- Avoid async runtimes in the first cut unless provider concurrency becomes a
  measured issue. Blocking HTTP is simpler and sufficient for Lite MCP.

## Lite MCP Tool Set

Marketplace Lite exposes the literature-provider surface that can be
implemented without the Python full runtime.

### Provider And Configuration

- `qiongli_config_status`
- `qiongli_configure_provider`
- `qiongli_save_provider_config`
- `qiongli_open_config_wizard`

The config wizard remains a local `127.0.0.1` browser page served by the MCP
runtime. Secrets are written to the shared provider config and never returned in
tool output.

### Literature Search

- `qiongli_literature_status`
- `qiongli_search_plan`
- `qiongli_literature_search`
- `qiongli_literature_export_evidence`

Provider support should match the current Lite provider scope:

- OpenAlex
- Semantic Scholar
- Crossref
- PubMed
- arXiv

The Lite runtime should preserve current search controls where practical:

- DOI and title lookup
- topic and review search modes
- `limit`, `per_provider_limit`, and `total_limit`
- query variants
- search depth
- document type and venue filters
- provider diagnostics
- deduped normalized results
- auditable evidence export

### Zotero Boundary

Marketplace Lite should include Zotero import-file support because it is local
file generation inside the MCP server:

- `qiongli_zotero_status`
- `qiongli_zotero_export_import_files`

When the Qiongli Zotero Companion is installed and Zotero Desktop is running,
the Lite runtime may also expose and support:

- `qiongli_zotero_search`
- `qiongli_zotero_upsert_references`

The Companion remains a separate Zotero-side install because it runs inside
Zotero's extension and permission model. The marketplace plugin must not claim
that it can silently install or bypass a Zotero extension. Without the
Companion, the Lite runtime should return `fallback_only` or
`companion_missing` and offer RIS, BibTeX, CSL JSON, and import-report files.

### Orchestrator Preview

Marketplace Lite can expose preview-only workflow helpers:

- `qiongli_orchestrator_route`
- `qiongli_task_plan`

These tools must not launch local agents. They can render route decisions and
task plans from embedded workflow metadata so desktop users can try the
workflow planning surface without installing Full CLI.

Execution tools such as `qiongli_task_run` remain Full CLI capabilities until
there is a separate self-contained execution-runtime design.

## Full CLI Relationship

The Python Full CLI remains the canonical complete runtime.

Full CLI keeps:

- `qiongli mcp serve --transport stdio`
- Python-backed literature tools
- orchestrator route and doctor tools
- `qiongli_task_plan`
- `qiongli_task_run`
- project guidance and trace writes
- local agent execution
- installer, doctor, release, and validation workflows

Marketplace Lite does not replace or weaken the Full CLI. It removes the Node
dependency from marketplace-installed literature tools.

Shared contracts between Rust Lite and Python Full:

- provider config file format: `~/.config/qiongli/providers.json`, or
  `$QIONGLI_CONFIG_HOME/providers.json`
- provider environment aliases
- MCP tool names and input schemas for overlapping tools
- normalized literature result schema
- diagnostics schema
- search-plan mode names
- secret redaction behavior
- Zotero import-file formats

Differences that should remain visible:

| Capability | Marketplace Lite | Full CLI |
|---|---:|---:|
| Runs without Node/Python | Yes | No, requires Python package/runtime |
| Literature provider search | Yes | Yes |
| Provider config wizard | Yes | Yes |
| Zotero import files | Yes | Yes |
| Zotero companion bridge | Optional Companion required | Optional Companion required |
| Orchestrator route preview | Yes | Yes |
| Task plan preview | Yes | Yes |
| Local agent execution | No | Yes |
| Project guidance writes | No, except explicit import/export files | Yes |
| Full doctor and installer | No | Yes |

## Consistency Model

Rust Lite and Python Full must stay compatible through external contracts, not
through shared implementation code.

Add versioned contract assets:

```text
content/mcp-contracts/
  lite-tools.json
  provider-config.schema.json
  literature-result.schema.json
  literature-diagnostics.schema.json
  search-plan.schema.json
  zotero-import-files.schema.json
  fixtures/
    openalex-search-response.json
    semantic-scholar-search-response.json
    crossref-search-response.json
    pubmed-summary-response.json
    arxiv-search-response.xml
    expected-normalized-results.json
```

Both runtimes consume or validate against these contracts:

- Rust unit tests validate Rust structs and serialized output.
- Python tests validate Full CLI handlers and output.
- Black-box parity tests launch both MCP servers over stdio and compare shared
  fields for overlapping tools.
- Node MCP remains a temporary reference implementation until Rust reaches
  parity for marketplace-lite tools.

The stable product contract is the schema and fixture set. Rust is the Lite
runtime implementation. Python is the Full runtime implementation.

## Repository Impact

This change is additive:

- Add `packages/qiongli-lite-mcp/` for the Rust Lite MCP runtime.
- Keep `packages/python-qiongli/` as the Full CLI runtime.
- Keep `packages/qiongli-literature-mcpb/` as the current Node reference and
  standalone MCPB path during migration.
- Update marketplace packaging to use the Rust runtime once it passes parity
  checks.
- Rebuild standalone MCPB artifacts around the Rust runtime after marketplace
  activation passes.

The existing Python implementation needs shared contract fixtures and parity
tests so Lite and Full stay compatible, but the Full CLI does not need to be
rewritten in Rust.

## Packaging And Install Behavior

### Codex Marketplace Plugin

The Codex plugin package should contain:

```text
plugin.json
.codex-plugin/plugin.json
.mcp.json
commands/
skills/qiongli-workflow/
bin/qiongli-literature-provider
```

`.mcp.json` launches the local binary, not `node`.

### Claude Code Marketplace Plugin

The Claude plugin package should contain:

```text
plugin.json
.claude-plugin/plugin.json
commands/
skills/qiongli-workflow/
bin/qiongli-literature-provider
```

`.claude-plugin/plugin.json` declares the plugin-local binary in `mcpServers`.

### Claude Desktop Direct Plugin

The direct plugin uses the same Rust Lite runtime binary and the same MCP server
declaration shape as Claude Code, subject to Desktop's direct-plugin manifest
rules.

### Standalone MCPB

The standalone MCPB remains available for users who want only provider tools,
but it should be rebuilt around the same Rust Lite runtime instead of the Node
runtime after migration. This keeps one provider implementation for marketplace
and MCPB users.

### Other MCP Clients

Other local MCP clients can use the same executable through standard stdio MCP
configuration:

```json
{
  "mcpServers": {
    "qiongli": {
      "command": "/path/to/qiongli-literature-provider",
      "args": ["--transport", "stdio"]
    }
  }
}
```

Registry metadata can publish the Lite runtime as MCPB or OCI package entries
after the marketplace artifacts are stable.

## Security And Privacy

- Do not store API keys in plugin manifests, marketplace metadata, release
  archives, logs, tests, or generated examples.
- Local config writes must create parent directories and write provider config
  with owner-only permissions where the OS supports it.
- `qiongli_configure_provider` binds only to `127.0.0.1` or `localhost`.
- Zotero connector URLs must remain loopback-only.
- Tool outputs must redact emails and keys from errors and diagnostics.
- Marketplace Lite must not execute local agents or arbitrary shell commands.
- Search providers are external network services; diagnostics must identify the
  provider and sanitized failure mode without exposing credentials.

## Migration Strategy

1. Add external contracts and fixtures before implementing Rust behavior.
2. Add the Rust Lite runtime behind tests and keep existing Node packaging
   unchanged.
3. Build platform-specific Rust Lite binaries in release artifacts.
4. Add marketplace packaging tests that assert plugin manifests no longer use
   `node`.
5. Switch prerelease marketplace plugins to the Rust Lite binary.
6. Keep standalone Node MCPB as a legacy fallback for one release train.
7. Rebuild the MCPB around the Rust Lite binary after marketplace activation
   passes.
8. Remove or de-emphasize the Node runtime only after parity and install checks
   cover all supported plugin targets.

## Non-Goals

- Do not remove Full CLI.
- Do not rewrite the Python orchestrator in the Lite MCP change.
- Do not migrate Full CLI to Rust in this phase.
- Do not silently install the Zotero Companion.
- Do not require provider API keys for MCP startup.
- Do not add remote MCP hosting as the default marketplace path.
- Do not make Marketplace Lite launch local Codex, Claude, or Antigravity
  agents.

## Acceptance Criteria

- Marketplace Lite plugin artifacts for Codex, Claude Code, and Claude Desktop
  include a plugin-local Rust-built MCP executable.
- Marketplace Lite manifests launch that executable and do not invoke `node`,
  `python`, `npm`, `pip`, `qiongli`, or shell wrapper scripts.
- A machine without Node and Python can run `initialize`, `tools/list`, and
  `qiongli_literature_status` against the bundled executable.
- `qiongli_literature_search` works for at least arXiv and one JSON provider in
  automated tests without user-installed runtimes.
- Provider config status and save tools share the same provider config file as
  Full CLI.
- Tool definitions and overlapping result shapes match Full CLI contract
  fixtures.
- Zotero import-file export works without Zotero Companion.
- Zotero local search and write tools report `companion_missing` when Zotero
  Companion is absent, and work only through loopback when Companion is present.
- Full CLI tests continue to pass without depending on the Rust Lite runtime.
- Release validation reports whether each artifact is `marketplace-lite-binary`,
  `python-full-runtime`, or `legacy-node-provider`.

## Direct Answers

Will this affect Full CLI users?

No, if implemented as an additive runtime. Full CLI remains Python-backed and
keeps complete orchestrator and task execution capabilities. The main effect is
that shared tool schemas and provider result contracts become more explicit.

Does Marketplace Lite need Rust?

Rust is now the selected implementation for the Lite MCP runtime because the
target is a self-contained local executable that does not require user-installed
Node or Python. Rust is not a requirement of the MCP protocol; it is the chosen
runtime implementation strategy.

Does Full CLI need to migrate to Rust later?

No. Full CLI should stay Python unless a separate future project proves that
the full orchestrator, installer, and agent execution model need a native
runtime. The current problem is marketplace-lite startup reliability, not Full
CLI language choice.

How do Rust Lite and Python Full stay consistent?

Use contract-first compatibility: external schemas, golden provider fixtures,
black-box MCP parity tests, shared provider config schema, and release gates
that compare overlapping tool behavior across Rust Lite and Python Full.

Does Zotero Companion still need separate installation?

Yes for local Zotero library search and direct writes. Import-file generation
does not need Companion. Companion-backed actions require Zotero Desktop plus
the Zotero-side extension because that is Zotero's application boundary.
