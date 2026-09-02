# Implementation Plan — lightweight Qiongli 2.x delivery

## 1. Close Alpha 5 truthfully

- Archive the superseded public-release task as an internal candidate decision.
- Update the Alpha 5 changelog and release note without changing versioned
  product source or publishing anything.

## 2. Separate task and Agent flow

- Add the Qiongli delivery lanes and authority chain to `.trellis/workflow.md`.
- Make task completion Focused by default; enter Slice only for a PR/integration
  checkpoint and Acceptance only for a release candidate.
- Enable existing Codex auto dispatch for planned complex tasks; keep small work
  outside a Trellis task inline and keep channel coordination opt-in.
- Condense the product-control verification contract without weakening trust,
  schema, path, authorization, or data-loss checks.

## 3. Simplify contributor and delivery entrypoints

- Replace obsolete migration-inventory detail in `CONTRIBUTING.md` with one
  roadmap/four-lane map and existing owner links.
- Condense `.github/delivery-checklists.md` while preserving required markers,
  evidence classes, ordered sections, and authorization boundaries.

## 4. Make lightweight PR classification real

- Extend `check_2x_native_change_boundary.sh` only for pure non-runtime docs,
  Trellis process files, and existing evidence paths.
- Add one focused regression covering the new path class; retain existing mixed,
  runtime, frozen, nested-fixture, unknown, and empty-diff tests.
- Update bilingual branch policy descriptions to match the classifier.

## 5. Restore one roadmap view

- Add a concise `NOW / NEXT / LATER` horizon to the master roadmap.
- State that the master owns order, while the ledger/index own state and evidence.
- Group `PLT-401`–`PLT-408` into three next work packages and
  `SEC-401`–`SEC-405` into one security package without changing task IDs.

## 6. Focused validation

```bash
.venv/bin/python -m unittest \
  tests.test_native_change_boundary \
  tests.test_branch_policy \
  tests.test_authorization_policy \
  tests.test_program_roadmap \
  tests.test_release_note_versions \
  tests.test_release_version_contract -v
python3 tooling/scripts/update_program_roadmap.py --check
python3 ./.trellis/scripts/task.py validate \
  .trellis/tasks/09-03-simplify-2x-delivery-flow
python3 ./.trellis/scripts/get_context.py --mode phase --step 2.2
git diff --check
```

Do not run a release candidate, package promotion, signing, publication, or
public download check for this process task.

## Validation result — 2026-09-03

- Shell syntax passed for `scripts/check_2x_native_change_boundary.sh`.
- All 60 focused boundary, branch-policy, authorization, roadmap, and release
  version/note tests passed.
- The 237-task program roadmap is current.
- Trellis task validation and `git diff --check` passed.
- No package, promotion, signing, publication, or announcement action ran.
