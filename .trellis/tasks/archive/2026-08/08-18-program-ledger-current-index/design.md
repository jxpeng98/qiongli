# Technical design: program ledger v1

## Authority boundary

Keep the existing master roadmap as the ordered description of the program.
Add one adjacent JSON state ledger and derive one adjacent Markdown status
index from both inputs:

```text
master roadmap descriptions/order + program ledger state/evidence
  -> stdlib validator/generator
  -> generated current index
  -> existing Evaluation Truth V1 CI check
```

No state is inferred from Markdown checkboxes. Acceptance ledgers and ADRs keep
their existing release and architecture authority.

## Files

- `docs/superpowers/roadmaps/qiongli-program-ledger-v1.json` — state authority.
- `docs/superpowers/roadmaps/qiongli-current-program-index.md` — generated view.
- `tooling/scripts/update_program_roadmap.py` — validator/generator.
- `tests/test_program_roadmap.py` — focused contract regressions.
- `.github/workflows/evaluation-truth.yml` — one `--check` step in its existing
  Python job; Native CI stays free of legacy runtimes.

## Ledger contract

The top level has an exact schema identity, the canonical roadmap path, and a
`tasks` array. Rows use only the nine required fields. The task prefix is the
workstream; milestone, description, and ordering are derived from the roadmap.

The validator rejects unknown keys and non-canonical values. Repository
evidence is a non-empty list of existing, repository-relative regular files.
Accepted rows require a full lowercase Git commit SHA and decimal Actions run
ID. Other states may preserve partial evidence without being promoted.

Dependencies are direct task IDs. A depth-first traversal detects cycles after
unknown, duplicate, and self references are rejected.

## Roadmap parsing

Parse only the canonical checklist syntax already used by the master roadmap:

```text
- [ ] `TASK-123` Description
- [x] `TASK-123` Description
```

Track the nearest `## ... Milestone M0` through `M7` heading and derive the
workstream from the task prefix. The checkbox token is parsed only to locate a
task row; it never affects ledger state.

## Deterministic index

Render fixed UTF-8 Markdown with `\n` newlines, fixed state order, roadmap task
order, and no timestamps generated at runtime. Each milestone contains
workstream sections and compact rows for ID, state, description, dependencies,
and evidence/blocker. `--check` compares exact bytes and reports the update
command when stale.

## Evidence closeout

The implementation commit keeps `GOV-401` through `GOV-404` active. After its
exact-head PR CI passes, an evidence-only commit records that commit and run,
marks only those four accepted, regenerates the index, and reruns the same
check. This avoids inventing future CI evidence.

## Failure and rollback

- Validation never edits files in `--check` mode.
- Update mode writes only the generated index after all inputs validate.
- A revert removes the ledger/index/check together and restores the prior
  roadmap authority prose; it changes no product or release receipt.
