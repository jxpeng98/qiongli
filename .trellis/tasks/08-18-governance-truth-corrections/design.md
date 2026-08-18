# Design: Correct architecture, ADR, and parity truth

## Boundaries

This is a governance-source correction. It changes documentation, validated
machine records, and their focused tests; it does not change product runtime or
package inputs.

## Architecture overview

Keep `docs/architecture.md` as the human source. Add assertions to the existing
ADR test module for the current Tauri/Svelte, Rust-native, ADR 0210, ADR 0211,
and Host-driven statements. The overview already satisfies GOV-405, so an
editorial rewrite would add churn without changing the outcome.

## Decision registry

Keep `tooling/architecture/arc-201-decisions.json` as the immutable bootstrap
baseline. Add `tooling/architecture/current-decisions.json` as the additive
current registry with exact top-level identity and ordered entries containing:

- `task_id`
- `adr_number`
- `title`
- `path`
- `status`

The existing validator will validate both records by default. Current-registry
validation compares the JSON paths with every numbered Markdown file in
`docs/architecture/decisions/`, rejects duplicate identity, requires numeric
order, and verifies title/status/task metadata against the referenced file.

ADR 0208 remains the earlier target-specific launcher decision. The later
Community Alpha distribution boundary moves to 0215, the next unused number.
All repository path references move with it.

## 1.x parity truth

Replace root `status` with `classification_status` and bump the internal ledger
schema from 1.0 to 1.1. Keep the existing capability `disposition` values and
evidence arrays unchanged. The existing Rust test remains the executable owner
and will assert classification completion separately from per-capability
implementation evidence and deferral nonclaims.

## Compatibility and rollback

- The ADR rename is a repository link migration; every checked-in reference is
  updated in the same commit.
- The frozen architecture baseline remains unchanged and its guard remains
  active.
- The parity schema change is explicit through version 1.1. Rollback restores
  the JSON, schema, and Rust decoder together.
- The current decision registry is additive and can be removed without touching
  accepted ADR contents if its validation boundary proves incorrect.
