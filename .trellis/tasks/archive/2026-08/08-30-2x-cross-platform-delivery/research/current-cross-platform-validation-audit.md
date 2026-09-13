# Current cross-platform validation audit

Audit date: 2026-08-30

## Executive finding

The repository already has the right three-tier policy: Focused checks during
development, one cross-platform Slice check for a frozen business slice, and
full package Acceptance only for an explicit candidate. The current cost comes
from trigger and closeout behavior, not from a missing test framework.

`Native CI` still runs the full Linux/macOS/Windows Rust source matrix for every
pull-request update and every merge push, including acceptance-receipt and
Trellis-archive-only changes. An explicit candidate dispatch then runs the same
source matrix again together with the intended package and lifecycle jobs.

## Repository evidence

- `.github/workflows/native-ci.yml:3-10` triggers on every push to `2.x`, every
  pull request targeting `2.x`, and manual dispatch.
- `.github/workflows/native-ci.yml:58-165` runs format, workspace check, Clippy,
  workspace tests, and the R4A acceptance test on Linux, macOS, and Windows.
- `.github/workflows/native-ci.yml:167-664` already restricts package assembly,
  packaged-product acceptance, candidate lifecycle acceptance, and promotion to
  manual dispatch.
- `.github/actions/setup-qiongli-desktop/action.yml` runs portable frontend
  checks only when requested and builds the static desktop assets on each
  native runner.
- `docs/maintainer/release-branch-policy.md` and its Chinese copy already define
  Focused, Slice, and Acceptance tiers. The policy says to run one exact-head
  Native CI after a business slice is frozen and reserve target packages for an
  explicit candidate.
- Ruleset `18800504` requires the change-boundary, Linux, macOS, Windows, and
  Evaluation Truth contexts with strict up-to-date checks. It forbids direct
  bypass, deletion, and non-fast-forward updates.
- The repository permits merge, squash, and rebase merges. With the strict
  up-to-date policy, the merged tree has already been checked on the PR; an
  explicit candidate dispatch remains the exact-commit/package authority.

## Observed Actions cost

From 00:00 UTC through the audit on 2026-08-30, GitHub recorded 29 Native CI
runs:

- 13 pull-request runs;
- 11 push runs;
- 5 manual candidate runs;
- 20 successes, 7 cancellations, and 2 failures.

Recent ordinary runs take about 20-21 wall-clock minutes. Their principal jobs
consume roughly 55-58 runner-minutes in aggregate:

- Linux native foundation: 18-19 minutes;
- macOS native foundation: 15-16 minutes;
- Windows native foundation: 20-21 minutes;
- Lite compatibility: about 1 minute.

Manual candidate run `33310992152` used the same source commit as the merge-push
run it cancelled, then correctly added three target packages, three lifecycle
checks, macOS packaged acceptance, and promotion. This shows that an immediate
candidate dispatch makes the merge-push source run redundant.

PRs 159/160 and 154/155 demonstrate the closeout multiplier. Each pair contains
only acceptance/ledger/Trellis closeout changes, yet both the PR and its merge
push ran the full three-platform source matrix. One completed task therefore
paid for multiple unrelated cross-platform runs after its product evidence was
already frozen.

## Root cause

The documented tier boundary is not encoded in the automatic trigger boundary:

1. full Slice checks run once on the PR and again after merge;
2. evidence-only and task-archive changes are treated like native source
   changes;
3. a manual Acceptance dispatch repeats the source matrix immediately after the
   merge run;
4. every additional PR commit starts the matrix again, although concurrency
   cancellation only saves the unfinished tail.

## Minimum viable correction

No new test runner, cache service, dependency, or platform farm is needed.

1. Keep the five ruleset context names unchanged.
2. Make PR the automatic Slice authority; keep manual dispatch as the exact
   candidate/Acceptance authority.
3. Stop automatically repeating the full matrix on the merge push.
4. Add an allowlisted evidence-only classifier. Required OS contexts should
   still report success but skip checkout/toolchain/build/test work when all
   changed paths are Trellis history, receipts, or generated program-state
   evidence.
5. Run full macOS and Windows source checks only on the final ready PR state;
   keep draft/intermediate development on focused local checks.
6. Keep security, schema, data-loss, path, ownership, CI, lockfile, native code,
   and packaging-input changes fail-safe: uncertainty selects the full matrix.

At audit time, the product decision was whether to remove the automatic
post-merge full matrix entirely or retain it for product-bearing merges. The
resolution is recorded below.

## Resolved decision and GitHub semantics

The user selected the speed-first boundary without reducing candidate rigor:
one final ready-PR Slice, no automatic post-merge Native CI, an evidence-only
fast path, and explicit dispatch for exact merged-candidate Acceptance.

GitHub's current status-check documentation confirms the implementation shape:

- a job skipped by a job-level condition reports success and does not block a
  required check;
- a whole workflow skipped by path or branch filtering can leave its required
  checks pending and block merge;
- strict required checks apply to the latest mergeable PR head.

One additional matrix constraint changes the first draft design. GitHub
evaluates `jobs.<job_id>.if` before applying `strategy.matrix`; a false
matrix-wide condition can therefore prevent the three named platform jobs from
being created. The safe ready-PR fast path always expands the matrix and
conditionally skips its expensive steps, leaving one lightweight successful
job on each platform. A matrix-wide condition is used only for draft/cancelled
runs, which cannot merge and receive a fresh event when marked ready. The
non-required Lite job may also be skipped at job level.

Primary references:

- https://docs.github.com/en/pull-requests/reference/status-checks
- https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks
- https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
