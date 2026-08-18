# Implementation Plan

## 1. Activate the bounded task

- [x] Review the PRD/design/plan and obtain the required post-summary approval.
- [x] Run `task.py start`, create `feat/protected-branch-review-policy` from the
      current `2.x`, and load `trellis-before-dev` product-control guidance.

## 2. Publish ownership and ruleset policy

- [x] Add `.github/CODEOWNERS` with ordered entries for security, schema,
      migration, release, research-Gate, authorization, and the policy files.
- [x] Add the closed v1 repository-review policy bound to ruleset `18800504` and
      the current blocked independent-review state.
- [x] Update the authorization product-control spec; do not add an ADR because
      this implements the existing roadmap/authorization decision.

## 3. Enforce the checked-in contract

- [x] Extend the existing standard-library authorization validator to load and
      validate the review policy and CODEOWNERS file.
- [x] Extend the existing focused unittest with valid-state and fail-closed
      mutations; keep the existing Evaluation Truth command path.
- [x] Mark `GOV-413` blocked with stable repository evidence and the exact
      second-reviewer blocker; regenerate the current index.

Focused checks:

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy -v
python tooling/scripts/update_program_roadmap.py --check
git diff --check
```

## 4. Update the live protected-branch rule safely

- [x] Open a Draft PR and require a passing `Evaluation Truth V1` observation
      for the exact head before changing repository settings.
- [x] Re-read ruleset `18800504`; abort if branch target, bypass list, rules,
      review settings, or native check inventory drifted from the plan.
- [x] Add only `Evaluation Truth V1` to required checks with a full guarded PUT;
      read back and compare all preserved fields.
- [x] Confirm the task PR now reports Evaluation Truth plus the four existing
      native contexts as required, with approval count zero and CODEOWNER review
      disabled.

## 5. Integrate and close

- [x] Run `trellis-check`, commit intentionally, resolve exact-head CI failures,
      and merge only through the protected PR path.
- [x] Archive/journal the Trellis task after merge. Leave `GOV-413` blocked until
      an independent eligible reviewer is explicitly nominated and configured.

## Risk and rollback points

- Never enable required review with only one eligible human.
- Never add a bypass actor or remove an existing required check.
- Treat any live precondition mismatch as a stop; do not overwrite drift.
- Product/package inputs are unchanged, so no package rebuild is claimed or
  required by this slice beyond repository CI that runs automatically.
