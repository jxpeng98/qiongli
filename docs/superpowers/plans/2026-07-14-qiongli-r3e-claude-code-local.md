# Qiongli 2.0 R3E Claude Code Local Execution Plan

Date: 2026-07-14  
Status: complete and accepted at `337cce74`

Branch: `feat/2x-native-alpha1`  
Rolling PR: `#63`  
Design: `docs/superpowers/specs/2026-07-14-qiongli-r3e-claude-code-local-design.md`

## Batch 1 — Canonical Claude metadata

- [x] Add the canonical `.claude-plugin/plugin.json` metadata template.
- [x] Include `.claude-plugin` in deterministic resource collection and fixtures.
- [x] Update the frozen resource-pack lock through the provided Rust lock updater.
- [x] Prove the Marketplace Lite projection exposes both platform templates without
  copying either template into another platform's skill payload.

Gate: focused `qiongli-content` and embedded-pack tests.

## Batch 2 — Native Claude package

- [x] Add `claude_bundle` composition and verification APIs.
- [x] Bind `ClaudeCodeLocal`, artifact, binary, resource pack, manifest, MCP, and
  complete-tree digests in a canonical receipt;
- [x] Generate `${CLAUDE_PLUGIN_ROOT}` MCP commands for Unix and Windows.
- [x] Preserve private staging, locking, no-replace promotion, owner/DACL checks,
  hard-link/reparse rejection, and committed verification;
- [x] Add direct verified removal through a transaction-owned quarantine.

Gate: focused deterministic, tamper, path, permission, Windows cross-check, and
empty-`PATH` MCP tests.

## Batch 3 — Claude local adapter

- [x] Add typed direct skills-directory and managed marketplace discovery.
- [x] Create the fixed `qiongli-local` marketplace catalog over the verified
  package source;
- [x] Implement deterministic `InstallPlan` preview with exact approvals and
  outstanding host action;
- [x] Implement receipt-backed apply, verify, repair, remove, and rollback for the
  Qiongli-owned marketplace catalog;
- [x] Reject adoption, overwrite, cache writes, settings writes, and ambiguous
  recovery.

Gate: focused lifecycle, idempotency, conflict, drift, approval, recovery, and
redaction tests.

## Batch 4 — Truthful command surface

- [x] Add read-only `qiongli install claude status`.
- [x] Report symbolic paths and typed local state without side effects.
- [x] Promote `claude-code-local` to `adapter-engine-ready` only when Batches 1–3
  pass;
- [x] Keep production grant, preview, apply, activation, and release unavailable.

Gate: CLI parsing, JSON schema, redaction, and side-effect tests.

## Batch 5 — Isolated real-client evidence

- [x] Use temporary `HOME` and `CLAUDE_CONFIG_DIR` only.
- [x] Run strict plugin validation and skills-directory discovery.
- [x] Run local marketplace add/install/list/uninstall/remove through Claude Code.
- [x] Verify isolated cache creation and launch the plugin-local MCP with empty
  `PATH`;
- [x] Record only redacted evidence and keep the external test ignored by default.

Gate: explicit real Claude Code acceptance plus the normal native workspace
gate.

## Batch 6 — Acceptance and rolling PR

- [x] Run format, locked workspace check, strict Clippy, all native tests, focused
  Lite compatibility, and Windows MSVC check/Clippy;
- [x] Commit and push cohesive checkpoints on the same branch.
- [x] Monitor Native CI and Cloudflare on the exact implementation head.
- [x] Update the accelerated roadmap, native README, and Draft PR #63 with factual
  capabilities, evidence, next batch, and non-claims.

## Local Acceptance Receipt

- the native pack contains 420 receipt-covered entries, including the canonical
  Claude plugin template, while each platform package excludes the other
  platform's manifest from its skill payload;
- deterministic composition, complete-tree verification, direct verified
  removal, and the marketplace lifecycle reject unmanaged targets, drift,
  links, hard links, permission changes, partial approval, and ambiguous
  recovery;
- all 199 normal native Rust tests, all 69 focused Lite compatibility tests,
  strict host Clippy, and Windows MSVC workspace check/Clippy pass;
- the explicit external test passes with Claude Code `2.1.206`, strict plugin
  validation, `qiongli@skills-dir` discovery, local marketplace add/install,
  cache verification, uninstall/remove, and all 12 Lite MCP tools under an
  empty `PATH`;
- the redacted direct and marketplace package receipt digest is
  `0974c4feccf3c9fd5108639cab9a491e144e5b8082e5e5c60c49fc17410000f9`;
- the external-client test remains ignored in the default workspace gate; and
- no Python or Node product suite is required or run.

## Remote Acceptance Receipt

- design checkpoint: `461c4839`;
- accepted implementation head:
  `337cce741ee2837165331a9a3cd5a53d2e7bf245`;
- Native CI run `29345585219` passed the boundary in 5s, focused Lite in 38s,
  Linux in 2m14s, Windows in 2m30s, and macOS in 2m42s; and
- Cloudflare Pages passed for the same implementation head.

## Explicit Non-claims

R3E does not provide Claude Desktop, cloud/web execution, public marketplace
publication, production grants or installers, managed in-place upgrade, UI,
Full MCP, agent execution, orchestrator execution, or an alpha release.
