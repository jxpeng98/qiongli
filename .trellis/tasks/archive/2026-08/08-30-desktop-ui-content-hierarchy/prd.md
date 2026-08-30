# Refine desktop UI content hierarchy

## Goal

Make the existing Qiongli desktop interface comfortable to read and scan during
normal daily use. Preserve the current Svelte 5 product flows while replacing
the visually heavy, uniformly dense presentation with a clear hierarchy between
primary work, supporting context, status, and technical detail.

## Background

- The desktop client already uses Geist Variable, light/dark themes, shared app
  components, and responsive layouts; this is a refinement of the current UI,
  not a framework or component-library rewrite
  (`packages/qiongli-desktop/src/app.css:1-13`,
  `packages/qiongli-desktop/src/lib/components/app/PageHeader.svelte:26-100`).
- The global body size is `13px`, the shared micro and label tokens resolve to
  `10px` and `11px`, and the current density tokens intentionally use `5-10px`
  gaps and padding (`packages/qiongli-desktop/src/app.css:76-125`,
  `packages/qiongli-desktop/src/app.css:214-224`).
- There are 324 literal `10px`, `11px`, or `12px` font-size declarations across
  desktop Svelte/CSS sources. On the populated research-library fixture at the
  default 1024x576 browser viewport, 68 of 133 visible text elements render at
  `12px` or smaller; 63 render below `12px`.
- Dense one-pixel grids and nested surfaces make supporting details compete with
  primary content (`packages/qiongli-desktop/src/lib/components/app/InfoGrid.svelte:32-57`,
  `packages/qiongli-desktop/src/lib/components/app/DescriptionGrid.svelte:32-58`).
- The research-library page exposes summary metrics, topology, filters, project
  rows, selected-project details, five common actions, priorities, and a danger
  action in one continuous reading flow
  (`packages/qiongli-desktop/src/routes/research-library/+page.svelte:421-649`).
- The current stack is healthy: the baseline completes 249 Desktop tests with
  one intentional skip, and `svelte-check` reports zero errors and zero
  warnings. Shared composition primitives already cover all ten routed
  workspaces (`packages/qiongli-desktop/src/lib/components/app/UnifiedUi.test.ts`).
- The compact presentation is a design-policy regression rather than a framework
  limit. Commit `f3868af2` intentionally reduced the body from `14px` to `13px`,
  section titles from `18px` to `16px`, and most shared spacing; later work
  tightened it further. Current source tests also pin several compact values
  (`packages/qiongli-desktop/src/app-css.test.ts`).
- Established product design systems converge on a semantic type scale, a
  readable default body size, consistent spacing, clear hierarchy, and
  progressive disclosure rather than making all information equally prominent.

## Requirements

- Establish one semantic typography scale for page titles, section titles,
  primary body copy, labels, metadata, and code/data values.
- Improve default reading comfort without losing the compactness expected of a
  desktop productivity tool.
- Rebalance spacing and surface treatment so borders, cards, badges, and helper
  copy communicate hierarchy instead of adding equal visual weight everywhere.
- Prioritize the current task, status, and primary action near the top/leading
  edge of each page; move secondary explanations and technical identifiers to
  quieter or progressively disclosed treatments.
- Apply the shared hierarchy consistently across overview, research-library,
  project-workspace, graph/data-heavy, integration, and settings/about surfaces.
- Use one comfortable daily-use density as the product default. Do not retain or
  add a separate compact-density preference.
- Preserve all current actions, status semantics, keyboard focus indicators,
  light/dark themes, Chinese/English content, and responsive behavior.
- Reuse the existing Geist font dependency, Svelte components, Tailwind 4,
  shadcn-svelte primitives, and design tokens unless evidence shows a specific
  primitive cannot express the required result.

## Acceptance Criteria

- [x] At 1024px and wider, primary body content renders at a comfortable default
      scale and no ordinary descriptive copy requires the current `10-11px`
      micro treatment.
- [x] Each representative page has one visually dominant title, clearly
      distinguishable section titles, readable body copy, and subdued metadata.
- [x] Research-library and another data-heavy project page make the primary task
      and current status visually dominant while keeping required topology or
      diagnostics available and separating destructive controls.
- [x] Shared cards/grids use spacing, grouping, or surface contrast intentionally;
      nested borders no longer make every block appear equally important.
- [x] Existing controls and navigation remain present and functional, with
      visible hover, pressed, disabled, loading, and keyboard-focus states.
- [x] Light and dark themes retain legible contrast, and Chinese/English labels
      do not clip at supported desktop and narrow layouts.
- [x] The populated source fixture has no horizontal page overflow at the
      existing responsive breakpoints and retains one intended vertical scroll
      owner per pane.
- [x] Desktop unit tests, `pnpm --dir packages/qiongli-desktop test`, and
      `pnpm --dir packages/qiongli-desktop check` pass.

## Out of Scope

- Rewriting application state, App API contracts, native behavior, or product
  workflows.
- Migrating from Svelte, Tailwind, shadcn-svelte, or Geist.
- Adding animation frameworks, a new component library, or a speculative
  user-configurable density system.
- Rewriting domain content whose meaning or product policy is not established by
  this task; content may be reordered, shortened where redundant, or disclosed
  progressively without changing its meaning.
