# Implementation Plan: editable packaged Plugin and Skills content

## 1. Start from the accepted target baseline

- [x] Fetch `origin/2.x`, create `feat/packaged-workflow-variants` from exact
      `237de9ba`, and update this task's branch/worktree metadata if the target
      moved before implementation.
- [x] Run Trellis before-dev context for content distribution, App API, Desktop
      frontend, native runtime, and product control.
- [x] Add a superseding ADR for local instruction-variant authority; preserve
      ADR 0213's fixed Host CLI and fresh-Ready boundaries.
- [x] Write one failing native test showing that an allowed Skill edit cannot
      currently remain managed and Ready.

## 2. Add the receipt-owned local variant

- [x] Implement the private managed variant document/tree with canonical path
      allowlisting, UTF-8/control/size limits, deterministic digest, revision,
      lock, atomic promotion, compare-and-swap, verify, and exact reset.
- [x] Reuse canonical JSON, SHA-256, path containment, private permissions, and
      transaction patterns already present in native crates.
- [x] Add focused tests for valid update/reset, unsupported path, stale base or
      revision, oversize/invalid content, link/drift, deterministic identity,
      and unrelated-file preservation.

Focused check:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib workflow_variant --locked
```

## 3. Derive standalone Skills and Plugin bundles from the variant

- [x] Extend shared profile projection with an optional validated override map.
- [x] Advance standalone materialization and Codex/Claude bundle receipts with
      backward-readable canonical-parent plus optional variant identity.
- [x] Keep existing canonical compose calls as thin no-override wrappers.
- [x] Make verification, update, drift, detach/remove, and managed/cache
      comparison exact for both canonical and customized receipts.
- [x] Test customized bytes, unchanged binary/MCP/manifests, receipt drift,
      canonical reset, and old canonical receipt compatibility.

Focused checks:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content -p qiongli-platform --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test codex_plugin_bundle --test claude_plugin_bundle --locked
```

## 4. Expose preview/apply and edit in the existing App surface

- [x] Evolve the App API content-customization model with resource editability,
      canonical/current digests, override state, and selected variant identity.
- [x] Add preview replace/reset intents that commit only through the existing
      confirmation operation.
- [x] Update Rust fixture, TypeScript schema/tests, browser transport, reducer,
      English/Chinese copy, and `WorkflowContentPanel` in the same slice.
- [x] Keep manifests read-only and project guidance separate; use a native
      textarea and existing controls, with focused accessibility/state tests.

Focused checks:

```bash
pnpm --dir packages/qiongli-app-api check
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop test -- WorkflowContentPanel
pnpm --dir packages/qiongli-desktop check
```

## 5. Bind activation and fresh Ready to the selected variant

- [x] Make variant changes mark affected standalone/Host destinations
      update/repair-required without silently changing them.
- [x] Reconcile through the existing packaged transaction and fixed official
      Codex/Claude CLI plans only.
- [x] Compare freshly observed managed/cache receipts with the exact selected
      variant and label Ready provenance as Canonical or Customized.
- [ ] Extend fake-client failure tests and packaged control-plane acceptance to
      cover customize -> reconcile -> fresh Ready -> reset -> reconcile.

Focused check:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib host_ --locked
```

## 6. Freeze and verify the exact product source

- [x] Run formatting, App API/Desktop checks/tests/build, capability validation,
      and the native workspace all-target/all-feature test suite.
- [x] Run the existing ignored real Codex and Claude clean-client tests with
      absolute installed client binaries and isolated temporary homes; do not
      use authenticated prompts or the normal profiles.
- [ ] Commit the frozen product source, then run the existing macOS
      packaged-product acceptance because the script requires an exact clean
      commit and the package inputs changed.
- [ ] Open the accepted App in its isolated manual home and manually verify one
      edit, reconcile, Ready label, reset, and canonical recovery.
- [x] Any later product/package change invalidates that receipt and reruns the
      package gate.

Quality gate:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
pnpm desktop:test
pnpm desktop:check
pnpm desktop:build
python3 scripts/validate_capability_contract.py
pnpm desktop:macos:acceptance
git diff --check
```

## 6A. Unify typography and connect Research Library to Academic Graph

- [x] Make native inputs, selects, and textareas inherit the existing App font;
      align SVG/Cytoscape labels with the same stack without adding a font.
- [x] Reuse `AcademicGraphPortfolio` in compact mode on Research Library,
      revision-bound to the current library snapshot and with explicit
      loading/failure states.
- [x] Preserve project selection and the existing full Academic Graph route;
      do not add a second graph model or editable inferred edges.
- [x] Add the smallest focused CSS/component tests and include them in Desktop
      check/test/build and packaged manual verification.

## 7. Review, integrate, and continue without another routine authorization

- [ ] Run Trellis check, update executable specs, commit the evidence-only
      closeout separately, push, create the PR, and resolve CI/review failures.
- [ ] Merge through branch protection only after exact-head checks pass; do not
      tag, publish, or decide a protected release environment.
- [ ] Archive this task, then create and execute the already approved
      `GOV-401`–`GOV-404` ledger/index task.

## Pre-freeze evidence

- App API: 32 tests passed; TypeScript check passed.
- Desktop: 247 tests passed; Svelte check reported 0 errors and 0 warnings;
  production bundle contract passed.
- Native: workspace `--all-targets --all-features --locked` and Clippy
  `-D warnings` passed; focused config/content/platform and bundle tests passed.
- Installed clients: isolated Codex CLI 0.147.0 and Claude Code 2.1.231 tests
  passed using temporary homes only.
- Capability Contract v2 and `git diff --check` passed.

## Rollback points

- Before any confirmed variant write: no managed destination changes.
- After storing a variant but before reconcile: reset the verified variant or
  leave destinations canonical.
- After customized reconcile: reset, then explicitly reconcile through the same
  official Host flow; never delete Host caches directly.
- After merge: use a new revert/fix PR; never rewrite `2.x` history.
