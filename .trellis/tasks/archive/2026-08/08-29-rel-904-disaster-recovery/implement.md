# REL-904 implementation plan

1. Prefix the existing interrupted-migration, missing-catalog, and durable
   transaction-boundary tests with `rel_904` so the focused filter reuses them.
2. Add one direct lost-registration test that re-registers an unchanged
   portable project manifest into an empty private Research Library index.
3. Let only `DeleteDerivedState` preview map `InvalidPortfolioCatalog` to the
   existing absent-catalog preview identity.
4. Extend `PortfolioCatalogStore::delete` to clear corrupt private catalog
   contents under the existing lock when the approved plan expects no valid
   catalog identity; retain fail-closed behavior for every other error.
5. Add one corrupt-catalog test proving approval, canonical byte preservation,
   reset, restart rebuild, and clean portfolio equivalence.
6. Add the seven-section REL-904 executable scenario to
   `.trellis/spec/product/control/public-schema-authority.md`.
7. Run focused format/test loops, then the full `qiongli-project` package,
   Clippy, public-schema policy, Capability Contract v2, roadmap validation,
   and Trellis task validation.
8. Commit product evidence, open a PR to `2.x`, record exact-head Slice CI,
   accept REL-904 in Program Ledger v1, archive the task, and merge only after
   all required checks pass.

## Focused validation

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-project rel_904 --locked
```

## Slice validation

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-project --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-project --all-targets --all-features --locked -- -D warnings
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy -v
.venv/bin/python scripts/validate_capability_contract.py
python tooling/scripts/update_program_roadmap.py --check
python -m unittest tests.test_program_roadmap -v
python3 ./.trellis/scripts/task.py validate \
  .trellis/tasks/08-29-rel-904-disaster-recovery
```

## Risk and rollback points

- Corrupt cleanup must remain inside the validated Portfolio catalog root and
  must not follow symlinks; any uncertainty stays fail closed.
- A stale reset plan must fail if a valid catalog appears or the library
  revision changes before apply.
- Do not accept REL-904 or merge if any of the five focused cases or exact-head
  required CI jobs fail.
