# Zotero Reference Notes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional Zotero child-note writes for Qiongli references during local Zotero upsert.

**Architecture:** Keep note normalization in MCPB record mapping and note persistence in the companion runtime. The bridge payload carries `qiongli_notes`; the companion writes child notes only when that array is present and `dry_run: false`.

**Tech Stack:** Node ESM, `node:test`, Zotero Desktop bootstrap extension JavaScript.

---

## File Structure

- Modify: `packages/qiongli-literature-mcpb/server/zotero/records.mjs`
  - Normalize record note fields and map them to `qiongli_notes`.
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
  - Expose note-related input fields in the upsert tool schema.
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
  - Add mapping and upsert payload tests for reading notes.
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
  - Assert schema fields exist.
- Modify: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
  - Add dry-run note planning and child-note write flow.
- Modify: `packages/qiongli-zotero-companion/bootstrap.js`
  - Implement `createChildNote` for Zotero runtime.
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
  - Add unit and VM tests for note writes.
- Modify: `packages/qiongli-zotero-companion/README.md`
  - Document child-note behavior.

## Task 1: Companion Note Writes

- [ ] Write failing tests for dry-run note planning and real child-note creation.
- [ ] Run `npm --prefix packages/qiongli-zotero-companion test` and confirm the note tests fail.
- [ ] Implement `qiongli_notes` handling and `runtime.createChildNote` calls.
- [ ] Re-run companion tests and confirm pass.

## Task 2: Bootstrap Child Notes

- [ ] Extend the bootstrap VM test to verify a Zotero `note` child item is saved
      under the parent reference.
- [ ] Run the companion test file and confirm failure.
- [ ] Implement bootstrap `createChildNote` with `new Zotero.Item("note")`,
      parent item id assignment, note content assignment, and `saveTx`.
- [ ] Re-run companion tests and confirm pass.

## Task 3: MCPB Note Mapping

- [ ] Write failing tests for schema fields and record-to-item note mapping.
- [ ] Run `npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs test/tools.test.mjs`.
- [ ] Implement note normalization in `records.mjs` and schema additions.
- [ ] Re-run MCPB focused tests and confirm pass.

## Task 4: Final Verification

- [ ] Run companion tests.
- [ ] Run focused MCPB tests.
- [ ] Run Zotero companion artifact tests.
- [ ] Run `git diff --check`.

