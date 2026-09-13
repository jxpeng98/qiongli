# PILOT-905 implementation plan

## 1. Freeze inputs

- [x] Record the SHA-256 and exact claims of the August 24 compatibility receipt
      and PILOT-903 machine receipt.
- [x] Confirm no later product/package input commit invalidates the PILOT-903
      product-source identity used by the matrix.

## 2. Publish the matrix

- [x] Add the canonical JSON receipt with the closed Host/capability/status
      inventories and source-bound evidence records.
- [x] Add matching English and Chinese capability pages.
- [x] Link each page from its documentation landing page.

## 3. Add the minimum guard

- [x] Add one dependency-free focused unittest for matrix closure, evidence
      bindings, privacy-safe values, and docs projection.
- [x] Run only that focused test while iterating.

## 4. Accept the Slice

- [x] Add a concise PILOT-905 acceptance record.
- [x] Set PILOT-905 to `accepted` in Program Ledger v1 and regenerate its index.
- [x] Run the focused test, docs build, roadmap freshness check, task validator,
      and `git diff --check`.
- [x] Commit, open a PR, wait for required Slice CI, merge, archive the Trellis
      task, and record the session.

## Risk and rollback points

- A positive cell without direct evidence is a false product claim; the focused
  test must fail it closed.
- Source identities must remain per receipt; never replace them with the matrix
  commit or current branch head.
- If docs and JSON diverge, fix the docs projection rather than adding a
  generator.
- Rollback is a single evidence/docs/test/ledger revert; no product or Host
  state is mutated.
