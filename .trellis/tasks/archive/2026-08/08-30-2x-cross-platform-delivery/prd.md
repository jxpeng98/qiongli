# Accelerate 2.x cross-platform delivery

## Goal

Shorten the Qiongli `2.x` delivery cycle without weakening Windows, macOS, or
release-candidate evidence. Normal development must receive focused feedback,
one frozen business slice must receive one cross-platform source matrix, and
only an explicit release candidate must receive target-package Acceptance.

This task accelerates the existing 2.0 replacement critical path. It does not
pull deferred post-2.0 roadmap work into the release scope.

## Background

- The repository already owns Focused, Slice, and Acceptance tiers in product
  control and the bilingual release-branch policy.
- The protected `2.x` ruleset requires the change-boundary, Linux, macOS,
  Windows, and Evaluation Truth contexts and requires PR branches to be current
  with `2.x`; there is no bypass actor.
- `Native CI` currently repeats its full source matrix on PR updates and merge
  pushes, including evidence-only and Trellis-closeout changes.
- The 2026-08-30 audit observed 29 Native CI runs: 13 PR, 11 push, and 5 manual
  dispatch runs. An ordinary run takes about 20-21 wall-clock minutes and
  roughly 55-58 aggregate runner-minutes.
- Manual dispatch already owns exact candidate packages, packaged acceptance,
  lifecycle checks, and promotion.
- GitHub reports a conditionally skipped job as a successful check, including
  when that job is required. Skipping the whole workflow with path filters would
  instead leave required checks pending.

## Requirements

- Keep Focused development local and package-specific. Draft PR updates must not
  start the expensive native source matrix.
- Let Apple Silicon maintainers complete the native development loop on macOS:
  run the macOS workspace locally, use third-party `cargo-xwin` to build the
  Windows x64 release and compile Windows tests, then run affected smoke paths
  in Windows. Cross-compilation is feedback, not Windows runtime evidence.
- Make the final ready PR head the sole automatic Slice authority. A
  non-evidence change must run the existing Linux, macOS, and Windows source
  jobs before merge.
- Remove automatic Native CI source-matrix runs from merge pushes. The strict
  up-to-date PR checks qualify the merged tree; an explicit candidate dispatch
  remains the exact merged-commit authority.
- Preserve the existing required status-check names and ruleset configuration.
  The Native CI workflow must still start on every PR targeting `2.x`. Draft
  PRs may skip the matrix before expansion because they cannot merge; every
  ready PR must create all three platform identities, conditionally skipping
  only their expensive steps.
- Skip Rust toolchain setup, frontend builds, Lite compatibility, and workspace
  tests only when every changed path belongs to the narrow evidence-only set:
  Trellis task history, Trellis workspace journals, acceptance receipts, or the
  generated Program Ledger/current index.
- Treat release fixtures, native or frontend source, tests, CI, lockfiles,
  scripts, policy inputs, and every unknown path as matrix-requiring. An empty
  or unclassifiable diff must also fail safe to the full matrix.
- Make every explicit `workflow_dispatch` on `2.x` run the complete source
  matrix and all existing candidate jobs regardless of path classification.
- Keep Evaluation Truth automatic and unchanged; it remains the inexpensive
  owner of architecture, authorization, evaluation, ledger, and generated-index
  truth.
- Keep failure ownership platform-specific. A flaky unchanged-source job may be
  rerun alone; a source fix creates a new final PR head and invalidates the old
  Slice result.
- Update executable policy tests, the product-control specification, and both
  language copies of the branch policy with the same trigger contract.

## Acceptance Criteria

- [x] Native CI starts automatically for PRs targeting `2.x` and by explicit
      dispatch, but not for pushes to `2.x`.
- [x] The four existing Native CI required context names remain unchanged.
- [ ] Draft PR updates do not expand the native foundation matrix; marking the
      same PR ready creates the required platform contexts on its current head.
- [ ] Allowlisted evidence-only ready PRs create all three required foundation
      contexts, complete them successfully through a lightweight fast-path
      step, and skip every toolchain/build/test step.
- [ ] Marking a matrix-requiring draft PR ready starts Linux, macOS, Windows,
      and Lite Slice jobs for that exact current PR head.
- [x] Any non-allowlisted, mixed, empty, or unclassifiable diff selects the full
      source matrix.
- [x] Release acceptance fixtures under `tooling/release/acceptance/fixtures/`
      select the full matrix even though top-level immutable Markdown receipts
      may use the evidence-only path.
- [x] A normal product slice performs at most one automatic full matrix for its
      final unchanged PR head and performs no post-merge duplicate, reducing
      automatic full-matrix work by at least 50% relative to the current
      PR-plus-push flow.
- [ ] An acceptance-receipt plus Trellis-archive closeout sequence performs zero
      full native matrices when no product, package, test, fixture, or CI input
      changes.
- [x] Explicit candidate dispatch always runs all three native source jobs,
      three target package/lifecycle paths, packaged acceptance, and the
      dependent promotion flow.
- [x] Policy-focused tests, the native change-boundary tests, authorization
      policy validation, and `git diff --check` pass locally.
- [x] The complete native Rust workspace passes on Apple Silicon macOS.
- [x] `cargo-xwin` builds the Windows x64 release workspace and compiles all
      Windows test targets with `--no-run`.
- [x] The three Windows release executables inspect as PE32+ x86-64 and retain
      identical SHA-256 hashes after transfer into Windows 11 Arm.
- [x] Windows 11 Arm x64 emulation passes CLI version/content, desktop startup,
      isolated persistence, restart readback, doctor, and revision-conflict
      smoke paths without claiming native x64 hardware validation.
- [x] The implementation PR itself completes one full ready-PR Slice because it
      changes CI policy; no candidate Acceptance run is claimed by this task.

## Out of Scope

- New CI services, runners, caches, test frameworks, or dependencies.
- Replacing the native Windows required context with `cargo-xwin`, or claiming
  signing, installer, clean-machine, performance, or native x64 certification.
- Reducing the commands inside one required Windows/macOS/Linux source job.
- Automatically classifying all documentation as evidence-only; the first
  allowlist remains intentionally narrow and fail-safe.
- Changing live GitHub ruleset identifiers, required contexts, merge methods,
  or repository permissions.
- Running or publishing a Community Alpha candidate.
- Implementing the remaining `2.x` product backlog or deferred post-2.0 work.

## Key Decisions

- Optimize by deleting redundant runs before attempting cache or compiler
  tuning.
- Use PR-only automatic Slice validation and explicit dispatch for exact merged
  candidate Acceptance.
- Preserve the three expanded contexts on every ready PR with step-level
  conditions. A matrix-wide condition may suppress draft/cancelled work only;
  workflow-level path filtering remains forbidden.
- Keep this as one atomic task because workflow behavior, classifier tests, and
  policy text must land together.
