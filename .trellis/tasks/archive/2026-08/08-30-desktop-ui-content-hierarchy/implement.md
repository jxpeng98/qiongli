# Desktop UI content hierarchy implementation

## 1. Establish the visual baseline

- [x] Record populated fixture measurements for Overview, Research Library,
      Academic Graph, Client Integrations, and About at the default desktop
      viewport, including rendered font-size distribution, overflow, and scroll
      owners.
- [x] Update the compact-Nova source contract in `src/app-css.test.ts` so it
      protects the chosen semantic scale rather than exact compact values.

## 2. Fix shared type and density at the owner

- [x] Add semantic body/supporting/label/micro type roles to `src/app.css` and
      restore comfortable page, section, state, metric, and tab tokens.
- [x] Keep Geist, the current theme bridge, neutral colors, radius model,
      reduced-motion behavior, focus rings, and coarse-pointer targets.
- [x] Align low-level card/control defaults only where they currently override
      the shared scale.
- [x] Add or update the smallest source/component tests that fail if ordinary
      body copy falls back to the micro role.

Validation:

```bash
pnpm --dir packages/qiongli-desktop test -- src/app-css.test.ts src/lib/components/app/UnifiedUi.test.ts
pnpm --dir packages/qiongli-desktop check
```

Rollback point: shared token and primitive changes form one reviewable unit and
can be reverted without affecting route behavior.

## 3. Rebalance shared composition primitives

- [x] Update `PageHeader` and `SectionHeader` hierarchy, title/action spacing,
      and supporting-description treatment.
- [x] Update `StatePanel`, `MetricCard`, `InfoGrid`, and `DescriptionGrid` so
      content grouping does not require nested equal-weight borders.
- [x] Update `ProjectWorkspaceBar`, tabs, status badges, and description tips to
      use the semantic roles and remain readable with Chinese labels.
- [x] Preserve existing component APIs unless one small optional prop avoids
      route duplication; do not add a parallel component family.

Validation:

```bash
pnpm --dir packages/qiongli-desktop test -- src/lib/components/app
pnpm --dir packages/qiongli-desktop check
```

## 4. Apply task-first hierarchy to routed surfaces

- [x] Refine Overview and Research Library first; verify their populated source
      fixtures before using the pattern elsewhere.
- [x] Refine Project Workspace routes: Artifacts, Captures, Academic Graph,
      Timeline, and Orchestrator. Separate controls/results and move long
      technical evidence into existing disclosure surfaces without hiding
      safety or recovery information.
- [x] Refine Client Integrations, workflow content, and About. Keep readiness and
      next actions visible; subordinate versions, paths, hashes, and diagnostics.
- [x] Replace route-local literal `10-12px` usage with semantic roles, retaining
      micro text only for genuine terse metadata.
- [x] Update English and Chinese copy together only where visible content is
      shortened or regrouped.

After each route group:

```bash
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop check
```

Rollback point: route groups remain separate from shared-system changes so an
individual page can return to its prior composition without losing the new
global readability scale.

## 5. Visual and responsive verification

- [x] Use `?fixture=source-read-only` to inspect populated light and dark modes
      for the five representative routes.
- [x] Verify normal desktop, 1024px-wide, sidebar-crowded, and narrow layouts for
      clipping, translated-label wrapping, horizontal overflow, and competing
      vertical scroll containers.
- [x] Confirm hover, pressed, focus-visible, disabled, loading, empty, error,
      warning, and destructive states remain distinguishable.
- [x] Compare post-change rendered font distribution with the recorded baseline;
      ordinary descriptions should be 13-14px, while 11-12px is limited to
      labels and technical metadata.

## 6. Quality gate

```bash
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop check
pnpm --dir packages/qiongli-desktop build
git diff --check
```

- [x] Review the final diff for accidental App API/native changes, new
      dependencies, duplicate primitives, hidden safety copy, and unrelated
      formatting churn.
- [x] Re-run the populated fixture audit after the production build.
