# Real Local-Agent Smoke And Runtime Hardening Design

## Goal

Turn the current preview-first subject runtime smoke into a release-grade
verification path for maintainers.

Qiongli should keep preview smoke as the default release gate, but it should
also provide an explicit opt-in local-agent smoke that can prove a real task run
loads project-local subject guidance, writes trace bundles inside an isolated
project root, and reports failures with enough context for diagnosis.

This is the third adaptive subject runtime slice after lifecycle controls and
subject guidance materialization. It does not add new subjects.

## Current Context

The repository already has the foundation:

- `tooling/scripts/run_subject_runtime_smoke.py` runs JSON smoke fixtures through
  the real MCP handler.
- The smoke runner already supports `--mode preview` and `--mode local-agent`.
- Local-agent mode already requires `QIONGLI_SMOKE_RUN_AGENTS=1`.
- Smoke cases run in isolated project directories with temporary `HOME`,
  `CODEX_HOME`, `QIONGLI_CONFIG_HOME`, `QIONGLI_GUIDANCE_HOME`,
  `XDG_CONFIG_HOME`, and `RESEARCH_CLI_LANG=en`.
- `tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json`
  confirms a finance subject before running a preview task.
- `qiongli_task_run` preview packets now expose `task_packet.local_guidance`.
- Real task runs write `data.local_guidance_trace` and include
  `data.task_packet.local_guidance`.

The remaining gap is confidence in the real execution path. Preview smoke proves
the MCP handler, router, materialized guidance, and packet construction. It does
not prove that a real local runtime agent receives that packet and emits a trace
after reading `.qiongli/guidance.d/subject-runtime.md`.

## Product Model

Normal users continue to see safe defaults:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

This remains preview-only and must never launch local agents.

Maintainers and release candidates can run:

```bash
QIONGLI_SMOKE_RUN_AGENTS=1 \
uv run python tooling/scripts/run_subject_runtime_smoke.py \
  --mode local-agent \
  --case confirmed_finance_guidance_loaded \
  --json
```

Local-agent smoke launches the configured local runtime agents only when both
conditions are true:

- `--mode local-agent` is passed.
- `QIONGLI_SMOKE_RUN_AGENTS=1` is present.

If either condition is missing, the smoke runner must fail closed with a clear
machine-readable error.

## Non-Goals

- Do not make local-agent smoke part of the default test or release gate.
- Do not install, configure, or mutate external agent CLIs.
- Do not add accounting, management, political economy, or any other new
  subject in this slice.
- Do not require marketplace or read-only clients to launch local agents.
- Do not call external literature providers from the smoke path.
- Do not guarantee the selected local runtime is offline. If the configured
  local agent CLI needs model access, that is owned by the maintainer's local
  runtime setup.
- Do not weaken preview-first MCP safety. `qiongli_task_run` must still default
  to `run_agents=false`.

## Runtime Smoke Contract

The smoke runner has two modes.

### Preview Mode

Preview mode remains the normal gate:

- Calls `qiongli_task_run` with `run_agents=false`.
- Verifies subject refinement decisions, resource activation levels, effective
  domains, borrowed lenses, and materialized guidance sources.
- Runs all subject smoke fixtures.
- Must pass without any local agent CLI.

### Local-Agent Mode

Local-agent mode is an opt-in maintainer gate:

- Calls `qiongli_task_run` with `run_agents=true`.
- Defaults to a small selected case set, starting with
  `confirmed_finance_guidance_loaded`.
- Uses bounded task-run options so the run is diagnostic, not a full research
  project.
- Verifies the real task packet includes loaded local guidance.
- Verifies the local guidance trace records the managed subject guidance
  fragment.
- Verifies returned trace paths are inside the isolated project root.
- Reports local runtime availability and runtime routing choices.

The first implementation should keep local-agent smoke narrow. One confirmed
finance case is enough to prove the materialized-guidance path. More cases can
be added after the single-case path is reliable.

## Isolation Contract

Every smoke case must run under:

```text
<workspace-root>/<case-name>/
```

The runner must set these environment roots inside the case project:

```text
<case-root>/.smoke-home/home
<case-root>/.smoke-home/codex
<case-root>/.smoke-home/qiongli-config
<case-root>/.smoke-home/qiongli-guidance
<case-root>/.smoke-home/xdg-config
```

The report must include the resolved environment values for each case.

Known-write assertions:

- `project_root` is inside the smoke workspace root.
- `HOME`, `CODEX_HOME`, `QIONGLI_CONFIG_HOME`, `QIONGLI_GUIDANCE_HOME`, and
  `XDG_CONFIG_HOME` are inside the case project root.
- `.qiongli/guidance_manifest.yaml`,
  `.qiongli/guidance.d/subject-runtime.md`, and `.qiongli/trace/**` are inside
  the case project root.
- Any returned `local_guidance_trace.run_dir`, `trace_index`, or trace artifact
  paths are inside the case project root.

The smoke runner cannot reliably prove that arbitrary third-party local agent
CLIs never touch their own caches. It must therefore set the common home/config
roots to isolated paths and audit the Qiongli-visible paths it returns.

## Local-Agent Task Shape

The local-agent smoke task should prioritize bounded execution:

```json
{
  "task_id": "C1",
  "paper_type": "empirical",
  "topic": "earnings announcement stock market reaction",
  "context": "Smoke test only: verify local guidance routing for a finance event-study task.",
  "domain": "auto",
  "guidance_mode": "propose",
  "run_agents": true,
  "max_revision_rounds": 0,
  "output_budget": 1,
  "skip_validation": true,
  "execution_mode": "solo",
  "controller": "codex"
}
```

The exact runtime choices may follow existing controller defaults when `codex`
is unavailable, but the report must record:

- requested controller and runtime overrides,
- resolved `runtime_plan`,
- actual draft and review runtime names when present,
- any runtime preflight or fallback notes.

## Report Schema

Extend the smoke report without breaking existing keys:

```json
{
  "schema_version": "1.1",
  "mode": "local-agent",
  "summary": {
    "total": 1,
    "passed": 1,
    "failed": 0
  },
  "cases": [
    {
      "name": "confirmed_finance_guidance_loaded",
      "status": "passed",
      "project_root": "/tmp/qiongli-smoke/case",
      "environment": {
        "HOME": ".../.smoke-home/home",
        "CODEX_HOME": ".../.smoke-home/codex"
      },
      "local_agent": {
        "requested": true,
        "env_opt_in": true,
        "will_launch_agents": true,
        "runtime_plan": {
          "primary_agent": "codex",
          "review_agent": "codex",
          "fallback_agent": "codex"
        }
      },
      "trace_assertions": {
        "trace_written": true,
        "subject_guidance_loaded": true,
        "subject_refinement_persisted": true
      },
      "write_boundary": {
        "known_paths_inside_project": true,
        "violations": []
      },
      "failures": []
    }
  ]
}
```

Existing preview consumers may continue to read `schema_version`, `summary`,
`cases`, `status`, `failures`, and `result`.

## Data Flow

```text
run_subject_runtime_smoke.py --mode local-agent
  -> load selected fixture
  -> create isolated case root and environment roots
  -> optional qiongli_subject_update confirm finance
  -> call qiongli_task_run with run_agents=true
  -> orchestrator.task_run builds task_packet
  -> effective_guidance reads .qiongli/guidance.d/subject-runtime.md
  -> local runtime agent receives task_packet and local guidance context
  -> orchestrator writes local guidance trace
  -> smoke runner asserts guidance, trace, and write boundaries
  -> JSON report exits 0 or 1
```

## Runtime Hardening

The implementation should harden the existing path instead of adding a separate
parallel runtime.

### Smoke Runner

Enhance `tooling/scripts/run_subject_runtime_smoke.py`:

- Add local-agent default case selection if `--mode local-agent` is used without
  `--case`.
- Add bounded local-agent task overrides for the selected fixture.
- Add local-agent assertions for `data.task_packet.local_guidance`,
  `data.local_guidance_trace`, and runtime routing notes.
- Add write-boundary checks for returned paths.
- Include the exact rerun command in failed local-agent reports.

### MCP Handler And Orchestrator

Keep `qiongli_task_run` preview-first.

If needed, tighten real run output so local-agent smoke can reliably inspect:

- `data.task_packet.local_guidance.guidance_files_read`,
- `data.task_packet.subject_refinement`,
- `data.local_guidance_trace`,
- `data.routing_notes`,
- runtime plan and controller metadata.

Do not add a second task-run implementation only for smoke.

### Release Readiness

Release readiness should consume smoke reports, not parse console prose.

The release checklist should distinguish:

- preview smoke: expected for every release,
- router eval: expected for every release,
- real local-agent smoke: optional but recommended for release candidates,
- local-agent smoke skipped: acceptable only when the report clearly states the
  missing opt-in or missing local runtime.

## Error Handling

- Missing `QIONGLI_SMOKE_RUN_AGENTS=1` in local-agent mode returns a JSON report
  with `summary.failed=1`.
- Missing or unavailable local runtime returns a failed local-agent case with
  runtime preflight notes.
- Local-agent timeout returns a failed case and includes the timeout, case name,
  project root, and rerun command.
- Missing `.qiongli/guidance.d/subject-runtime.md` in the confirmed case fails
  the case.
- Missing `data.local_guidance_trace` in a real run fails the case.
- Any returned trace path outside the case project root fails the case.
- Preview mode must continue to reject accidental agent launch by asserting
  `run_agents=false`.

## Testing Plan

Unit tests:

- Local-agent mode still requires `QIONGLI_SMOKE_RUN_AGENTS=1`.
- Local-agent mode selects only the confirmed finance case by default.
- Local-agent task args include bounded options.
- Write-boundary checks pass for inside-project paths.
- Write-boundary checks fail for outside-project trace paths.
- Local-agent assertion fails when `local_guidance_trace` is missing.
- Local-agent assertion fails when subject guidance is not listed in
  `guidance_files_read`.
- Error reports include the rerun command and workspace root.

Integration tests:

- Preview suite still passes with all subject smoke fixtures.
- A mocked local-agent result with `run_agents=true`, loaded subject guidance,
  and a valid `local_guidance_trace` passes.
- A mocked local-agent runtime failure is reported as a failed case, not as an
  unhandled exception.

Manual maintainer verification:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
uv run python tooling/scripts/evaluate_subject_router.py --json
QIONGLI_SMOKE_RUN_AGENTS=1 \
uv run python tooling/scripts/run_subject_runtime_smoke.py \
  --mode local-agent \
  --case confirmed_finance_guidance_loaded \
  --json
git diff --check
```

## Success Criteria

- Preview smoke remains unchanged as the safe default.
- Local-agent smoke cannot launch agents without both opt-ins.
- Confirmed finance local-agent smoke proves
  `.qiongli/guidance.d/subject-runtime.md` is loaded in `task_packet`.
- Local-agent smoke proves a local guidance trace was written for the real run.
- Local-agent smoke reports all Qiongli-visible paths inside the isolated case
  root.
- Failures include the exact rerun command, isolated root, case name, and trace
  path when available.
- Release readiness can include preview smoke, router eval, and optional
  local-agent smoke summaries without scraping human text.

## Rollback

Repository rollback is simple because this slice should be isolated to:

- `tooling/scripts/run_subject_runtime_smoke.py`,
- `tests/test_subject_runtime_smoke.py`,
- optional release-readiness report parsing,
- docs.

Runtime rollback is also safe:

- Do not run `--mode local-agent`.
- Keep using preview smoke and router eval.
- Remove temporary smoke workspaces if a maintainer supplied a persistent
  `--workspace-root`.

## Risks

- Local runtime variability: mitigated by making local-agent smoke opt-in and
  reporting unavailable runtimes clearly.
- Slow or expensive agent runs: mitigated by bounded task options, single-case
  default, and explicit maintainer opt-in.
- False confidence from preview-only checks: mitigated by requiring trace and
  loaded-guidance assertions in local-agent mode.
- External CLI cache writes: mitigated by isolated home/config roots and
  Qiongli-visible path audits, while documenting that arbitrary third-party
  cache behavior cannot be fully proven from inside the smoke runner.
- Scope creep into subject expansion: mitigated by keeping this slice focused
  on runtime confidence before adding new disciplines.

## Implementation Scope

This spec is suitable for one focused implementation plan:

1. Update smoke report schema and local-agent case selection.
2. Add bounded local-agent task arguments and rerun diagnostics.
3. Add local-agent assertions for loaded guidance and trace output.
4. Add write-boundary checks for returned paths.
5. Add unit tests and mocked local-agent integration tests.
6. Run preview smoke, router eval, focused tests, optional local-agent smoke
   when maintainer opt-in is available, and whitespace checks.
