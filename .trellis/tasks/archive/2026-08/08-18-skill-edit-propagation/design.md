# Editable Skill propagation design

## Boundary

This task changes evidence, not production architecture. The existing flow is:

1. Desktop loads verified editable resources from App API v18.
2. A replace intent writes a receipt-owned private workflow variant with CAS
   revision and digest checks.
3. Explicit Skills/Integration reconciliation projects that variant into
   standalone targets and Codex/Claude Plugin bundles.
4. Fresh verification derives current/customized state from receipts.
5. Reset removes the override, then reconciliation restores canonical targets.

## Changes

### Desktop fixture and component test

Add the real context-maintainer Skill to `sourceFixtureTransport()` and select
it through the existing `Preview resource` control. Assert the resulting
preview target contains the exact nested Skill path. No component behavior or
App API type changes are needed.

### Packaged-product acceptance

Reuse `exercise_workflow_variant_reconcile_reset` with:

- source path:
  `skills/Z_cross_cutting/academic-context-maintainer.md`
- standalone projected path: unchanged from the source path
- Plugin projected path:
  `skills/qiongli-workflow/skills/Z_cross_cutting/academic-context-maintainer.md`

The existing flow already covers preview, commit, update-required observation,
three standalone reconciliations, both Plugin reconciliations, receipt digest
checks, reset, canonical reconciliation, and fresh snapshot verification.

## Compatibility and safety

- Public contracts and receipt schemas remain unchanged.
- `WorkflowOverrides` continues to reject non-Markdown/non-Skill resources.
- Existing bundle verifiers protect manifests, MCP descriptors, binaries, and
  generated adapters from unreceipted drift.
- Acceptance uses isolated temporary homes and fake official-host fixtures;
  separate ignored tests cover current real Codex/Claude CLIs in isolated
  homes.

## Rollback

Revert the fixture/test/acceptance changes. No user data or migration exists.
