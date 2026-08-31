# Implementation Plan

## 0. Start only after approval

- [x] Present the converged PRD/design/plan and wait for explicit user approval.
- [x] Run `task.py start` only after that approval, create
      `feat/gov-414-delivery-checklists` from current `2.x`, and load the
      product-control/shared specs with `trellis-before-dev`.
- [x] Mark only `GOV-414` `active`; regenerate the current Program Index.

## 1. Add the two operator surfaces

- [x] Add `.github/delivery-checklists.md` with the four bounded stages,
      Machine/Human evidence markers, existing repository commands, tier
      selection, stale-evidence invalidation, and non-transitive authority rule.
- [x] Add `.github/pull_request_template.md` with the roadmap minimum fields and
      one link to the canonical checklist.
- [x] Keep release commands parameterized and point to the existing wrappers;
      do not copy release logic or imply that native dry-run permits publication.

## 2. Extend the existing governance contract

- [x] Add the checklist to `authorization-policy-v1.json` evidence and add both
      files to `repository-review-policy-v1.json` evidence.
- [x] Extend `validate_authorization_policy.py` with one literal checklist/PR
      contract check; preserve its existing public validation functions.
- [x] Add one focused valid-state assertion and one mutation table to
      `tests/test_authorization_policy.py`.
- [x] Update `.trellis/spec/product/control/authorization-policy-v1.md` with the
      two new canonical artifacts and validation contract.

## 3. Run the focused gate before expanding CI

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy -v
python tooling/scripts/update_program_roadmap.py --check
python -m unittest tests.test_program_roadmap -v
git diff --check
```

- [x] Verify the diff changes no Native CI classifier, required-check name,
      CODEOWNER identity, release script, product/package input, or user data.
- [x] Run `trellis-check` and resolve only findings in this task's bounded scope.

## 4. Obtain exact-head integration evidence once

- [x] Commit and push the bounded branch, then open a Draft PR against `2.x`.
- [x] Keep it Draft while local/focused evidence changes. Make it ready only
      after the intended implementation head is frozen so the full required
      Linux/macOS/Windows Slice runs once deliberately.
- [x] Confirm all protected contexts, including `Evaluation Truth V1`, pass on
      the exact implementation commit.
- [x] Record `GOV-414` as accepted with that full commit SHA, exact-head
      `Evaluation Truth V1` run ID, and stable evidence; regenerate the index and
      rerun the focused gate. If exact evidence is unavailable, keep it active
      instead of fabricating acceptance.
- [x] Merge only through the protected PR path, then archive/journal the Trellis
      task before selecting the next Program Ledger item.

## Risk and rollback points

- Checklist/template drift fails through the existing validator and test path.
- A new push invalidates exact-head CI and review evidence; refresh it before
  merge or Ledger acceptance.
- Rollback is a straight revert of the governance-only files; no product or
  persisted-data migration exists.
