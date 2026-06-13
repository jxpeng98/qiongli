# Worker Orchestration Design

## Goal

Add a platform-neutral worker orchestration layer to Qiongli so one controller
can delegate independent goals to multiple workers, merge their outputs, and
run an explicit final review without hard-coding Codex subagents or Claude
cowork into the academic workflow model.

The design should make these cases first-class:

- Codex controls a task, dispatches subagents for scoped goals, then reviews and
  merges their results.
- Claude controls a task, delegates cowork-style units, then performs the same
  merge and review gates.
- Any runtime without native worker support can still execute the same plan
  through a generic prompt fallback.

The immediate problem is that Qiongli already has multi-runtime orchestration
(`codex`, `claude`, `gemini`) and limited `team-run` fanout/fanin, but it lacks a
single contract for "multiple workers inside one controller platform." As a
result, Codex subagent and Claude cowork behavior would have to be improvised in
platform-specific instructions.

## Non-Goals

- Do not replace `task-run`, `team-run`, controller modes, or the existing
  `codex|claude|gemini` runtime registry.
- Do not implement a platform-specific Codex or Claude API before the neutral
  contract exists.
- Do not make generated plugin payloads editable source. This follows the
  current single-source distribution model on `dev`: canonical changes belong in
  `content/`, Python runtime source, tests, and docs.
- Do not require every task to use worker orchestration. The existing solo, duo,
  and triad paths remain valid defaults.
- Do not create new academic task semantics. Worker orchestration only changes
  execution structure, not Task IDs, required artifacts, quality gates, or skill
  ownership.

## Current Context

The latest `dev` branch has these relevant pieces:

- `content/standards/agent-run-contract.yaml` defines runtime agents, run
  packets, review packets, and handoff fields.
- `content/standards/mcp-agent-capability-map.yaml` maps Task IDs to MCP
  requirements, required skills, functional owners, primary runtime agents,
  review runtime agents, fallback agents, and `team_run_config`.
- `packages/python-qiongli/src/qiongli/bridges/orchestrator.py` implements
  `task-run`, `parallel`, `team-run`, runtime fallback, draft/review revision
  loops, and triad audits.
- `docs/advanced/controller-modes.md` documents controller, primary, reviewer,
  verifier, and solo role gate flags.
- `docs/superpowers/specs/2026-06-12-single-source-plugin-distribution-design.md`
  and the current source tree remove checked-in plugin package mirrors. Future
  platform wrappers must be generated from canonical source, not hand-edited
  under `packages/qiongli-plugin/` or `packages/qiongli-next-plugin/`.

The gap is below the runtime layer. Existing orchestration can pick Codex as the
draft runtime and Claude as the reviewer, but it cannot say that Codex should
spawn three isolated workers with separate goals, write boundaries, and review
requirements before the controller merges them.

## Design Overview

Introduce a new worker orchestration contract with three layers:

1. **Neutral contract**
   - Defines `worker_plan`, worker packets, merge policies, final review gates,
     allowed artifacts, forbidden artifacts, and degraded execution semantics.
   - Lives in canonical `content/` so it is distributed to every platform.

2. **Runtime strategy**
   - Extends the Python orchestrator so `task-run` and selected `team-run`
     paths can request a worker plan.
   - The first implementation can run through `generic_prompt`, preserving
     behavior even before native Codex subagent or Claude cowork adapters exist.

3. **Platform adapters**
   - Maps the same neutral worker plan to Codex subagents, Claude cowork, or a
     generic single-runtime prompt.
   - Adapter names are metadata, not academic task owners. The academic contract
     remains Task ID -> skills -> artifacts -> quality gates.

This keeps the existing hierarchy intact:

```text
Task ID
  -> capability map
  -> task packet
  -> optional worker plan
  -> platform adapter
  -> worker outputs
  -> merge output
  -> independent review
  -> validator gate
```

## Canonical Contract

Add `content/standards/worker-orchestration-contract.yaml`.

The contract should define:

- `contract_version`
- `orchestration_modes`: `none`, `delegated_workers`, `review_swarm`
- `platform_adapters`: `generic_prompt`, `codex_subagent`, `claude_cowork`
- `worker_statuses`: `planned`, `running`, `passed`, `failed`, `blocked`,
  `skipped`
- `merge_policies`: `synthesize_with_conflict_matrix`,
  `consensus_then_gaps`, `controller_adjudication`
- `required_worker_plan_fields`
- `required_worker_fields`
- `required_merge_fields`
- `required_final_review_fields`

The core packet shape:

```yaml
worker_plan:
  orchestration_mode: delegated_workers
  controller_runtime: codex
  platform_adapter: generic_prompt
  task_id: B1
  paper_type: systematic-review
  topic: ai-in-education
  workers:
    - id: literature_search_worker
      goal: Build and execute the search strategy within the assigned scope.
      functional_role: literature-agent
      required_skills:
        - academic-searcher
      allowed_artifacts:
        - RESEARCH/[topic]/runs/[run_id]/workers/literature_search_worker/**
      forbidden_artifacts:
        - RESEARCH/[topic]/search_strategy.md
        - RESEARCH/[topic]/search_results.csv
      review_required: true
      stop_conditions:
        - required_mcp_unavailable
        - evidence_provenance_missing
  merge:
    agent: controller
    policy: synthesize_with_conflict_matrix
    output_artifacts:
      - RESEARCH/[topic]/runs/[run_id]/worker-merge-report.md
  final_review:
    reviewer: independent_runtime_or_worker
    gate: accept_revise_block
```

Worker outputs must be isolated by default. Workers write only to a run-scoped
worker directory. The controller merge step is the only phase allowed to update
canonical task outputs unless a future task explicitly opts into direct writes.

## Templates

Add these canonical templates:

- `content/templates/worker-run-packet.json`
- `content/templates/worker-review-packet.md`
- `content/templates/worker-merge-report.md`

`worker-run-packet.json` should contain safe defaults for the required fields:

- `run_id`
- `worker_id`
- `controller_runtime`
- `platform_adapter`
- `task_id`
- `paper_type`
- `topic`
- `goal`
- `functional_role`
- `required_skills`
- `required_mcp`
- `allowed_artifacts`
- `forbidden_artifacts`
- `artifacts_read`
- `artifacts_written`
- `warnings`
- `blocking_issues`
- `status`
- `confidence`

`worker-review-packet.md` should mirror the existing agent review packet style
and require explicit `ACCEPT`, `REVISE`, or `BLOCK` status.

`worker-merge-report.md` should require:

- worker status table
- accepted worker outputs
- rejected or blocked worker outputs
- conflict summary
- gap summary
- controller adjudication
- canonical output update plan
- final review request

## Capability Map Integration

Extend `content/standards/mcp-agent-capability-map.yaml` with an optional
`worker_orchestration_config` block. This should be sparse and task-specific,
similar to existing `team_run_config`.

Initial candidate tasks:

- `B1`: literature search and screening can be sharded by query family, paper
  batch, or database/source group.
- `H3`: peer-review simulation can use persona workers such as methodologist,
  domain expert, and reviewer 2.
- `I8`: reproducibility review can split method fidelity, test evidence, data
  leakage, and packaging checks.

Example:

```yaml
worker_orchestration_config:
  B1:
    default_mode: delegated_workers
    adapter_preference:
      codex: codex_subagent
      claude: claude_cowork
      gemini: generic_prompt
    partition_strategy: by_search_facet
    max_workers: 4
    worker_pool:
      - literature_search_worker
      - screening_worker
      - extraction_worker
    merge_policy: synthesize_with_conflict_matrix
    barrier_rules:
      min_success_ratio: 0.6
      on_failure: degrade
```

The capability map remains the source for required MCP providers, required
skills, functional ownership, and fallback runtimes. The worker config only
describes how one controller can split execution.

## Orchestrator Changes

Add small, testable helpers rather than expanding `task_run` with inline logic:

- `_load_worker_orchestration_config(task_id)`
- `_resolve_worker_adapter(controller_runtime, requested_adapter, profile_cfg)`
- `_build_worker_plan(task_packet, worker_config, run_id, adapter)`
- `_build_worker_prompt(worker_packet, task_packet, mcp_evidence, skill_cards)`
- `_execute_worker_plan(worker_plan, adapter, cwd, profile_cfg)`
- `_apply_worker_barrier(worker_results, barrier_rules)`
- `_build_worker_merge_prompt(worker_results, worker_plan, task_packet)`
- `_build_worker_final_review_prompt(merge_output, worker_plan, task_packet)`

The first implementation should support `generic_prompt` end to end. Native
platform adapters can then plug into `_execute_worker_plan` without changing the
contract or task packet shape.

CLI controls should be explicit:

- `--worker-mode none|auto|delegated-workers|review-swarm`
- `--worker-adapter auto|generic-prompt|codex-subagent|claude-cowork`
- `--max-workers <n>`

Default behavior stays `none` for `task-run` until the contract and smoke tests
are stable. `team-run` may use worker orchestration internally later, but the
first pass should avoid merging two execution models prematurely.

## Platform Adapter Semantics

`generic_prompt`

- Runs workers as structured prompts against the selected controller runtime or
  resolved runtime pool.
- Requires no platform-native subagent API.
- Produces the same worker packets and merge report as native adapters.
- Serves as the baseline for tests and CI.

`codex_subagent`

- Maps each `workers[]` entry to a Codex subagent-style delegated task.
- Each subagent receives its goal, allowed artifacts, forbidden artifacts,
  required skills, MCP evidence summary, and expected output schema.
- The controller receives all worker reports and performs the merge.
- If native dispatch is unavailable, falls back to `generic_prompt` and records
  an adapter degradation note.

`claude_cowork`

- Maps each `workers[]` entry to Claude cowork-style delegated work.
- Preserves the same artifact boundaries and review status values.
- If cowork is unavailable, falls back to `generic_prompt` and records an
  adapter degradation note.

Adapters must not alter Task IDs, required outputs, quality gates, or skill
requirements. They only decide how worker prompts are launched and collected.

## Data Flow

1. `task-run` builds the existing task packet from the research contract and
   capability map.
2. If `--worker-mode` enables delegation and the task has config, the
   orchestrator builds a worker plan.
3. The adapter executes worker packets and returns structured worker results.
4. Barrier rules classify the run as `ok`, `degraded`, or `blocked`.
5. If not blocked, the controller merges worker results into a merge report.
6. An independent reviewer reviews the merge report and issues `ACCEPT`,
   `REVISE`, or `BLOCK`.
7. The normal validator gate checks canonical outputs.

Worker artifacts are run-scoped:

```text
RESEARCH/[topic]/runs/[run_id]/workers/[worker_id]/
RESEARCH/[topic]/runs/[run_id]/worker-merge-report.md
RESEARCH/[topic]/runs/[run_id]/worker-review-report.md
```

Canonical task outputs remain under their existing paths. The merge phase may
produce or update them only after worker barrier rules pass.

## Error Handling

- Missing contract file: fail fast in strict validation and skip worker
  orchestration in non-strict mode with a routing warning.
- Unsupported adapter: fall back to `generic_prompt` unless strict adapter mode
  is requested.
- Worker failure: apply task-specific barrier rules. `block` stops before merge;
  `degrade` continues only if the minimum success ratio is met.
- Forbidden artifact write: record a blocking issue and reject that worker
  output from merge.
- Missing evidence provenance: reviewer should block affected claims or move
  them to gap notes.
- Merge conflict: preserve disagreement in `worker-merge-report.md`; do not
  average incompatible findings.
- Final review block: keep worker outputs and merge report as evidence, but do
  not claim the canonical task passed.

## Documentation

Update:

- `docs/guide/multi-agent.md`: explain worker orchestration as a layer below
  runtime collaboration.
- `docs/advanced/controller-modes.md`: clarify that controller/primary/reviewer
  choose runtime accountability, while worker plans split work inside a
  controller.
- `docs/advanced/agent-skill-collaboration.md`: add the standard pattern
  `plan -> mcp-evidence -> worker-plan -> worker-execute -> merge -> review ->
  validator-gate`.
- `content/workflow/references/platform-routing.md`: document adapter mapping
  for Codex subagent, Claude cowork, and generic prompt fallback.

Generated platform plugin docs and wrappers must come from materialization, not
tracked plugin payload edits.

## Test Strategy

Add source-level tests first:

- Contract test: `worker-orchestration-contract.yaml` defines required enums and
  fields.
- Template test: worker run, review, and merge templates contain the required
  fields/headings.
- Capability map test: worker configs reference valid Task IDs, skills,
  functional roles, merge policies, and barrier rules.
- Orchestrator parser test: `--worker-mode`, `--worker-adapter`, and
  `--max-workers` parse supported values and reject invalid values.
- Generic adapter test: a mocked orchestrator executes a worker plan, records
  worker packets, applies barrier rules, merges successful worker outputs, and
  runs final review.
- Degraded path test: one worker failure with sufficient success ratio produces
  a degraded merge note; insufficient success ratio blocks.
- Artifact boundary test: worker prompts include allowed and forbidden artifact
  boundaries.

Validation commands:

```bash
python3 -m unittest tests.test_agent_run_contract tests.test_agent_routing_policy -v
python3 -m unittest tests.test_controller_agnostic_orchestration -v
python3 scripts/validate_research_standard.py --strict
```

Implementation should add a focused worker orchestration test module rather than
overloading existing controller tests.

## Migration Plan

1. Add the worker contract and templates under `content/`.
2. Add contract/template tests.
3. Add sparse `worker_orchestration_config` for one or two MVP tasks, likely
   `B1` and `H3`.
4. Add CLI flags and parser tests with default behavior disabled.
5. Implement `generic_prompt` worker plan execution behind explicit flags.
6. Add docs for runtime collaboration versus worker delegation.
7. Add adapter placeholders for `codex_subagent` and `claude_cowork` that
   degrade to `generic_prompt` with clear routing notes.
8. After generic behavior is stable, implement native platform adapter dispatch
   when the host runtime exposes a reliable API.

## Acceptance Criteria

- Worker orchestration is represented by a canonical contract and templates in
  `content/`.
- `task-run` can run without worker orchestration exactly as it does now.
- With explicit worker flags, a task can produce a worker plan, isolated worker
  outputs, a merge report, and a final worker review report.
- Codex subagent and Claude cowork are represented as adapter names and
  documented semantics, but unsupported native dispatch falls back to
  `generic_prompt`.
- Worker outputs cannot silently write canonical task artifacts before merge.
- Barrier and review statuses are visible in the final orchestration result.
- Tests cover contract shape, templates, parser behavior, generic execution,
  degraded execution, and artifact boundaries.
- No generated plugin payload source is edited or committed.

## Risks

- `orchestrator.py` is already large. Keep worker logic in helper methods and
  consider a later extraction to `worker_orchestration.py` if the implementation
  grows.
- Platform-native subagent and cowork capabilities may differ or change. The
  neutral contract and `generic_prompt` fallback prevent platform churn from
  changing academic workflow semantics.
- Worker fanout can increase cost and latency. Keep explicit flags and
  `max_workers` limits, and default to disabled until local validation is solid.
- If workers write canonical outputs directly, review becomes unreliable. The
  first implementation should enforce run-scoped worker outputs and merge-only
  canonical updates.
