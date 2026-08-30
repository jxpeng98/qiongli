# REL-910 implementation plan

## 1. Correct workflow identity and control

- [x] Add a default-false publication-authorization workflow input.
- [x] Bind `build_run_url` to the current promotion run attempt instead of the
      qualifying Native CI run.
- [x] Gate only the protected authorization job on the explicit input; preserve
      all rebuild, aggregate, and authorization internals.

## 2. Add the minimum regression guard

- [x] Extend the existing branch-policy test with the exact build-attempt URL,
      default-false input, safe dispatch, and gated authorization invariants.
- [x] Run the focused branch-policy method and closest Community Alpha Rust
      contract tests.

## 3. Freeze and merge the implementation

- [x] Run Trellis task validation and `git diff --check`.
- [x] Commit, open a PR, wait for required Slice CI, and merge to `2.x`.

## 4. Produce and verify one exact candidate

- [x] Dispatch Native CI on the exact merged `2.x` source.
- [x] Wait for successful three-target candidate aggregation with protected
      authorization skipped.
- [x] Download the candidate into a private temporary directory and verify its
      canonical receipt, source/build identities, target/file inventories,
      sizes, and SHA-256 values.

## 5. Accept and close REL-910

- [x] Add the source-bound acceptance receipt.
- [x] Set REL-910 to `accepted`, regenerate the Program Ledger index, and run
      the focused evidence/roadmap/task/diff checks.
- [x] Commit and merge the evidence-only closeout, archive the Trellis task,
      record the session, and merge the archive commit.

## Risk and rollback points

- Never record the qualifying Native CI URL as the builder of artifacts rebuilt
  by the promotion run.
- Never request or infer protected publication authorization during REL-910.
- Do not claim production signing or byte-identical reproducibility.
- If an exact-source or digest check fails, discard the candidate and rerun only
  from the current accepted `2.x` source after fixing the smallest owner.
