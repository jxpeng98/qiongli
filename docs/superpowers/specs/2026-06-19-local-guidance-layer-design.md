# Local Guidance Layer Design

## Status

Approved for design by the user on 2026-06-19.

## Problem

Qiongli already has two customization mechanisms:

- `--custom-dir` for materializing local subject overlays into generated packages.
- `--profile-file` for runtime agent profiles and task overrides.

Neither mechanism solves project-local learning across multi-turn work. Current task runs can
also leave important context only in printed output. When helper files are written, they are not
always linked as a coherent run history, so later agents cannot reliably explain why guidance
changed or which artifacts justified it.

The requested feature is a local, self-updating customization layer that can steer later work
without mutating canonical skills, workflow contracts, or release payloads.

## Goals

- Persist project-local guidance that evolves from multi-turn use, project context, and explicit
  user preferences.
- Inject that guidance into later task runs so it can influence drafting, review, artifact policy,
  and follow-up behavior.
- Keep the guidance layer isolated from canonical Qiongli source files and installed skill
  packages.
- Restore traceability by writing linked auxiliary artifacts for each run, even when formal
  `RESEARCH/[topic]/...` outputs are missing.
- Make guidance updates auditable, reversible, and conservative by default.

## Non-Goals

- Do not rewrite `content/workflow/SKILL.md`, `content/skills/**`, subject overlays, or packaged
  release payloads as part of local learning.
- Do not replace formal research artifacts under `RESEARCH/[topic]/...`.
- Do not store project run traces in the user-global Qiongli directory by default.
- Do not infer long-term user preferences from a single run without producing reviewable evidence.

## Storage Boundary

Project-specific traceability and guidance belong in the project directory.

```text
<project>/
├── .qiongli/
│   ├── local_guidance.md
│   └── trace/
│       ├── index.jsonl
│       └── runs/
│           └── <run_id>/
│               ├── task_packet.json
│               ├── guidance_context.md
│               ├── draft.md
│               ├── review.md
│               ├── merged_analysis.md
│               ├── validator_gate.json
│               └── guidance_update_proposal.md
└── RESEARCH/
    └── [topic]/
```

The optional global layer is limited to user defaults:

```text
~/.qiongli/preferences.md
```

Global preferences may supply defaults such as language, output compactness, or preferred review
strictness. They must not contain project run evidence. Project-local guidance overrides global
preferences when both exist.

## Guidance Files

### `~/.qiongli/preferences.md`

Optional user-level defaults. This file is read-only from task-run by default. It can be edited by
the user or by an explicit future preference command, but task execution must not silently update it.

Recommended sections:

- `## Communication Preferences`
- `## Artifact Preferences`
- `## Review Preferences`
- `## Stable User Constraints`

### `<project>/.qiongli/local_guidance.md`

The current project-local guidance file. This file is the effective customization layer for future
runs in the same project.

Required sections:

- `# Qiongli Local Guidance`
- `## Scope`
- `## Active Guidance`
- `## Artifact Policy`
- `## Project Preferences`
- `## Trace Anchors`
- `## Revision History`

The file should be concise. It should contain stable rules and pointers to trace entries, not full
chat transcripts or long draft content.

## Trace Run Bundle

Each task-run writes a run bundle under `.qiongli/trace/runs/<run_id>/`. The bundle is created even
when validator gates fail or formal contract outputs are missing.

Required files:

- `task_packet.json`: effective task packet after guidance injection.
- `guidance_context.md`: merged global and project-local guidance used for the run.
- `draft.md`: primary draft response, or an error note if no draft was produced.
- `review.md`: review response, or an error note if no review was produced.
- `merged_analysis.md`: final merged analysis returned to the caller.
- `validator_gate.json`: disk validation result for required outputs.
- `guidance_update_proposal.md`: conservative proposal for guidance changes.

The run bundle is diagnostic support. It does not satisfy formal workflow outputs unless a task
contract explicitly lists these files.

## Trace Index

`.qiongli/trace/index.jsonl` stores one JSON object per run.

Required fields:

```json
{
  "run_id": "",
  "created_at": "",
  "task_id": "",
  "paper_type": "",
  "topic": "",
  "cwd": "",
  "guidance_mode": "off|read|propose|apply",
  "run_dir": ".qiongli/trace/runs/<run_id>",
  "required_outputs": [],
  "found_outputs": [],
  "missing_outputs": [],
  "guidance_files_read": [],
  "guidance_proposal": ".qiongli/trace/runs/<run_id>/guidance_update_proposal.md",
  "applied_guidance_update": false
}
```

This index lets later agents locate the relevant history without scanning every helper file.

## Runtime Flow

1. Resolve project root from the task-run `cwd`.
2. Read optional global preferences from `~/.qiongli/preferences.md`.
3. Read project guidance from `<project>/.qiongli/local_guidance.md`.
4. Merge the guidance into a `local_guidance_context` block.
5. Add that block to the task packet and draft/review prompts.
6. Run draft, review, revision loop, triad, and validator gates as usual.
7. Write the trace run bundle under `<project>/.qiongli/trace/runs/<run_id>/`.
8. Append a line to `<project>/.qiongli/trace/index.jsonl`.
9. Produce a guidance update proposal.
10. Apply the proposal only when the selected update mode permits it.

## Update Modes

The default mode is `propose`.

| Mode | Reads guidance | Writes trace bundle | Writes proposal | Updates local guidance |
|---|---:|---:|---:|---:|
| `off` | no | yes | no | no |
| `read` | yes | yes | no | no |
| `propose` | yes | yes | yes | no |
| `apply` | yes | yes | yes | yes |

`apply` updates only `<project>/.qiongli/local_guidance.md`. It never updates global preferences
or canonical source files.

## Guidance Update Rules

Guidance changes must be evidence-backed.

Acceptable update sources:

- Explicit user preference stated during the run.
- Repeated user correction that affects future work.
- Stable project constraint already reflected in artifacts.
- Validator or review outcome showing a recurring artifact-policy problem.

Rejected update sources:

- One-off phrasing in a draft.
- Model speculation without user confirmation or artifact evidence.
- Preferences that conflict with canonical workflow contracts.
- Instructions that would suppress required validation, evidence, or safety checks.

Each `guidance_update_proposal.md` must include:

- proposed additions, edits, or removals
- evidence source
- affected future behavior
- rejected alternatives
- whether the proposal was applied

## Orchestrator Integration

The orchestrator should gain a small guidance runtime component with four responsibilities:

- resolve paths
- read and merge guidance
- write trace bundles and index records
- prepare and optionally apply guidance update proposals

The component should be called from `task_run` after the task packet is built and before draft
prompt construction. Trace writing should happen after validator gate evaluation so the bundle can
include output coverage.

The task packet should include a `local_guidance` object:

```json
{
  "enabled": true,
  "mode": "propose",
  "project_guidance_file": ".qiongli/local_guidance.md",
  "global_preferences_file": "~/.qiongli/preferences.md",
  "trace_dir": ".qiongli/trace/runs/<run_id>",
  "summary": "",
  "guidance_context": ""
}
```

Draft and review prompts should state that local guidance is advisory unless it conflicts with
task contracts, required outputs, evidence gates, or safety constraints.

## CLI Surface

Add a `qiongli guidance` command group:

- `qiongli guidance init --project-dir .`
- `qiongli guidance show --project-dir .`
- `qiongli guidance trace --project-dir .`
- `qiongli guidance apply --project-dir . --proposal <path>`

Extend task-run with:

- `--guidance-mode off|read|propose|apply`

The default should be `propose`.

## Error Handling

- Missing guidance files are not errors.
- Malformed guidance files should produce a warning and continue without that layer.
- Trace write failure should not erase task output, but it should be surfaced in routing notes.
- `apply` mode must fail closed if the existing `local_guidance.md` changed after the proposal was
  generated.
- Guidance must never override required outputs, quality gates, or strict MCP/skill validation.

## Testing Plan

Unit tests:

- path resolution for project-local and global guidance
- missing-file behavior
- merge precedence: project guidance overrides global preferences
- trace index JSONL record generation
- proposal creation and apply behavior

Orchestrator tests:

- task packet includes `local_guidance`
- draft prompt includes guidance context
- trace bundle is written after task-run
- validator failure still writes a trace bundle
- `off`, `read`, `propose`, and `apply` modes differ as specified

Safety tests:

- guidance apply never modifies `content/workflow/SKILL.md`
- guidance apply never modifies `content/skills/**`
- guidance apply never modifies subject materializer source or release payloads
- global preferences are not updated by task-run

CLI tests:

- `guidance init` creates project-local files
- `guidance show` reads effective guidance
- `guidance trace` summarizes index entries
- `guidance apply` applies an explicit proposal and records revision history

## Rollout

Phase 1:

- implement project-local `.qiongli/local_guidance.md`
- implement trace bundles and index
- add `--guidance-mode off|read|propose|apply`
- default to `propose`

Phase 2:

- add optional global `~/.qiongli/preferences.md`
- add CLI helpers for inspecting and applying guidance proposals

Phase 3:

- consider promotion tooling that helps users decide whether a project-local rule should remain
  local, become a user preference, or become a canonical contribution.

## Acceptance Criteria

- A task-run can read project-local guidance and inject it into the task packet.
- A task-run writes a linked trace bundle under `<project>/.qiongli/trace/runs/<run_id>/`.
- `.qiongli/trace/index.jsonl` links each run to its guidance context, proposal, and output
  coverage.
- Guidance updates are proposed by default and applied only in explicit `apply` mode or by a
  dedicated apply command.
- Canonical skills, workflow contracts, subject overlays, and release payloads are not modified by
  local learning.
- Existing formal artifacts under `RESEARCH/[topic]/...` remain the authoritative research outputs.
