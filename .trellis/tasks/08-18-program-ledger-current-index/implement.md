# Implementation plan: GOV-401 through GOV-404

## 1. Freeze inputs and contract

- [x] Confirm the protected `2.x` base, 233 unique roadmap IDs, and accepted
      EVAL merge/run evidence.
- [x] Read product-control and shared reuse/cross-layer specs.
- [x] Add one failing focused test for a missing ledger task and stale index.

## 2. Add ledger validator/generator

- [x] Implement strict stdlib JSON and roadmap parsing.
- [x] Validate exact schema, IDs/order, states, evidence, dates, dependency
      closure, and cycles.
- [x] Render the deterministic milestone/workstream index and support `--check`.

Focused check:

```bash
python3 -m unittest tests.test_program_roadmap -v
```

## 3. Seed all 233 tasks and current evidence

- [x] Generate the ledger inventory in roadmap order without descriptions.
- [x] Record accepted EVAL protected merge/run evidence; classify unresolved,
      blocked, deferred, and historical work without using checkbox state.
- [x] Keep `GOV-401` through `GOV-404` active pending exact-head CI.
- [x] Generate the current index and update roadmap authority links/prose.

## 4. Own freshness in existing CI

- [x] Add `python tooling/scripts/update_program_roadmap.py --check` to the
      existing 2.x `Evaluation Truth V1` Python job.
- [x] Add focused rejection tests for invalid state, duplicate/missing ID,
      unknown/cyclic dependency, missing accepted evidence, and stale output.

Quality gate:

```bash
python3 tooling/scripts/update_program_roadmap.py --check
python3 -m unittest tests.test_program_roadmap -v
python3 -m unittest tests.test_branch_policy -v
git diff --check
```

## 5. Review, PR, and evidence closeout

- [x] Run Trellis check, update executable specs if a reusable contract changed,
      commit, push, and open a PR against `2.x`.
- [ ] Resolve CI/review failures without broadening scope.
- [x] After exact-head CI passes, mark only `GOV-401` through `GOV-404`
      accepted with that exact commit/run and regenerate the index.
- [ ] Merge only after final branch protection passes; do not publish or mutate
      GitHub roadmap objects.

## Rollback points

- Before ledger generation: task-only changes.
- Before CI wiring: ledger/index/script/tests are locally removable as one unit.
- After merge: use a normal revert/fix PR; do not rewrite `2.x` history.
