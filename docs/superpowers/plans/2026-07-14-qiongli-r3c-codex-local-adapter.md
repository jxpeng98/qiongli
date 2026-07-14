# Qiongli 2.0 R3C Codex Local Adapter Execution Plan

Date: 2026-07-14  
Status: in progress  
Branch: `feat/2x-native-alpha1`  
Rolling PR: `#63`

## Goal

Complete the first receipt-backed Codex current-user adapter vertical without
writing Codex's plugin cache or claiming client activation.

## 1. Freeze The Boundary

- [x] Verify the current official personal-marketplace, local-source, cache,
  and enablement contracts.
- [x] Fix the v1 current-user marketplace, private state, and plugin-source
  symbolic paths.
- [x] Freeze discovery, plan, approval, receipt, merge, transaction, and
  non-claim rules in the R3C design.
- [ ] Commit and push the design checkpoint to the rolling Draft PR.

## 2. Make The Embedded Lite Projection A Codex Source

- [ ] Add a skills-only `.codex-plugin/plugin.json` to canonical content.
- [ ] Include and validate it in every applicable resource-pack projection.
- [ ] Regenerate the checked resource-pack lock and update deterministic
  fixture expectations.
- [ ] Prove the `marketplace-lite` materialization contains and receipts the
  manifest without adding a Python or Node launch path.

## 3. Implement Discovery And Preview

- [ ] Add typed redacted current-user Codex discovery.
- [ ] Validate the fixed R3B materialized source and Codex manifest.
- [ ] Parse bounded personal marketplace documents and classify absent,
  registered, conflict, drift, and recovery states.
- [ ] Build a deterministic one-operation `RegisterPluginSource` install plan
  with exact approvals, inverse, source digest, and outstanding host action.
- [ ] Add pure document-merge tests and plan tamper/approval tests.

## 4. Implement Receipt-backed Registration Lifecycle

- [ ] Add canonical registration, lifecycle, state, and journal schemas.
- [ ] Add private-root locking, compare-and-swap marketplace writes, rollback,
  and post-commit verification on Unix and Windows.
- [ ] Implement apply, verify, repair, remove, and rollback.
- [ ] Reject source, marketplace, receipt, ownership, path, permission,
  duplicate-entry, and recovery drift without deleting user data.
- [ ] Add idempotence, preservation, drift, rollback, and fault tests.

## 5. Expose Conservative Product Status

- [ ] Add side-effect-free `qiongli install codex status` output with symbolic
  locations only.
- [ ] Change the Codex install target from `contract-only` only to an accurate
  adapter-engine state; keep production `launch_grant`, `preview`, and `apply`
  unavailable in the source build.
- [ ] Update native README and accelerated roadmap with exact capabilities and
  non-claims.

## 6. Verify And Land The Batch

- [ ] Run formatting and the focused `qiongli-platform`, `qiongli-content`, and
  CLI Rust tests.
- [ ] Run the full native Rust workspace tests locally.
- [ ] Run the native boundary and focused Lite gates.
- [ ] Cross-check the Windows target and obtain the real Windows CI result.
- [ ] Commit and push the implementation and local receipt checkpoints.
- [ ] Update Draft PR `#63` with factual files, tests, limitations, and the
  exact head.
- [ ] Accept R3C only after the exact implementation-and-receipt head is green
  on Native CI and Cloudflare Pages.

## Explicit Non-goals

- direct writes to `~/.codex/plugins/cache` or Codex enablement state;
- Desktop install/enable automation or activation claims;
- direct Codex MCP config mutation;
- public Marketplace submission or publication;
- repository-scoped Codex marketplaces;
- Claude Code or Claude Desktop adapters;
- plugin packaging with the native executable;
- UI, updater, release, or clean-machine artifact claims.

## Receipt

Pending implementation and exact-head CI evidence.
