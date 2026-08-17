# Design: Integrate PR 124 into 2.x

## Boundary

GitHub's protected PR path owns the integration. The task may update roadmap and
PR metadata, but it does not alter the delivered product or release authority.
The exact PR head remains immutable once its final checks start.

## Integration Flow

1. Future-proof stale roadmap integration wording and update PR metadata.
2. Commit only the roadmap change; keep `.trellis/tasks/08-16-*` untracked.
3. Push the source head and wait for every required PR check.
4. Re-fetch `2.x`, re-evaluate mergeability and bind a merge commit to that head.
5. Verify the merge commit on `origin/2.x` and its push-triggered CI.
6. Observe automatic non-publishing promotion dispatch without acting on the
   protected environment.
7. Archive and journal this task on the retained source branch.

## Contracts

### Pre-merge

- `baseRefName == "2.x"`
- `behind_by == 0`
- `mergeable == "MERGEABLE"`
- `mergeStateStatus == "CLEAN"`
- unresolved review threads: `0`
- required checks: success; expected PR promotion dispatch: skipped
- local final head equals remote PR head

Any changed head or base invalidates the snapshot and restarts this preflight.

### Merge

Use `gh pr merge 124 --merge --match-head-commit <final-head>`. Do not pass
`--delete-branch`. Merge/squash/rebase are all allowed by repository settings,
but merge commit is selected because it preserves the tested head and its 38
commits as ancestry.

### Post-merge

- PR state is merged and exposes one merge commit.
- final PR head is an ancestor of `origin/2.x` and a parent of the merge commit.
- `Evaluation Truth` and `Native CI` runs for the merge SHA succeed.
- promotion dispatch, if present, is recorded but its environment decision is
  untouched; Alpha tag/release remain absent.

## Compatibility and Rollback

Before merge, stopping is a no-op on `2.x`. After merge, history must not be
rewritten; a regression is handled through a new protected revert/fix PR.
Post-merge CI failure blocks task closeout but does not trigger an automatic
revert or publication action.

## Trade-offs

- Merge commit adds one commit but retains exact-head traceability; squash and
  rebase are smaller history but discard that identity.
- Evidence-gated roadmap wording avoids a second status-only PR after merge.
- Trellis closeout remains on the retained source branch so an in-progress task
  is neither merged into `2.x` nor followed by another target-branch CI cycle.
