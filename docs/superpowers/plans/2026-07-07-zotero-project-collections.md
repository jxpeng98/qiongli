# Zotero Project Collections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put Zotero writes into project-specific collections, with explicit paths first and derived project-title paths as a fallback.

**Architecture:** Keep project-context resolution in the literature MCPB and Zotero mutation in the companion. The MCPB resolves one `collection_path`; the companion resolves or creates nested Zotero collections and adds created, updated, or unchanged items to that collection.

**Tech Stack:** Node ESM, `node:test`, Zotero Desktop bootstrap extension JavaScript, existing MCPB handler tests.

---

## File Structure

- Modify: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
  - Add testable collection path normalization, dry-run reporting, and upsert
    collection membership behavior.
- Modify: `packages/qiongli-zotero-companion/bootstrap.js`
  - Mirror collection behavior in the Zotero runtime adapter.
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
  - Add failing tests for collection dry runs and writes.
- Modify: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
  - Resolve derived collection paths from `project_title`, `research_title`, or
    `topic` when no explicit/default path exists.
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
  - Expose project-title fields in the upsert tool schema.
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
  - Assert derived paths are sent to the companion.
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
  - Assert schema fields exist.
- Modify: `packages/qiongli-zotero-companion/README.md`
  - Document collection write behavior.

## Task 1: Companion Collection Behavior

- [ ] **Step 1: Write failing tests**

Add tests showing that `upsertItems` reports `collection_path` during dry runs,
creates a missing nested collection during writes, and adds duplicate existing
items to the collection.

- [ ] **Step 2: Verify red**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: fail because collection mutation is not implemented.

- [ ] **Step 3: Implement minimal companion helpers**

Add path normalization, `runtime.ensureCollectionPath`, and
`runtime.addItemToCollection` calls inside `upsertItems`.

- [ ] **Step 4: Verify green**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: pass.

## Task 2: Zotero Bootstrap Runtime

- [ ] **Step 1: Write failing VM test**

Extend the bootstrap VM test with a mocked `Zotero.Collection`, collection save,
and item collection assignment.

- [ ] **Step 2: Verify red**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test -- test/bridge.test.mjs
```

Expected: fail until bootstrap collection APIs are implemented.

- [ ] **Step 3: Implement runtime collection APIs**

Add nested collection path listing, creation, and item membership methods to
`createRuntime`.

- [ ] **Step 4: Verify green**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test -- test/bridge.test.mjs
```

Expected: pass.

## Task 3: MCPB Derived Collection Path

- [ ] **Step 1: Write failing MCPB tests**

Add tests for `project_title`, `research_title`, and `topic` schema fields and
for derived `Qiongli/<slug>` payloads.

- [ ] **Step 2: Verify red**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs test/tools.test.mjs
```

Expected: fail because derived paths are not implemented.

- [ ] **Step 3: Implement MCPB path derivation**

Add a small slug helper in `server/zotero/tools.mjs` and use it after explicit
and configured collection paths have been checked.

- [ ] **Step 4: Verify green**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs test/tools.test.mjs
```

Expected: pass.

## Task 4: Documentation And Final Verification

- [ ] **Step 1: Update companion README**

Document explicit collection paths and derived project-title fallback behavior.

- [ ] **Step 2: Run focused verification**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs test/tools.test.mjs
```

Expected: pass.

