# Establish schema authority and compatibility classification

## Goal

Make future Qiongli 2 public contract changes traceable to one Rust-owned
semantic source and force every such change to declare one compatibility class.
This removes the current ambiguity between native Rust producers, the App's
handwritten Zod decoder, checked-in MCP JSON Schemas, and public CLI JSON.

## Background

- `GOV-408` requires one schema-authority ADR: Rust domain types generate
  versioned JSON Schema, while TypeScript/Zod, MCP, and public CLI consumers use
  generated contracts and golden fixtures.
- `GOV-409` requires every public schema change to be classified as
  `additive`, `migratable-breaking`, or `unsupported-breaking`.
- The App boundary currently has a useful Rust-generated fixture consumed by
  `packages/qiongli-app-api/scripts/run-tests.mjs`, but
  `packages/qiongli-app-api/src/schema.ts` remains a separately maintained Zod
  model.
- MCP v2 has a complete checked-in registry and JSON Schemas under
  `content/mcp-contracts/v2/`, currently maintained as contract content and
  consumed by the native runtime.
- Public CLI JSON is emitted by Rust `Serialize` types and checked by native
  integration tests, but it does not have one complete generated schema set.
- Converting all three established surfaces in one change would be a broad,
  high-risk migration. M1 needs an accepted authority and transition rule
  before later domain schemas are introduced; it does not require a big-bang
  rewrite of every existing contract.

## Requirements

### R0 — Preserve the already-prioritized product spine

Before changing governance files, re-audit the current
`App -> native CLI -> Plugin/Skills -> Lite/Full MCP` path from current-source
tests and exact package evidence. In particular, prove that an allowed
Workflow/Skill Markdown edit is stored as a receipt-owned variant, marks
installed destinations repair-required, is propagated only after explicit
reconciliation through the official Host CLI, and returns Ready only after a
fresh exact variant/Plugin/Skill/MCP observation. If any required link is
missing or contradicted, fix and requalify that link before implementing
`GOV-408`/`GOV-409`.

### R1 — One accepted authority decision

Add one accepted ADR that makes Rust domain types the semantic authority for
every new or modified Qiongli 2 public schema. Generated, versioned Draft
2020-12 JSON Schema is the interchange artifact; generated schemas and
Rust-produced golden fixtures are outputs, never second authorities.

### R2 — Explicit public boundary inventory

Keep one machine-readable policy record that covers exactly the current public
contract families:

1. App IPC decoded by TypeScript/Zod;
2. MCP tool input/output contracts;
3. documented public CLI JSON output.

For each family, record its Rust owner or producer, current version source,
checked-in contract/fixture consumers, and transition state. Existing
handwritten contracts must be labelled as frozen migration baselines rather
than falsely reported as generated.

### R3 — Closed compatibility classification

Define and validate exactly these classes for every post-baseline public schema
change:

- `additive`: old valid payloads and supported consumers remain valid; no
  existing field, variant, meaning, or bound is narrowed;
- `migratable-breaking`: a version bump plus a bounded dual-read, adapter, or
  migration path preserves supported data or callers;
- `unsupported-breaking`: no safe automatic migration exists, so the old
  version remains explicit, the new boundary fails closed, and separate
  product/release approval is required before support is removed.

An initial frozen baseline is not retroactively called a change. Every later
record must name its predecessor and one of the three classes.

### R4 — Fail-closed governance validation

Add the smallest deterministic validator and focused tests that reject:

- missing, duplicate, reordered, or unknown public boundary records;
- unknown authority, schema draft, transition state, or compatibility class;
- missing repository paths or paths outside the repository;
- a post-baseline version without a predecessor and compatibility class;
- a `migratable-breaking` entry without an explicit migration/adapter path;
- an `unsupported-breaking` entry without an explicit removal approval gate;
- a changed/added accepted ADR that is absent from the current decision
  registry.

Run the validator in the existing required `Evaluation Truth` workflow; do not
create another workflow or umbrella test.

### R5 — Truthful roadmap state

During implementation, mark `GOV-408` and `GOV-409` active in Program Ledger
v1. Mark them accepted only after exact implementation commit and required CI
evidence exist, then regenerate the current program index.

## Acceptance Criteria

- [x] Current-source automated and packaged evidence proves CLI install/test,
      Plugin activation, expected Skill/MCP contents, editable Skill save,
      reconcile-to-Ready, reset, and canonical recovery; any discovered gap is
      repaired before governance implementation.
- [x] ADR 0216 is accepted, registered once, and states Rust ownership,
      generated JSON Schema/golden-fixture consumption, transition rules, and
      rollback/non-claims.
- [x] One machine-readable record identifies the App IPC, MCP, and public CLI
      baselines without claiming that handwritten legacy surfaces are already
      generated.
- [x] The compatibility policy accepts only `additive`,
      `migratable-breaking`, and `unsupported-breaking` for changes after each
      baseline.
- [x] Focused negative tests prove missing coverage, duplicate identities,
      unknown fields/classes, invalid repository paths, and missing migration
      or approval evidence fail closed.
- [x] Existing ADR validation, Program Ledger validation, App API contract
      tests, MCP capability validation, and focused public CLI JSON tests remain
      green.
- [x] `Evaluation Truth` runs the new governance check on pull requests to
      `2.x`.
- [ ] Program Ledger evidence for `GOV-408` and `GOV-409` names the exact merged
      implementation commit and successful required run before either is
      `accepted`.

## Out of Scope

- Rewriting the full 3,806-line Zod contract in this governance task.
- Adding a code-generation framework or dependency before a concrete public
  Rust type is migrated.
- Converting all 25 MCP tools or every existing CLI JSON command at once.
- Changing any App, MCP, or CLI wire shape.
- Applying compatibility classification retroactively to pre-policy history.
- Persistent internal state, private receipts, release authorization, and
  migration-file schemas unless they are explicitly exposed as a supported
  public contract in a later change.

## Technical Notes

- Reuse `tooling/architecture/current-decisions.json`,
  `tooling/scripts/validate_arc_201_adrs.py`, Program Ledger v1, and the existing
  `Evaluation Truth` job.
- The policy record is governance metadata, not a runtime registry and not a
  source for generating product code.
- The first later wire-shape change in each family must migrate that touched
  contract to the ADR's Rust-generated path in the same pull request.
