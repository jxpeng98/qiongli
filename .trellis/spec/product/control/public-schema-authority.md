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

## Scenario: REL-901 release contract freeze

### 1. Scope / Trigger

Use this contract when qualifying a 2.x release or changing an already frozen
App IPC, MCP, CLI JSON, project, or global-state compatibility claim. It
prevents an unchanged schema ID from acquiring new meaning.

### 2. Signatures

```bash
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy -v
python3 scripts/validate_capability_contract.py
```

The executable freeze remains in
`tooling/architecture/public-schema-policy.json`; do not add another registry.

### 3. Contracts

- App IPC schema `19` is an exact bundled-product contract, not a cross-version
  App/native interoperability promise.
- MCP registry/schema v2 and CLI JSON schema `1` retain their accepted meanings
  throughout the Qiongli `2.x` release line.
- An unchanged public ID has immutable semantics. Retirement requires a
  separately accepted release gate.
- Persisted project/global state supports current plus two predecessor versions
  through forward-only, rollback-capable migration.
- A persisted version newer than the running binary fails closed and remains
  unmodified.

### 4. Validation & Error Matrix

- missing/extra freeze field or family -> policy failure;
- App Rust/TypeScript version differs from the freeze -> policy failure;
- MCP registry version or root `$id` differs from the freeze -> policy failure;
- weakened N-2, rollback, future-file, or removal rule -> policy failure;
- runtime/schema change without a classified successor -> policy failure.

### 5. Good / Base / Bad Cases

- Good: add a classified successor identity, preserve the frozen predecessor,
  and attach required migration or removal evidence.
- Base: change implementation internals without changing any frozen identity or
  semantic meaning; the policy stays unchanged.
- Bad: edit bytes or meanings under the same ID, or claim compatibility from a
  version bump and green CI alone.

### 6. Tests Required

- Mutation tests reject every global compatibility-window field and every
  family identity, definition, semantic, and support-window field.
- The policy validator reads the live App and MCP identity owners.
- Capability Contract validation remains green without MCP registry drift.
- `REL-902` separately proves N-2 migration/rollback; `REL-903` separately
  proves future files remain unchanged.

### 7. Wrong vs Correct

Wrong: create a second release manifest or hash whole implementation files as a
proxy for public meaning.

Correct: extend the existing closed policy, validate stable identity owners,
and require a new classified identity only when the public contract changes.

## Scenario: REL-902 persisted-state migration and rollback

### 1. Scope / Trigger

Use this contract when changing the supported project or global provider-state
migration window. It proves the REL-901 current-plus-two policy against exact
published predecessors without turning release labels into new schema IDs.

### 2. Signatures

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib rel_902 --locked
```

The native owners are `ProjectStateService::{preview_migrate,apply_migration,
preview_migration_rollback,apply_migration_rollback}` and the private provider
`stage_legacy_provider_config`, `verify_legacy_provider_config`, and
`rollback_legacy_provider_config` functions.

### 3. Contracts

- The fixture manifest contains exactly N-1 `v1.19.0-beta.1` at
  `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`, then N-2
  `v1.18.0-beta.3` at `12aea420bff9a3fbfa5e421c482ae8da2588c9ed`.
- Each row supplies a 1.x project tree and legacy `providers.json`; execution
  occurs under a fresh test-owned home and project root.
- Migration registers a readable current project and replaces provider
  plaintext secrets with secret-store references.
- Rollback removes only receipt-owned current project state, restores prior
  current provider settings and created secrets, and does not rewrite legacy
  input bytes.

### 4. Validation & Error Matrix

- missing, extra, reordered, or relabelled predecessor -> fixture identity
  assertion failure;
- project not registered/readable or destination not removed -> project state
  assertion failure;
- missing SecretRef, wrong secret bytes, or rollback drift -> provider state
  assertion failure;
- changed legacy project or provider bytes -> exact byte comparison failure.

### 5. Good / Base / Bad Cases

- Good: both exact predecessors complete migrate, verify, and rollback through
  the shared native owners with identical source bytes afterward.
- Base: both rows share the supported 1.x document family; no artificial schema
  version is invented to distinguish releases.
- Bad: copy fixture files directly, test only N-1, or delete the legacy source
  and call recreation a rollback.

### 6. Tests Required

- One fixture-driven native test asserts the closed predecessor identity list.
- For each row, assert registration and readable migrated research state before
  rollback, then absent destination and registration afterward.
- For each row, assert current SecretRef plus stored secret bytes before
  rollback, then exact prior settings, secrets, project bytes, and provider
  bytes afterward.

### 7. Wrong vs Correct

Wrong: add a second migration registry or a release-specific migration path.

Correct: bind exact release provenance in the fixture and exercise the existing
project and provider migration owners for both rows.
