# Experience And Promotion Loop Design

## Goal

Turn Qiongli's existing local guidance, trace bundles, subject evidence memory,
worker orchestration, and validation results into a structured experience layer
that can improve later local runs and produce reviewable candidates for
global or canonical improvements.

The immediate goal is not to build a Trellis-scale database. The first version
should make Qiongli's current run history queryable, replayable, measurable,
and safe to promote. This creates the missing bridge between project-local
learning and framework-level improvement.

## Current Baseline

As of July 6, 2026, Qiongli already has the necessary primitives:

- Project-local guidance under `.qiongli/local_guidance.md` and
  `.qiongli/guidance.d/*.md`.
- Project-local trace bundles under `.qiongli/trace/runs/<run_id>/`.
- A trace index at `.qiongli/trace/index.jsonl`.
- Subject evidence memory at `.qiongli/trace/subject_evidence.json`.
- Guidance update proposals that can be applied to local guidance.
- Preview-first `qiongli_task_run` and explicit `run_agents=true` execution.
- Worker orchestration state for delegated workers, merge, and final review.
- Validator gates, review blockers, missing-output reports, and subject
  refinement packets.
- Canonical skill sources under `content/skills/**`, consolidated skill
  summaries in `content/skills-core.md`, and generated workflow packages under
  `qiongli-workflow/**`.

The gap is that this information is still mostly an audit trail. Later runs do
not have a stable query surface for prior lessons, and there is no explicit
promotion path from repeated local evidence into global preferences, skill
strengthening, or canonical contracts.

## Decision

Add an experience layer with five staged capabilities:

1. **Experience Record Contract**: normalize each run into a stable
   machine-readable record.
2. **Experience Query And Replay**: expose local experience through CLI and MCP
   query surfaces.
3. **Planner Experience Injection**: let `task-plan` and `task-run` consume
   relevant prior lessons before drafting.
4. **Evidence-Backed Skill Reinforcement**: use recurring experience patterns
   to update existing skill source and skill summaries before broader core
   promotion.
5. **Local-to-Global/Core Promotion Gate**: generate promotion candidates and
   require eval gates before global or canonical changes.

This order is deliberate. Qiongli should not strengthen canonical skills from
intuition alone. It should first record experience, query it, identify repeated
patterns, then update skills and contracts through tested promotion candidates.

## Non-Goals

- Do not silently mutate canonical source files from task execution.
- Do not promote project-local guidance into global or canonical behavior
  without explicit maintainer action.
- Do not require a database in the first implementation.
- Do not replace `.qiongli/local_guidance.md`; the experience layer complements
  it.
- Do not make real local-agent execution the default release path.
- Do not treat all model output as reliable training data.
- Do not store secrets, raw API keys, private browser state, or unredacted
  provider credentials in experience records.
- Do not edit generated `qiongli-workflow/**` packages by hand.

## Product Model

### Local Learning

Local learning remains project-scoped:

```text
<project>/.qiongli/
+-- local_guidance.md
+-- guidance.d/
+-- trace/
    +-- index.jsonl
    +-- subject_evidence.json
    +-- experience.jsonl
    +-- runs/<run_id>/
        +-- task_packet.json
        +-- validator_gate.json
        +-- subject_refinement.json
        +-- experience_record.json
        +-- guidance_update_proposal.md
```

The local experience index records what happened. `local_guidance.md` records
what the project has accepted as future guidance. A later run may read both, but
canonical contracts still win over local guidance and experience hints.

### Global Preferences

User-global behavior belongs in `~/.qiongli/preferences.md` or a future
user-global experience index only after explicit user action. It should capture
stable user preferences, not project evidence.

### Canonical Improvement

Canonical improvement means changing checked-in source such as:

- `content/skills/**`
- `content/skills-core.md`
- `content/workflow/references/**`
- `content/standards/**`
- `content/standards/mcp-agent-capability-map.yaml`
- tests, eval fixtures, and docs that verify the change

Canonical promotion must go through normal repository review, tests, and release
materialization. A task run may create a candidate, but it must not apply it.

## Experience Record Contract

Add a stable `experience_record` schema. The first implementation can live as
documentation plus tests before introducing a formal JSON Schema file.

Required top-level fields:

```json
{
  "schema_version": "1.0",
  "run_id": "",
  "created_at": "",
  "project_root": "",
  "task": {
    "task_id": "",
    "paper_type": "",
    "topic": "",
    "workflow": "",
    "stage": ""
  },
  "execution": {
    "run_agents": false,
    "execution_mode": "preview|solo|duo|triad|team|worker",
    "controller": "",
    "primary_agent": "",
    "review_agent": "",
    "verifier_agent": "",
    "worker_mode": "none|delegated_workers|review_swarm"
  },
  "inputs": {
    "guidance_files_read": [],
    "project_manifest": {},
    "subject_refinement": {},
    "provider_status": {},
    "mcp_evidence": []
  },
  "outputs": {
    "required_outputs": [],
    "found_outputs": [],
    "missing_outputs": [],
    "artifacts_written": [],
    "trace_files": []
  },
  "quality": {
    "validator_status": "passed|failed|blocked|skipped",
    "review_status": "passed|failed|blocked|skipped",
    "blocking_issues": [],
    "warnings": [],
    "confidence": 0.0
  },
  "experience": {
    "lessons": [],
    "failure_modes": [],
    "reusable_guidance": [],
    "promotion_candidates": []
  },
  "privacy": {
    "redaction_status": "redacted|not_needed|blocked",
    "contains_user_corpus": false,
    "contains_provider_metadata": false
  }
}
```

The record should be compact. Large drafts, reviews, and artifacts remain in the
run directory and are referenced by path. The record stores summaries and stable
links.

## Experience Write Path

`task-run` should write `experience_record.json` after validator and guidance
trace writing. It should also append one line to
`.qiongli/trace/experience.jsonl`.

Write behavior:

- Preview runs may write experience records if they already write trace bundles.
- `guidance_mode=off` may still write an experience record because experience
  is audit metadata, not guidance.
- If experience write fails, the user-facing result should include a warning,
  but formal research artifacts should not be deleted or rewritten.
- Experience records must use project-relative paths where possible.
- Experience records must include redaction status.

## Experience Query And Replay

Add a local query surface before introducing any global or canonical promotion.

CLI:

```bash
qiongli experience list --project-dir .
qiongli experience show --project-dir . --run-id <run_id>
qiongli experience search --project-dir . --task-id B1 --topic ai-in-education
qiongli experience lessons --project-dir . --task-id F3
qiongli experience replay-plan --project-dir . --run-id <run_id>
```

MCP:

- `qiongli_experience_query`
- `qiongli_experience_show`
- `qiongli_experience_lessons`

Query behavior:

- Default to local project experience only.
- Filter by task ID, stage, topic, subject, validator status, failure mode,
  guidance source, and worker mode.
- Return compact summaries and paths, not full drafts.
- Support read-only clients by returning proposed commands or exported records
  without writing new files.

Replay behavior:

- `replay-plan` does not rerun agents.
- It reconstructs the task packet summary, guidance sources, validator result,
  and next-action recommendation.
- It can be used to debug failed runs or prepare a safer rerun.

## Planner Experience Injection

Once query works, `task-plan` and `task-run` should include a bounded
`prior_experience` block in the task packet.

Example:

```json
{
  "prior_experience": {
    "query": {
      "task_id": "B1",
      "topic": "ai-in-education",
      "limit": 5
    },
    "records": [
      {
        "run_id": "abc123",
        "status": "failed",
        "failure_modes": ["missing_search_diagnostics"],
        "reusable_guidance": [
          "Write search diagnostics before claiming review-grade coverage."
        ],
        "trace_path": ".qiongli/trace/runs/abc123/experience_record.json"
      }
    ]
  }
}
```

Injection rules:

- Prior experience is advisory and cannot override canonical contracts.
- A failing prior run should become a warning or checklist item, not a hidden
  hard block.
- Injection should be size bounded and deterministic.
- The task packet should record the query that selected the prior records.
- If local guidance conflicts with prior experience, the task packet should
  record the conflict and prefer accepted local guidance unless canonical
  contracts say otherwise.

## Skill Reinforcement Phase

Skill reinforcement should happen after query/replay exists and before core
promotion. This is the safest point because Qiongli can identify repeated
failures by Task ID, stage, skill, and validator result before editing
canonical skill source.

Targets:

- `content/skills/**`: detailed skill instructions and output requirements.
- `content/skills-core.md`: token-efficient core behavior.
- `content/skills/registry.yaml`: summaries, usage triggers, inputs, outputs,
  and tags when the skill's activation boundary changed.
- `content/workflow/references/**`: shared contracts when a pattern crosses
  several skills.
- `content/standards/mcp-agent-capability-map.yaml`: routing, MCP, worker, or
  reviewer requirements when runtime behavior needs stronger ownership.

Candidate triggers:

- A skill repeatedly produces missing required artifacts.
- A reviewer repeatedly blocks the same skill output for the same reason.
- A validator repeatedly reports the same quality gate failure.
- Local guidance proposals repeatedly add the same rule for a task or skill.
- Worker merge reports repeatedly show conflict patterns tied to a skill.
- Subject-specific overlays repeatedly compensate for missing core skill
  language.

Output artifact:

```text
.qiongli/trace/promotion/skill-reinforcement-candidate-<date>.md
```

Required sections:

- affected skill IDs
- supporting experience records
- repeated failure or improvement pattern
- proposed canonical source change
- expected behavior change
- required eval or regression test
- rollback path

The first implementation should generate candidates only. Maintainers apply the
actual `content/` changes in a normal branch with tests.

## Local-To-Global/Core Promotion Gate

Promotion has three levels:

1. **Project-local**: accepted into `.qiongli/local_guidance.md`.
2. **User-global**: accepted into `~/.qiongli/preferences.md`.
3. **Canonical candidate**: proposed for repository source under `content/`,
   standards, tests, or docs.

Add a promotion command group:

```bash
qiongli experience promote \
  --project-dir . \
  --scope local|user-global|skill-candidate|canonical-candidate \
  --task-id B1 \
  --min-support 3
```

Promotion rules:

- Local promotion may use a single explicit user decision.
- User-global promotion requires explicit user approval and must not include
  project-specific evidence or private corpus details.
- Skill-candidate promotion requires repeated evidence from experience records.
- Canonical-candidate promotion requires a proposed test or eval gate.
- No promotion command applies canonical source edits automatically.

Canonical candidate output:

```text
docs/superpowers/specs/<date>-<topic>-design.md
docs/superpowers/plans/<date>-<topic>.md
```

or, for smaller changes:

```text
.qiongli/trace/promotion/canonical-candidate-<date>.md
```

The maintainer can decide whether the candidate becomes a full spec, a plan, or
a smaller reviewed patch.

## Metrics

Add experience-derived metrics before claiming self-improvement:

- validator pass rate by task ID and stage
- missing artifact rate by task ID and skill
- review blocker count by blocker type
- guidance proposal acceptance rate
- repeated local guidance rule frequency
- subject routing confirmation, dismissal, and correction rate
- worker merge failure rate
- final review block rate for worker runs
- provider/search diagnostic failure rate for literature tasks

The first metrics surface can be:

```bash
qiongli experience metrics --project-dir .
```

The command should produce a compact JSON summary and a human-readable table.

## Error Handling And Safety

- Missing experience files are not errors.
- Malformed experience records should be skipped with warnings.
- Query should tolerate older trace bundles that do not yet have
  `experience_record.json`.
- Promotion should fail closed when supporting records are missing, malformed,
  or privacy-blocked.
- User-global promotion must redact project-specific paths and source details.
- Canonical promotion candidates must list tests before recommending source
  edits.
- Generated candidates must not include secrets or raw provider credentials.
- Read-only clients should receive exported candidate text instead of write
  attempts.

## Testing Requirements

Unit tests:

- experience record construction from a representative task packet,
  validator gate, guidance trace, subject refinement, and worker state
- redaction and path normalization
- JSONL append and malformed-record skip behavior
- query filtering by task ID, stage, topic, validator status, and failure mode
- replay-plan output for successful and failed runs

Orchestrator tests:

- task-run writes `experience_record.json`
- preview-only task-run records `run_agents=false`
- worker orchestration state appears in the experience record
- failed validator gates produce reusable failure modes
- prior experience injection is bounded and deterministic

CLI/MCP tests:

- `qiongli experience list/show/search/lessons/replay-plan`
- `qiongli_experience_query` and `qiongli_experience_show`
- read-only or missing-project behavior
- promotion candidate generation without source edits

Skill reinforcement tests:

- repeated synthetic experience records produce a skill reinforcement candidate
- candidate includes affected skill IDs, supporting run IDs, proposed test, and
  rollback path
- candidate generation does not edit `content/skills/**`

Promotion tests:

- local promotion can reference an accepted guidance proposal
- user-global promotion refuses project-specific private evidence
- canonical-candidate promotion requires minimum support and test text
- no promotion command edits generated workflow payloads

Regression checks:

```bash
.venv/bin/python -m unittest tests.test_guidance_runtime tests.test_mcp_tool_handlers
.venv/bin/python -m unittest tests.test_worker_orchestration_runtime tests.test_agent_run_contract
.venv/bin/python -m unittest tests.test_skill_contract_alignment tests.test_skill_structure_lint
git diff --check
```

## Rollout

### Phase 1: Experience Record Contract

- Add experience record builder and tests.
- Write `experience_record.json` into each trace run.
- Append compact records to `.qiongli/trace/experience.jsonl`.
- Keep existing trace index behavior intact.

### Phase 2: Query And Replay

- Add CLI list/show/search/lessons/replay-plan.
- Add MCP query/show tools.
- Support older traces gracefully.
- Keep all queries local-project scoped by default.

### Phase 3: Planner Injection

- Add bounded `prior_experience` to task-plan and task-run packets.
- Record query parameters and selected run IDs.
- Add prompt language that treats prior experience as advisory.

### Phase 4: Skills Reinforcement

- Generate skill reinforcement candidates from repeated experience patterns.
- Update existing skills only through normal source edits under `content/`.
- Update `content/skills-core.md` when a behavior must be visible in the
  token-efficient execution path.
- Add or update evals/tests before applying any canonical skill change.

### Phase 5: Promotion Gates

- Add local, user-global, skill-candidate, and canonical-candidate promotion
  scopes.
- Require explicit approval for user-global changes.
- Require test or eval evidence for canonical candidates.
- Document promotion behavior in CLI reference and maintainer docs.

### Phase 6: Metrics And Release Readiness

- Add `experience metrics`.
- Add release-readiness checks for experience record compatibility.
- Add docs that explain local learning, skill reinforcement, and canonical
  promotion boundaries.

## Acceptance Criteria

- Every task-run that writes a trace bundle also writes an experience record or
  reports why it could not.
- A maintainer can query prior failed runs by task ID and failure mode without
  reading every trace directory manually.
- `task-plan` can include bounded prior lessons from local experience.
- Repeated local evidence can generate a skill reinforcement candidate that
  names affected existing skills and required tests.
- Canonical skill or contract changes remain explicit source edits under
  `content/`, never automatic task-run side effects.
- Promotion candidates preserve privacy boundaries and never include secrets.
- The roadmap clearly places skill reinforcement after query/replay and before
  canonical promotion.

## Rollback

If the experience layer becomes noisy or unsafe:

- Disable planner experience injection while keeping records for audit.
- Keep CLI query read-only and remove promotion commands from release docs.
- Ignore `.qiongli/trace/experience.jsonl` in older runtimes.
- Preserve existing `.qiongli/local_guidance.md` and trace bundles.
- Revert only the new experience and promotion surfaces; local guidance and
  subject refinement remain valid.
