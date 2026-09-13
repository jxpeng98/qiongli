# Implementation Plan: GOV-416 decision separation

## 0. Enter implementation only after final approval

- [ ] Present the final Goal/In Scope/Out of Scope/Acceptance/Decisions/Risks
      summary and obtain a subsequent explicit implementation approval.
- [ ] Run `task.py start`, create `feat/gov-416-decision-separation` from current
      `2.x`, and load product-control/shared specs with `trellis-before-dev`.
- [ ] Mark only GOV-416 active and regenerate the Program Index.

## 1. Add the missing announcement action

- [ ] Add `publication.announce-release` to the existing authorization policy
      and receipt action enum; preserve all receipt fields and existing actions.
- [ ] Add the three ordered negative transitions separating announcement from
      merge, publication, and CI evidence.
- [ ] Update the Release Approver/Publication descriptions only where needed;
      add no role, policy file, receipt store, or runtime owner.

## 2. Fail closed through the existing validator

- [ ] Extend expected action, rule, authorizer, default, evidence, and count
      constants in `validate_authorization_policy.py`.
- [ ] Require announcement plan digest, verified artifact digests, and channels
      together through one action-specific binding guard.
- [ ] Reuse `required_bindings` in `validate_receipt` so declared plan/artifact
      digest bindings must be present for every action.
- [ ] Add one focused mutation table in `tests/test_authorization_policy.py` for
      action/schema, authorizer, bindings/evidence, transitions, and delivery
      wording; keep existing public validation functions unchanged.

## 3. Sync operator and maintenance contracts

- [ ] Update the release checklist with a distinct announcement decision/receipt
      after public verification and before announcement.
- [ ] Update the PR confirmation so green checks authorize none of merge,
      release, or announcement.
- [ ] Update `.trellis/spec/product/control/authorization-policy-v1.md`; do not
      change product code, release automation, workflows, or remote settings.

## 4. Run the Slice gate

- [ ] Run:

```bash
python3 tooling/scripts/validate_authorization_policy.py
python3 -m unittest tests.test_authorization_policy tests.test_program_roadmap -v
python3 tooling/scripts/update_program_roadmap.py --check
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
git diff --check
python3 .trellis/scripts/task.py validate .trellis/tasks/08-31-gov-416-decision-separation
```

- [ ] Run `trellis-check` and resolve only GOV-416 findings.
- [ ] Confirm the diff contains no native release, package, workflow, dependency,
      service, receipt-storage, GitHub-setting, or user-data change.

## 5. Obtain one implementation matrix and merge

- [ ] Commit/push the bounded branch and open a Draft PR against `2.x`.
- [ ] Freeze the implementation head before Ready; run the required Linux,
      macOS, Windows, Native boundary, and Evaluation Truth contexts once.
- [ ] Merge through the protected PR path while GOV-416 remains active.

## 6. Evidence-only closeout

- [ ] From merged `2.x`, create `chore/gov-416-closeout`.
- [ ] Record GOV-416 accepted with implementation SHA, exact-head Evaluation
      Truth run, and stable evidence; regenerate the Program Index.
- [ ] Archive/journal with `trellis-finish-work`, push the allowlisted closeout
      PR, confirm matrix-required false, merge, and leave local `2.x` clean.

## Risk and Rollback Points

- A receipt action mismatch fails closed; never reinterpret an old receipt.
- Any implementation push invalidates current-head CI/review evidence.
- Rollback is a single governance-diff revert with no runtime or data migration.
