# REL-901 Technical Design

## Boundary

The existing governance chain remains authoritative:

`Rust/App + MCP registry + CLI JSON -> public-schema-policy.json -> validator -> Evaluation Truth`

No runtime dispatch, storage, App API, MCP schema, or CLI serializer changes.

## Policy Shape

Add one closed top-level compatibility-window object and one closed
`release_freeze` object to each of the existing three contract rows.

The compatibility window records:

- same schema identity means immutable semantics;
- current plus two predecessor persisted-state versions are supported;
- persisted migrations are forward-only and rollback-capable;
- future persisted versions fail closed and remain unmodified;
- public-ID removal requires a separately accepted release gate.

Each family freeze records a schema identity, a bounded semantic statement, and
its support window:

- App IPC: schema `19`, snapshot/intent/event protocol, exact bundled product;
- MCP: capability registry/schema v2, tool/profile/input/output/error,
  side-effect and security semantics, the `2.x` line;
- CLI: command-scoped JSON schema `1`, stable field/error/redaction meanings,
  the `2.x` line.

## Validation

Extend `validate_public_schema_policy.py` with exact-key and exact-value checks.
The validator also reads the current Rust and TypeScript App constants and the
MCP registry/schema documents to reject a freeze that disagrees with its live
identity owner. Existing capability-contract validation remains the owner of
the complete MCP tool/schema graph.

Unit mutations cover missing/extra keys, weakened global windows, each family
identity/meaning/support value, App Rust/TypeScript mismatch, and MCP
registry/schema mismatch. No new dependency or test framework is needed.

## Compatibility And Rollback

This is governance metadata only, so rollback is a normal revert before it is
used as accepted release evidence. Once accepted, future changes append a
classified successor or pass the named removal gate; accepted history is not
rewritten.

`REL-902` and `REL-903` consume this freeze as their acceptance input. They are
not implemented here.

## Risks

- A free-form policy could claim compatibility without code evidence. Mitigate
  with closed values, source identity checks, and mutation tests.
- Hashing full source files would create false policy churn. Avoid hashes and
  bind only stable schema/version owners.
- Duplicating full MCP validation would drift. Keep detailed MCP graph checks in
  `validate_capability_contract.py` and validate only the frozen root identity
  here.
