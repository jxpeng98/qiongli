# Implementation Plan

## 1. Lock The Failing Contracts

- [x] Add the six bounded EVAL-408 fixture families.
- [x] Add one table-driven test asserting each fixture's case status, stable
      reason, assertion outcome, and relevant truth counter.
- [x] Run the focused test and confirm all six scenarios are observed through
      the existing runner.

## 2. Move Batch Ownership

- [x] Add `evals/runner/run_suite.py` by moving the current batch loop and result
      contract from the academic-quality script.
- [x] Default the canonical command to the 12 repository cases and fixtures;
      retain explicit case/fixture inputs and fail closed on zero cases.
- [x] Replace the tooling implementation with a compatibility delegation and
      leave the root wrapper unchanged.
- [x] Add CLI tests for no-argument canonical execution, non-root cwd path
      resolution, empty/failing suites, and legacy parity.

## 3. Transfer CI Authority

- [x] Add a dedicated `2.x` Evaluation Truth workflow using Python 3.12, pinned
      PyYAML, and only `python evals/runner/run_suite.py` as the eval invocation.
- [x] Add a source contract test that prevents CI from reverting to the legacy
      wrapper, indirect unit-test ownership, or Python inside Native CI.

## 4. Verify And Record

- [x] Run focused eval and CI-source tests.
- [x] Run the canonical command directly.
- [x] Run the full Python suite and strict research-standard validation.
- [x] Run `py_compile`, `git diff --check`, and Trellis check/update-spec.
- [ ] Mark only EVAL-408/EVAL-410 complete, commit, push, monitor PR checks, and
      archive the task after all checks are green.

## Review Focus

- The multi-case path must call the existing single-case truth owner.
- Empty, failed, blocked, and malformed inputs must never produce suite success.
- Compatibility modules must not retain a second batch loop.
- CI must invoke the canonical path directly on the active `2.x` workflow.

## Rollback Point

Revert the implementation commit. No project data, research artifacts, Host
state, package identity, release receipt, or remote policy is mutated.
