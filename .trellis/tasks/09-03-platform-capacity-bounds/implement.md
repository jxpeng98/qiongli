# Implementation Plan -- platform capacity and bounds baseline

## 0. Record active roadmap state

- [x] Mark only `PLT-401`--`PLT-403` active in Program Ledger v1 and regenerate
  the current program index.

## 1. Add the deterministic project fixture owner

- Add one `#[cfg(test)]` module to `qiongli-project`; do not expose fixture or
  limit APIs from production builds.
- Generate independent small, medium, exact-limit, and `limit + 1` Library,
  Capture, Graph, Portfolio, and portable-project cases from current native
  constants and builders.
- Leave one normal, cheap contract test for explicit limit identities and
  percentile calculation. Keep filesystem-heavy exact/one-over cases in the
  ignored capacity run.
- Measure project snapshot, refresh, Capture load, Graph build/query, Portfolio
  rebuild, portable export/import, and resident memory; write the project JSON
  receipt part.

## 2. Add the Desktop receipt part

- Add one `#[cfg(test)]` module to the `qiongli` library.
- Reuse isolated `CommandEnvironment`, embedded content,
  `validate_desktop_startup`, and `app_snapshot_json` rather than adding a new
  Desktop surface.
- Measure startup and snapshot construction and record IPC payload bytes for the
  same profiles, source, target, sample count, and fixture identity.
- Write a separate Desktop JSON receipt part in the shared output directory.

## 3. Keep the heavy run manual

- Add a `workflow_dispatch`-only capacity step and artifact upload to the
  existing Linux/macOS/Windows Rust foundation matrix.
- Do not add a workflow, PR trigger, schedule, package job, release job, or
  performance threshold.
- Document the one release-mode workspace command and its output contract in
  the existing local Desktop build guide.

## 4. Focused local validation

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli-project --lib platform_capacity_contract --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --lib platform_capacity_contract --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --lib --release --no-run --locked platform_capacity_baseline
.venv/bin/python -m unittest tests.test_branch_policy -v
python3 ./.trellis/scripts/task.py validate \
  .trellis/tasks/09-03-platform-capacity-bounds
git diff --check
```

The full product-limit measurement is intentionally absent from this local
Focused gate.

## 5. Obtain explicit remote-run authority and collect evidence

- After local review, request separate authorization to commit/push the feature
  branch and manually dispatch Native CI; implementation approval alone does
  not grant repository push or workflow-run authority.
- Dispatch the exact feature-branch head once. Confirm Linux, macOS, and Windows
  foundation jobs upload their two-part receipt sets and branch-restricted
  package/candidate/promotion jobs remain skipped.
- Download and validate all receipt parts against the exact source, target set,
  sample count, metric inventory, profile sizes, and fixture identities.

## 6. Close evidence and roadmap state

- Add one concise acceptance record with exact source/run IDs and all P50/P95
  observations; do not describe hosted-runner data as an SLO.
- Move `PLT-401`, `PLT-402`, and `PLT-403` from `proposed` to `accepted` only
  when the three target receipts and fail-closed cases are complete, then
  regenerate the current program index.
- Run the roadmap, branch-policy, task, Rust formatting, affected native tests,
  and diff checks again. A ready source-affecting PR later owns the normal exact-
  head Slice; no release action is part of this task.

## Rollback points

- Before the manual run: revert only the test modules, module declarations,
  workflow steps, and guide entry.
- After the manual run: preserve the historical run identity, revert the code
  and ledger acceptance together, and state that the observation was withdrawn.
- No rollback touches user projects, config roots, packages, tags, or releases.
