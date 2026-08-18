# Implementation Plan

## 0. Re-audit CLI, Plugin, Skills, and MCP effectiveness

- [x] Trace current canonical content through embedded pack, standalone Skills,
      Codex/Claude Plugin composition, official Host activation, fresh Ready,
      and App edit/preview/confirm/reconcile/reset flows.
- [x] Run the focused current-source contract and isolated temporary-home Host
      tests, then the existing exact-source macOS packaged acceptance covering
      edit -> reconcile -> Ready -> reset -> canonical recovery.
- [x] If any link fails or lacks direct evidence, repair it at the existing
      shared owner, add one focused regression, and rerun the package gate before
      continuing. No product gap was found; the environment-only missing
      `PyYAML` failure was resolved by using the repository `.venv`.

## 1. Activate one bounded governance slice

- [x] After final plan approval, run `task.py start` and create
      `feat/schema-authority-compatibility` from the current local `2.x` head.
- [x] Load the product-control, App API, native runtime, reuse, and cross-layer
      Trellis specs with `trellis-before-dev`.
- [x] Mark only `GOV-408` and `GOV-409` active and regenerate the current index.

## 2. Record authority and transition truth

- [x] Add ADR 0216 and register it in
      `tooling/architecture/current-decisions.json`; keep the frozen ARC-201
      inventory unchanged.
- [x] Add one closed public-schema policy record covering App IPC, MCP, and
      public CLI JSON.
- [x] Record existing surfaces as frozen migration baselines and leave their
      initial change histories empty.

## 3. Enforce compatibility classification

- [x] Add one standard-library validator for exact keys, ordered coverage,
      canonical paths, closed classes, predecessor/version continuity, and
      breaking-change evidence.
- [x] Add one focused `unittest` module with positive repository validation and
      the minimum mutation cases required by the PRD.
- [x] Run the validator in the existing `Evaluation Truth` workflow.

Focused checks:

```bash
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy tests.test_arc_201_adrs -v
python tooling/scripts/update_program_roadmap.py --check
```

## 4. Verify unaffected consumers

- [x] Run the existing App Rust-fixture/Zod check, MCP v2 validator, and focused
      native CLI JSON tests without changing their wire shapes.

```bash
pnpm --dir packages/qiongli-app-api check
pnpm --dir packages/qiongli-app-api test
python scripts/validate_capability_contract.py
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test cli --locked
git diff --check
```

## 5. Integrate and record exact evidence

- [x] Run `trellis-check`, update the product-control spec, commit, push, open
      the PR, and resolve CI/review failures.
- [x] After exact implementation CI passes, update Program Ledger evidence for
      `GOV-408` and `GOV-409`, regenerate the index, and rerun required checks.
- [x] Merge only after exact-head required checks pass; archive and journal the
      task, then deploy the next machine-ordered Program Ledger item.

## Risk and rollback points

- Before policy adoption, no runtime or wire state changes.
- A validator false positive is contained to governance CI and can be fixed
  without changing product payloads.
- Reverting the PR removes the new decision/policy; it does not migrate or
  delete user data.
- Never rewrite the frozen ARC-201 registry or protected `2.x` history.
