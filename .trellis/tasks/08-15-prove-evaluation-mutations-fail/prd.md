# Prove evaluation mutations fail

## Goal

Close `EVAL-411` by proving that two concrete mutations turn the same freshly
passing Evaluation Truth V1 case non-green: deleting required evidence and
changing a conserved count.

## Background

- `tests/test_eval_cases.py` already owns a temporary scientific case whose
  nine assertions pass before mutation.
- The existing mismatch test changes a PRISMA total and checks failure, but it
  does not prove the same case passed immediately before that mutation.
- Academic-quality coverage removes required finding text, while `EVAL-411`
  explicitly requires deletion of evidence and count mutation.
- Diagnostic execution confirms the existing runner already returns the exact
  required outcomes; no product-code fix or new fixture family is needed.

## Requirements

### R1. Prove the baseline in each mutation case

- Materialize the existing temporary scientific case independently for each
  mutation.
- Evaluate it before mutation and require `case-passed`, nine executed
  assertions, zero missing/failed/blocked/unknown counters, and success.

### R2. Prove deleted evidence fails closed

- Delete the required `record.json` evidence from the passing case.
- Require non-success with case status `fail`, reason
  `required-artifact-missing`, `required_missing == 1`, and a matching failing
  `schema` outcome.

### R3. Prove changed counts fail semantically

- Change `Records screened` from `5` to `6` while its parts remain `2 + 3`.
- Require non-success with case status `fail`, reason `assertion-failed`,
  `failed_assertions == 1`, and a `count-conservation-failed` outcome.

### R4. Keep one truth owner and one focused test owner

- Reuse `_materialize_scientific_case` and `_evaluate_case`; do not add a
  mutation framework, new fixtures, dependencies, runner branches, or a second
  success predicate.
- Fold the assertions into the existing scientific mismatch coverage instead
  of creating overlapping tests.
- Keep all mutation work in disposable temporary directories.

### R5. Record only proven progress

- Mark `EVAL-411` complete only after focused, full-suite, strict-validation,
  diff-hygiene, and exact-head CI checks pass.
- Do not claim M1 exit, target-branch integration, Alpha qualification, or
  publication authority.

## Acceptance Criteria

- [x] Each mutation starts from a freshly evaluated passing case with the exact
      baseline counters.
- [x] Deleting required evidence produces the exact missing-artifact status,
      reason, counter, and outcome.
- [x] Changing the conserved count produces the exact assertion-failure status,
      counter, and outcome.
- [x] Existing scientific validator mismatch coverage remains intact without a
      new runner, fixture corpus, dependency, or test framework.
- [x] Focused eval tests, the canonical 12-case suite, the full Python suite,
      strict research validation, and diff hygiene pass.
- [x] Only `EVAL-411` is newly checked, while M1 and release claims remain open.

## Out Of Scope

- New mutation infrastructure, generated mutants, persistent receipts, or new
  adversarial fixture directories.
- Changes to `run_eval.py`, `run_suite.py`, case schemas, validators, or CI
  workflow topology unless the focused test exposes a real defect.
- Mutation coverage for every validator or every academic-quality fixture.
- Governance, security, platform-baseline, M1 exit, merge, tag, or release work.
