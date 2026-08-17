# Close evaluation false-green cases

## Goal

Make the existing shared golden-eval runner fail honestly when required
evidence is missing or no executable validation ran. This task closes the
roadmap's `EVAL-401` through `EVAL-405` slice without starting the later
scientific-validator, receipt, CI-owner, or academic-quality work.

## Confirmed Facts

- `evals/runner/run_eval.py::run_case` is the only owner of the four shared
  golden cases and is imported directly by `tests/test_eval_cases.py`.
- A missing artifact or empty artifact directory currently prints `SKIP`, and
  the case still succeeds because the final predicate is only `failed == 0`.
- The four cases use executable-but-unversioned `must_contain` lists plus
  unused free-form `validation` strings. Their only required semantics are
  "contains all" and "contains any".
- The repository already depends on PyYAML and CI discovers
  `tests/test_eval_cases.py`; no dependency or new test harness is needed.

## Requirements

### R1 — Explicit artifact presence

- Every entry in the four shared cases must declare `required: true` or
  `required: false`.
- A missing or unusable required artifact is non-success. A missing optional
  artifact may be skipped, but cannot make an otherwise empty case pass.

### R2 — Versioned typed assertions

- Each case must declare `schema_version: "1.0"`.
- Replace `must_contain` and scalar `validation` fields with an `assertions`
  list containing typed assertion objects.
- Version 1 supports only the two validators required by current cases:
  `contains_all` and `contains_any`, each with a non-empty string `values`
  list. Unknown types and unsupported versions are non-success.

### R3 — Honest success predicate

The runner may return success only when all conditions hold:

```text
required_missing == 0
executed_assertions > 0
failed_assertions == 0
blocked_assertions == 0
unknown_validation_types == 0
```

### R4 — Failure semantics

- Malformed YAML, malformed case/assertion data, artifact read failure,
  unavailable validators, and required SKIP/BLOCKED states must return false
  and make the CLI exit non-zero.
- A present optional artifact is validated normally; only its absence may be
  skipped.
- The runner must report enough counters/reasons in its existing text output
  to explain why the predicate failed.

### R5 — Minimal compatibility boundary

- Preserve the existing `run_case(case_path, output_dir) -> bool` API and CLI
  invocation.
- Migrate all four owned cases atomically; do not retain a legacy parser that
  silently accepts `must_contain` or free-form `validation`.
- Use the existing Python standard library and PyYAML only.

## Acceptance Criteria

- [x] All four shared cases use schema version 1, explicit requiredness, and
      typed assertions; none contains `must_contain` or scalar `validation`.
- [x] Minimal valid fixture output still passes every shared case.
- [x] An empty output directory and a missing/empty required artifact fail.
- [x] A case with zero executed assertions fails, even when no assertion failed.
- [x] Unknown assertion types, unsupported schema versions, malformed assertion
      payloads, artifact read failures, and malformed YAML fail rather than
      raising an uncaught success-path exception or being ignored.
- [x] A missing optional artifact is allowed only when at least one other typed
      assertion executes successfully.
- [x] `python -m unittest tests.test_eval_cases -v` and the one final
      repository unit-test pass succeed; `git diff --check` is clean.
- [x] The master roadmap marks only `EVAL-401` through `EVAL-405` complete after
      the checks pass, without changing M0 or Alpha 3 publication status.

## Out Of Scope

- EVAL-406 scientific validators, EVAL-407 JSON/JUnit receipts, EVAL-408
  adversarial fixture expansion, EVAL-409 academic-quality conversion, and
  EVAL-410 CI command consolidation.
- Changes to the academic-quality, controller-mode, or subject-specialization
  evaluators.
- New dependencies, schema frameworks, eval services, UI work, providers,
  release packaging, exact-head CI, tags, or Community Alpha publication.

## Open Questions

None. The roadmap and repository evidence fully determine this slice.

## Notes

- This task targets Alpha 4 development and does not claim that M0 has exited.
- At the user's direction, stale distribution and native-release baseline
  failures blocking the final repository gate were repaired as prerequisite
  maintenance. Those changes do not expand the EVAL roadmap slice or grant
  release/publication authority and should be committed separately.
- Final evidence on 2026-08-12: focused eval tests passed; the repository suite
  passed all `1737` tests with `18` environment skips; `git diff --check` and
  Capability Contract v2 validation passed.
