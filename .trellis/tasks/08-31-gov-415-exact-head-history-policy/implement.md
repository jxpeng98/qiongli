# Implementation Plan: GOV-415 exact-head and history policy

## 0. Enter implementation only after final approval

- [x] Present the final Goal/In Scope/Out of Scope/Acceptance/Decisions/Risks
      summary and obtain a subsequent explicit implementation approval.
- [x] Run `task.py start`, create
      `feat/gov-415-exact-head-history-policy` from current `2.x`, and load the
      product-control/shared specs with `trellis-before-dev`.
- [x] Mark only GOV-415 `active` and regenerate the Program Index.

## 1. Extend the existing governance contract

- [x] Add one closed `history_policy` object to
      `tooling/architecture/repository-review-policy-v1.json` with exact-head,
      protected-ref, and exceptional feature-rewrite rules from the approved
      design.
- [x] Update `.github/delivery-checklists.md` and
      `.github/pull_request_template.md` so head-change invalidation, protected
      no-force-push, and the bounded `--force-with-lease` exception agree with
      the JSON contract.
- [x] Update `.trellis/spec/product/control/authorization-policy-v1.md`; do not
      add another policy file, hook, workflow, dependency, or remote setting.

## 2. Fail closed through the existing validator

- [x] Extend `validate_authorization_policy.py` with the smallest exact v1
      history-policy constant/check and delivery markers. Preserve existing
      public functions and live-ruleset validation.
- [x] Add one mutation table to `tests/test_authorization_policy.py` covering
      stale head evidence, protected ref narrowing/force push, unsafe feature
      rewrite, missing authority/notice, plain `--force`, and retained replaced-
      commit receipts.
- [x] Keep Evaluation Truth unchanged because it already invokes the shared
      validator and focused tests.

## 3. Run focused and task-scope gates

- [x] Run:

```bash
python3 tooling/scripts/validate_authorization_policy.py
python3 -m unittest tests.test_authorization_policy tests.test_program_roadmap -v
python3 tooling/scripts/update_program_roadmap.py --check
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
git diff --check
python3 .trellis/scripts/task.py validate .trellis/tasks/08-31-gov-415-exact-head-history-policy
```

- [x] Read back GitHub rulesets without mutation and report the partial hosted
      enforcement truthfully.
- [x] Run `trellis-check` and fix only GOV-415 scope findings.

## 4. Obtain one full implementation matrix

- [ ] Commit/push the bounded branch and open a Draft PR against `2.x`.
- [ ] Freeze the intended implementation head before making the PR ready; run
      the full Linux/macOS/Windows Slice once deliberately.
- [ ] Confirm required Native contexts and `Evaluation Truth V1` pass on that
      exact implementation SHA, then merge through the protected PR path while
      GOV-415 remains `active`.

## 5. Close out without a second full matrix

- [ ] From merged `2.x`, create an evidence-only closeout branch.
- [ ] Record GOV-415 `accepted` with the implementation SHA, exact-head
      Evaluation Truth run ID, and stable evidence; regenerate the index.
- [ ] Archive the task and add the session journal using `trellis-finish-work`.
- [ ] Commit/push the allowlisted Ledger/Trellis/workspace diff, confirm the
      lightweight protected contexts, merge the closeout PR, and leave local
      `2.x` clean.

## Risk and Rollback Points

- Policy/template drift fails through the existing validator and mutation test.
- Any implementation push invalidates its current-head CI/review evidence; do
  not record acceptance until exact evidence is available.
- A hosted GitHub setting change is outside scope and must stop for separate
  authorization rather than being folded into this task.
- Rollback is a straight governance-file revert; no product, package, ref, tag,
  remote rule, or persisted-data rollback is required.
