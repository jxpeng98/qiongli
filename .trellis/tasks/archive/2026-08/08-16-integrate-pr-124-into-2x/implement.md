# Implementation Plan: Integrate PR 124 into 2.x

## 1. Prepare the integration head

- [x] Change only stale roadmap integration wording to evidence-gated language.
- [x] Update PR #124 title/body to summarize Host activation, Plugin quality,
      Evaluation Truth, tests, risks and release non-claims.
- [x] Run `git diff --check` and strict research validation.
- [x] Commit the roadmap change without the current Trellis task directory.
- [x] Push the source branch and freeze its new head SHA.

## 2. Requalify the exact PR head

- [x] Wait for Evaluation Truth, Native CI effective jobs and Cloudflare to pass.
- [x] Fetch `origin/2.x` and the source branch again.
- [x] Require zero-behind ancestry, clean merge state, no unresolved threads and
      all live ruleset requirements satisfied.
- [x] Reconfirm the Alpha 3 tag and GitHub Release are absent.

## 3. Merge through protection

- [x] Run `gh pr merge 124 --merge --match-head-commit <final-head>` without
      deleting the branch.
- [x] Capture the merge SHA and verify the PR state, parents and `origin/2.x`
      ancestry.

## 4. Verify the target branch

- [x] Wait for merge-SHA Evaluation Truth and Native CI success.
- [x] Record the automatic promotion run ID/state without taking any protected
      environment action.
- [x] Confirm no Alpha 3 tag or GitHub Release exists.

## 5. Close operational records

- [x] Check every PRD acceptance item and record merge/run identities in task
      metadata or notes.
- [x] Archive the task and add the Trellis journal entry on the retained source
      branch.
- [x] Commit and push only those operational records; verify the closed PR is not
      extended and the worktree is clean.

## Rollback Points

- Before step 3: stop; `2.x` is unchanged.
- After step 3: do not reset or rewrite; use a new protected revert/fix PR if the
  post-merge checks expose a regression.
