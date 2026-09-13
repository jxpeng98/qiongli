# Implementation Plan: GOV-418 self-authorization negative policy

## 0. Enter implementation only after final approval

- [x] Present the final Goal/In Scope/Out of Scope/Acceptance/Decisions/Risks
      summary and obtain a subsequent explicit implementation approval.
- [x] Run `task.py start`, create
      `test/gov-418-self-authorization-policy` from current `origin/2.x`, and
      load product-control/shared specs with `trellis-before-dev`.
- [x] Mark only `GOV-418` active and regenerate the Program Index.

## 1. Add the exhaustive negative matrix

- [x] Extend `tests/test_authorization_policy.py` with one table-driven matrix
      across all twelve v1 actions.
- [x] Reject Agent/CI and `ci-green` authorizer insertion for every action.
- [x] Validate a human-authorized Agent/CI executor receipt baseline per action,
      then reject its Agent/CI self-authorized mutation.
- [x] Remove every declared action binding one at a time and require fail-closed
      validation.
- [x] Remove and positively reverse each privileged `ci-green` transition and
      require fail-closed validation.

## 2. Synchronize the executable contract

- [x] Update `.trellis/spec/product/control/authorization-policy-v1.md` with the
      full negative-matrix requirement.
- [x] Leave the policy JSON, receipt schema, validator, workflow, and product
      code unchanged unless the test exposes a real shared-owner gap.
- [x] Keep `GOV-413`, `SEC-820`, and unrelated roadmap states unchanged.

## 3. Run the local Slice gate

- [x] Run:

```bash
python3 tooling/scripts/validate_authorization_policy.py
python3 -m unittest tests.test_authorization_policy tests.test_program_roadmap -v
python3 tooling/scripts/update_program_roadmap.py --check
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
git diff --check
python3 .trellis/scripts/task.py validate .trellis/tasks/09-01-gov-418-self-authorization-negative-policy
```

- [x] Run `trellis-check` inline and resolve only GOV-418 findings.
- [x] Confirm no runtime, schema, package, Host, release, dependency, or
      user-data input changed.

## 4. Freeze, review, and merge once

- [ ] Commit and push the bounded branch, then open a Draft PR against `2.x`.
- [ ] Freeze the implementation head before Ready and obtain the required
      Linux, macOS, Windows, Native boundary, and Evaluation Truth contexts.
- [ ] Review exact-head diff/evidence and merge through the protected PR path.

## 5. Evidence-only closeout

- [ ] From merged `2.x`, create `chore/gov-418-closeout`.
- [ ] Record `GOV-418` accepted with the implementation SHA and exact-head
      Evaluation Truth run; regenerate the Program Index.
- [ ] Archive/journal with `trellis-finish-work`, push the allowlisted closeout
      PR, merge it, and leave local `2.x` clean.

## Risk and Rollback Points

- A malformed negative fixture could pass for the wrong reason; each per-action
  receipt first proves its valid human-authorized baseline.
- This policy-as-code task does not claim general runtime receipt-use
  enforcement; that remains with action consumers and the deferred SEC matrix.
- Any implementation push invalidates current-head CI/review evidence.
- Rollback is one test/spec/evidence revert with no migration or data repair.
