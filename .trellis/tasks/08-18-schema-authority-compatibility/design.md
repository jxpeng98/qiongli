# Technical Design

## Architecture and boundaries

Implementation begins with a read-only/product-acceptance audit of the existing
CLI/Plugin/Skills/MCP spine. This is a precondition, not a parallel registry or
new feature. Any failed edit/reconcile/Ready link returns the task temporarily
to that existing shared owner and must be repaired before schema governance is
added.

Add ADR 0216 as the architecture owner and one small machine-readable public
schema policy record under `tooling/architecture/`. The record inventories the
three supported public boundary families and their frozen current baselines.
It does not replace the App API schema, MCP capability registry, CLI parser, or
Program Ledger.

The authority flow after adoption is:

```text
Rust domain type
  -> versioned Draft 2020-12 JSON Schema + Rust-produced fixtures
  -> App Zod adapter / MCP declaration / CLI JSON conformance
  -> existing consumer and integration tests
```

Existing App, MCP, and CLI shapes enter as truthful transition baselines. A
later change may keep reading a baseline, but it cannot change its semantics
without migrating the touched contract to the generated flow.

## Policy record

Use one closed JSON object with:

- policy schema version and record type;
- authority language and JSON Schema draft;
- the three ordered compatibility class names;
- exactly three ordered contract families;
- for each family: stable ID, boundary, version source, Rust owner/producer,
  consumers/fixtures, baseline state, and an append-only `changes` list.

The initial `changes` lists are empty because adoption records current state; it
does not invent historical classifications. A later entry binds a new version
to its predecessor, class, generated schema, golden fixtures, and either a
migration path or explicit removal gate when required.

## Validation

A standard-library Python validator reads the policy and repository paths. It
enforces exact keys, order, uniqueness, closed enums, canonical contained
paths, existing regular files, baseline truth, and conditional evidence for
the two breaking classes. Focused `unittest` mutations exercise each branch.

The existing `Evaluation Truth` workflow invokes this validator beside the ADR
and Program Ledger checks. No runtime dependency, generator, or additional CI
workflow is introduced.

## Compatibility and rollout

- Adoption is wire-neutral: no schema version or payload changes.
- `additive` still requires explicit versioning and fixtures; adding an enum
  variant or narrowing a bound is not additive for exhaustive/older clients.
- `migratable-breaking` requires a tested dual-read, adapter, or migration path
  and an old-version support boundary.
- `unsupported-breaking` requires a new explicit version, fail-closed behavior,
  and a named product/release approval gate; green CI alone is insufficient.
- Rollback reverts the policy/ADR change. It does not rewrite existing public
  payloads or claim generated coverage that never existed.

## Trade-offs

The design deliberately does not introduce `schemars`, TypeScript generation,
or a general schema compiler during governance adoption. That keeps this task
small and truthful. The first concrete schema migration can select the minimum
tooling based on an actual type graph and prove it with one consumer, rather
than scaffolding a generator before it has a real owner.
