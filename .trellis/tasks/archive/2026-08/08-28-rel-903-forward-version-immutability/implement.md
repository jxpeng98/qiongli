# REL-903 implementation plan

1. Add one `rel_903` test beside the REL-902 persisted-state test in
   `packages/qiongli-native/apps/qiongli/src/legacy_migration_cli.rs`.
2. Generate current global/project state through `GlobalSettingsStore` and
   `ProjectStateService`; retain exact supported bytes.
3. Mutate only the schema version for global settings, the research-library
   index, and the portable project manifest; assert each normal owner fails
   closed and leaves exact bytes unchanged.
4. Keep production code unchanged unless the test exposes a shared-owner bug;
   do not add fixtures, registries, migration helpers, or downgrade behavior.
5. Add the seven-section REL-903 executable scenario to
   `.trellis/spec/product/control/public-schema-authority.md`.
6. Run focused verification:

   ```bash
   cargo test --manifest-path packages/qiongli-native/Cargo.toml \
     -p qiongli --lib rel_903 --locked
   cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
   ```

7. Run affected Slice verification: qiongli app library tests, affected Rust
   Clippy with warnings denied on the repository-pinned toolchain, public-schema
   validator/tests, Capability Contract validator, roadmap tests, and task
   validation.
8. Commit the product proof, open a PR to `2.x`, and wait for exact-head
   Evaluation Truth plus required Native CI contexts.
9. After those checks pass, write one acceptance note, set `REL-903` to
   `accepted`, regenerate the current program index, archive the Trellis task,
   commit closeout evidence, and merge only after final branch protection
   passes.

## Review gates

- Before implementation: PRD, design, and this plan approved; task status
  changed from `planning` to `in_progress`.
- Before product commit: focused and local Slice checks pass.
- Before ledger acceptance: exact product commit and exact CI run are known and
  successful.
- Before merge: all current-head protected checks pass and PR is mergeable.

## Rollback points

- Test/spec failure: revert only the focused test/spec edit.
- CI failure: reproduce with the smallest affected command before changing
  product code.
- Evidence mismatch: keep `REL-903` proposed and do not merge an acceptance
  claim.
