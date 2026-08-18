# Public Schema Authority

## Scenario: new or changed public App, MCP, or CLI JSON contract

### 1. Scope / Trigger

Use this contract whenever a pull request adds or changes App IPC, MCP tool
input/output, or documented public CLI JSON. It prevents Rust, Zod, checked-in
JSON Schema, fixtures, and documentation from becoming competing authorities.

### 2. Signatures

```bash
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy -v
```

Authority and inventory are owned by:

- `docs/architecture/decisions/0216-rust-owned-public-schema-authority.md`;
- `tooling/architecture/public-schema-policy.json`.

### 3. Contracts

- Rust domain types own every new or modified public wire contract.
- Emit deterministic versioned Draft 2020-12 JSON Schema and Rust-produced
  golden fixtures; checked-in outputs are derived, not editable authorities.
- Zod, MCP declarations, and CLI checks consume the generated contract without
  adding fields, defaults, variants, or weaker bounds.
- Existing App IPC, MCP v2, and CLI JSON remain truthful frozen migration
  baselines. The first later wire change migrates only the touched contract.
- Every post-baseline change names predecessor/successor versions and exactly
  one class: `additive`, `migratable-breaking`, or `unsupported-breaking`.
- `migratable-breaking` requires a tested migration/adapter path;
  `unsupported-breaking` requires a separate product/release removal gate.
- Green CI and a version bump never authorize old-version removal.

### 4. Validation & Error Matrix

- missing, duplicate, reordered, or unknown boundary -> policy failure;
- unsafe, missing, linked, or non-file evidence path -> policy failure;
- unknown authority state/class or unknown fields -> policy failure;
- missing predecessor, generated schema, fixture, or consumer check -> policy
  failure;
- non-Draft-2020-12 generated schema -> policy failure;
- breaking class without its required migration/removal control -> policy
  failure.

### 5. Good / Base / Bad Cases

- Good: change one Rust type, bump its version, regenerate one schema and
  fixture set, update the affected consumer, and append one classified record.
- Base: read and validate an unchanged frozen baseline without introducing a
  generator or change record.
- Bad: hand-edit Zod or an MCP schema, narrow a bound, and call the change
  additive because current tests still serialize one fixture.

### 6. Tests Required

- Policy unit tests reject structural, path, predecessor, class, and
  conditional-evidence mutations.
- The touched Rust producer and each affected consumer validate the same golden
  fixtures.
- Existing unrelated App, MCP, and CLI baselines remain unchanged and green.

### 7. Wrong vs Correct

Wrong: add a second schema source or a general generator before any concrete
contract needs migration.

Correct: migrate the first actually changed contract from its recorded baseline
to the smallest Rust-generated path, then verify its real consumers.
