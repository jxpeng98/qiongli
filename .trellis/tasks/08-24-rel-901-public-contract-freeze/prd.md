# REL-901: Freeze public compatibility contracts

## Goal

Turn the accepted Qiongli 2 App, MCP, CLI, project, and global-state behavior
into one executable release freeze so later migration and recovery work cannot
silently reinterpret an existing schema identity or compatibility promise.

## Background

- `PLT-320`, `PLT-321`, and `PLT-322` are accepted, satisfying every declared
  dependency of `REL-901`.
- ADR 0216 and `tooling/architecture/public-schema-policy.json` already own the
  three public wire families: App IPC, MCP tools, and public CLI JSON.
- The accepted identities are App API schema `19`, MCP capability registry v2
  (`2.0.0-preview.5`), and command-scoped CLI JSON schema `1`.
- ADR 0204 already requires versioned project/global state, forward-only
  migration, rollback, and fail-closed handling of future documents.
- The master roadmap assigns N-2 project/global-state migration proof to
  `REL-902` and future-version immutability proof to `REL-903`; this task freezes
  those promises but does not perform those later acceptance runs.

## Requirements

1. Extend the existing public-schema policy rather than create another public
   contract registry or runtime abstraction.
2. Record one closed release-freeze entry for each existing public family:
   App IPC schema `19`, MCP registry/schema v2, and CLI JSON schema `1`.
3. State the semantic meaning owned by each identity and require unchanged
   meaning for unchanged schema IDs.
4. Declare the support boundary truthfully:
   App IPC is supported only inside its exact bundled product version, while
   MCP v2 and CLI JSON v1 remain supported for the Qiongli `2.x` release line.
5. Declare a persisted-state window of current plus two predecessor versions,
   forward-only migration with rollback, and fail-closed/unmodified treatment
   of versions newer than the running binary.
6. Require a separately accepted release/removal gate before a frozen public ID
   can be retired; a green test or version bump is insufficient.
7. Extend the existing standard-library validator and unit tests so missing,
   reordered, unknown, weakened, or source-inconsistent freeze data fails.
8. Keep all existing runtime schemas, tool names, serialized wire shapes,
   project files, and global state unchanged.

## Acceptance Criteria

- [ ] The policy contains exactly the three accepted public-family freeze rows
  with their current IDs, semantic meanings, and support windows.
- [ ] The policy records same-ID semantic immutability, N-2 persisted-state
  support, forward-only rollback-capable migration, future-version
  fail-closed/unmodified behavior, and a separate removal gate.
- [ ] Validation fails for a changed App schema ID, MCP v2 identity, CLI JSON
  identity, compatibility depth, semantic statement, support window, or
  removal rule.
- [ ] Validation confirms the App schema version agrees in Rust and TypeScript
  and the MCP registry/schema identity agrees with the checked-in v2 contract.
- [ ] `python tooling/scripts/validate_public_schema_policy.py` passes.
- [ ] `python -m unittest tests.test_public_schema_policy -v` passes.
- [ ] `python3 scripts/validate_capability_contract.py` passes without MCP
  registry or schema changes.
- [ ] Evaluation Truth and the required exact-head Native CI Slice pass before
  `REL-901` is recorded as accepted in Program Ledger v1.

## Out of Scope

- Runtime wire-shape changes, schema-version bumps, new schema generators, or
  a second contract registry.
- N-2 migration/rollback execution (`REL-902`) and future-version mutation
  testing (`REL-903`).
- Candidate packaging, publication, Stable promotion, or legacy-path removal.
- Graph v2, additional Hosts/providers, or post-2.0 kernel work.

## Resolved Decisions

- Reuse and harden `public-schema-policy.json`; do not add a parallel release
  manifest for the same identities.
- Freeze meanings as closed policy values and cross-check live identity owners;
  do not hash implementation files, because unrelated source edits are not
  public semantic changes.
- Treat App IPC as an exact-bundle contract rather than claim unsupported
  cross-version App/native interoperability.
- Use the roadmap-defined N-2 and fail-closed promises as requirements; no
  additional user-owned product decision remains open.
