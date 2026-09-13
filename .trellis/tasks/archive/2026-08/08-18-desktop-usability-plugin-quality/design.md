# Technical Design: receipt-bound local workflow variants

## Boundary

Extend the existing content path in place:

```text
verified embedded pack
  + optional receipt-owned local Markdown overrides
  -> deterministic selected variant
  -> existing standalone Skills / Codex / Claude materializers
  -> existing preview + confirmation
  -> fixed official Host CLI plan
  -> fresh managed/cache/Skill/MCP observation
  -> Ready (canonical or customized, explicitly labeled)
```

There is no generic file editor, new package manager, custom MCP runner, Host
cache writer, or second canonical content tree.

## Local variant owner

Add one small native owner under the existing private v2 configuration root. It
stores a managed directory containing:

- a canonical JSON receipt with schema version, canonical pack/content identity,
  monotonic revision, sorted override entries, base/current digests, total size,
  and deterministic variant digest;
- one regular UTF-8 Markdown file per overridden canonical resource.

The owner validates the complete staged tree, uses owner-private permissions,
locks the target, writes a sibling stage, verifies it, and atomically promotes
it. Reset removes only a verified receipt-owned variant. Reuse the repository's
existing containment, lock, canonical-JSON, digest, and quarantine patterns;
do not introduce a database.

Allowed paths are computed from the verified embedded pack, not supplied by the
UI. They are limited to:

- `workflow/SKILL.md`;
- canonical `ResourceKind::Skill` Markdown entries included by the selected
  content profile.

Plugin manifests remain previewable but read-only. MCP declarations, binaries,
schemas, standards, roles, templates, metadata, and unknown paths are never
accepted as override targets. Each file remains within the current 128 KiB
preview bound; the complete variant receives a bounded aggregate limit.

## App API

Evolve the existing content-customization event rather than add a parallel
route. Each bounded resource exposes:

- path and format;
- whether it is editable;
- canonical SHA-256;
- current SHA-256/content;
- whether a local override is active.

The event also exposes the current variant revision/digest or canonical state.
Add two mutation previews:

- replace one allowed resource, bound to expected variant digest/revision and
  the resource's expected current digest;
- reset one overridden resource with the same compare-and-swap binding.

Both return the existing operation preview and commit through
`confirm-operation`. The App API version, Rust fixture, TypeScript schema,
browser transport, and reducers move together. Project guidance remains a
separate intent and storage owner.

## UI

Refine `WorkflowContentPanel.svelte`:

- keep the existing profile/destination and source selector;
- show an editable textarea only for allowed workflow/Skill Markdown;
- show Plugin manifests in the existing read-only preview with an explicit
  immutable-contract note;
- show Canonical or Customized state, preview save/reset, and the exact
  destinations requiring reconcile;
- reload customization state after a confirmed mutation and retain focus;
- keep project-local guidance in its existing clearly separated editor.

No rich-text/Monaco dependency is added. A native textarea, existing buttons,
and current confirmation dialog are sufficient.

## Derived materialization and receipts

Add an optional validated override map to the shared resource projection. The
canonical pack remains the parent authority; only selected resource bytes and
their entry digests change.

- Standalone Skills receipts advance to a backward-readable version that binds
  the canonical pack SHA and optional local variant SHA.
- Codex and Claude bundle receipts advance similarly and bind the exact derived
  package content root.
- Existing canonical callers remain thin wrappers around the variant-capable
  composer with no overrides.
- Existing verifiers accept prior canonical receipt versions and the new exact
  variant version, but never an unreceipted or partially changed tree.

The signed launch grant continues to authorize the packaged binary and
canonical parent pack. The user's digest-bound confirmation separately
authorizes the local instruction overlay. An ADR records that this authority
cannot change executable/MCP/tool permissions or become publication evidence.

## Activation and Ready

A local variant change does not mutate installed destinations. Snapshot
derivation compares each destination's recorded variant identity with the
selected identity and reports update/repair required.

On explicit reconcile:

1. regenerate the receipt-owned managed source from canonical parent plus the
   selected variant;
2. execute only the existing target-specific official Host CLI plan;
3. clear old Host observations;
4. verify the managed and Host cache receipts match exactly;
5. verify Full MCP, plus Claude's exact Skill component;
6. report Ready with `canonical` or `customized` provenance.

Codex customized Ready still does not claim that a live model invoked the Skill.

## Failure and rollback

- A stale variant draft fails before any materialization.
- Materialization/Host failure leaves the selected variant stored but the
  destination non-Ready and retryable; it never edits cache files to compensate.
- Resetting the variant makes existing customized destinations update-required.
  Reconcile then restores canonical bundles through the same transaction.
- Reverting the product commit leaves prior canonical receipts readable. New
  variant receipts remain untouched and unsupported by the older binary rather
  than being destructively guessed or removed.

## Follow-on execution train

Typography and graph-backed Research Library continuity were absorbed into
this task after the user expanded its scope. After this task is accepted,
`GOV-401`–`GOV-404` adds one machine-readable 233-task ledger, a strict
   stdlib validator/generator, and a generated current index. Historical
   checkboxes become presentation only.
