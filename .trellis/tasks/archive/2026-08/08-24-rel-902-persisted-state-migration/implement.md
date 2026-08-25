# REL-902 Implementation Plan

## Checklist

1. Add the closed two-row persisted-state predecessor fixture manifest.
2. Add one native acceptance test beside `legacy_migration_cli` that loops both
   rows and calls the existing project/global migration and rollback owners.
3. Keep production code unchanged unless the test exposes a concrete defect;
   fix such a defect only at its shared owner with the smallest regression
   check.
4. Add the executable REL-902 scenario to the persisted/public schema Trellis
   spec and keep REL-903/REL-904 nonclaims explicit.
5. Run focused Rust format/test checks, then task-scope policy and Native Slice
   checks.
6. Commit product evidence, open a PR to `2.x`, record exact-head acceptance,
   archive the task, and merge only after required checks pass.

## Focused Validation

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib rel_902 --locked
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy -v
python3 tooling/scripts/update_program_roadmap.py --check
```

## Rollback Points

- Fixture/test failure: revert only the fixture and focused test; production
  behavior remains unchanged.
- Shared-owner defect: keep the failing predecessor row, revert unrelated
  changes, and repair only the owner that both migration surfaces call.
- CI failure: reproduce the smallest failing platform check before changing
  the accepted scope.
