# Editable Skill propagation implementation

## 1. Desktop evidence

- [x] Add the context-maintainer Skill to the existing development transport
      customization fixture.
- [x] Extend `WorkflowContentPanel.test.ts` to select, edit, and preview that
      exact Skill without replacing the existing workflow-root coverage.

## 2. Native packaged evidence

- [x] Point the existing workflow-variant packaged acceptance scenario at the
      real context-maintainer Skill.
- [x] Verify the standalone source path and the host-specific nested Plugin
      projection for customized and canonical states.

## 3. Focused checks

```bash
pnpm --dir packages/qiongli-desktop test -- WorkflowContentPanel
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test codex_plugin_bundle --test claude_plugin_bundle
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content -p qiongli-config
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
git diff --check
```

## 4. Full release-shaped checks

- [x] Run Desktop test/check/build and affected native workspace tests.
- [x] Run content/distribution and capability-contract checks.
- [x] Run `pnpm desktop:macos:acceptance` and retain the schema-3
      non-publishing receipt.
- [x] Commit, push, open a PR, and resolve exact-head required CI.

## Rollback points

- Keep the change test-only; any production contract change returns to design.
- Do not write to normal Codex/Claude homes.
- Do not weaken receipt verification to make the marker assertion pass.
