# Qiongli 2.0 R3D Codex Native Plugin Execution Plan

Date: 2026-07-14  
Status: active  
Branch: `feat/2x-native-alpha1`  
Rolling PR: `#63`

## Goal

Build and prove one dependency-free, target-native Codex plugin package without
writing the developer's normal Codex cache or enablement state.

## 1. Freeze The Boundary

- [x] Confirm the current Codex manifest, skills, MCP, personal marketplace,
  cache, and enablement contracts.
- [x] Confirm Plugin Creator's canonical `skills/` and `.mcp.json` validation.
- [x] Freeze package layout, grant binding, receipt, transaction, client-owned
  activation, and non-claim rules in the R3D design.
- [ ] Commit and push the design checkpoint to rolling Draft PR `#63`.

## 2. Implement Native Plugin Composition

- [ ] Add typed plugin-bundle target, receipt, entry, error, compose, and
  verify APIs to `qiongli-platform`.
- [ ] Generate the Codex manifest and platform-specific native Lite MCP
  declaration deterministically.
- [ ] Project Marketplace Lite content below
  `skills/qiongli-workflow/` without a tracked generated mirror.
- [ ] Copy only the grant-matched target-native Qiongli executable.
- [ ] Add private staging, target lock, no-replace promotion, and committed
  verification on Unix and Windows.

## 3. Migrate The R3C Source Boundary

- [ ] Require a verified R3D plugin-bundle receipt during Codex discovery.
- [ ] Bind registration state to the plugin-bundle receipt and package root.
- [ ] Refactor R3C fixtures to compose complete native plugin sources.
- [ ] Keep registration, cache ownership, enablement, and non-claim semantics
  unchanged.

## 4. Prove Structure And Runtime

- [ ] Test deterministic layout, metadata, MCP command, modes, and digests.
- [ ] Test grant, binary, pack, target, collision, oversize, extra-file, link,
  hard-link, mode, receipt, and content tampering failures.
- [ ] Test no unmanaged overwrite and target-lock contention.
- [ ] Launch the packaged executable with an empty `PATH` and prove MCP
  `initialize` plus the exact Lite `tools/list` response.
- [ ] Cross-check Windows-only production and test code locally before push.

## 5. Obtain Real Clean-client Evidence

- [ ] Validate the generated root with Plugin Creator.
- [ ] Create an isolated home, `CODEX_HOME`, and personal marketplace.
- [ ] Install with the actual Codex CLI and confirm list, cache, and enablement
  evidence in that isolated environment.
- [ ] Launch the cached MCP command with an empty `PATH` and record a redacted
  acceptance receipt.
- [ ] Remove the isolated fixture without touching normal user state.

## 6. Verify And Land The Batch

- [ ] Run format, focused Rust tests, strict Clippy, and the native workspace
  test gate.
- [ ] Update the accelerated roadmap, native README, and rolling Draft PR with
  exact facts, limitations, tests, and head SHA.
- [ ] Commit and push the implementation/local-evidence checkpoint.
- [ ] Accept R3D only after exact-head Native CI and Cloudflare Pages are green.

## Explicit Non-goals

- public Codex Marketplace submission or publication;
- direct writes to the developer's normal Codex cache or enablement state;
- Codex cloud or ChatGPT web access to a local executable;
- production grant issuance, release signing, notarization, or alpha release;
- package upgrade, repair, removal, or rollback;
- Claude Code, Claude Desktop, Antigravity, Hermes, UI, updater, Full MCP,
  agent execution, or orchestrator execution completion;
- Python or Node compatibility-suite execution.

