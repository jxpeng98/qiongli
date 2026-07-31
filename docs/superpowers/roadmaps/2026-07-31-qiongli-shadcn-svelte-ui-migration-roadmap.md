# Qiongli shadcn-svelte UI Migration Roadmap

Status: completed and locally accepted on July 31, 2026. The migration is
implemented on `feat/shadcn-svelte-migration`, with recoverable Conventional
Commit checkpoints for the baseline, foundation, route waves, regression
coverage, and legacy removal.

Visual direction amendment: the final product styling now uses the current
shadcn-svelte Rhea + Neutral baseline. Rhea's compact controls, tighter gaps,
Inter typography, black/white primary actions, neutral
opaque surfaces, fine borders, and restrained shadows are applied through the
generated component layer and Qiongli semantic tokens. This amendment
supersedes later roadmap text that requires a teal primary accent, Liquid Glass,
Nova, or Vega in the application shell.

Geometry amendment: Qiongli intentionally restrains the registry-default Rhea
rounding. Controls use an 8px radius, inset information groups 10px, cards 16px,
and dialogs 18px. Repeated facts use one bordered `InfoGrid` with internal
dividers instead of individually rounded nested cards. Pill and circular shapes
remain limited to status, avatar, and graph-shape semantics. These rules are
part of the accepted component boundary and must be preserved when generated
components are refreshed.

Decision date: July 31, 2026

Current worktree: `.worktrees/ui-visual-refinement`

Implementation branch: `feat/shadcn-svelte-migration`

Product surface: `packages/qiongli-desktop`

## Executive Decision

Migrate the Qiongli Desktop UI to one layered system:

```text
Qiongli routes and feature components
  -> Qiongli application patterns
  -> shadcn-svelte components (Rhea, Neutral base)
  -> Bits UI behavior primitives
  -> Svelte 5 + Tailwind CSS 4
```

shadcn-svelte owns the reusable visual component layer. Bits UI remains the
behavior and accessibility foundation used by shadcn-svelte, but routes and
feature components must not import it directly. Qiongli uses a Neutral primary
palette, keeps domain status colors and the graph palette, and uses opaque
surfaces throughout the application shell.

This migration does not add Skeleton, Melt UI, UnoCSS, Flowbite, or a second
component framework. It also does not rewrite business behavior, Tauri command
contracts, project state, or Cytoscape rendering.

## Why This Migration Is Bounded but Material

At the decision checkpoint, the Svelte application already had a useful
transition boundary:

- Svelte 5.56.6, Tailwind CSS 4.3.3, and Bits UI 2.18.1 are installed;
- `src/lib/shared/ui/primitives.ts` is the only direct Bits UI import;
- shared Tabs, dialog, page-state, header, badge, and metric patterns exist;
- `src/app.css` already separates primitive, semantic, and component tokens;
- persistent light and dark modes and reduced-transparency fallbacks exist;
- document-level horizontal overflow is guarded and currently tested.

The remaining migration surface is still substantial:

- 12 route files;
- 139 native buttons;
- 34 selects and 12 inputs;
- 43 Svelte files with local style blocks;
- one data table with local horizontal scrolling;
- several dense interaction surfaces in Captures and Academic Graph.

The roadmap therefore uses a compatibility bridge and page waves. It does not
perform a repository-wide generated-code replacement in one commit.

## Design-System Authority

The final system uses three token layers:

```text
Primitive tokens
  -> raw color ramps, spacing, radius, typography, duration, shadows

Semantic tokens
  -> background, foreground, card, muted, primary, destructive, border, ring

Component tokens and variants
  -> button, input, badge, state panel, workspace bar, solid shell surfaces
```

Rules:

- routes and feature components consume semantic utilities or component APIs;
- raw color values are forbidden in general-purpose components;
- graph visualization colors are an explicit documented exception;
- dark mode overrides semantic tokens, not individual pages;
- component states follow `disabled > loading > active > focus > hover > default`;
- focus indicators, normal text, and UI boundaries retain measurable contrast;
- gradients, backdrop blur, and glass-specific tokens are not part of the
  application surface system.

## Target Source Layout

```text
packages/qiongli-desktop/src/
  lib/
    components/
      ui/                 # shadcn-svelte open-code components
      app/                # Qiongli application patterns
      legacy/             # temporary migration adapters only
    features/             # domain components and behavior
    styles/
      shadcn.css          # semantic bridge consumed by generated components
    utils.ts              # cn() and shared presentation utilities
  routes/                 # page composition, no local design system
  app.css                 # primitive, semantic, component, and base tokens
```

Import policy after migration:

| Consumer | May import | Must not import |
|---|---|---|
| Route | `components/app`, `components/ui`, feature views | `bits-ui`, legacy UI |
| Feature | `components/app`, `components/ui` | `bits-ui`, route styles |
| App pattern | `components/ui` | route code |
| shadcn component | Bits UI and shared utilities | feature or route code |

## Critical Path

```text
UI-0 checkpoint current baseline
  -> UI-1 bootstrap shadcn-svelte safely
  -> UI-2 converge tokens and theme ownership
  -> UI-3 migrate atomic and behavioral primitives
  -> UI-4 rebuild Qiongli application patterns
  -> UI-5 migrate the application shell
  -> UI-6 migrate all route waves
  -> UI-7 close responsive and accessibility gaps
  -> UI-8 remove compatibility code and build the local App
```

No milestone may begin before the preceding milestone has a green local gate
and a recoverable Conventional Commit checkpoint.

## Milestone Ledger

| ID | Outcome | Size | Depends on |
|---|---|---:|---|
| UI-0 | Recoverable visual baseline | S | current worktree |
| UI-1 | Reproducible shadcn-svelte foundation | M | UI-0 |
| UI-2 | One light/dark token authority | M | UI-1 |
| UI-3 | Shared atomic and behavioral components | L | UI-2 |
| UI-4 | Product-level Qiongli patterns | L | UI-3 |
| UI-5 | Unified shell, navigation, and overlays | L | UI-4 |
| UI-6A | Low-risk route migration | M | UI-5 |
| UI-6B | Workflow route migration | L | UI-6A |
| UI-6C | Data-heavy route migration | L | UI-6B |
| UI-6D | Captures and Academic Graph migration | XL | UI-6C |
| UI-7 | Responsive, contrast, and keyboard closure | L | UI-6D |
| UI-8 | Legacy removal and packaged acceptance | M | UI-7 |

## UI-0 — Checkpoint the Current Visual Baseline

Purpose: protect the current visual work and make every later migration step
reversible.

Deliverables:

- review and commit the current `feat/ui-visual-refinement` changes;
- record the exact commit as the visual comparison authority;
- create `feat/shadcn-svelte-migration` from that checkpoint;
- retain the current screenshots for light, dark, desktop, and narrow layouts;
- record current check, test, production-build, and local-App results;
- ensure unrelated conversations no longer mutate the migration worktree.

Exit gate:

- clean migration worktree;
- baseline commit is reachable from the migration branch;
- `pnpm desktop:check`, `pnpm desktop:test`, and `pnpm desktop:build` pass;
- no migration dependency or generated file has been added.

Recommended commit:

```text
refactor(ui): establish unified visual system baseline
```

## UI-1 — Bootstrap shadcn-svelte Without Overwriting the Baseline

Purpose: introduce a reproducible component-generation boundary without
allowing the initializer to replace the existing global design work.

Implementation:

1. Run shadcn-svelte initialization in a temporary SvelteKit project using the
   same Svelte and Tailwind major versions.
2. Select the current Rhea style (the compact, rounded product baseline),
   Neutral base color, CSS variables, and
   `$lib/components/ui` as the UI alias.
3. Review the generated `components.json`, utility module, CSS variables, and
   dependency changes.
4. Apply only the reviewed files and token blocks to Qiongli.
5. Add explicit aliases to `svelte.config.js` if required by the generated
   configuration.
6. Pin the generator version used by maintenance documentation or scripts.

Initial dependency policy:

- retain `bits-ui` and `@lucide/svelte`;
- add only dependencies required by imported components;
- do not add every registry component in advance;
- review `pnpm-lock.yaml` after each component batch;
- treat `components/ui` as owned open code, not an untouchable vendor bundle.

Exit gate:

- `components.json` and `cn()` resolve correctly;
- one generated Button renders in a focused test fixture;
- existing routes remain visually and behaviorally unchanged;
- `app.css` has not been replaced wholesale;
- dependency versions and registry source are reproducible.

Recommended commit:

```text
chore(ui): initialize shadcn-svelte foundation
```

## UI-2 — Converge Tokens and Theme Ownership

Purpose: make shadcn-svelte and existing Qiongli surfaces read from one theme
without losing the accepted teal identity or dark-mode contrast.

Semantic mapping:

| Existing Qiongli token | Target semantic token |
|---|---|
| `--color-canvas` | `--background` |
| `--color-ink` | `--foreground` |
| `--color-surface` | `--card`, `--popover` |
| `--color-surface-subtle` | `--muted`, `--secondary` |
| `--color-muted` | `--muted-foreground` |
| `--color-accent-strong` | `--primary` |
| `--color-on-accent` | `--primary-foreground` |
| `--color-accent-soft` | `--accent` |
| `--color-danger` | `--destructive` |
| `--color-border` | `--border`, `--input` |
| `--color-focus` | `--ring` |

Implementation:

- convert semantic colors to one reviewed OKLCH palette;
- retain Qiongli success, warning, info, and graph-layer extensions;
- provide temporary aliases for existing token names;
- make `.dark` the final theme selector;
- during migration, synchronize `.dark` and the existing `data-theme` value
  from one controller;
- restore the saved theme before the first visible paint;
- retain system preference, explicit user choice, and `color-scheme` metadata;
- add automated contrast checks for light and dark semantic pairs.

Dark glass target:

- allowed only for the sidebar, project workspace bar, and blocking overlays;
- use approximately 94-97% opaque dark surfaces;
- reduce blur from 16px to approximately 8-10px;
- keep saturation at or below 1.0;
- reduce highlight opacity to approximately 0.03-0.04;
- remove content-card glass and stacked inset highlights;
- retain reduced-transparency and increased-contrast fallbacks.

Exit gate:

- light and dark modes use one semantic token set;
- normal text reaches at least 4.5:1 contrast;
- controls and focus indicators reach at least 3:1 contrast;
- theme switching has no visible first-paint flash;
- ordinary cards remain opaque in both themes.

Recommended commit:

```text
feat(theme): map qiongli tokens to shadcn semantics
```

## UI-3 — Migrate Atomic and Behavioral Primitives

Purpose: replace page-level classes and the temporary Bits UI gateway with a
complete, tested component foundation.

Batch A, visual atoms:

- Button;
- Badge;
- Card;
- Separator;
- Input and Label;
- Native Select and Select;
- Checkbox and Switch;
- Alert;
- Skeleton and Progress.

Batch B, behavioral primitives:

- Tabs;
- Alert Dialog and Dialog;
- Dropdown Menu;
- Popover and Tooltip;
- Collapsible;
- Sidebar or Sheet where required.

Variant mapping:

| Existing style | shadcn-svelte target |
|---|---|
| `.button-primary` | `Button` default |
| `.button-secondary` | `Button` secondary or outline |
| `.button-quiet` | `Button` ghost |
| `.button-danger` | `Button` destructive |
| `StatusBadge` tones | `Badge` semantic variants |
| shared Tabs wrappers | shadcn Tabs |
| confirmation overlay | Alert Dialog |

Behavior gate for every interactive component:

- keyboard traversal and activation;
- visible focus and focus return;
- Escape and outside-interaction behavior where applicable;
- disabled and loading precedence;
- accessible labels and descriptions;
- portal and z-index behavior;
- touch target and pointer-density variants.

Exit gate:

- no route or feature adds a new raw `.button-*` usage;
- direct `bits-ui` imports exist only inside `components/ui`;
- component tests cover state and keyboard contracts;
- existing confirmation and Tabs flows retain behavior.

Recommended commits:

```text
refactor(ui): migrate visual primitives to shadcn-svelte
refactor(ui): migrate interactive primitives to shadcn-svelte
```

## UI-4 — Rebuild Qiongli Application Patterns

Purpose: preserve a stable product vocabulary above generic shadcn components.

Patterns:

- PageHeader;
- SectionHeader;
- StatePanel;
- FeedbackBanner;
- MetricCard and MetricGrid;
- ProjectWorkspaceBar;
- AppSidebar;
- FilterBar;
- ActionGroup;
- ResponsiveDataView;
- destructive-operation confirmation content.

These components own Qiongli composition, density, icon placement, wording
slots, and responsive rules. They must not duplicate shadcn primitive behavior.

Exit gate:

- all patterns render in light and dark component tests;
- state patterns expose loading, empty, warning, error, and recovery semantics;
- repeated page structures have no local clones;
- pattern APIs accept content and actions without route-specific imports.

Recommended commit:

```text
refactor(ui): rebuild qiongli application patterns
```

## UI-5 — Migrate the Application Shell

Purpose: establish the final visual authority before migrating individual
pages.

Scope:

- global sidebar and primary navigation;
- brand block;
- language selector;
- theme toggle;
- refresh and native runtime status;
- project workspace navigation;
- global notice layer;
- confirmation dialog boundary;
- responsive mobile navigation behavior.

Shell glass policy:

- glass is a shell material, not a card variant;
- mobile Sheet and Dialog surfaces prioritize readability over translucency;
- dark glass is flatter and more opaque than light glass;
- backdrop filters always have opaque fallbacks.

Exit gate:

- global navigation and project navigation remain distinct;
- all navigation is keyboard reachable;
- active route and current project are visually unambiguous;
- shell works at 360px width and 200% zoom;
- no document-level horizontal overflow exists.

Recommended commit:

```text
refactor(shell): rebuild desktop shell with shadcn-svelte
```

## UI-6 — Route Migration Waves

### UI-6A — Low-Risk Information Routes

Routes:

- Overview;
- Artifacts;
- About.

Primary components: Card, Button, Badge, Alert, Progress, StatePanel, and
SectionHeader.

Exit gate: all three routes use shadcn primitives or Qiongli patterns and have
no page-local button, control, card, or state-system definitions.

### UI-6B — Workflow Routes

Routes:

- Portfolio;
- Timeline;
- Orchestrator.

Primary components: FilterBar, Select, Date/Input controls, Badge, ActionGroup,
Collapsible, confirmation actions, and responsive result cards.

Exit gate: filtering, pagination, run control, destructive actions, loading,
and recovery behavior pass existing and new focused tests.

### UI-6C — Data-Heavy Management Routes

Routes:

- Client Integrations;
- Research Library.

Primary components: Tabs, Select, Checkbox, Dropdown Menu, Dialog, Alert
Dialog, batch ActionGroup, and dense metadata layouts.

Exit gate: install, verify, reconcile, remove, create, migrate, archive,
restore, and unregister presentation flows retain their existing typed intents
and confirmation boundaries.

### UI-6D — Complex Research Routes

Routes:

- Captures;
- Academic Graph.

Primary components: Tabs, FilterBar, responsive list/detail composition,
Inspector patterns, graph toolbar, conflict-resolution forms, and controlled
overlays.

Cytoscape graph algorithms and styles remain domain-owned. Only surrounding
controls, panels, status presentation, legend chrome, and responsive layout
migrate.

Exit gate:

- capture intake, review, conflict, outbox, and continuity views pass;
- graph filters, selection, paths, overlays, inspector, minimap, and revision
  views pass;
- no graph interaction regresses due to Dialog, Popover, or focus changes.

Compatibility routes `model-backend` and `workflow-content` are verified after
UI-6D for redirect, query-parameter, and navigation behavior.

Recommended commit pattern:

```text
refactor(<route>): migrate <route> to shadcn-svelte
```

Each route or tightly coupled route pair receives its own green checkpoint.

## UI-7 — Responsive and Accessibility Closure

Purpose: prove that unified components solve the original usability problems
on every route rather than merely changing appearance.

Responsive rules:

- every flex and grid child that can shrink uses `min-width: 0`;
- action groups wrap or stack instead of forcing viewport width;
- long paths and identifiers wrap or truncate with an accessible full-value
  disclosure;
- fixed minimum widths are removed unless the component is locally bounded;
- desktop tables switch to semantic card lists on narrow screens;
- the Academic Graph data table no longer exposes a visible horizontal
  scrollbar on narrow layouts;
- 200% zoom is treated as a narrow layout, not a desktop overflow exception.

Viewport matrix:

| Width | Purpose |
|---:|---|
| 360px | narrow mobile and high zoom |
| 390px | common mobile layout |
| 768px | tablet and collapsed shell |
| 1024px | compact desktop window |
| 1440px | standard desktop baseline |

Accessibility matrix:

- keyboard-only navigation;
- visible focus and logical focus order;
- dialog focus trap and return;
- Tabs arrow-key behavior;
- labels, descriptions, status, alert, and busy announcements;
- no state communicated by color alone;
- reduced motion and reduced transparency;
- increased contrast;
- Light and Dark at 100% and 200% zoom.

Automated contracts:

- fail when routes or features import `bits-ui` directly;
- fail when legacy UI classes are added;
- fail on document-level horizontal overflow;
- fail when non-graph components add hardcoded colors;
- validate light/dark semantic contrast pairs;
- retain the z-index ordering between notices, scrims, and dialogs.

Recommended commit:

```text
test(ui): add shadcn migration regression coverage
```

## UI-8 — Remove Legacy UI and Build the Local App

Removal scope:

- `src/lib/shared/ui` compatibility exports;
- `materialClass`, `surfaceClass`, and the existing class concatenation helper;
- `.button-primary`, `.button-secondary`, `.button-quiet`, `.button-danger`;
- generic `.surface` presentation rules;
- unused `--ui-*` component tokens;
- repeated route-local control, card, state, and badge CSS;
- unused UI dependencies and generated components;
- temporary `.dark` plus `data-theme` dual selectors.

Final validation:

```bash
pnpm desktop:check
pnpm desktop:test
pnpm desktop:build
pnpm desktop:macos
pnpm desktop:macos:open
```

The local macOS application must be inspected in both appearances with the
real Tauri window, including title-bar composition, scrolling, overlays, focus,
project navigation, and narrow window resizing.

Exit gate:

- all required commands pass at the same commit;
- all 12 routes pass the viewport and theme matrix;
- no legacy imports, classes, or redundant token aliases remain;
- the packaged local App starts and supports the critical interaction paths;
- the migration branch is clean and ready for review.

Recommended commits:

```text
refactor(ui): remove legacy ui compatibility layer
build(desktop): prepare migrated local app for validation
```

## Validation Tiers

Run after every cohesive component or route commit:

```bash
pnpm desktop:check
pnpm desktop:test
pnpm desktop:build
```

Run browser visual inspection after every route wave:

- every route in Light and Dark;
- the complete viewport matrix;
- document `scrollWidth === clientWidth`;
- no unexpected console errors or accessibility warnings;
- screenshots compared with the UI-0 baseline.

Run native packaging only at UI-5 and UI-8 unless shell or Tauri integration is
changed by another milestone.

## Risk Register

| Risk | Prevention | Recovery |
|---|---|---|
| shadcn initializer overwrites `app.css` | initialize in a temporary project and import reviewed changes | restore the UI-0 checkpoint |
| generator or registry drift | pin the used CLI version and review generated diffs | regenerate from the pinned version |
| two theme sources diverge | one controller synchronizes transition selectors | revert the theme milestone independently |
| focus and portal regressions | migrate behavioral primitives before routes and add interaction tests | keep the compatibility adapter for the affected primitive |
| bundle growth | add only used components and dependencies | remove unused generated components per wave |
| page-local CSS recreates divergence | source-contract tests and pattern ownership | block the route checkpoint until duplication is removed |
| graph visuals lose domain meaning | exempt and document graph semantic colors | revert only graph chrome migration |
| narrow layouts regain horizontal scroll | viewport automation plus mobile card alternatives | revert the affected route checkpoint |
| dark glass reduces readability | opaque fallback, contrast gate, reduced-transparency support | disable glass for dark mode without affecting tokens |

## Branch and Commit Policy

- checkpoint the current visual branch before initializing shadcn-svelte;
- use one dedicated migration branch and worktree;
- do not combine migration with business behavior changes;
- keep each commit buildable and reviewable;
- migrate dependencies in the same commit as their first use;
- never overwrite generated components without reviewing the diff;
- do not delete the compatibility layer until every consumer is migrated;
- use Conventional Commits as the rollback boundary.

## Reuse and Future Expansion

The first objective is one coherent Qiongli implementation, not an immediate
public design-system package. After UI-8 is accepted, reusable shadcn
components and Qiongli-neutral tokens may be extracted into a versioned custom
registry for other Svelte projects.

Extraction is allowed only when:

- the component has at least two real consumers or a demonstrated reuse case;
- its API has no Qiongli domain vocabulary;
- theme tokens are documented and have Light/Dark examples;
- accessibility and interaction tests travel with the component;
- registry installation is reproducible without copying Qiongli feature code.

Qiongli-specific patterns remain in `components/app`; portable primitives may
move to a future registry package. This prevents the migration from becoming a
premature framework-maintenance project.

## Definition of Done

The shadcn-svelte migration is complete only when all of the following are
true:

- every Desktop route uses the same shadcn-svelte and Qiongli pattern system;
- Rhea and Neutral are the locked generated baseline;
- Neutral black/white remains the primary interaction language;
- direct Bits UI imports exist only inside shadcn UI components;
- the legacy shared UI boundary and global button/surface classes are removed;
- one three-layer token authority drives Light and Dark modes;
- Light and Dark application surfaces remain opaque and readable;
- every route avoids a visible horizontal scrollbar at supported widths;
- text, controls, focus, keyboard, motion, transparency, and zoom gates pass;
- existing business behavior and Tauri command contracts remain unchanged;
- unit, contract, production-build, browser, and local macOS App validation pass
  at the same final commit.

## Reference Authority

- shadcn-svelte homepage visual reference:
  <https://www.shadcn-svelte.com/>
- shadcn-svelte SvelteKit installation:
  <https://shadcn-svelte.com/docs/installation/sveltekit>
- shadcn-svelte CLI:
  <https://shadcn-svelte.com/docs/cli>
- shadcn-svelte `components.json`:
  <https://shadcn-svelte.com/docs/components-json>
- shadcn-svelte theming:
  <https://shadcn-svelte.com/docs/theming>
- shadcn-svelte Tailwind v4 migration:
  <https://shadcn-svelte.com/docs/migration/tailwind-v4>
- Bits UI introduction:
  <https://bits-ui.com/docs/introduction>

## Immediate Next Slice

No migration implementation slice remains. The completed acceptance record is:

- all UI-0 through UI-8 milestones completed on
  `feat/shadcn-svelte-migration`;
- `svelte-check` completed with 0 errors and 0 warnings;
- 32 App API tests and 221 Desktop tests passed;
- the production SvelteKit static build passed;
- 12 routes passed Light and Dark at the 360, 390, 768, 1024, and 1440
  viewport matrix, for 120 verified states with no document overflow;
- Tabs keyboard navigation, dialog focus entry and return, compatibility
  redirects, and persistent theme switching passed in the browser;
- the release-profile, ad-hoc-signed local App was generated at
  `dist/macos/Qiongli.app`;
- the real macOS window was inspected in Light and Dark, at the 760px native
  minimum width, with sidebar navigation, project navigation, a shadcn Dialog,
  and the native workspace picker.

The next action is product review of the generated local App. Reusable neutral
components may be extracted into a versioned custom registry later under the
criteria in **Reuse and Future Expansion**; that work is outside this migration.
