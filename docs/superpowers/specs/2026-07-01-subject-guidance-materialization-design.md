# Subject Guidance Materialization Design

## Goal

Make confirmed or locked runtime subject state affect future Qiongli runs through
project-local guidance.

After a user or client explicitly confirms or locks a subject, Qiongli should
write a small managed guidance fragment under `.qiongli/guidance.d/`. Future
`effective_guidance()` reads should include that fragment, so installed
adaptive core packages can keep using core workflow guidance while adding the
right subject-specific instructions for the current project.

This is the second implementation slice after adaptive subject lifecycle
controls. It does not expand the subject catalog beyond the current official
subjects, and it does not make install-time subject choice part of the normal
user flow.

## Current Context

The repository already has the necessary runtime foundations:

- `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py` infers
  `no_subject`, `borrow_lens`, `suggest_subject`, `confirm_subject`, and
  `lock_subject` decisions with signal records and resource activation plans.
- `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py` exposes
  project-local subject actions: `status`, `confirm`, `dismiss`, `reset`,
  `lock`, and `unlock`.
- `packages/python-qiongli/src/qiongli/bridges/project_manifest.py` stores
  `.qiongli/guidance_manifest.yaml` with `active_subject`, `subject_mode`,
  `secondary_subjects`, `venue_profiles`, `method_lenses`, and `strictness`.
- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py` already
  reads `.qiongli/local_guidance.md` and sorted `.qiongli/guidance.d/*.md`
  fragments through `effective_guidance()`.
- CLI and MCP already expose subject lifecycle actions through `qiongli subject
  ...`, `qiongli_subject_status`, and `qiongli_subject_update`.
- The subject router has a curated evaluation runner, and the subject runtime
  smoke harness can verify preview-first MCP behavior.

The remaining gap is durable execution influence. Confirming a subject updates
the manifest and evidence memory, but it does not yet create a concise local
guidance fragment that future agent runs can read without re-inferring the same
state from scratch.

## Product Model

Normal installation remains full adaptive core:

```bash
qiongli install --profile full --target codex
```

The user does not choose a subject during normal install. During project use:

1. Runtime subject refinement suggests or borrows subject resources.
2. A user or client explicitly confirms or locks a subject.
3. Qiongli writes both the project manifest and a managed subject guidance
   fragment.
4. Future task runs read `.qiongli/guidance_manifest.yaml`,
   `.qiongli/local_guidance.md`, and `.qiongli/guidance.d/subject-runtime.md`.
5. Reset or unlock lifecycle actions make the managed fragment match the new
   project subject state.

This keeps canonical skills stable while allowing each project to accumulate
subject-specific guidance only after explicit user or client action.

## Non-Goals

- Do not change the default install flow into a subject selection wizard.
- Do not auto-confirm a subject from repeated evidence.
- Do not write subject guidance for `dismiss`; dismissal remains evidence
  memory only.
- Do not rewrite `content/workflow/SKILL.md`, canonical skills, subject
  overlays, or release payloads during project use.
- Do not store project subject guidance in a user-global directory.
- Do not silently overwrite user-authored guidance outside the managed subject
  block.
- Do not make npm-lite expose subject materialization without the Python
  runtime.
- Do not add new subjects in this slice.

## Storage Boundary

Subject materialization remains project-local:

```text
<project>/
└── .qiongli/
    ├── guidance_manifest.yaml
    ├── local_guidance.md
    ├── guidance.d/
    │   └── subject-runtime.md
    └── trace/
        ├── index.jsonl
        └── subject_evidence.json
```

`subject-runtime.md` is a managed project fragment. It is read by the existing
guidance fragment loader and should appear in `guidance_files_read`,
`guidance_sources`, and trace records like any other project fragment.

## Managed Fragment Contract

The managed fragment path is fixed:

```text
.qiongli/guidance.d/subject-runtime.md
```

The file contains a generated block with stable markers:

```markdown
# Qiongli Subject Runtime Guidance

<!-- qiongli:subject-runtime:start -->
schema_version: 1.0
managed_by: qiongli
active_subject: finance
subject_mode: confirmed
updated_at: 2026-07-01T12:00:00+00:00
updated_by: cli
lifecycle_action: confirm
run_id: run-123

## Active Subject

- Use the canonical Qiongli workflow as the base.
- Add the `finance` subject layer when interpreting project-specific methods,
  evidence standards, venue norms, and quality checks.
- Treat this guidance as project-local. It does not change global user
  preferences or installed canonical skills.

## Method Lenses

- event-study
- asset-pricing

## Resource Activation

- core: active
- subject_overlay: confirmed
- subject_skill: confirmed
- method_pack: confirmed

## Evidence And Trace Anchors

- manifest: `.qiongli/guidance_manifest.yaml`
- subject evidence: `.qiongli/trace/subject_evidence.json`
- latest action: `confirm`
<!-- qiongli:subject-runtime:end -->
```

The exact generated text should be concise. It should contain enough guidance
for future runs to apply the subject layer, but it should not copy full subject
skills or long discipline references into the project directory.

The fragment may contain user-authored text outside the managed markers. Qiongli
owns only the marked block.

## Lifecycle Semantics

### Status

`subject_status(project_root)` should report subject guidance state:

```json
{
  "subject_guidance": {
    "path": ".qiongli/guidance.d/subject-runtime.md",
    "exists": true,
    "managed_block": "active",
    "active_subject": "finance",
    "subject_mode": "confirmed",
    "updated_at": "2026-07-01T12:00:00+00:00",
    "warnings": []
  }
}
```

If the file is missing, `exists` is false and `managed_block` is `missing`.
If the file exists but has no managed markers, `managed_block` is `absent`.
If marker order is invalid or the block cannot be parsed, `managed_block` is
`invalid` and a warning explains the problem.

### Confirm

`confirm <subject>` writes:

- `.qiongli/guidance_manifest.yaml`
- `.qiongli/trace/subject_evidence.json`
- `.qiongli/guidance.d/subject-runtime.md`

The managed fragment should use `subject_mode: confirmed`. It should include
method lenses from the project manifest and, when available, a compact summary
of the latest resource activation plan for the confirmed subject.

### Lock

`lock <subject>` has the same materialization behavior as confirm, except the
managed fragment uses `subject_mode: locked`.

Locked subject guidance should state that automatic subject replacement is not
allowed. Borrowed neighboring method lenses remain allowed when the router
detects method-only evidence.

### Unlock

`unlock` preserves a concrete active subject and changes `subject_mode` from
`locked` to `confirmed`. The managed fragment should be rewritten from locked
to confirmed.

If the current active subject is `auto` or `core`, `unlock` returns the project
to adaptive core and disables the managed fragment.

### Reset

`reset` returns the project to adaptive core and disables the managed fragment.

The preferred behavior is to replace the managed block with a disabled block
that records the reset event:

```markdown
<!-- qiongli:subject-runtime:start -->
schema_version: 1.0
managed_by: qiongli
active_subject: auto
subject_mode: auto
status: disabled
updated_at: 2026-07-01T12:00:00+00:00
lifecycle_action: reset

## Active Subject

- No project-specific subject is confirmed or locked.
- Use adaptive core inference for future runs.
<!-- qiongli:subject-runtime:end -->
```

This keeps an auditable local record without forcing future runs to activate a
subject. If the file has no user-authored text outside the managed block, the
implementation may delete it instead. The behavior must be deterministic and
covered by tests.

### Dismiss

`dismiss <subject>` does not write or delete `subject-runtime.md`.

Dismissal only updates subject evidence memory and suppresses repeated
promotion prompts until new evidence appears. It must not remove an existing
confirmed or locked subject fragment for another subject.

## Guidance Runtime Integration

`effective_guidance(project_root, mode="read")` already reads sorted project
fragments from `.qiongli/guidance.d/*.md`. The implementation should preserve
that generic fragment loading behavior and make subject materialization fit it.

Additional runtime expectations:

- `subject-runtime.md` should appear as a `project-fragment` source.
- The trace index should record the fragment in `guidance_files_read` and
  `guidance_sources`.
- `guidance_bootstrap_status()` should count the subject fragment like other
  project fragments.
- `lint_project_guidance()` should check the subject fragment for forbidden
  override language.
- MCP preview output should continue to use the effective guidance packet, so
  clients can see that subject guidance was loaded without knowing file paths.

The fragment must remain advisory below canonical workflow contracts. If it
conflicts with required outputs, evidence gates, safety checks, or reporting
standards, canonical contracts win.

## User Edit Preservation

Qiongli owns only this block:

```markdown
<!-- qiongli:subject-runtime:start -->
...
<!-- qiongli:subject-runtime:end -->
```

Rules:

- If the file does not exist, create it with the managed block.
- If the file exists with one valid managed block, replace only that block.
- If the file exists with user text before or after the managed block, preserve
  that text byte-for-byte except for normalizing final newline handling.
- If the file exists with no managed block, append a new managed block after the
  existing content and report `managed_block: appended`.
- If the file contains multiple managed blocks, do not write. Return a
  structured error that asks the user to repair the file.
- If the start marker appears after the end marker, do not write. Return a
  structured error.

This prevents silent loss of user-authored project guidance while keeping the
managed subject layer deterministic.

## Data Flow

```text
qiongli subject confirm finance
  -> subject_lifecycle.apply_subject_action(...)
  -> update_project_manifest(active_subject=finance, subject_mode=confirmed)
  -> append lifecycle event to subject_evidence.json
  -> subject_guidance_materializer.write_subject_guidance(...)
  -> .qiongli/guidance.d/subject-runtime.md

future qiongli task run
  -> guidance_runtime.effective_guidance(...)
  -> read project manifest
  -> read .qiongli/local_guidance.md
  -> read .qiongli/guidance.d/subject-runtime.md
  -> include subject fragment in prompts and trace records
```

The materializer should not call the subject router. It should use lifecycle
inputs, the project manifest, existing evidence memory, and optional latest
activation plan data if already available.

## Module Boundaries

Create a focused module:

```text
packages/python-qiongli/src/qiongli/bridges/subject_guidance.py
```

Responsibilities:

- Resolve `.qiongli/guidance.d/subject-runtime.md`.
- Render active and disabled managed subject blocks.
- Replace, append, disable, or inspect the managed block.
- Return structured status packets for CLI, MCP, tests, and trace records.

Keep `subject_lifecycle.py` responsible for lifecycle actions and state
transitions. It may call `subject_guidance.py` after manifest and evidence
updates succeed.

Keep `guidance_runtime.py` responsible for reading guidance sources and writing
trace records. It should not know subject lifecycle semantics beyond reporting
the fragment it already loads.

## CLI And MCP Surface

No new top-level command group is required in this slice.

Existing lifecycle commands should gain richer output:

```bash
qiongli subject status --cwd . --json
qiongli subject confirm finance --cwd . --json
qiongli subject lock economics --cwd . --json
qiongli subject reset --cwd . --json
```

The JSON packet should include `subject_guidance`. Human output should include a
short line:

```text
subject guidance: active (.qiongli/guidance.d/subject-runtime.md)
```

MCP tools should return the same `subject_guidance` packet through:

- `qiongli_subject_status`
- `qiongli_subject_update`

Clients that cannot write project files should receive a structured write
error, not a partially updated lifecycle state. The implementation should write
manifest, evidence memory, and managed guidance in an order that avoids
reporting success when the managed fragment failed.

## Error Handling

- Missing `.qiongli/` directories are created only for lifecycle actions that
  write state.
- `status` must not create files.
- Invalid subject names still raise `SubjectLifecycleError`.
- Invalid managed block markers should block writes to `subject-runtime.md` and
  return a clear error.
- A failed subject guidance write should not silently report lifecycle success.
- A malformed existing subject evidence file should retain current recovery
  behavior and should not prevent writing subject guidance unless lifecycle
  action validation fails.
- Permission or filesystem errors should include the attempted relative path.

## Testing Plan

Unit tests should cover the materializer independently:

- Create active guidance for confirmed finance.
- Create locked guidance for economics.
- Disable guidance on reset.
- Rewrite only the managed block.
- Preserve user text before and after the managed block.
- Append a managed block when the file has user text but no markers.
- Reject multiple managed blocks.
- Reject invalid marker order.
- Inspect missing, active, disabled, absent, and invalid states.

Lifecycle tests should cover integration:

- `confirm` writes manifest, evidence memory, and active subject guidance.
- `lock` writes locked subject guidance.
- `unlock` rewrites locked guidance to confirmed guidance.
- `reset` disables or removes managed subject guidance.
- `dismiss` does not touch subject guidance.
- lifecycle status includes subject guidance status without creating files.

Guidance runtime tests should cover future-run behavior:

- `effective_guidance()` reads `.qiongli/guidance.d/subject-runtime.md`.
- `guidance_sources` includes the subject fragment as a project fragment.
- trace records include the subject fragment in `guidance_files_read`.
- `lint_project_guidance()` checks the subject fragment.

CLI and MCP tests should cover:

- `qiongli subject confirm finance --json` returns `subject_guidance.exists`.
- human status output includes a subject guidance line.
- `qiongli_subject_update` returns the same subject guidance packet.
- write failure returns a structured error and non-zero CLI exit.

Smoke tests should add one fixture:

- Confirm finance in an isolated project.
- Run the preview subject runtime smoke.
- Assert the effective guidance sources include
  `.qiongli/guidance.d/subject-runtime.md`.
- Assert the run remains preview-first and does not launch local agents.

## Success Criteria

- Confirming or locking a subject writes a managed project guidance fragment.
- Future `effective_guidance()` calls include the managed subject fragment.
- Reset returns the project to adaptive core and disables subject guidance.
- Dismissal does not mutate subject guidance.
- User-authored text outside the managed block is preserved.
- Invalid managed block states fail closed with actionable errors.
- CLI and MCP lifecycle packets expose subject guidance status.
- Existing subject router eval and preview smoke remain green.

## Rollback

Rollback is project-local:

1. Run `qiongli subject reset --cwd <project>`.
2. If necessary, remove `.qiongli/guidance.d/subject-runtime.md`.
3. Keep `.qiongli/trace/subject_evidence.json` for audit history unless the
   user explicitly wants to delete project trace data.

Repository rollback is also straightforward because the feature is isolated in
`subject_guidance.py` plus call sites in lifecycle, CLI, MCP, tests, and docs.

## Risks

- Over-activation: mitigated by requiring explicit confirm or lock before
  writing active subject guidance.
- User trust risk: mitigated by managed markers, status visibility, and reset.
- File safety risk: mitigated by replacing only the managed block and failing
  closed on invalid markers.
- Client parity risk: mitigated by returning the same `subject_guidance` packet
  through CLI and MCP.
- Guidance conflict risk: mitigated by existing guidance lint rules and by
  keeping canonical workflow contracts authoritative.

## Implementation Scope

This spec is suitable for a single implementation plan with six tasks:

1. Add `subject_guidance.py` and focused unit tests.
2. Connect subject guidance writes to lifecycle actions.
3. Extend subject status packets, CLI output, and MCP output.
4. Extend guidance runtime and trace assertions for subject fragments.
5. Add smoke coverage for confirmed-subject future runs.
6. Update docs and run focused verification.
