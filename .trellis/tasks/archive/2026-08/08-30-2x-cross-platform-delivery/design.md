# Design: PR-only Slice validation with evidence fast paths

## Problem statement

Qiongli already has the correct verification ladder, but Native CI does not
encode it. A business slice is checked on its PR, checked again after merge,
and often followed by an explicit candidate run that checks the same source
matrix again. Acceptance records and Trellis archival then trigger more native
matrices despite changing no executable input.

The minimum correction is to make one ready PR the automatic Slice boundary,
make explicit dispatch the exact candidate boundary, and conditionally skip
required source jobs for narrowly classified evidence-only changes.

## Scope and authority

This remains one implementation task. The following pieces describe one
contract and would drift if delivered separately:

- `.github/workflows/native-ci.yml` owns automatic and candidate execution;
- `scripts/check_2x_native_change_boundary.sh` already owns changed-path
  inspection for `2.x` and will also emit the conservative matrix decision;
- `tests/test_native_change_boundary.py` and `tests/test_branch_policy.py` own
  executable policy coverage;
- `.trellis/spec/product/control/index.md` and the bilingual release-branch
  policy own development guidance.

No program-ledger task state, product code, package format, or live GitHub
ruleset setting changes.

## Event and tier model

| Tier | Trigger | Work | Evidence |
| --- | --- | --- | --- |
| Focused | Local edit loop and draft PR | Smallest affected package/test command; required risk-boundary negatives; Native CI boundary may run but the native matrix does not expand | Local command result |
| Slice | Ready, current PR with any matrix-requiring path | Existing change-boundary, Linux/macOS/Windows native foundation, and Linux Lite jobs | Required checks on the final PR head |
| Evidence fast path | Ready PR whose complete diff is allowlisted evidence | Change-boundary and unchanged Evaluation Truth; each native foundation job runs one lightweight report step and skips toolchain/build/test work; Lite is skipped | Three expanded required contexts conclude successful |
| Acceptance | Explicit Native CI dispatch on current merged `2.x` | Full source matrix plus existing target packages, packaged acceptance, lifecycle checks, and promotion dispatch | Exact run, source, package, and receipt identities |

Native CI no longer has a `push` trigger. Evaluation Truth keeps its current
push, PR, and manual triggers because it is bounded to five minutes and owns the
ledger/governance files used by the fast path.

Draft PR behavior uses explicit pull-request activity types so `ready_for_review`
starts the Slice and `converted_to_draft` can cancel stale expensive work through
the existing concurrency group. A new commit on a ready PR remains a normal
`synchronize` event and starts a fresh Slice.

## Changed-path classification

The existing boundary script already calculates `base...head` once. Extend its
single loop to retain frozen-path rejection and emit a boolean job output named
`native-matrix-required`.

The default is full matrix. The result becomes false only when at least one
changed path exists and every path is one of:

- `.trellis/tasks/**`;
- `.trellis/workspace/**`;
- `docs/superpowers/acceptance/**`;
- `docs/superpowers/roadmaps/qiongli-current-program-index.md`;
- `docs/superpowers/roadmaps/qiongli-program-ledger-v1.json`;
- top-level immutable Markdown receipt files under
  `tooling/release/acceptance/`.

Nested release-acceptance paths, especially
`tooling/release/acceptance/fixtures/**`, require the full matrix. Any other
path, any mixed diff, a failed git query, or an empty diff also requires the
full matrix. This is deliberately an allowlist; it never attempts to enumerate
all risky source paths.

The script writes only a fixed boolean to `GITHUB_OUTPUT`, so changed filenames
cannot inject workflow output syntax. Its normal human-readable pass/fail output
remains available locally.

## Required-check preservation

`native-change-boundary` exposes the classifier output. `rust-native-foundation`
and `lite-runtime-compatibility` depend on it. Native build work runs only when
either:

1. the event is `workflow_dispatch`; or
2. the event is a non-draft PR and `native-matrix-required` is true.

The workflow itself still starts for every PR. A draft-only job condition may
suppress matrix expansion because a draft cannot merge and `ready_for_review`
starts a new run for the same current head. Every non-draft PR must expand the
matrix: GitHub evaluates `jobs.<job_id>.if` before applying the matrix, so using
the evidence classifier in that condition would not reliably create the three
required Linux/macOS/Windows identities. A job-level environment boolean
therefore gates every existing toolchain/build/test step, while one lightweight
report step lets each evidence-only platform job finish successfully.

This uses a small amount of native-runner startup time but avoids the observed
15-21 minutes of work per platform. `lite-runtime-compatibility` is not a
required context and may use a job-level condition. Workflow-level path filters
remain forbidden because GitHub documents that they can leave required checks
pending.

Manual dispatch ignores the classifier result. This preserves exact-candidate
qualification even if the most recent commit contains only evidence.

## Development operating loop

1. Create one business-slice branch and keep its PR draft during focused loops.
2. Run only affected local checks; run trust, schema, path, security, and
   data-loss negatives as soon as those boundaries change.
3. On Apple Silicon, native cross-platform work may add the complete macOS
   workspace plus `cargo xwin build` and `cargo xwin test --no-run` for
   `x86_64-pc-windows-msvc`. Run affected startup, persistence, and failure
   smoke paths in Windows; compilation alone is not runtime evidence.
4. Synchronize with `2.x`, freeze the slice, and mark the PR ready once.
5. Treat the single required cross-platform Slice as the merge gate. Rerun only
   an unchanged-source flaky job; any fix creates a new head and new Slice.
6. Merge and start the next bounded 2.x task immediately; there is no automatic
   post-merge source matrix to wait for.
7. Record receipts and archive Trellis work through the evidence fast path.
8. Dispatch Acceptance only for a named alpha/beta/RC checkpoint or when
   changed package inputs explicitly invalidate an existing candidate.

This removes repeated work from the critical path without allowing Slice green
status to authorize release.

## Tests and observability

`tests/test_native_change_boundary.py` will prove:

- Trellis/receipt-only diffs emit false;
- release fixtures emit true;
- mixed and ordinary source diffs emit true;
- empty diffs emit true;
- frozen paths still fail before classification can weaken the guard.

`tests/test_branch_policy.py` will prove:

- Native CI has PR and manual triggers but no push trigger;
- ready/draft activity types are declared;
- job conditions consume the boundary output;
- draft events suppress matrix expansion, while all three contexts expand on a
  ready evidence fast path and every expensive foundation step is gated;
- all required context names and three platform commands remain unchanged;
- every manual candidate job remains dispatch-only.

The implementation PR necessarily selects the full matrix because it changes
workflow and script inputs. After merge, the next evidence-only closeout PR is
the first live observation of skipped foundation contexts; Evaluation Truth
must still pass.

## Compatibility and rollout

- Existing ruleset `18800504` and its five required contexts are unchanged.
- Existing package and promotion workflows are unchanged.
- Existing candidate receipts remain bound to exact source/run identities.
- Existing ordinary PRs continue to run the full matrix unless they are drafts
  or entirely evidence-only.
- General documentation remains full-matrix by default. Expand the allowlist
  only after repeated measured cases establish another safe class.

## Risks and rollback

- **Misclassification:** the allowlist is narrow, mixed/unknown/empty diffs run
  full, and fixtures are explicitly negative-tested.
- **Required checks remain pending:** allow matrix-wide suppression only while
  draft/cancelled; on ready PRs keep the matrix expandable and gate its steps,
  not the whole matrix or workflow. If the first live fast-path PR does not
  produce three successful contexts, revert the condition before merging it.
- **Candidate misses source checks:** `workflow_dispatch` forces all source and
  candidate jobs regardless of classification.
- **Draft never receives Slice:** `ready_for_review` is an explicit trigger and
  branch policy still requires all contexts on the latest mergeable head.
- **Governance drift:** update the product-control spec, English policy, Chinese
  policy, and executable tests in the same change.

Rollback is one normal revert of the script, workflow, tests, and policy text.
No user data, package, release, or repository-setting rollback is involved.
