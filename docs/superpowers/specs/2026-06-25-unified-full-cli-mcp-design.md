# Unified Full CLI MCP Design

## Problem

Qiongli currently exposes its full capability through multiple surfaces:

- Native marketplace plugins provide client-native skills and a bundled Node
  literature-provider MCP.
- The `qiongli` CLI provides workflow asset installation, provider setup,
  doctor checks, and a Python-backed full MCP with orchestrator tools.
- Claude Desktop/Web use skill ZIPs and a separate literature MCPB.

This is defensible as a distribution strategy, but it is fragmented as a product
model. A user who wants "full Qiongli" has to understand plugin MCP versus CLI
MCP versus literature MCP versus provider config. The desired product behavior is
one full local installation path and one full MCP server.

## Product Direction

The `qiongli` CLI becomes the canonical full product. Marketplace plugins remain
client-native lightweight distribution and no-CLI fallback.

The full product contract is:

```bash
qiongli install --profile full --target codex
```

After installation, the target client should have one Qiongli MCP server named
`qiongli`, backed by:

```bash
qiongli mcp serve --transport stdio
```

That server must expose both literature-provider tools and orchestrator tools.

## User-Visible Capability Tiers

| Tier | Entry | Purpose |
|---|---|---|
| Lite | marketplace plugin | No CLI environment; skill workflows and lightweight literature fallback. |
| Full | `qiongli install --profile full` | Complete local Qiongli: skills, literature tools, provider config, orchestrator, doctor, task-run. |
| Dev/Release | source checkout and release scripts | Maintainer-only packaging, tests, validation, and artifact publishing. |

Marketplace plugins should not be required for full CLI usage. CLI full should
not require marketplace installation.

## Unified Full MCP Tool Set

`qiongli mcp serve` should expose one complete tool set:

### Literature

- `qiongli_literature_status`
- `qiongli_literature_search`
- `qiongli_literature_export_evidence`

### Provider And Config

- `qiongli_config_status`
- `qiongli_configure_provider`
- `qiongli_save_provider_config`
- `qiongli_open_config_wizard`
- `qiongli_list_provider_env`
- `qiongli_test_provider`
- `qiongli_collect_evidence`

### Orchestrator

- `qiongli_orchestrator_route`
- `qiongli_orchestrator_doctor`
- `qiongli_task_plan`
- `qiongli_task_run`

The Node literature-provider MCP remains in marketplace/MCPB packages as a
fallback for users without a CLI runtime. It is no longer the full-product path.

## Literature Implementation Strategy

The Python full MCP should implement literature tools natively instead of
requiring the client to register a second Node MCP server. This keeps the MCP
surface unified and avoids duplicate `qiongli_config_status` tools.

The Python implementation should reuse existing query planning, normalization,
deduplication, diagnostics, evidence export, and provider config modules where
possible:

- `bridges.providers.literature_query`
- `bridges.providers.literature_search`
- `bridges.providers.literature_diagnostics`
- `bridges.providers.literature_artifacts`
- `bridges.provider_config`

Provider execution should reach parity with the current Node literature MCP for
the supported provider set:

- Semantic Scholar
- OpenAlex
- Crossref
- PubMed

Zotero-specific tools can remain out of the first unified full MCP cut unless a
follow-up explicitly brings the Zotero companion workflow into the full MCP. The
first goal is to merge the core literature search/status/evidence tools.

## Install Behavior

`qiongli install --profile full` should include MCP registration by default.

Suggested install parts:

- `globals`: workflow assets
- `project`: project-facing assets
- `cli`: shell CLI wrappers when applicable
- `mcp`: target client MCP registration
- `doctor`: final capability check

`partial` remains workflow-only:

- `globals`
- `project`

The installer must be conservative:

- Dry-run prints the MCP config path and server entry it would write.
- Existing managed Qiongli MCP entries are updated.
- Existing unmanaged `qiongli` entries are skipped unless the user passes an
  explicit overwrite option.
- Remove should support `--parts mcp` for managed entries.
- Targets without a stable writable config path should print an exact config
  fragment instead of silently doing nothing.

## Doctor And Check Output

`qiongli doctor`, `qiongli mcp doctor`, and `qiongli check --json` should make
the unified model visible:

- CLI runtime installed
- workflow assets installed
- full MCP configured for the target when detectable
- literature tools available
- orchestrator tools available
- provider capability mode: `provider_connected` or `strategy_only`
- missing provider keys, without printing raw secrets

## Documentation Contract

Docs should describe the split as:

- Use marketplace for lite/no-CLI installs.
- Use `qiongli install --profile full` for complete local Qiongli.
- Use the Node literature MCP only as plugin/MCPB fallback.

Docs should stop implying that users need both marketplace plugin MCP and CLI MCP
for full usage.

## Non-Goals

- Do not remove marketplace plugin MCP in this change.
- Do not make marketplace plugins depend on Python.
- Do not require provider API keys for installation to succeed.
- Do not launch local runtime agents during MCP install or doctor.
- Do not change canonical Task IDs or workflow contracts.

## Acceptance Criteria

- `qiongli mcp serve --transport stdio` lists literature, provider/config, and
  orchestrator tools from one MCP server.
- `qiongli_literature_search` works through the full CLI MCP and returns
  provider capability, search plan, diagnostics, and results without requiring a
  second MCP server.
- `qiongli install --profile full --target codex --dry-run` reports MCP
  registration as part of the install plan.
- `qiongli install --profile full --target codex` can register the unified MCP
  without overwriting unmanaged user MCP config.
- Docs position marketplace as lite/fallback and CLI full as the complete local
  product.
