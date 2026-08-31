# Implement GOV-414 delivery checklists

## Goal and user value

Turn the accepted repository-delivery policy into four short, version-controlled
checklists for pre-commit, pre-push, pull request, and release work. The operator
must be able to choose the smallest valid verification tier during development,
run the expensive Linux/macOS/Windows matrix only at the Slice boundary, and
retain exact-head evidence without confusing successful checks with authorization.

## Background and confirmed facts

- The generated Program Index lists `GOV-414` as the first unblocked proposed
  M1 item. `REL-314` appears earlier but depends on blocked `REL-313`.
- Master roadmap Sections 19.5 through 19.7 already own the delivery sequence,
  minimum PR fields, non-transitive authorization rule, and release sequence.
- `docs/maintainer/release-branch-policy.md` already owns Focused, Slice, and
  Acceptance verification tiers. Draft pull requests avoid the native matrix;
  ready source-affecting pull requests run the required Linux, macOS, and Windows
  Slice; merge pushes do not start a duplicate Native CI run.
- `tooling/architecture/authorization-policy-v1.json` already requires
  `staged-diff` and `local-checks` for commit, `clean-checkpoint` and
  `pre-push-checklist` for push, `exact-head` and `scope-nonclaims` for PR work,
  and exact commit, asset digests, and release approval for publication.
- `tooling/scripts/validate_authorization_policy.py` and
  `tests/test_authorization_policy.py` already provide the machine-verifiable
  governance path used by `Evaluation Truth V1`.
- `scripts/release_ready.sh` and `scripts/release_preflight.sh` are the existing
  public release-entry wrappers. Native 2.x runs remain non-publishing until
  later release gates remove their explicit blocker.
- `.github` has no pull-request template today. Its existing CODEOWNERS rule
  already covers all files under `.github/`.

## Requirements

### R1 — One canonical delivery checklist

- Add `.github/delivery-checklists.md` with exactly four operational sections:
  pre-commit, pre-push, pull request, and release.
- Mark each item as machine-verifiable or human/authority evidence so automation
  is never represented as approval.
- Reuse repository commands and owners for branch/diff state, focused checks,
  exact-head required checks, and release preflight. Do not duplicate build or
  release logic in the checklist.
- State the tier rule explicitly: Focused while editing, one exact-head Slice
  when the PR is ready, and Acceptance only for an explicit candidate.

### R2 — Default pull-request contract

- Add `.github/pull_request_template.md` linked to the canonical checklist.
- Capture the roadmap minimums: problem/outcome, in-scope paths and non-goals,
  architecture/schema/security/research impact, tests and exact-head evidence,
  migration/rollback/compatibility, risks/follow-ups, and required reviewers.
- Require stale exact-head evidence to be replaced after every new push and keep
  merge and release authorization separate from PR completion.

### R3 — Reuse the existing governance gate

- Extend `tooling/scripts/validate_authorization_policy.py` with the smallest
  literal contract check for the four checklist stages, required evidence
  markers/commands, non-transitive authorization warning, PR-template fields,
  and canonical checklist link.
- Add one focused mutation test group to `tests/test_authorization_policy.py`.
- Add the checklist and PR template to the existing authorization/review-policy
  evidence lists; add no new validator, schema, dependency, workflow, or service.
- Keep `Evaluation Truth V1` unchanged because it already runs the validator and
  focused test module.

### R4 — Preserve the fast cross-platform loop

- Do not require broad builds at every commit or push. The checklist must route
  ordinary iteration to task-focused checks and reserve the full native matrix
  for a ready source-affecting PR.
- Keep the implementation PR in Draft until focused governance checks pass, then
  make it ready once so the required Linux/macOS/Windows matrix runs against one
  deliberate exact head.
- Do not change the Native CI classifier, required contexts, release scripts,
  product/runtime behavior, package inputs, or user data.

### R5 — Truthful Program Ledger lifecycle

- After explicit implementation approval and task start, mark only `GOV-414`
  `active` and regenerate the current Program Index.
- Mark `GOV-414` `accepted` only after the implementation commit has a passing
  exact-head `Evaluation Truth V1` run, recording its full 40-character commit
  SHA, decimal run ID, and stable repository evidence. Otherwise leave it
  `active`.

## Acceptance criteria

- [ ] One canonical file contains all four named checklists and distinguishes
      machine evidence from human authority.
- [ ] The checklists direct development through Focused -> Slice -> Acceptance
      without requiring repeated cross-platform builds during ordinary edits.
- [ ] A new PR automatically receives every minimum field required by roadmap
      Section 19.6 and links the canonical checklist.
- [ ] The existing authorization validator fails when a checklist stage,
      required evidence marker/command, authorization warning, PR field, or link
      is removed.
- [ ] `python tooling/scripts/validate_authorization_policy.py` passes.
- [ ] `python -m unittest tests.test_authorization_policy -v` passes, including
      focused checklist/template mutations.
- [ ] Program Ledger validation and generated-index freshness checks pass.
- [ ] The ready implementation head passes all protected `2.x` required checks;
      `GOV-414` acceptance records that exact implementation commit and
      `Evaluation Truth V1` run.
- [ ] No new dependency, hook manager, workflow, service, product/package input,
      release, publication, or user-data mutation is introduced.

## Out of scope

- Installing or enforcing local Git hooks, adding a new CLI, or auto-running
  every possible test before each commit or push.
- Changing branch protection, CODEOWNER identities, required-check names,
  Native CI classification, merge methods, or the blocked `GOV-413` reviewer
  state.
- Granting push, merge, tag, release, or publication authority.
- Performing a release, assembling a candidate, publishing assets, or claiming
  native Windows hardware/signing/installer acceptance.
- `GOV-415` through `GOV-418`, product features, runtime schemas, and package
  changes.

## Risks and constraints

- Markdown can drift from executable policy, so the existing authorization gate
  must validate a small stable set of stage/field/command markers.
- Overly broad checklists would recreate the slow build loop. Each checklist
  therefore points to the existing tier owner instead of copying every package
  command.
- The `.github` and validator changes make this a source-affecting PR, so one
  full exact-head native matrix is expected before merge.
