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
- [x] Commit and push design checkpoint `8c76f5f2` to the rolling Draft PR.

## 2. Make The Embedded Lite Projection A Codex Source

- [x] Add a skills-only `.codex-plugin/plugin.json` to canonical content.
- [x] Include and validate it in every applicable resource-pack projection.
- [x] Regenerate the checked resource-pack lock and update deterministic
  fixture expectations.
- [x] Prove the `marketplace-lite` materialization contains and receipts the
  manifest without adding a Python or Node launch path.

## 3. Implement Discovery And Preview

- [x] Add typed redacted current-user Codex discovery.
- [x] Validate the fixed R3B materialized source and Codex manifest.
- [x] Parse bounded personal marketplace documents and classify absent,
  registered, conflict, drift, and recovery states.
- [x] Build a deterministic one-operation `RegisterPluginSource` install plan
  with exact approvals, inverse, source digest, and outstanding host action.
- [x] Add pure document-merge tests and plan tamper/approval tests.

## 4. Implement Receipt-backed Registration Lifecycle

- [x] Add canonical registration, lifecycle, state, and journal schemas.
- [x] Add private-root locking, compare-and-swap marketplace writes, rollback,
  and post-commit verification on Unix and Windows.
- [x] Implement apply, verify, repair, remove, and rollback.
- [x] Reject source, marketplace, receipt, ownership, path, permission,
  duplicate-entry, and recovery drift without deleting user data.
- [x] Add idempotence, preservation, drift, rollback, and fault tests.

## 5. Expose Conservative Product Status

- [x] Add side-effect-free `qiongli install codex status` output with symbolic
  locations only.
- [x] Change the Codex install target from `contract-only` only to an accurate
  adapter-engine state; keep production `launch_grant`, `preview`, and `apply`
  unavailable in the source build.
- [x] Update native README and accelerated roadmap with exact capabilities and
  non-claims.

## 6. Verify And Land The Batch

- [x] Run formatting and the focused `qiongli-platform`, `qiongli-content`, and
  CLI Rust tests.
- [x] Run all 185 native Rust workspace tests locally.
- [x] Run the native boundary and all 69 focused Lite compatibility tests.
- [x] Cross-check all workspace targets for Windows MSVC locally.
- [ ] Obtain the real Windows CI result.
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

Local implementation receipt, before checkpoint commit:

- official Codex manual cache was current on 2026-07-14 and confirmed the
  personal marketplace, local source, client-owned cache, and enablement
  boundaries used by the design;
- the resource pack now has 419 entries with content root
  `db362981aa9c9ad1f84eeecbdb4f119924c7f442096be3a8e657afa424aea2f2`
  and pack digest
  `04a0ac3a980194c741ba57f0cc8da8490bb1c77c9d445f6f2144bf5464b3818d`;
- seven focused Codex adapter tests pass, including post-activation rollback
  and ambiguous concurrent-state preservation;
- strict workspace Clippy passes;
- all 185 native Rust tests pass;
- the native change boundary and all 69 focused Lite compatibility tests pass;
  and
- the Windows MSVC all-target, all-feature workspace check passes.

Implementation commit, push, and exact-head Native CI/Cloudflare evidence remain
pending.
