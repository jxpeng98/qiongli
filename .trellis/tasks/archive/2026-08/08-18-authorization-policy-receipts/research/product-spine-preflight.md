# Product-spine and editability preflight

## Result

The current App -> CLI -> Plugin/Skills -> MCP product spine is complete for
this task's entry gate. No product-code repair is required before implementing
`GOV-410` through `GOV-412`.

The only failed attempt used a `mise` shim inside a deliberately cleared Host
environment. The shim tried to resolve `latest` over the network and failed DNS.
Rerunning the same isolated tests with the already-installed exact binaries
passed. This is an invocation-environment issue, not a Qiongli product defect.

## Current-source checks

Source under test: `6be44aaad44533d7e4c147efea67bc5b78b8cbc7`.

| Boundary | Evidence |
|---|---|
| App API | 32 tests passed; TypeScript check passed |
| Desktop | 247 tests passed; Svelte check reported 0 errors and 0 warnings |
| Capability Contract v2 | canonical validator passed |
| Plugin/Skill quality | 106 focused tests passed |
| Marketplace artifacts | Codex/Claude artifacts, invocation, bundled MCP startup, and 179/180 desktop Skill file budget passed |
| Workflow variant storage | 43 `qiongli-config` tests passed, including revision-safe replace/reset |
| Receipt-owned content override | focused `qiongli-content` materialization test passed |
| Native CLI | 31 copied-binary and public CLI tests passed |
| Full MCP | 7 stdio tests passed, including empty-runtime-path and Full routing |
| Codex/Claude bundles | 7 non-Host bundle tests passed; 2 real-CLI tests run separately |

## Exact official Host checks

- Codex CLI `0.147.0`: isolated local marketplace registration, Plugin install,
  enablement, cache receipt, customized Skill bytes, 32 Full MCP tools under an
  empty runtime `PATH`, MCP inventory, exact removal, and client absence passed.
- Claude Code `2.1.231`: isolated strict validation, direct Skills discovery,
  local marketplace install, Skill/MCP component inventory, cache receipt,
  customized Skill bytes, 32 Full MCP tools under an empty runtime `PATH`, and
  marketplace removal passed.
- Both tests used private temporary homes/config roots and left the normal Host
  profiles untouched.

## Exact macOS packaged vertical

`pnpm desktop:macos:acceptance -- --diagnostics` produced schema-3 status
`accepted-ad-hoc-nonpublishing` for the exact source above.

- canonical SHA-256:
  `fa6bf38bd316ff0bf48e47bb844345747f8d5ef331964d74e69514a9c8aead1a`
- signed archive SHA-256:
  `b23fe85e97975cd16c1748278c285f42dd1736e8bef30f3aa2775ecbd054960a`
- product-control SHA-256:
  `34651c38dea7c1a0662a63afe7a483d89fb0d980335e8e4ea2910bbb273732e9`
- publication allowed: `false`

All 28 receipt checks are true. The editability-specific evidence is:

- `workflow_variant_edit_reconcile_reset=true`;
- `standalone_skills_all_targets=true`;
- `cli_plugin_reconcile_remove=true`;
- `cli_schema3_app_authority=true`;
- `codex_install_verify_remove=true`;
- `claude_install_verify_remove=true`;
- `skills_materialize_verify_refresh=true`;
- `lite_mcp_self_test=true`;
- `empty_path_startup=true`.

The same accepted App also completed the R5D Zotero automated acceptance with
status `accepted-automated-nonpublishing`.

## Decision

Continue with the governance-only task. Do not modify product code, add another
editability abstraction, or rebuild the package again unless a later change
touches a product/package input.
