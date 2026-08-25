# REL-902 persisted state migration and rollback

## Goal

Prove that the current Qiongli 2 native runtime can migrate supported project
and global provider state from both supported predecessor releases, then return
to a predecessor-usable state without data loss or dependence on Python/Node.

## Background

- `REL-901` froze current plus two predecessor versions as the persisted-state
  support window and assigned the executable migration/rollback proof here.
- The two published predecessor releases are `v1.19.0-beta.1` at
  `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f` and `v1.18.0-beta.3` at
  `12aea420bff9a3fbfa5e421c482ae8da2588c9ed`.
- Both predecessors use the 1.x project layout, including
  `context/research_state.md` and `.qiongli/guidance_manifest.yaml`.
- `ProjectStateService` already owns copy-on-migrate and receipt-owned rollback.
  `qiongli-config` plus `legacy_migration_cli` already own legacy provider
  conversion, secret indirection, verification, and exact rollback on failure.

## Requirements

1. Bind the N-1 and N-2 fixtures to the exact predecessor tag and commit
   identities above; do not derive acceptance from an unlabelled synthetic
   legacy directory.
2. For each predecessor, exercise the existing native project migration owner,
   verify the migrated canonical project, then exercise receipt-owned rollback.
3. For each predecessor, exercise the existing native legacy provider-state
   conversion, including secret-store indirection, then exercise its exact
   rollback path.
4. Hash or byte-compare predecessor project and global-state inputs before and
   after the complete journey. Rollback must remove only migration-owned
   current state and must not rewrite predecessor bytes.
5. Keep all execution isolated under test-owned temporary roots. The proof must
   not inspect or mutate the developer's real home, Host profiles, or projects.
6. Run without a Python/Node product runtime. Python may continue to run the
   existing repository governance checks only.
7. Preserve the frozen App IPC, MCP, CLI JSON, and current persisted-state
   identities. This task adds proof, not a speculative schema bump.

## Acceptance Criteria

- [ ] A checked-in fixture manifest names exactly `v1.19.0-beta.1` as N-1 and
      `v1.18.0-beta.3` as N-2 with their exact peeled commits.
- [ ] One focused native test runs the same project and global-state
      migrate/verify/rollback journey for both predecessor rows.
- [ ] Each migrated project is registered and readable before rollback; after
      rollback its migration-owned destination and registration are absent.
- [ ] Each migrated provider configuration is valid current global state with
      plaintext secrets replaced by secret references before rollback; after
      rollback current provider settings and newly created secret entries equal
      their pre-migration state.
- [ ] The original project tree and legacy `providers.json` are byte-identical
      after successful migration and rollback for both predecessors.
- [ ] Focused Rust formatting and test commands pass, followed by the task-scope
      Slice checks and exact-head Native CI on Linux, macOS, and Windows.
- [ ] Acceptance evidence records the exact product commit and CI run before
      `REL-902` becomes `accepted` in Program Ledger v1.

## Out of Scope

- Forward-version rejection and immutability, owned by `REL-903`.
- Interrupted migration, missing index, corrupt derived state, lost
  registration, and partial-update disaster recovery, owned by `REL-904`.
- New persisted-state schemas, new migration frameworks, automatic legacy-home
  discovery, legacy runtime execution, package publication, or release
  authorization.
