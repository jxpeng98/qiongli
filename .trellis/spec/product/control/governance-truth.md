# Governance Truth Records

## Scenario: current ADR and 1.x parity truth

### 1. Scope / Trigger

Use this contract when an accepted ADR is added, renamed, or renumbered, or
when the Qiongli 1.x product-outcome parity record changes. It prevents the
frozen bootstrap inventory from becoming a stale current registry and prevents
classification completeness from being reported as implementation completeness.

### 2. Signatures

```bash
python scripts/validate_arc_201_adrs.py
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-platform --test product_parity_ledger --locked
```

The records are:

- frozen bootstrap: `tooling/architecture/arc-201-decisions.json`;
- current ADR registry: `tooling/architecture/current-decisions.json`;
- parity record and schema: `tooling/migration/qiongli-1x-product-parity.json`
  and `tooling/migration/qiongli-1x-product-parity.schema.json`.

### 3. Contracts

- Never modify the frozen ARC-201 inventory or ADR 0201-0207. Superseding or
  later decisions belong only in the additive current registry.
- The current registry has exactly `schema_version`, `record_type`, `branch`,
  and `decisions`. Each decision has exactly `task_id`, `adr_number`, `title`,
  `path`, and `status`.
- Registry paths equal every numbered ADR Markdown file exactly once and in
  filename/number order. Number, title, status, and task ID match the file.
- Accepted ADR numbers and paths are unique. A collision is repaired by moving
  the later decision and all repository references to the next unused number.
- Parity schema 1.1 exposes root `classification_status` and forbids ambiguous
  root `status`. `complete` means every outcome is classified; it does not
  change any capability `disposition` or establish implementation evidence.

### 4. Validation & Error Matrix

- missing, extra, duplicate, or reordered ADR entry -> validator failure;
- path/number mismatch or non-canonical path -> validator failure;
- title, status, or task metadata mismatch -> validator failure;
- unknown registry fields or wrong registry identity -> validator failure;
- parity root `status`, schema version before 1.1, or missing
  `classification_status` -> Rust/schema contract failure;
- a deferred outcome without `defer-to-r4`, R4 ownership, and a nonclaim ->
  Rust/schema contract failure.

### 5. Good / Base / Bad Cases

- Good: add ADR 0216, append one matching current-registry row, update every
  checked-in reference, and pass both validator and focused tests.
- Base: read or validate the current decision set without changing the frozen
  ARC-201 baseline.
- Bad: append a later ADR to `arc-201-decisions.json`, reuse an ADR number, or
  treat `classification_status: complete` as proof that deferred work ships.

### 6. Tests Required

- Python: assert current architecture wording and reject missing, extra,
  duplicate, reordered, path-mismatched, and metadata-mismatched ADR entries.
- Frozen baseline: prove the ARC-201 record and ADR 0201-0207 remain protected.
- Rust: parse parity schema 1.1, require `classification_status`, reject root
  `status`, and assert the exact deferred capability set remains deferred.
- CI: run the default ADR validator so frozen and current records are checked
  together.

### 7. Wrong vs Correct

Wrong: rewrite the frozen ARC-201 inventory to make it look current, or rename
`status` only in the parity JSON while leaving its schema and Rust owner stale.

Correct: keep ARC-201 byte-frozen, update the additive current registry, and
change parity JSON, schema, and Rust contract test in one revision.
