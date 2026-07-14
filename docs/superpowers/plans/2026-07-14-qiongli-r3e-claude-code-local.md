# Qiongli 2.0 R3E Claude Code Local Execution Plan

Date: 2026-07-14  
Status: active  
Branch: `feat/2x-native-alpha1`  
Rolling PR: `#63`  
Design: `docs/superpowers/specs/2026-07-14-qiongli-r3e-claude-code-local-design.md`

## Batch 1 — Canonical Claude metadata

- add the canonical `.claude-plugin/plugin.json` metadata template;
- include `.claude-plugin` in deterministic resource collection and fixtures;
- update the frozen resource-pack lock through the provided Rust lock updater;
- prove the Marketplace Lite projection exposes both platform templates without
  copying either template into another platform's skill payload.

Gate: focused `qiongli-content` and embedded-pack tests.

## Batch 2 — Native Claude package

- add `claude_bundle` composition and verification APIs;
- bind `ClaudeCodeLocal`, artifact, binary, resource pack, manifest, MCP, and
  complete-tree digests in a canonical receipt;
- generate `${CLAUDE_PLUGIN_ROOT}` MCP commands for Unix and Windows;
- preserve private staging, locking, no-replace promotion, owner/DACL checks,
  hard-link/reparse rejection, and committed verification;
- add direct verified removal through a transaction-owned quarantine.

Gate: focused deterministic, tamper, path, permission, Windows cross-check, and
empty-`PATH` MCP tests.

## Batch 3 — Claude local adapter

- add typed direct skills-directory and managed marketplace discovery;
- create the fixed `qiongli-local` marketplace catalog over the verified
  package source;
- implement deterministic `InstallPlan` preview with exact approvals and
  outstanding host action;
- implement receipt-backed apply, verify, repair, remove, and rollback for the
  Qiongli-owned marketplace catalog;
- reject adoption, overwrite, cache writes, settings writes, and ambiguous
  recovery.

Gate: focused lifecycle, idempotency, conflict, drift, approval, recovery, and
redaction tests.

## Batch 4 — Truthful command surface

- add read-only `qiongli install claude status`;
- report symbolic paths and typed local state without side effects;
- promote `claude-code-local` to `adapter-engine-ready` only when Batches 1–3
  pass;
- keep production grant, preview, apply, activation, and release unavailable.

Gate: CLI parsing, JSON schema, redaction, and side-effect tests.

## Batch 5 — Isolated real-client evidence

- use temporary `HOME` and `CLAUDE_CONFIG_DIR` only;
- run strict plugin validation and skills-directory discovery;
- run local marketplace add/install/list/uninstall/remove through Claude Code;
- verify isolated cache creation and launch the plugin-local MCP with empty
  `PATH`;
- record only redacted evidence and keep the external test ignored by default.

Gate: explicit real Claude Code acceptance plus the normal native workspace
gate.

## Batch 6 — Acceptance and rolling PR

- run format, locked workspace check, strict Clippy, all native tests, focused
  Lite compatibility, and Windows MSVC check/Clippy;
- commit and push cohesive checkpoints on the same branch;
- monitor Native CI and Cloudflare on each exact accepted head;
- update the accelerated roadmap, native README, and Draft PR #63 with factual
  capabilities, evidence, next batch, and non-claims.

R3E closes only after exact-head CI and the explicit real-client evidence pass.
