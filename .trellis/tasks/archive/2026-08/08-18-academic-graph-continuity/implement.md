# Implementation Plan

## 1. Activate the bounded task

- [x] Review PRD/design/plan against the user's standing implementation
      approval and start `fix/academic-graph-continuity` from the latest
      protected `2.x` lineage.
- [x] Load `trellis-before-dev` for product control, native runtime, Desktop,
      and content distribution.

## 2. Correct readiness truth at the native owner

- [x] Add the smallest failing readiness regression for artifact-only
      containment and scholarly nodes with only structural edges.
- [x] Exclude structural nodes/relations only in readiness classification;
      preserve Graph v1 output and public counts.

Focused check:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-project academic_graph_readiness --locked
```

## 3. Correct Desktop meaning

- [x] Display semantic relations from existing relation counts and update the
      English/Chinese recovery copy to use Run in client plus rebuild.
- [x] Extend the existing readiness-panel test; add no route, state store, or
      graph renderer.

Focused check:

```bash
pnpm --dir packages/qiongli-desktop test -- AcademicGraphReadinessPanel
```

## 4. Make Plugin/Skill graph continuity executable

- [x] Add one Academic Graph continuity reference with stage-close and legacy
      append/update/preview/verify rules.
- [x] Route the root workflow and `academic-context-maintainer` through it.
- [x] Extend the existing academic-context continuity test with exact contract
      assertions.

Focused check:

```bash
python3 -m unittest tests.test_academic_context_continuity -v
```

## 5. Align priority and staged distribution

- [x] Update current product-control priority without changing the master
      233-task inventory or activating Graph v2/Kernel work.
- [x] Materialize Plugin/Skill outputs to a temporary staging directory, run
      the existing package audit and Skill quick validation, and confirm no
      generated source-tree payload changed.

## 6. Verify and package

- [x] Run `trellis-check`, native formatting/focused workspace tests, Desktop
      test/check/build, content/capability tests, roadmap check, and
      `git diff --check`.
- [x] Commit intentionally, push, open a PR, and resolve exact-head CI failures.
- [x] Build a local non-publishing macOS App from the exact tested source and
      report its path for manual inspection; do not claim publication authority.

## Risk and rollback points

- Do not reinterpret `relationCount`; App API still binds it to total edges.
- Do not infer semantic facts from arbitrary prose.
- Do not edit generated payloads in place.
- Do not mutate the user's live research project as part of repository tests.
- Any product/package-input change after package evidence requires rebuilding.
