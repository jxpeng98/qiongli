# Build latest 2.x local macOS package

## Goal

Build and open a local macOS `Qiongli.app` from the exact latest
`origin/2.x` source so the user can inspect the integrated desktop manually.

## Confirmed Facts

- Latest fetched `origin/2.x` is
  `237de9ba9e235f2b5067cc9704aef49eee3ce9c6` on an Apple Silicon Mac.
- The repository-owned `pnpm desktop:macos` command builds embedded Svelte
  assets and the release-profile native executable, ad-hoc signs the App, and
  runs startup, CLI-version and embedded-content checks.
- This ordinary local source App deliberately has no packaged-product authority;
  Plugin/Skills installation requires the separate acceptance build.

## Requirements

- Re-fetch `origin/2.x` and build its exact head in a clean detached temporary
  worktree.
- Install the locked workspace dependencies if the temporary worktree lacks
  them, then reuse `pnpm desktop:macos` without changing source code.
- Copy the resulting App to
  `dist/local-2x-237de9ba/Qiongli.app` in the main workspace and open it for
  manual inspection.
- Verify the copied App's ad-hoc signature, startup check and bundled CLI version.

## Acceptance Criteria

- [x] The build source equals the latest fetched `origin/2.x` SHA and its
      temporary worktree is clean before building.
- [x] `dist/local-2x-237de9ba/Qiongli.app` exists and passes signature, startup
      and CLI-version checks.
- [x] The App opens for manual inspection.
- [x] No tracked source, real Host profile, tag, release or publication state is
      changed.

## Result

- Source: `237de9ba9e235f2b5067cc9704aef49eee3ce9c6`
- App: `dist/local-2x-237de9ba/Qiongli.app`
- Checks: ad-hoc signature valid; startup ready; bundled CLI
  `qiongli 2.0.0-alpha.3`; App opened successfully.

## Out of Scope

- Automated packaged-product acceptance, real Codex/Claude/Zotero mutations,
  Plugin/Skills installation authority, notarization, DMG creation or release
  qualification.
