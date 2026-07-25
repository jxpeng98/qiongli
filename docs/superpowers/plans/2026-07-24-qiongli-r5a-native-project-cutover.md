# Qiongli R5A Native Project Cutover Plan

Status: complete; source implementation and isolated macOS project-data manual
acceptance passed

Date: July 24, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: R4 replacement migration commit `0638262c`

Roadmap:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

## Goal

Complete the project-data part of the Qiongli 1.x to 2.x cutover without
reviving any 1.x runtime:

- copy only a user-selected legacy project into a new 2.x destination;
- preserve the source as the rollback boundary;
- register the verified destination in the native Research Library;
- rebuild derived graph/search state deterministically;
- resume safely after interruption; and
- make the same preview, approval, result, and recovery semantics available to
  the CLI and packaged macOS App.

R5A begins after the global client/plugin/provider replacement transaction in
R4M. It does not scan the user's home directory and does not treat a 1.x
installation as an active compatibility mode.

## Product sequence

```text
Select legacy project + empty destination
  -> preview bounded copy and exclusions
  -> approve exact digest
  -> commit verified 2.x files and migration receipt
  -> register the destination
  -> write the registration completion marker
  -> rebuild and verify derived indexes
  -> reopen after restart
```

If interruption occurs after the file commit, the destination receipt is the
recovery authority. Recovery verifies the exact source, destination, manifest,
inventory, project identity, original plan digest, and Research Library
revision boundary before completing registration. It never copies the files a
second time.

## Batches

### A1 — Durable copy-to-registration recovery

Status: implemented in source

- extend the migration receipt with source inventory, manifest, symbolic path,
  and expected-library-revision digests;
- reconstruct a verified recovery plan from an already committed destination;
- expose `qiongli project migrate recover preview|apply`;
- require the original migration plan digest and explicit filesystem approval;
- preserve tombstone and recovery-floor protection so an intentional
  unregister cannot be silently undone; and
- prove restart reconstruction and idempotent replay without retaining the
  original in-memory plan.

Acceptance:

- process loss between file commit and Library registration is recoverable;
- source or destination drift blocks recovery;
- wrong digest or missing approval performs no registration;
- a successful replay does not duplicate the project; and
- the source remains unchanged.

### A2 — Packaged macOS migration experience and derived-state qualification

Status: implemented and accepted in the isolated macOS App

- add an opaque source/destination folder selection flow to the Research
  Library;
- display copy, exclusion, source-retention, and recovery consequences before
  confirmation;
- keep a visible Resume migration action after restart without scanning the
  user's home directory;
- rebuild the Academic Graph and search index twice from the migrated
  canonical state and require identical projection identities;
- reopen the project from the packaged App and copied CLI in an isolated home;
  and
- keep manual host activation separate from project-data acceptance.

Acceptance:

- the packaged macOS App completes preview, apply, restart, and recovery;
- no absolute private path crosses the App API;
- the migrated project and its source-backed graph survive restart; and
- the derived index can be deleted and rebuilt without changing canonical
  academic state.

### A3 — Reconciliation and rollback closure

Status: implemented and accepted against disposable macOS fixtures

- report per-artifact reconciliation for research state, decisions, evidence,
  captures, semantic links, and continuity gaps;
- add a receipt-owned rollback preview that unregisters the destination and
  removes only the exact migration-owned destination;
- refuse rollback after destination academic drift unless the user exports or
  explicitly resolves the changed project;
- add Doctor repair guidance for incomplete marker/index states; and
- document the final 1.x project support boundary.

Acceptance:

- failure never deletes or mutates the legacy source;
- rollback cannot remove an unrelated or changed directory;
- incomplete and conflicting states produce an item-scoped recovery action;
  and
- the product no longer needs a Python or Node runtime to read or migrate a
  supported project.

## Current implementation boundary

Batches A1 through A3 now cover the native service, CLI, strict App API v4,
opaque Desktop folder selection, Svelte migration/recovery/rollback controls,
per-artifact reconciliation, structured graph qualification, Doctor guidance,
restart reconstruction tests, and deterministic frontend fixtures. Rollback
revalidates the source, exact receipt-owned destination, Library revision,
registration state, marker, manifest, and every migrated artifact immediately
before unregistering and removing the destination. A changed or unrelated
destination is never removed.

The release-profile, ad-hoc-signed macOS App completed the project-data manual
interaction receipt in an isolated home. The acceptance migrated six academic
files, restarted through a simulated missing registration marker without
copying again, rolled back one exact unchanged destination, proved all 1.x
source digests unchanged, and blocked a second rollback after one 2.x academic
artifact changed. The blocked preview disabled confirmation and retained both
the changed destination and legacy source.

The interaction receipt is local, non-publishing evidence at
`dist/macos-r5a-manual/current/r5a-project-manual-acceptance.receipt.json`.
Because the worktree is intentionally still uncommitted, the broader
product-controlled client-install acceptance script was not run: it correctly
requires a clean commit identity. That client-install authority gate is
separate from R5A project-data mutation authority. Windows/Linux interactive
packaging, Homebrew/Scoop/WinGet publication, and Beta promotion remain
unclaimed.

The validation scope is focused Rust formatting, compile, unit, parser,
project-service, App API, Svelte component, type, and production-build tests.
No formal cybersecurity scan is added.

## Manual acceptance result

1. normal migration copied 6 files / 1,190 bytes and produced identical graph
   identities across two rebuilds;
2. restart recovery recreated a missing registration marker and explicitly
   reported that files were not copied again;
3. exact rollback reconciled 6 matched artifacts, 0 drifted artifacts, and 0
   continuity gaps before deleting only the migrated destination;
4. the original source remained byte-identical after migration, recovery, and
   rollback;
5. a changed `context/research_state.md` produced 5 matches and 1 drifted item,
   disabled confirmation, retained the destination, and directed the user to
   export or explicitly resolve it; and
6. Doctor reported the retained drift fixture as academic revision drift while
   keeping its migration receipt and registration marker ready.

R5A can now close. R5B starts with cross-project graph/search continuity and
packaged distribution work. It must not add a Qiongli 1.x runtime or automatic
home-directory project discovery.

## R5A completion gate

R5A is complete when:

1. normal migration and interrupted-registration recovery share one canonical
   receipt and Library identity;
2. CLI and packaged macOS App expose the same digest-bound operation;
3. source, destination, Library, and graph drift are detected rather than
   overwritten;
4. restart and idempotent replay pass in an isolated home;
5. the source remains the usable rollback copy until the user separately
   retires it; and
6. project cutover runs without invoking Qiongli 1.x, Python, Node, a model
   provider, Codex, or Claude Code.
