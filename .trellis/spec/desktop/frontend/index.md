# Desktop Frontend

The Svelte 5 client lives in `packages/qiongli-desktop/src/lib/`. It renders
typed App API snapshots and sends intents through the transport; product state
and filesystem authority remain native.

## Local Pattern

- `app-state.svelte.ts` owns snapshot/event reduction and selected UI state.
- `dev-transport.ts` is a typed browser fixture, not a second product backend.
- Feature components under `features/` render one product area and emit typed
  intents; they do not construct native plans or filesystem paths.
- User customization of bundled Workflow/Skill Markdown is a
  draft/preview/confirm flow into one private receipt-owned variant. Canonical
  embedded content remains immutable; managed destinations change only through
  a later explicit reconciliation.
- A pane has one intentional vertical scroll owner. Nested preview content may
  scroll only when its parent is not also the same-axis scroll container.

## Visual Hierarchy Contract

- Use the semantic roles owned by `src/app.css`: body `14px`, supporting
  `13px`, label `12px`, and micro `11px`. Route and feature styles consume the
  variables instead of reintroducing literal `10-13px` declarations.
- Ordinary copy and controls use body or supporting text. Field names and tabs
  use label text. Micro text is reserved for terse technical metadata such as
  versions, paths, hashes, timestamps, and status codes.
- Prefer spacing and `--color-surface-subtle` to nested equal-weight borders.
  Use native `<details>` for secondary diagnostics or topology when it competes
  with the current task; safety, recovery, and destructive consequences remain
  visible.

```css
/* Correct: readable supporting context. */
.description { font-size: var(--font-size-supporting); }

/* Wrong: ordinary prose treated as technical metadata. */
.description { font-size: var(--font-size-micro); }
```

## Pre-Development Checklist

- Read the App API schema and native producer for every changed field or intent.
- Reproduce the behavior in the smallest component/state test.
- Check keyboard labels and disabled/loading states for new controls.

## Quality Check

- `pnpm --dir packages/qiongli-desktop test`
- `pnpm --dir packages/qiongli-desktop check`
- Keep `src/app-css.test.ts` aligned with the semantic type and density tokens.
- Verify browser fixtures and native snapshots express the same contract.
- Inspect narrow and tall layouts for clipping or competing scrollbars.

Reference examples:

- `src/lib/features/client-integrations/LiteratureProvidersPanel.svelte`
- `src/lib/features/client-integrations/WorkflowContentPanel.svelte`
- `src/lib/features/project-workspace/ProjectArtifactViewer.svelte`

Executable contracts:

- [Tauri prototype hardening](tauri-prototype-hardening.md) — deferred UI
  dependencies must load after `Object.prototype` is frozen.
