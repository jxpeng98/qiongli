# Design

## Boundary

Keep the single-case truth engine unchanged. Add one thin multi-case owner next
to it:

```text
evals/runner/run_suite.py
  -> sorted academic-quality case YAML
  -> evals.runner.run_eval.run_case(case, fixture/case-id)
  -> case pass/fail counts and process exit
```

`tooling/scripts/run_academic_quality_evals.py` re-exports and delegates to this
owner. The existing root `scripts/` wrapper continues delegating to `tooling/`.
No compatibility path retains its own iteration or success predicate.

## Canonical Command

```bash
python evals/runner/run_suite.py
```

With no arguments it resolves the repository-owned academic-quality cases and
fixtures from the command file's repository root. A positional case directory
and optional fixture root retain focused/custom execution. Resolution must not
depend on the caller's current working directory.

The suite result preserves `case_count`, `passed_cases`, `failed_cases`, and
`success`. Success remains `case_count > 0 and failed_cases == 0`; per-case truth
continues to come only from `run_case`.

## EVAL-408 Fixture Contract

Place the six adversarial families under one test-fixture root. Each directory
contains a V1 case and only the minimal output material needed to demonstrate
its failure:

| Fixture | Required observation |
|---|---|
| empty-project | required artifact missing from a nonexistent output root |
| missing-artifact | required artifact missing from an otherwise present root |
| malformed | case load blocked with `case-load-failed` |
| keyword-only | detailed required evidence fails `contains_all` |
| contradictory | cross-artifact equality fails |
| stale-artifact | exact-byte digest fails |

The focused test reads the structured result and asserts case status, reason,
and relevant truth counter/assertion reason. It does not bless a generic
non-zero exit as sufficient evidence.

## CI Ownership

Add a dedicated `evaluation-truth.yml` workflow for `2.x` with checkout, Python
3.12, PyYAML 6.0.3, and the canonical no-argument command. A separate workflow
is required because Native CI intentionally proves that the shipped native
runtime starts no Python or Node toolchain. This keeps command ownership visible
without enabling the large legacy compatibility workflow for `2.x`.

## Compatibility

- Existing imports of `run_evals`, `EvalRunResult`, and `main` keep working.
- `scripts/run_academic_quality_evals.py <case-dir>` keeps its current behavior.
- The single-case CLI and opt-in per-case receipts stay unchanged.
- The batch command does not aggregate receipts; EVAL-407 deliberately made
  per-case receipts portable, and EVAL-410 does not require a new receipt schema.

## Risks And Rollback

- Risk: path defaults accidentally depend on the checkout working directory.
  Resolve them from `__file__` and execute the CLI from a non-root test cwd.
- Risk: compatibility shims drift. Test canonical and legacy CLIs against the
  same temporary suite and compare exit/summary behavior.
- Risk: adding the job to Native CI would violate its runtime boundary. Keep a
  source-contract test for both the dedicated command and existing branch rule.
- Rollback is one code/workflow commit. Fixtures and tests are read-only and do
  not mutate project or Host state.
