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

## Styling and themes

- Use shadcn semantic classes or the Qiongli semantic tokens from `app.css`.
- Do not add raw colors outside the documented Academic Graph visual language.
- `data-theme="light|dark"` is the only theme selector.
- Use the shadcn-svelte Rhea + Neutral baseline: Inter type, near-black primary
  actions, compact controls, tighter gaps, fine borders, and restrained shadows.
- Drive density through the shared `--ui-page-*`, `--ui-panel-*`,
  `--ui-section-gap`, and `--ui-empty-min-height` tokens. Desktop layouts should
  feel compact without reducing coarse-pointer controls below 44px or disabling
  translated-label wrapping.
- Apply Qiongli's conservative geometry roles instead of restoring Rhea's
  registry-default 24px card radius: controls use `--radius-control`, inset
  information groups use `--radius-inset`, cards use `--radius-card`, and
  dialogs use `--radius-dialog`. Pills and circles are reserved for statuses,
  avatars, and shape semantics.
- Use `components/app/InfoGrid.svelte` for related facts and summaries. It
  provides one outer boundary with internal dividers; do not rebuild the same
  content as a cluster of individually rounded mini-cards.
- Preserve Qiongli's responsive adaptations when regenerating Rhea components:
  translated button and tab labels may wrap, while icon-only controls remain
  fixed-size.
- Regeneration must preserve the semantic geometry classes on Button, Card,
  Input, NativeSelect, Tabs, Dialog, Alert, and Dropdown Menu.
- Do not add gradients, backdrop blur, tinted glass, or decorative depth to
  application surfaces. Sidebar, workspace context, dialogs, and cards remain
  opaque in both themes.
- New controls must pass keyboard, focus-return, reduced-motion,
  contrast, and narrow-layout checks.

## Reuse in other Svelte projects

Portable primitives can later move into a versioned shadcn custom registry when
they have at least two consumers, no Qiongli domain vocabulary, documented
Light/Dark tokens, and self-contained accessibility tests. Product-specific
patterns remain in `components/app`; feature code is never copied into the
registry.
