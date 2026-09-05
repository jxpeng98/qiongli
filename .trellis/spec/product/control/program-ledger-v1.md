# Program Ledger v1

## 1. Scope / Trigger

Use this contract whenever a master-roadmap task is added, reordered, changes
live state, gains acceptance evidence, or when the generated current index is
updated. It prevents Markdown checkboxes and stale prose from becoming a second
status authority.

## 2. Signatures

```bash
python3 tooling/scripts/update_program_roadmap.py
python3 tooling/scripts/update_program_roadmap.py --check
```

Inputs and output:

- descriptions/order: `docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md`;
- state/evidence: `docs/superpowers/roadmaps/qiongli-program-ledger-v1.json`;
- generated view: `docs/superpowers/roadmaps/qiongli-current-program-index.md`.

## 3. Contracts

- Top-level JSON keys are exactly `schema_version`, `roadmap`, and `tasks`;
  schema identity is `qiongli-program-ledger/v1`.
- Every task row has exactly `id`, `state`, `owner`, `dependencies`, `evidence`,
  `commit`, `run`, `updated_at`, and `blocker`.
- State is one of `proposed`, `active`, `accepted`, `blocked`, `deferred`, or
  `superseded`.
- Ledger IDs and order exactly match the 249 canonical roadmap checklist rows.
- `accepted` requires an existing repository evidence file, a 40-character
  lowercase commit SHA, and a decimal Actions run ID.
- The generated index derives descriptions, milestone, workstream, and order
  from the roadmap; it derives all live state from the ledger.
- The 2.x Evaluation Truth workflow owns `--check`. Native CI remains free of
  Python and other legacy-language runtime launches.

## 4. Validation & Error Matrix

- invalid/missing/extra schema field -> fail;
- duplicate, missing, extra, or reordered task ID -> fail;
- unknown state or malformed date/commit/run/path -> fail;
- unknown, duplicate, self, or cyclic dependency -> fail;
- `accepted` without complete existing evidence -> fail;
- `blocked` without a blocker -> fail;
- missing or byte-stale generated index in `--check` mode -> fail with the
  update command.

## 5. Good / Base / Bad Cases

- Good: an exact-head CI run is recorded with repository evidence and the task
  becomes `accepted`; regeneration changes only the derived index.
- Base: a proposed or deferred task has empty evidence/commit/run fields and
  remains visible in roadmap order.
- Bad: checking a roadmap box or merging a PR without exact run evidence is
  treated as acceptance.

## 6. Tests Required

- Assert the repository has 249 equal, ordered IDs and a byte-current index.
- Mutate state, ID inventory, dependency graph, accepted evidence, and index
  bytes; assert each mutation fails.
- Assert `.github/workflows/evaluation-truth.yml` runs the same `--check`
  command and existing branch-policy tests keep Native CI Python-free.

## 7. Wrong vs Correct

Wrong: edit a checkbox/current-status paragraph and infer that the task is
accepted.

Correct: update the ledger with exact evidence, regenerate the index, and let
`--check` prove that roadmap identity/order and live state still agree.
