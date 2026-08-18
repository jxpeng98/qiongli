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

## Pre-Development Checklist

- Read the App API schema and native producer for every changed field or intent.
- Reproduce the behavior in the smallest component/state test.
- Check keyboard labels and disabled/loading states for new controls.

## Quality Check

- `pnpm --dir packages/qiongli-desktop test`
- `pnpm --dir packages/qiongli-desktop check`
- Verify browser fixtures and native snapshots express the same contract.
- Inspect narrow and tall layouts for clipping or competing scrollbars.

Reference examples:

- `src/lib/features/client-integrations/LiteratureProvidersPanel.svelte`
- `src/lib/features/client-integrations/WorkflowContentPanel.svelte`
- `src/lib/features/project-workspace/ProjectArtifactViewer.svelte`
