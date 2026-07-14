# Qiongli 2.0 R3D Codex Native Plugin Execution Plan

Date: 2026-07-14  
Status: implementation and local acceptance complete; remote acceptance pending
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
- [x] Commit and push design checkpoint `e2e5c814` to rolling Draft PR `#63`.

## 2. Implement Native Plugin Composition

- [x] Add typed plugin-bundle target, receipt, entry, error, compose, and
  verify APIs to `qiongli-platform`.
- [x] Generate the Codex manifest and platform-specific native Lite MCP
  declaration deterministically.
- [x] Project Marketplace Lite content below
  `skills/qiongli-workflow/` without a tracked generated mirror.
- [x] Copy only the grant-matched target-native Qiongli executable.
- [x] Add private staging, target lock, no-replace promotion, and committed
  verification on Unix and Windows.

## 3. Migrate The R3C Source Boundary

- [x] Require a verified R3D plugin-bundle receipt during Codex discovery.
- [x] Bind registration state to the plugin-bundle receipt and package root.
- [x] Refactor R3C fixtures to compose complete native plugin sources.
- [x] Keep registration, cache ownership, enablement, and non-claim semantics
  unchanged.

## 4. Prove Structure And Runtime

- [x] Test deterministic layout, metadata, MCP command, modes, and digests.
- [x] Test grant, binary, pack, target, collision, oversize, extra-file, link,
  hard-link, mode, receipt, and content tampering failures.
- [x] Test no unmanaged overwrite and target-lock contention.
- [x] Launch the packaged executable with an empty `PATH` and prove MCP
  `initialize` plus the exact Lite `tools/list` response.
- [x] Cross-check Windows-only production and test code locally before push.

## 5. Obtain Real Clean-client Evidence

- [x] Validate the generated root with Plugin Creator.
- [x] Create an isolated home, `CODEX_HOME`, and personal marketplace.
- [x] Install with the actual Codex CLI and confirm list, cache, and enablement
  evidence in that isolated environment.
- [x] Launch the cached MCP command with an empty `PATH` and record a redacted
  acceptance receipt.
- [x] Remove the isolated fixture without touching normal user state.

## 6. Verify And Land The Batch

- [x] Run format, focused Rust tests, strict Clippy, and the native workspace
  test gate.
- [x] Update the accelerated roadmap and native README with exact local facts,
  limitations, tests, and evidence.
- [ ] Update rolling Draft PR `#63` with the implementation checkpoint and
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

## Local Acceptance Receipt

- the normal package suite composes the complete embedded Marketplace Lite
  projection, verifies deterministic metadata and receipts, and rejects an
  invalid target name, existing data, lock contention, signed-binary drift,
  extra files, links, wrong modes, hard links, receipt corruption, content
  drift, and binary tampering;
- the packaged native executable completes MCP `initialize` and exact
  `tools/list` with an empty `PATH`; all 12 canonical Lite tools are present;
- Plugin Creator validates the generated package;
- Codex CLI `0.144.1` installs and lists the plugin from an isolated personal
  marketplace, writes enablement only below the isolated `CODEX_HOME`, copies
  the exact receipt-covered tree into its cache, and launches the cached MCP
  with an empty `PATH`;
- the redacted test-bundle receipt digest is
  `5c6a6b442bde758864224f11f58656f6821f10f028fcc97c5cade6335b33aeab`;
- format, locked workspace check, strict host and Windows MSVC Clippy, all 187
  normal native Rust tests, and all 69 focused Lite compatibility tests pass;
- the external-client test is ignored in the default workspace gate and passed
  explicitly with the current Plugin Creator validator and Codex CLI; and
- no Python or Node product suite was run or required. Python was used only to
  execute Plugin Creator's development-time structural validator, never by the
  generated plugin or cached MCP runtime.
