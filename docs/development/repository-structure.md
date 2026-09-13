# Repository structure

`main` contains the integrated native product. `2.x` remains the development and
prerelease branch. The old Python runtime is retained for compatibility work;
new CLI and MCP behaviour belongs in the Rust workspace.

| Path | Owner |
|---|---|
| `content/workflow/` | Main Skill, shortcuts and stage references |
| `content/skills/`, `roles/`, `templates/`, `subjects/` | Research guidance and reusable outputs, all under `content/` |
| `content/standards/` | Research task and artifact contracts |
| `content/mcp-contracts/` | Public MCP tools, schemas and fixtures |
| `content/distribution/` | Shared Plugin metadata |
| `packages/qiongli-native/apps/qiongli/` | Native product entry and CLI/Host adapters |
| `packages/qiongli-native/crates/` | Shared domain, project, runtime and installation services |
| `packages/qiongli-zotero-companion/` | Extension installed inside Zotero |
| `tooling/scripts/native_*.py` | Native release, registry and Marketplace packaging |
| `tooling/release/` | Release contracts and evidence |
| `docs/`, `docs/zh/` | English and Chinese documentation |
| `docs/architecture/decisions/` | Accepted and superseded architecture decisions |
| `docs/superpowers/` | Plans, roadmap and evidence ledger |
| `tests/`, `evals/` | Regression and behavioural checks |

## Retained sources

`packages/qiongli-desktop/` and `packages/qiongli-app-api/` support the retained
Svelte/Tauri Desktop. Desktop maintenance is separate from the current CLI delivery.
`packages/python-qiongli/`, `packages/npm-qiongli/`, `packages/qiongli-lite-mcp/`
and the older MCPB sources preserve compatibility; they are not the native registry
package builders. In particular, their old READMEs are not the README shipped in
current native npm or Python packages.

## Generated outputs

Plugin trees, `qiongli-workflow/`, `.agent/`, package payloads and installed caches
are derived from canonical sources. Keep them out of source edits. Native packaging
builds in a staging directory and uses common CLI metadata while retaining a
channel-specific wrapper and installation instructions.

Root `scripts/` keeps stable command wrappers; edit `tooling/scripts/` for their
implementation. Legacy materialization details remain in the explicitly labelled
[compatibility guide](distribution-materialization.md).
See [editing conventions](../conventions.md) before adding a new owner.
