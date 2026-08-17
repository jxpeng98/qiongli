# Integrate PR 124 into 2.x

## Goal

Integrate the exact verified contents of PR #124 into protected branch `2.x`
without rewriting its tested history, overstating release readiness, or taking
any protected publication decision.

## Background

- Planning baseline: PR #124 is open, non-draft, `MERGEABLE`/`CLEAN`, at
  `884b434d711b675c010ddfacd07d36c5829f7f8d`, 38 commits ahead and zero behind
  `2.x` commit `6494b004ac141bcef0e6a799552d4b63720b7b7f`.
- Its 12 effective checks passed; the PR-only promotion dispatch was correctly
  skipped. There are zero review threads and the live ruleset requires zero
  approvals plus strict Native boundary and three-platform Rust checks.
- The repository permits merge, squash and rebase, but only a merge commit
  retains the exact tested PR head as a parent without rewriting its commits.
- The PR title and body still describe only App-mediated Host activation; the
  branch now also contains bounded Plugin-quality and Evaluation Truth V1 work.
- A successful `2.x` Native CI push automatically dispatches the Community
  Alpha promotion workflow. Its rebuild and aggregate stages are non-publishing;
  authorization is protected by `community-alpha-publication`. The Alpha 3 tag
  and GitHub Release are absent at planning time.

## Requirements

### R1. Make the integration surface truthful

- Update the PR title and body to cover Host activation, Plugin quality and
  Evaluation Truth, including executed checks and explicit release non-claims.
- Replace only roadmap wording that would become false immediately after merge
  with evidence-gated language that is correct both before and after integration.
- Do not change product behavior, release policy, or the Alpha acceptance ledger.

### R2. Preserve exact-head evidence

- Keep the source branch history intact: no rebase, squash, force push or branch
  deletion.
- Before merge, fetch `2.x` again and require the PR to be zero commits behind,
  mergeable/clean, free of unresolved threads, and green on every required check.
- Merge with GitHub's merge-commit method and bind the command to the final head
  using `--match-head-commit`.

### R3. Verify target integration

- Require the PR to report merged and the merged PR head to be an ancestor of
  current `origin/2.x`.
- Capture the merge commit and verify its parentage includes the exact PR head.
- Require the post-merge `Evaluation Truth` and `Native CI` runs for that merge
  commit to complete successfully.

### R4. Keep publication authority separate

- Observe and record the automatically dispatched promotion run, if created,
  but do not approve, reject, cancel or bypass its protected environment.
- Confirm integration creates no `v2.0.0-alpha.3` tag or GitHub Release.
- Do not claim M0/M1 exit, Alpha qualification, package acceptance, publication
  authorization or public observation.

### R5. Close Trellis without polluting the merged tree

- Keep this task directory out of PR #124; it was created after the verified PR
  head and is operational integration evidence, not product input.
- After the merge and post-merge checks, archive the task and record its journal
  on the retained source branch, then push that branch without reopening or
  extending the merged PR.

## Acceptance Criteria

- [x] PR #124 accurately summarizes all integrated slices and non-claims.
- [x] The final PR head is current with `2.x`, has no unresolved review thread,
      and every required exact-head check passes.
- [x] PR #124 is merged by merge commit with its exact final head as a parent.
- [x] `origin/2.x` contains the merged head and its merge-commit Evaluation Truth
      and Native CI runs succeed.
- [x] Automatic promotion is observed without a publication-plane decision, and
      no Alpha 3 tag or GitHub Release is created.
- [x] Roadmap language remains truthful across the merge; no release or milestone
      acceptance is newly claimed.
- [x] The task is archived on the retained source branch and the worktree is clean.

## Out of Scope

- Squash/rebase merge, force push, source-branch deletion or direct `2.x` push.
- Approving, rejecting, cancelling or bypassing protected publication.
- Updating Alpha package receipts, accepting A6-A9, tagging or publishing.
- Additional product code, evaluator expansion or later M1/GOV/SEC/PLT work.
