# Product spine preflight — 2026-08-18

## Verdict

The current App -> CLI -> Plugin/Skills -> Lite/Full MCP path is complete for
the requested local product boundary. No product-code repair was required
before GOV-408/GOV-409.

## Current-source evidence

- App API Rust fixture -> Zod decoder: 32 tests passed.
- Desktop editor and state/UI contracts: 247 tests passed.
- Workflow variant/config/content materialization: 89 Rust tests passed,
  including revision-safe replace/reset and exact receipt-managed overrides.
- Codex/Claude bundle tests: 7 passed, 2 real-client tests intentionally
  ignored in the ordinary suite, 0 failed.
- Capability Contract v2 validator passed.
- Plugin/Skill/MCP cross-surface contract suite: 45 tests passed.

## Isolated official Host evidence

- Codex CLI 0.147.0: local Plugin install/enable/list, cache receipt, customized
  Skill bytes, enabled MCP inventory, Full MCP startup with empty `PATH`, remove,
  and absence verification passed in a temporary home.
- Claude Code 2.1.231: strict Plugin validation, Skills-directory discovery,
  marketplace install, Skill/MCP inventory, cache receipt, customized Skill
  bytes, Full MCP startup with empty `PATH`, uninstall, and marketplace removal
  passed in a temporary home.
- Neither test used the normal Codex or Claude profile or invoked a model.

## Exact macOS package evidence

- Product source: `6c5bf2136e1dd2745ca4f4c71d660bc93bac2e0d`.
- Receipt: `dist/macos-acceptance/current/qiongli-packaged-product-acceptance.receipt.json`.
- Status: `accepted-ad-hoc-nonpublishing`; `publication_allowed=false`.
- Canonical binary SHA-256:
  `f49c62a563df23290fb28688368e8185cb1e8c0e677912baa442125cf6209306`.
- Signed archive SHA-256:
  `b9be476f9c8892450e10c48903da5203c2c809b832f229d937a510b09a9c0c3a`.
- Receipt checks include:
  `workflow_variant_edit_reconcile_reset`,
  `standalone_skills_all_targets`,
  `cli_plugin_reconcile_remove`,
  `codex_install_verify_remove`,
  `claude_install_verify_remove`,
  `lite_mcp_self_test`, and
  `project_app_cli_library_full_mcp_parity`.
- The edit journey proved update/repair-required before reconciliation,
  receipt/cache-bound Customized Ready afterward, reset-induced staleness, and
  explicit reconciliation back to Canonical Ready.

This receipt is local acceptance evidence only. It grants no publication or
release authority.
