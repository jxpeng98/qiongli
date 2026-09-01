# Implementation Plan: GOV-417 authorization exception lifecycle

## 0. Enter implementation only after final approval

- [x] Present the final Goal/In Scope/Out of Scope/Acceptance/Decisions/Risks
      summary and obtain a subsequent explicit implementation approval.
- [x] Run `task.py start`, set the base to `2.x`, create
      `feat/gov-417-authorization-lifecycle`, and load the product-control/shared
      specs with `trellis-before-dev`.
- [x] Mark only GOV-417 active and regenerate the Program Index.

## 1. Add the closed lifecycle policy

- [x] Add the ordered `authorization_lifecycle` array to
      `tooling/architecture/authorization-policy-v1.json` with denial, expiry,
      revocation, repository-only emergency hotfix, and reconciliation paths.
- [x] Keep the existing planes, roles, actions, authorizers, negative rules,
      and receipt schema unchanged.
- [x] Encode new-authorization recovery, immutable receipt evidence, and the
      ban on reviving or retroactively authorizing invalid work.

## 2. Fail closed through the existing validator

- [x] Add the lifecycle root key and one exact
      `EXPECTED_AUTHORIZATION_LIFECYCLE` constant to
      `validate_authorization_policy.py`.
- [x] Add one equality guard that rejects missing, reordered, unknown,
      broadened, or weakened lifecycle definitions.
- [x] Reuse stdlib and current validation entry points; add no parser,
      abstraction, dependency, schema field, or runtime state machine.

## 3. Add one focused check and synchronize guidance

- [x] Add one lifecycle mutation table to
      `tests/test_authorization_policy.py`, including emergency scope and
      reconciliation non-authorization cases.
- [x] Add `### Authorization exceptions` to the existing delivery checklist and
      protect its fail-closed, repository-only, no-bypass, and reconciliation
      wording with the current marker validator.
- [x] Update `.trellis/spec/product/control/authorization-policy-v1.md` with the
      same consumer and maintenance rules; leave the PR template unchanged.

## 4. Run the local Slice gate

- [x] Run:

```bash
python3 tooling/scripts/validate_authorization_policy.py
python3 -m unittest tests.test_authorization_policy tests.test_program_roadmap -v
python3 tooling/scripts/update_program_roadmap.py --check
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
git diff --check
python3 .trellis/scripts/task.py validate .trellis/tasks/09-01-gov-417-authorization-lifecycle
```

- [x] Run `trellis-check` inline and resolve only GOV-417 findings.
- [x] Confirm no product/native code, workflow, dependency, remote setting,
      receipt schema, publication path, or user-data behavior changed.

## 5. Freeze, review, and merge once

- [ ] Commit and push the bounded branch, then open a Draft PR against `2.x`.
- [ ] Freeze the implementation head before Ready and run the required Linux,
      macOS, Windows, Native boundary, and Evaluation Truth contexts once.
- [ ] Review the exact-head diff and evidence, then merge through the protected
      PR path while GOV-417 remains active.

## 6. Evidence-only closeout

- [ ] From merged `2.x`, create `chore/gov-417-closeout`.
- [ ] Record GOV-417 accepted with the implementation SHA, exact-head
      Evaluation Truth run, and stable evidence; regenerate the Program Index.
- [ ] Archive/journal with `trellis-finish-work`, push the allowlisted closeout
      PR, confirm matrix-required false, merge, and leave local `2.x` clean.

## Risk and Rollback Points

- The policy is an auditable governance contract, not new runtime enforcement;
  consumers remain responsible for checking receipt state and time.
- An emergency receipt never bypasses the protected PR/check path or expands
  into research/publication authority.
- Any implementation push invalidates current-head CI/review evidence.
- Rollback is one governance-diff revert with no migration or data repair.
