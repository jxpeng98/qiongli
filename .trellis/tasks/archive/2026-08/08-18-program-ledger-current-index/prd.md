# Create machine-readable roadmap ledger and current index

## Goal

Implement `GOV-401` through `GOV-404` so the 233-task master roadmap has one
machine-readable live-state authority and one deterministic current index.
Markdown checkboxes remain long-term presentation and never establish accepted
state by themselves.

## Confirmed baseline

- The task starts from protected `2.x` merge `17f5b83177d7c4c4fae25588032b12645db9786a`.
- The master roadmap contains 233 task IDs, each exactly once.
- `EVAL-401` through `EVAL-407` entered protected `2.x` in merge
  `237de9ba9e235f2b5067cc9704aef49eee3ce9c6`; post-merge Evaluation Truth run
  `31984053266` passed that source.
- Historical Alpha 3 receipts remain release evidence only for their exact
  source. They do not make a reopened current release task accepted.

## Requirements

### R1 — One versioned ledger

- Add one JSON ledger containing every roadmap task ID exactly once and in
  roadmap order.
- Every row has exactly `id`, `state`, `owner`, `dependencies`, `evidence`,
  `commit`, `run`, `updated_at`, and `blocker`.
- Allowed states are `proposed`, `active`, `accepted`, `blocked`, `deferred`,
  and `superseded`.
- Dependencies reference known task IDs, contain no duplicates or self edges,
  and form an acyclic graph.

### R2 — Accepted means evidenced

- `accepted` requires at least one existing repository-relative evidence path,
  one exact 40-character commit identity, and one exact numeric Actions run ID.
- A checked roadmap box, local-only work, stale package, or historical receipt
  for a different source cannot establish current acceptance.
- Seed `EVAL-401` through `EVAL-407` from their protected merge and post-merge
  run. Keep unresolved work non-accepted.
- Keep `GOV-401` through `GOV-404` active until this PR has exact-head CI
  evidence; an evidence-only follow-up may then mark only those four accepted.

### R3 — One stdlib validator/generator

- Add one Python-standard-library command with update and `--check` modes.
- Validate schema, ID equality/order, states, owners, dependency closure/cycles,
  accepted evidence, dates, and deterministic generated-output freshness.
- Fail with concise actionable errors and a non-zero exit status.

### R4 — Generated current index

- Generate one Markdown index grouped by roadmap milestone and workstream.
- Preserve roadmap task descriptions and ordering by deriving them from the
  roadmap, not duplicating them in the JSON ledger.
- Show state counts, state meanings, dependencies, blockers, and accepted
  evidence compactly; label the file generated and non-editable.
- Update roadmap authority prose and its README to point live status at this
  index while preserving the master roadmap as sequencing authority.

### R5 — Existing CI owner

- Run `--check` in the existing 2.x `Evaluation Truth V1` Python job; preserve
  the native workflow's no-Python boundary.
- Add focused standard-library unit tests for every required rejection class.
- Do not add a dependency, service, database, umbrella governance framework,
  GitHub mutation, release action, or duplicate backlog.

## Acceptance Criteria

- [x] Ledger and roadmap contain the same 233 unique IDs in the same order.
- [x] Invalid state, duplicate/missing ID, unknown/cyclic dependency,
      accepted-without-evidence, and stale generated index fail locally.
- [x] The generated index is byte-deterministic and groups all tasks by
      milestone/workstream while retaining their roadmap descriptions.
- [x] `EVAL-401` through `EVAL-407` cite protected merge/post-merge evidence;
      unresolved items are not upgraded from checkboxes.
- [x] The existing 2.x Evaluation Truth CI owner runs the same `--check` command.
- [x] After exact-head CI evidence exists, only `GOV-401` through `GOV-404` are
      advanced from active to accepted in the evidence closeout.

## Out of scope

- Editing GitHub Issues, Project fields, Milestones, tags, Releases, or public
  release state.
- Implementing `EVAL-408`, later GOV tasks, or product behavior.
- Rewriting long-term roadmap descriptions into JSON or replacing acceptance
  ledgers and ADRs.
