# Prove editable Skill propagation

## Goal

Prove that a user can edit one real `skills/**/*.md` resource through the
existing App customization flow, explicitly reconcile that variant into every
supported Skills and Plugin target, and reset all targets to canonical bytes.

## Background

- Native already allows `workflow/SKILL.md` and Markdown resources classified
  as `ResourceKind::Skill`; manifests, MCP descriptors, and binaries are
  intentionally read-only.
- App API v18, Desktop editing, receipt-owned variants, reconciliation, reset,
  CLI, MCP, Codex Plugin, and Claude Code Plugin paths are implemented and their
  existing checks pass.
- Isolated real-host tests pass with Codex CLI `0.147.0` and Claude Code
  `2.1.231`.
- Existing App and packaged-product edit scenarios mutate only
  `workflow/SKILL.md`. The remaining gap is direct evidence for one nested
  Skill resource and its host-specific projected path.

## Requirements

### R1 — Exercise the existing App editor with a real Skill

- Add the canonical
  `skills/Z_cross_cutting/academic-context-maintainer.md` resource to the
  Desktop development transport fixture as editable Markdown.
- The component test must select that resource, edit it, and preview the
  existing `preview-workflow-resource-replace` intent with the exact Skill
  path.
- Keep the existing `workflow/SKILL.md` UI test and all App API contracts.

### R2 — Prove end-to-end propagation and reset

- Reuse the existing packaged-product
  `exercise_workflow_variant_reconcile_reset` flow with the real Skill path.
- After customization and explicit reconciliation, the marker must exist in
  all three standalone Skills targets and in both source/cache forms of the
  Codex and Claude Code Plugins.
- Plugin projection must use
  `skills/qiongli-workflow/skills/Z_cross_cutting/academic-context-maintainer.md`.
- Receipts and fresh App snapshots must report the customized variant as
  current; reset plus reconciliation must remove the marker and restore the
  canonical variant state.

### R3 — Preserve existing boundaries

- Do not add a new editor, store, API action, public schema, automatic host
  reload, dependency, or editable manifest/MCP/binary path.
- Existing receipt and bundle verification remains authoritative for all
  non-edited resources and generated host adapters.
- Test only inside isolated temporary homes; never mutate normal host profiles.

### R4 — Verify release-shaped behavior

- Run focused Desktop and native tests, full affected checks, exact-head CI,
  and a local non-publishing macOS packaged-product acceptance build.

## Acceptance Criteria

- [ ] The App fixture exposes the real Skill and the component test previews a
      save for its exact path.
- [ ] A receipt-owned variant of the real Skill reaches all standalone Skills,
      Codex Plugin, and Claude Code Plugin source/cache targets.
- [ ] Reset removes the marker everywhere and fresh snapshots return to
      canonical/current.
- [ ] Plugin manifests, MCP descriptors, binaries, public schemas, and normal
      user profiles remain outside the editable boundary.
- [ ] Focused/full tests, `git diff --check`, exact-head CI, and local macOS
      packaged-product acceptance pass.

## Out of Scope

- Editing Plugin manifests, `.mcp.json`, generated adapters, or binaries.
- Adding automatic client restart/reload or writing directly into host caches.
- Adding a second customization model, graph editor, or generalized file
  browser.

## Key Decisions

- Reuse the existing resource selector and workflow-variant intents.
- Change the packaged edit scenario from the already-covered workflow root to
  one representative nested Skill; retain existing workflow-root unit/UI
  coverage.
- Use explicit reconciliation and receipt verification as the activation
  boundary.

## Notes

- This is the already-approved third step of the active Plugin/Skills/CLI/MCP
  completion plan; no product or risk decision remains open.
