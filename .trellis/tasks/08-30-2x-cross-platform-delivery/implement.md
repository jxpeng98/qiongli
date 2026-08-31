# Implementation plan: accelerate 2.x cross-platform delivery

## 1. Start only after final planning approval

- [x] Obtain explicit approval of the final PRD/design/implementation summary.
- [x] Run `task.py start`, then load `trellis-before-dev` and the product-control
      guidance before editing product-owned files.
- [x] Record and preserve unrelated worktree changes; limit edits and validation
      review to this task's files.

## 2. Lock the behavior with focused tests

- [x] Extend `tests/test_native_change_boundary.py` with evidence-only, nested
      fixture, mixed/source, empty-diff, and fixed-output cases.
- [x] Update `tests/test_branch_policy.py` to require PR/manual-only Native CI,
      explicit draft/ready events, classifier-driven job conditions, unchanged
      context names, and dispatch-only candidate jobs.
- [x] Run the focused tests once to confirm they fail for the current workflow.

## 3. Reuse the native change-boundary classifier

- [x] Extend `scripts/check_2x_native_change_boundary.sh` to calculate one
      conservative `native-matrix-required` decision while retaining every
      frozen-path rejection.
- [x] Allow only Trellis history/journals, named acceptance evidence, generated
      program state, and top-level immutable Markdown release receipts to skip.
- [x] Make nested release fixtures, mixed/unknown paths, and empty diffs require
      the full matrix.
- [x] Emit a fixed boolean to `GITHUB_OUTPUT` when present and a concise local
      summary without adding a new script or dependency.

Focused checks:

```bash
bash -n scripts/check_2x_native_change_boundary.sh
python3 -m unittest tests.test_native_change_boundary -v
```

## 4. Make ready PR the sole automatic Slice

- [x] Remove the `push` trigger and its obsolete `paths-ignore` block from
      `.github/workflows/native-ci.yml`.
- [x] Declare PR activity types for opened, synchronized, reopened,
      ready-for-review, and converted-to-draft events.
- [x] Export the boundary script's matrix decision from
      `native-change-boundary`.
- [x] Make `rust-native-foundation` depend on that output but always expand its
      three-platform matrix for non-draft PRs and manual dispatches unless the
      run is cancelled. Draft/converted-to-draft events may suppress expansion.
      Store the evidence run/skip decision in a job environment boolean, run
      one lightweight fast-path report when false, and gate every existing
      toolchain/build/test step when true.
- [x] Make non-required `lite-runtime-compatibility` run only for ready
      matrix-requiring PRs or any manual dispatch.
- [x] Preserve all three platform entries, commands, required job names,
      concurrency cancellation, and dispatch-only candidate conditions.

Focused check:

```bash
python3 -m unittest tests.test_branch_policy -v
```

## 5. Synchronize policy authority

- [x] Update the Three-tier verification scenario in
      `.trellis/spec/product/control/index.md`: draft/Focused, ready-PR/Slice,
      evidence-fast-path, no merge-push duplicate, explicit Acceptance.
- [x] Update `docs/maintainer/release-branch-policy.md` and
      `docs/zh/maintainer/release-branch-policy.md` with equivalent operational
      rules and nonclaims.
- [x] Preserve the 1.x freeze, exact-candidate evidence, 90-day support window,
      and release-authorization boundaries.

## 6. Run the task-scope quality gate

- [x] Run shell syntax and the two focused policy modules.
- [x] Validate the repository authorization/ruleset contract, which must retain
      the same required contexts.
- [x] Run whitespace/diff validation.
- [x] Inspect the final diff for fail-safe classification and synchronized
      English/Chinese/spec wording.

Validation commands:

```bash
bash -n scripts/check_2x_native_change_boundary.sh
python3 -m unittest \
  tests.test_native_change_boundary \
  tests.test_branch_policy \
  tests.test_authorization_policy \
  -v
python3 tooling/scripts/validate_authorization_policy.py
git diff --check
```

## 7. Establish the macOS-first Windows feedback loop

- [x] Confirm Apple Silicon, the pinned Rust toolchain, native workspace,
      Windows-specific dependencies/build output, installed LLVM, and Parallels.
- [x] Install the missing Windows MSVC target and reuse `cargo-xwin 0.23.1`
      after explicit Microsoft SDK licence approval.
- [x] Run the complete macOS workspace tests, Windows x64 release cross-build,
      Windows test compilation, PE inspection, hashes, and Windows 11 Arm smoke.
- [x] Keep the generated deterministic `windows-schema.json` alongside the
      existing desktop/macOS Tauri schemas.
- [x] Document exact commands and nonclaims in product control and both branch
      policy languages without adding a wrapper, dependency, or CI service.
- [x] Rerun Rust format/Clippy, policy tests, Windows test compilation, and diff
      validation after the final edits.

## 8. Slice and rollout evidence

- [ ] Keep the implementation PR draft until the focused gate passes, then
      mark it ready once on a current `2.x` base.
- [ ] Require the unchanged change-boundary, Linux, macOS, Windows, and
      Evaluation Truth contexts on that final head. This is the task's Slice.
- [ ] Do not manually dispatch candidate Acceptance for this CI-policy task.
- [ ] After merge, confirm no Native CI push run starts.
- [ ] Use the next evidence-only closeout PR to observe three lightweight
      successful native foundation contexts while Evaluation Truth remains
      green.

## Rollback points

- Classifier script and classifier tests revert together.
- Native CI triggers/conditions and branch-policy tests revert together.
- Product-control and bilingual policy text revert with workflow behavior.
- A live required-context mismatch blocks the fast-path PR; revert before merge
  rather than changing the ruleset or bypassing checks.
