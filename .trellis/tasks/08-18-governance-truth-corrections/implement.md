# Implementation: Correct architecture, ADR, and parity truth

## Ordered checklist

- [ ] Mark `GOV-405` through `GOV-407` active and regenerate the current index.
- [ ] Rename the Community Alpha ADR from 0208 to 0215 and update all references.
- [ ] Add the complete current ADR registry and extend the existing validator.
- [ ] Add focused architecture/registry regression tests.
- [ ] Rename parity root status to classification status, bump schema 1.1, and
  update the existing Rust contract test.
- [ ] Run focused validators and tests.
- [ ] Run Program Ledger, evaluation, format, and diff checks.
- [ ] Commit, push, open a PR, and resolve required CI/review failures.
- [ ] Record exact-head evidence, mark `GOV-405` through `GOV-407` accepted,
  regenerate the index, and merge after required checks pass.

## Focused validation

```bash
python3 scripts/validate_arc_201_adrs.py
python3 -m unittest tests.test_arc_201_adrs -v
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-platform --test product_parity_ledger
python3 tooling/scripts/update_program_roadmap.py --check
```

## Final validation

```bash
python3 -m unittest tests.test_arc_201_adrs tests.test_program_roadmap \
  tests.test_branch_policy -v
.venv/bin/python evals/runner/run_suite.py
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
git diff --check
```

## Risk and rollback points

- Before the ADR rename, enumerate every old path reference; after editing,
  require zero matches for `0208-community-alpha-distribution-boundary`.
- Do not stage a change to the frozen ARC-201 inventory or ADR 0201-0207.
- Keep parity disposition arrays byte-equivalent except for formatting caused by
  the root field/version edit.
- If exact-head CI fails, leave the three roadmap tasks active and fix only the
  failing owner; do not record acceptance from an older run.
