# Packaged content chain audit

Audit baseline: `origin/2.x` at
`237de9ba9e235f2b5067cc9704aef49eee3ce9c6`, 2026-08-18.

| Surface | Current outcome | Evidence owner | Disposition |
| --- | --- | --- | --- |
| Packaged native CLI | Exact packaged bytes can be installed, PATH-configured, tested from a fresh shell, verified, updated, and removed through digest-bound plans. | `desktop.rs`, `command.rs`, `native_packaged_product_acceptance.rs` | Complete; preserve. |
| Official Host CLI execution | One confirmation binds fixed Codex/Claude argv, uses no shell, stops on failure, then discards old observations. | `desktop.rs`, ADR 0213, archived activation task | Complete; preserve. |
| Codex Plugin | Receipt-owned source is installed through official Codex CLI; exact version/source/cache and Full MCP are freshly checked. Exact bundle identity includes Skill bytes, but does not prove live Skill invocation. | `codex_bundle.rs`, `codex_plugin_bundle.rs`, Host probes | Complete for canonical content. |
| Claude Plugin | Receipt-owned source is installed through official Claude CLI; exact version/scope/cache, one `qiongli-workflow` Skill, and one Full MCP component are freshly checked. | `claude_bundle.rs`, `claude_plugin_bundle.rs`, Host probes | Complete for canonical content. |
| Standalone Skills | Qiongli-managed, current-project, and selected custom-folder targets support preview/apply, verify, update, detach, and exact remove. | `qiongli-content`, `desktop.rs`, packaged acceptance | Complete for canonical content. |
| Lite/Full MCP | Plugin-local packaged binary launches with an empty PATH; public tool inventories and Host attachment are checked independently. | `mcp.rs`, runtime tests, bundle/packaged acceptance | Complete; keep immutable under customization. |
| Plugin/Skill quality | Evaluation Truth and the executable 12-case academic-quality suite are integrated; strict Skill audit and mutation evidence are owned by PR #124. | Evaluation Truth run `31984053266`, Native CI `31984053292` | Complete at merged source; no model-quality claim. |
| App content preview | The App previews `workflow/SKILL.md` and target manifests, and can edit per-project local guidance. | `WorkflowContentPanel.svelte`, App API v17 | Partial. |
| Effective Plugin/Skill editing | Editing installed trees causes drift; no receipt-bound user variant feeds standalone Skills and Plugin composition/cache verification. | materialization and bundle verifiers | Missing; current task. |

## Evidence limits

- PR #124's merge-source CI proves cross-platform source and packaged-control
  behavior, not public release authorization.
- The archived activation task records isolated real Codex and Claude tests at
  source `fdfd5323`. The current product change must rerun them.
- `dist/macos-acceptance/current` is also from `fdfd5323`; it is not evidence
  for `237de9ba` or future edited source.
- `dist/local-2x-237de9ba/Qiongli.app` is useful manual UI evidence but has no
  packaged-product install grants.

## Root cause

The UI term “customize” currently combines two different contracts:

1. verified Plugin/Skill source preview (read-only); and
2. editable project-local guidance (writeable).

No native owner represents a user-authored, derived workflow-content identity.
Consequently, any direct edit is correctly treated as drift and cannot pass
managed/cache receipt comparison. The fix must introduce one contained derived
identity, not weaken drift checks or write Host caches.
