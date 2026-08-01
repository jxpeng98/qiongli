# Qiongli Desktop UI

The Desktop UI has one layered implementation:

1. `components/ui` contains reviewed shadcn-svelte open code. Bits UI imports
   are allowed only here.
2. `components/app` contains reusable Qiongli patterns such as page headers,
   state panels, metrics, workspace navigation, and confirmation flows.
3. `features` owns domain presentation and behavior. Routes compose those
   features without defining another component system.

## Adding a component

Run the pinned generator from `packages/qiongli-desktop`, then review every
generated file before committing it:

```bash
pnpm dlx shadcn-svelte@1.4.2 add <component>
```

Prefer an existing primitive or app pattern first. Keep a generated component
only when it has a real consumer, and remove unused generated groups before the
change is complete.

## Local frontend preview

Start the Svelte frontend without building the native application from the
repository root:

```bash
pnpm run dev
```

The root script forwards to `packages/qiongli-desktop` and serves the local
fixture-capable Vite app at `http://127.0.0.1:1421`. Port 1420 remains reserved
for the Desktop/Tauri development flow.

## Styling and themes

- Use shadcn semantic classes or the Qiongli semantic tokens from `app.css`.
- Do not add raw colors outside the documented Academic Graph visual language.
- `data-theme="light|dark"` is the only theme selector.
- Use the shadcn-svelte Nova + Neutral baseline: Geist type, near-black primary
  actions, reduced spacing, compact controls, fine borders, and restrained shadows.
- Drive density through the shared `--ui-page-*`, `--ui-panel-*`,
  `--ui-section-gap`, and `--ui-empty-min-height` tokens. Desktop layouts should
  feel compact without reducing coarse-pointer controls below 44px or disabling
  translated-label wrapping.
- Keep block spacing on the Nova scale: default cards use 10px, compact cards
  use 8px, and nested information groups use 6–8px. Add larger local spacing
  only when it communicates a new hierarchy level.
- Keep Qiongli geometry on Nova's default `0.5rem` radius scale: controls use
  `--radius-control`, while inset groups, cards, and dialogs resolve to the
  default `--radius`. Pills and circles are reserved for statuses, avatars, and
  shape semantics.
- Use `components/app/InfoGrid.svelte` for related facts and summaries. It
  provides one outer boundary with internal dividers; do not rebuild the same
  content as a cluster of individually rounded mini-cards.
- Preserve Qiongli's responsive adaptations when regenerating Nova components:
  translated button and tab labels may wrap, while icon-only controls remain
  fixed-size.
- Regeneration must preserve the semantic geometry classes on Button, Card,
  Input, NativeSelect, Tabs, Dialog, Alert, and Dropdown Menu.
- Selected Tabs use the semantic `primary / primary-foreground` pair for a
  clearly visible state in both themes. Hover and press transitions use the
  shared short motion curve and must keep the reduced-motion fallback.
- Do not add gradients, backdrop blur, tinted glass, or decorative depth to
  application surfaces. Sidebar, workspace context, dialogs, and cards remain
  opaque in both themes.

## Page composition

- Every routed workspace uses `PageLayout`; routes provide only the translated
  header copy, optional actions, business state, and feature components.
- Use `ContentGrid` for responsive card or panel arrangements. Its `columns`,
  `collapse`, `gap`, and `lastSpan` props replace page-specific grid wrappers.
- Use `DescriptionGrid` for semantic `dt`/`dd` facts, `InfoGrid` for mixed
  information cells, and `MetricGrid` for numeric summaries.
- Use `SectionHeader variant="panel"` instead of wrapping a section header in a
  page-owned `panel-header` div. Use `StatePanel` directly instead of adding
  `state-block`, `empty-state`, or `loading` wrappers for spacing.
- New page-level layout behavior belongs in the shared app components. Route
  styles should remain limited to business-specific visualization or content
  presentation that cannot be expressed by the shared composition API.
- New controls must pass keyboard, focus-return, reduced-motion,
  contrast, and narrow-layout checks.

## Reuse in other Svelte projects

Portable primitives can later move into a versioned shadcn custom registry when
they have at least two consumers, no Qiongli domain vocabulary, documented
Light/Dark tokens, and self-contained accessibility tests. Product-specific
patterns remain in `components/app`; feature code is never copied into the
registry.
