# Make one eval command own CI

## Goal

Close the missing adversarial-fixture prerequisite, then make one canonical
Evaluation Truth V1 suite command the direct academic-quality gate in 2.x CI.
This removes indirect unit-test-only confidence without creating another
validator or changing release status.

## Background

- The master roadmap leaves `EVAL-408`, `EVAL-410`, and `EVAL-411` open while
  `EVAL-409` is complete on the current branch.
- The roadmap audit classifies `EVAL-410` as dependent on the `EVAL-408` and
  `EVAL-409` fixture corpus. Starting CI ownership before `EVAL-408` would
  standardize an incomplete suite.
- `evals/runner/run_eval.py` already owns V1 case parsing, typed assertions,
  truth counters, receipts, and fail-closed exit status.
- `tooling/scripts/run_academic_quality_evals.py` owns the current 12-case batch,
  while `scripts/run_academic_quality_evals.py` is a repository compatibility
  wrapper. No active 2.x CI job invokes the batch command directly.
- `.github/workflows/native-ci.yml` deliberately forbids Python and Node so the
  shipped native runtime stays independent. The eval gate therefore needs its
  own `2.x` workflow instead of weakening that boundary or enabling legacy CI.

## Requirements

### R1. Complete EVAL-408 before changing CI ownership

- Check in exactly the six named adversarial fixture families: empty project,
  missing artifact, malformed case, keyword-only evidence, contradictory
  artifacts, and stale artifact.
- Exercise the fixtures through the existing V1 runner. Each fixture must fail
  for its intended counter or stable reason code; no fixture may rely only on
  an exception or a free-form output substring.
- Keep the fixtures offline, deterministic, bounded, and free of research data,
  Host state, network access, models, credentials, and absolute paths.

### R2. Establish one canonical multi-case command

- Move the existing academic-quality batch loop under `evals/runner/` and reuse
  `run_case`; do not duplicate parsing, assertions, counters, or success logic.
- The canonical command must run the repository's 12 checked-in V1
  academic-quality cases by default, print pass/fail case counts, and return
  non-zero for an empty case set or any failed/blocked case.
- Preserve explicit case-directory and fixture-root inputs needed by focused
  tests and downstream maintainers.

### R3. Preserve compatibility without duplicate implementations

- Keep both existing academic-quality Python entry paths working as thin,
  tested compatibility shims over the canonical batch owner.
- Preserve the current `run_evals` result fields and the positional case
  directory behavior. Additive options are allowed.

### R4. Make the command directly own 2.x CI

- Add one explicit Evaluation Truth workflow for `2.x` pull requests and pushes.
- CI must invoke the canonical command itself, not depend on unit-test discovery
  to reach it and not invoke a legacy wrapper.
- Pin only the already-used Python and PyYAML runtime needed by the command.
- Keep Native CI free of Python, Node, and legacy-language test orchestration.

### R5. Record only proven roadmap progress

- Mark `EVAL-408` and `EVAL-410` complete only after focused, full, and CI-shape
  checks pass.
- Leave `EVAL-411`, the M1 exit gate, target-branch acceptance, Alpha
  qualification, and publication open.

## Acceptance Criteria

- [x] All six EVAL-408 fixture families exist and each produces its asserted
      fail-closed counter/status/reason through the shared runner.
- [x] The canonical no-argument suite command runs all 12 academic-quality
      cases, reports `12 passed, 0 failed`, and exits zero.
- [x] An empty suite and a suite containing a failed or blocked case exit
      non-zero.
- [x] The canonical and legacy entry paths return the same result for the same
      case and fixture roots; only the canonical module owns the batch loop.
- [x] One dedicated 2.x workflow directly invokes the canonical suite, while
      Native CI remains legacy-runtime-free.
- [x] Focused eval tests, the full Python suite, strict research validation,
      `py_compile`, and `git diff --check` pass.
- [x] Only `EVAL-408` and `EVAL-410` are newly checked after verification.

## Out Of Scope

- EVAL-411's broader evidence-deletion and count-mutation program.
- Rewriting controller-mode, subject-router, or subject-specialization evaluators
  into V1; they are separate pre-existing contracts.
- Suite-level JSON/JUnit aggregation, hosted dashboards, model judges, live
  Plugin runs, benchmark scoring, or new dependencies.
- Native or legacy CI runtime-boundary changes, release preflight policy,
  App/Host behavior, tags, merges, publication, or release qualification.

## Open Questions

None. The roadmap dependency, existing runner boundary, and active 2.x CI
workflow determine the minimum implementation.
