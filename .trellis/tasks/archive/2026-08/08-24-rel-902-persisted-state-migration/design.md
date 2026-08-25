# REL-902 Design

## Boundary

Use one fixture-driven native acceptance test in the existing App crate because
that module can call both shared owners without exposing new production APIs:

- `qiongli_project::ProjectStateService` for project migrate and rollback;
- the existing private legacy-provider stage, verify, and rollback functions in
  `legacy_migration_cli.rs` for global state.

No production migration path changes unless the focused proof exposes a real
defect.

## Fixture Contract

Add one JSON fixture manifest under the App test fixtures. It contains exactly
two rows, ordered N-1 then N-2. Each row carries the release tag, peeled source
commit, a small representative 1.x project file map, and legacy provider JSON.
The test asserts the two exact release identities before using the payloads.

The fixture contains only synthetic research text, public emails, and fake
secret values. It contains no private paths or credentials.

## Data Flow

For each row:

1. materialize the legacy project and `providers.json` under a fresh test root;
2. snapshot every predecessor input byte;
3. preview and apply project migration through `ProjectService`;
4. discover, preview, stage, and verify provider conversion through the existing
   legacy migration owners and a memory-only test secret store;
5. assert current project registration/content and redacted global settings;
6. preview and apply receipt-owned project rollback;
7. invoke exact provider rollback;
8. assert current migration-owned state is gone and every predecessor byte is
   unchanged.

## Compatibility and Rollback

N-1 is `v1.19.0-beta.1`; N-2 is `v1.18.0-beta.3`. The releases share the
supported 1.x project/provider document family, so the test proves release
window compatibility without inventing unused schema versions.

Rollback is not a downgrade writer. It removes only receipt-owned current
project state, restores pre-migration current provider settings/secrets, and
leaves the predecessor source intact for the predecessor binary.

## Risks

- A test that bypasses shared owners would prove only fixture copying. The test
  must call `ProjectService` and the existing provider migration functions.
- A loose label could falsely claim provenance. Exact tag/commit assertions are
  part of the executable test.
- `REL-903` future-file behavior must stay separate; no future schema fixture is
  added here.
