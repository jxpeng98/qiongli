# Make Plugin quality executable

## Goal

Replace the current academic-quality score theater with deterministic,
executable artifact findings, then repair the smallest canonical Plugin content
batch already proven incomplete.

This P1 task improves the bundled Plugin's contract and regression quality. It
does not claim that a model's scholarly output improved without an observed
model evaluation.

## Dependency

Start only after `.trellis/tasks/08-13-close-app-cli-plugin-activation` is
accepted or explicitly deferred with evidence. The execution head must contain
Evaluation Truth V1 (`EVAL-401` through `EVAL-407`); integrate that prerequisite
instead of reimplementing it.

## Confirmed Baseline

- `tooling/scripts/run_academic_quality_evals.py` reads 12 YAML files and
  averages their declared `expected_dimensions`. It never evaluates an input,
  Plugin output, or finding.
- `evals/runner/run_eval.py` already owns typed artifact assertions, contained
  paths, deterministic receipts, and the five-clause fail-closed predicate.
- The canonical Skill audit currently scans 82 files and reports 74 complete.
  The stale checked-in report still says 71/71.
- The eight incomplete canonical files are:
  - `content/skills/L_coursework/assignment-brief-analyzer.md`;
  - `content/skills/L_coursework/coursework-architect.md`;
  - `content/skills/L_coursework/coursework-reviser.md`;
  - `content/skills/L_coursework/rubric-mapper.md`;
  - `content/skills/M_dissertation/chapter-architect.md`;
  - `content/skills/M_dissertation/dissertation-planner.md`;
  - `content/skills/M_dissertation/dissertation-readiness-checker.md`;
  - `content/skills/M_dissertation/supervisor-feedback-integrator.md`.
- Their exposed gaps are explicit insufficient-input behavior, calibrated
  finding/interpretation/implication rules, and—on five files—an explicit ban
  on invented citations/data.
- Canonical sources live under `content/`; generated Codex/Claude Plugin trees
  are outputs and must not be hand-edited.

## Requirements

### Q1. Make the 12 academic-quality cases executable

- Remove every `expected_dimensions` field and all aggregate dimension-score
  output.
- Convert each case to Evaluation Truth V1 shape with a real input topic,
  required captured artifact(s), and typed assertions for exact expected
  findings or positive evidence.
- Store bounded case outputs under a deterministic academic-quality fixture
  root. Negative cases must require their named gate/severity/finding; positive
  cases must require concrete scholarly contract evidence rather than a number.
- Reuse `evals/runner/run_eval.py` for truth. The academic-quality entry point
  may remain only as a thin 12-case batch wrapper; it must not recalculate
  quality or preserve the declared-score API.
- Missing input/output, zero executed assertions, malformed case data, unknown
  assertion types, or a missing expected finding must make the batch non-zero.
- Add one mutation regression showing that removing a required finding from a
  previously passing captured artifact makes that case fail. This is bounded
  EVAL-409 evidence, not a claim that the full EVAL-411 mutation program is done.

### Q2. Repair only the current canonical Skill batch

- Update the eight named files at their existing Inputs, Process, Quality Bar,
  or Common Pitfalls owner; do not paste keywords into unrelated prose.
- Each file must state task-specific behavior when required inputs are missing
  or insufficient.
- Each file must distinguish evidence-backed finding, interpretation, and
  implication at the strength the available evidence permits.
- Where applicable, explicitly forbid inventing citations, sources, data,
  sample sizes, rules, feedback, statistics, or results.
- Preserve each Skill's existing purpose, stage, artifact paths, degree/course
  boundaries, and platform-neutral behavior.
- If executable cases expose a content owner outside these eight files, pause
  and return to planning unless the fix is a shared sentence in an already
  listed canonical owner.

### Q3. Verify the actual bundled Plugin payload

- Regenerate `docs/maintainer/skill-quality-gap-report.md` from the canonical
  audit; do not edit its counts manually.
- Require the strict audit to report 82/82 complete.
- Materialize all distribution targets into a fresh temporary directory and
  run the existing distribution/capability audits.
- Verify the staged Codex and Claude Plugin bundles contain the canonical
  `qiongli-workflow` and the repaired content through existing bundle tests.
- Do not commit generated package mirrors or temporary eval receipts.

### Q4. Preserve honest quality claims

- Captured artifact fixtures prove deterministic detection/contract behavior,
  not live model performance, factual correctness, or expert agreement.
- `claude plugin eval` with ablation may be run later under explicit model/cost
  authorization; it is not CI and not required here.
- Mark only `EVAL-409` complete after all deterministic checks pass. Do not mark
  EVAL-410, EVAL-411, M1 exit, Alpha qualification, or Stable readiness.

## Acceptance Criteria

- [ ] All 12 academic-quality cases have Evaluation Truth V1 inputs, required
      artifact assertions, and no `expected_dimensions` or self-declared scores.
- [ ] The academic-quality command delegates case truth to the existing V1
      runner, reports case pass/fail counts, and exits non-zero on any blocked,
      failed, empty, or unexecuted case.
- [ ] Removing one required expected finding makes the focused mutation test
      fail.
- [ ] Focused tests cover all 12 cases, the batch failure predicate, and path /
      malformed-data containment without network or model calls.
- [ ] The eight named canonical Skills express task-specific insufficient-input,
      claim-calibration, and non-fabrication behavior where required.
- [ ] `python3 scripts/audit_skill_sections.py --strict` reports 82/82 complete,
      and the generated report matches that result.
- [ ] Capability validation, staged all-target distribution audit, and existing
      Codex/Claude Plugin bundle tests pass from canonical content.
- [ ] No generated Plugin mirror, raw model output, private path, credential, or
      temporary receipt is committed.
- [ ] Completion wording is limited to executable fixture and Skill-contract
      quality; no live model-quality or release claim is made.

## Out of Scope

- Live/paid model execution, LLM judges, cross-model ablation, expert review,
  benchmark score targets, or CI credentials.
- Rewriting all 82 Skills, changing the Skill registry, adding a new evaluator
  framework, or completing EVAL-408/EVAL-410/EVAL-411.
- Changes to Plugin activation, App UI, Host cache state, public release, or
  later Research Kernel/Gate runtime milestones.
