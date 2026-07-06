# Adaptive Subject Lifecycle And Evaluation Design

## Goal

Build the first user-controllable loop around runtime subject refinement.
Qiongli should be able to evaluate whether its subject router is behaving
correctly, then let users or clients inspect, confirm, dismiss, reset, lock, and
unlock project subject state without changing installation-time subject
selection.

This spec covers the first implementation slice. It does not expand subject
coverage beyond the current economics and finance runtime.

## Current Context

The repository already has the core runtime pieces:

- `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py` produces
  subject refinement packets with decision class, structured signals, borrowed
  lenses, loaded resources, and resource activation plans.
- `packages/python-qiongli/src/qiongli/bridges/subject_resources.py` centralizes
  resource activation plans.
- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py` writes trace
  bundles, proposal text, and `.qiongli/trace/subject_evidence.json`.
- `packages/python-qiongli/src/qiongli/bridges/project_manifest.py` defines
  `.qiongli/guidance_manifest.yaml` with `active_subject`, `subject_mode`,
  `secondary_subjects`, `venue_profiles`, `method_lenses`, and `strictness`.
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` exposes
  preview-first task running through `qiongli_task_run`.
- `tooling/scripts/run_subject_runtime_smoke.py` verifies subject runtime
  behavior through the real MCP handler without launching local agents.

The remaining gap is operational control. The runtime can suggest a subject,
but users cannot yet accept or reject the suggestion through a stable local
workflow. The router also lacks a curated evaluation corpus that measures
whether `borrow_lens`, `suggest_subject`, and `no_subject` decisions are
correct.

## Product Model

Installation remains simple:

```bash
qiongli install --profile full --target codex
```

or the equivalent plugin or marketplace install.

The installed agent starts from adaptive core. During use:

1. Runtime subject refinement observes task text, manifest state, and trace
   memory.
2. The router emits `no_subject`, `borrow_lens`, `suggest_subject`,
   `confirm_subject`, or `lock_subject`.
3. Repeated evidence may produce a `promotion_recommendation`.
4. A user or client explicitly confirms, dismisses, resets, locks, or unlocks
   subject state.
5. Future runs read the manifest and evidence memory to refine behavior.

## Non-Goals

- Do not add new disciplines in this slice.
- Do not make install-time subject choice part of the default flow.
- Do not auto-confirm a subject from repeated evidence.
- Do not generate full subject local guidance fragments in this slice.
- Do not launch local agents in the evaluation runner.
- Do not change npm-lite into a Python runtime surface.
- Do not remove or rewrite existing project manifest semantics.

## User-Facing Controls

Add subject lifecycle operations through CLI and MCP.

CLI shape:

```bash
qiongli subject status --cwd <project>
qiongli subject confirm finance --cwd <project>
qiongli subject dismiss finance --cwd <project>
qiongli subject reset --cwd <project>
qiongli subject lock economics --cwd <project>
qiongli subject unlock --cwd <project>
```

MCP shape:

- `qiongli_subject_status`
- `qiongli_subject_update`

`qiongli_subject_update` accepts an `action` enum:

- `confirm`
- `dismiss`
- `reset`
- `lock`
- `unlock`

MCP clients should not need to know file paths. They pass `cwd`, `action`, and
optional `subject`, and receive a structured before/after packet.

## Lifecycle Semantics

### Status

`status` reads:

- `.qiongli/guidance_manifest.yaml`
- `.qiongli/trace/subject_evidence.json`
- latest subject refinement summary when available

It returns:

```json
{
  "active_subject": "auto",
  "subject_mode": "auto",
  "secondary_subjects": [],
  "method_lenses": [],
  "suggestions": [
    {
      "subject": "finance",
      "suggestion_count": 2,
      "last_confidence": 0.85,
      "last_decision": "suggest_subject",
      "promotion_status": "recommend_confirmation"
    }
  ],
  "dismissed_subjects": [],
  "locked": false
}
```

### Confirm

`confirm <subject>` writes the project manifest:

```yaml
active_subject: finance
subject_mode: confirmed
```

It may preserve existing `secondary_subjects`, `venue_profiles`,
`method_lenses`, and `strictness`. It records a lifecycle event in subject
evidence memory:

```json
{
  "event": "confirm",
  "subject": "finance",
  "source": "user",
  "write_manifest": true
}
```

Confirmed state influences future runtime packets through the existing project
manifest path.

### Dismiss

`dismiss <subject>` does not edit `active_subject`. It writes dismissal metadata
to subject evidence memory:

```json
{
  "dismissed_subjects": {
    "finance": {
      "dismissal_count": 1,
      "last_dismissed_run_id": "run-123",
      "last_suggestion_count": 2
    }
  }
}
```

Dismissed subjects should not produce another confirmation prompt until the
subject has accumulated new suggestion evidence after the dismissal. Dismissal
does not block `borrow_lens`; method-level protections should still work.

### Reset

`reset` returns the project to adaptive core:

```yaml
active_subject: auto
subject_mode: auto
secondary_subjects: []
venue_profiles: []
method_lenses: []
strictness: standard
```

It clears dismissal metadata and confirmation recommendations from subject
evidence memory while preserving historical run traces.

### Lock

`lock <subject>` writes:

```yaml
active_subject: economics
subject_mode: locked
```

Locked state prevents automatic subject replacement. Runtime inference may
still borrow neighboring method lenses.

### Unlock

`unlock` preserves the current active subject but changes mode from `locked` to
`confirmed`. If the manifest is already `auto`, `unlock` is a no-op with a clear
status packet.

## Evaluation Runner

Add a curated evaluation runner:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

The runner should call `infer_subject_refinement()` directly rather than the
full MCP preview path. This keeps evaluation fast, deterministic, and focused
on router behavior. The existing real preview smoke remains the integration
test for MCP behavior.

Fixture directory:

```text
tests/fixtures/subject_router_eval/
```

Fixture schema:

```json
{
  "name": "finance_event_study_clear",
  "task_packet": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "earnings announcement stock returns",
    "context": "Use CRSP abnormal returns and an event study for Journal of Finance."
  },
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto"
  },
  "expected": {
    "decision": "suggest_subject",
    "primary_subject": "finance",
    "method_lenses": ["event-study"],
    "borrowed_lenses": []
  },
  "tags": ["finance", "clear-positive"]
}
```

Required fixture categories:

- Clear finance subject.
- Clear economics subject.
- Finance method-only borrowed lens.
- Economics method-only or causal-method suggestion.
- Mixed economics and finance.
- Weak signal that should remain core-only.
- Near-miss finance language that should not suggest finance.
- Locked subject with borrowed neighboring lens.

Report schema:

```json
{
  "schema_version": "1.0",
  "summary": {
    "total": 8,
    "passed": 8,
    "failed": 0
  },
  "metrics": {
    "decision_accuracy": 1.0,
    "primary_subject_accuracy": 1.0,
    "suggest_subject_precision": 1.0,
    "near_miss_false_positives": 0
  },
  "failures": []
}
```

Initial thresholds:

- `decision_accuracy >= 0.90`
- `primary_subject_accuracy >= 0.90`
- `suggest_subject_precision >= 0.85`
- `near_miss_false_positives == 0`

The runner exits non-zero when thresholds fail.

## Architecture

Add focused modules rather than expanding existing large files unnecessarily.

Suggested files:

- `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`
  - Reads manifest and subject evidence memory.
  - Applies lifecycle actions.
  - Returns before/after packets.
- `tooling/scripts/evaluate_subject_router.py`
  - Loads eval fixtures.
  - Calls `infer_subject_refinement()`.
  - Computes metrics and threshold failures.
- `tests/test_subject_lifecycle.py`
  - Tests status, confirm, dismiss, reset, lock, and unlock.
- `tests/test_subject_router_eval.py`
  - Tests fixture loading, metric calculation, and threshold failures.
- `tests/fixtures/subject_router_eval/*.json`
  - Curated router behavior cases.

Modify existing files:

- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Add MCP definitions and handlers for subject status and update.
- `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
  - Add CLI entrypoints if this is the current MCP CLI dispatcher.
- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Reuse existing subject evidence helpers where possible.
  - Suppress promotion recommendations for dismissed subjects until new
    suggestion evidence appears.
- `tests/test_mcp_tool_handlers.py`
  - Cover MCP lifecycle packets.
- Existing CLI tests
  - Cover command routing and JSON output.

## Data Flow

Status flow:

```text
CLI/MCP subject status
  -> subject_lifecycle.load_subject_status(cwd)
  -> load_project_manifest(cwd)
  -> read subject_evidence.json
  -> return status packet
```

Confirm flow:

```text
CLI/MCP subject confirm finance
  -> subject_lifecycle.apply_subject_action(action=confirm, subject=finance)
  -> update_project_manifest(active_subject=finance, subject_mode=confirmed)
  -> append lifecycle event to subject_evidence.json
  -> return before/after packet
```

Dismiss flow:

```text
CLI/MCP subject dismiss finance
  -> subject_lifecycle.apply_subject_action(action=dismiss, subject=finance)
  -> update dismissed_subjects in subject_evidence.json
  -> leave guidance_manifest.yaml unchanged
  -> return before/after packet
```

Evaluation flow:

```text
evaluate_subject_router.py
  -> load JSON fixtures
  -> infer_subject_refinement(task_packet, manifest_state)
  -> compare actual packet to expected fields
  -> compute metrics
  -> exit 0 or 1 based on thresholds
```

## Error Handling

- Unsupported subjects return structured errors that include available subjects.
- `confirm`, `dismiss`, and `lock` require a subject.
- `reset`, `status`, and `unlock` do not require a subject.
- Invalid or malformed `subject_evidence.json` should not abort lifecycle
  commands. The command should preserve a warning and continue from safe
  defaults.
- Manifest validation errors should return a structured error and should not
  partially write lifecycle state.
- Eval fixture parse errors should include fixture path and field name.
- Threshold failures should be reported as evaluation failures, not Python
  exceptions.

## Testing Strategy

Unit tests:

- `test_subject_lifecycle_status_reads_manifest_and_memory`
- `test_subject_confirm_writes_confirmed_manifest`
- `test_subject_dismiss_does_not_change_manifest`
- `test_subject_reset_returns_to_auto`
- `test_subject_lock_prevents_auto_subject_switch`
- `test_subject_unlock_changes_locked_to_confirmed`
- `test_malformed_subject_memory_does_not_abort_lifecycle`
- `test_dismissed_subject_suppresses_repeated_promotion`

Evaluation tests:

- Fixture loading reads all curated cases.
- Clear finance case passes as `suggest_subject`.
- Method-only finance case passes as `borrow_lens`.
- Near-miss case fails if it suggests finance.
- Threshold failure exits non-zero in the script entrypoint.

Integration tests:

- MCP status returns manifest and memory state.
- MCP update confirm writes project-local manifest.
- MCP update dismiss writes memory only.
- Existing subject runtime smoke still passes.

Verification commands:

```bash
uv run python -m unittest tests.test_subject_lifecycle tests.test_subject_router_eval tests.test_mcp_tool_handlers
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/run_subject_runtime_smoke.py --mode preview --json
uv run python -m unittest discover -s tests
node --test packages/npm-qiongli/test/*.test.mjs
git diff --check
```

## Acceptance Criteria

- Evaluation runner ships with at least eight curated fixtures.
- Evaluation runner exits non-zero when threshold metrics fail.
- Users can inspect subject status through CLI and MCP.
- Users can confirm, dismiss, reset, lock, and unlock subject state.
- Confirm and lock write `.qiongli/guidance_manifest.yaml`.
- Dismiss writes only subject evidence memory and does not change manifest.
- Reset returns the project to adaptive core and clears dismissal state.
- Promotion recommendations respect dismissals until new evidence appears.
- Runtime preview remains preview-first and does not launch local agents.
- Existing full test suites remain green.

## Release Notes Draft

This feature adds subject lifecycle controls and router evaluation for Qiongli's
adaptive subject runtime. Qiongli still installs as a complete core agent, but
users can now inspect, confirm, dismiss, reset, lock, and unlock inferred
subject state as the project evolves. Maintainers also gain a curated evaluation
runner that measures whether subject routing is precise enough before expanding
to more disciplines.
