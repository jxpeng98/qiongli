# REL-905 data lifecycle policy implementation plan

## Implementation

- [x] Author the English policy from current native owners and release policy.
- [x] Add a faithful Simplified Chinese policy page.
- [x] Link both pages from their Guide index and VitePress sidebar.
- [x] Add one dependency-free policy regression test and run it in Evaluation
      Truth.
- [x] Record the executable REL-905 scenario in the product-control spec.

## Validation

- [x] `python -m unittest tests.test_data_lifecycle_policy -v`
- [x] `pnpm docs:build`
- [x] `python tooling/scripts/update_program_roadmap.py --check`
- [x] `python -m unittest tests.test_program_roadmap -v`
- [x] `python3 ./.trellis/scripts/task.py validate 08-29-rel-905-data-lifecycle-policy`
- [x] `git diff --check`
- [x] Exact-head Evaluation Truth and Native CI pass at Slice tier.

## Review gates

- No claim may imply that portable export contains credentials or private
  runtime state.
- No removal command may be represented as full data deletion.
- No fixed 1.x end date may be invented before Qiongli 2 Stable publication.
- Candidate packaging, promotion, publication, and REL-906 remain out of scope.

## Rollback

Revert the policy Slice. No user state or product artifact requires migration.
