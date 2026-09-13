# Design: protected branch and review ownership

## Boundary

This slice joins one checked-in contract to one existing GitHub ruleset:

```text
repository-review-policy-v1.json + .github/CODEOWNERS
  -> existing authorization validator + focused mutations
  -> Evaluation Truth V1
  -> exact read/modify/read of ruleset 18800504
```

It does not introduce a service, workflow bot, approval token, or generic
GitHub-management abstraction.

## Canonical records

- `.github/CODEOWNERS` owns GitHub path-to-reviewer routing.
- `tooling/architecture/repository-review-policy-v1.json` owns the six domain
  inventory, expected `2.x` ruleset invariants, required checks, and the
  blocked/enforced review state.
- `tooling/scripts/validate_authorization_policy.py` remains the shared
  governance validator because GOV-413 operationalizes the repository merge
  action already defined by authorization policy v1.
- `tests/test_authorization_policy.py` supplies the smallest mutation check.
- `.trellis/spec/product/control/authorization-policy-v1.md` records the
  maintenance contract.

## Policy shape

The new JSON root is closed and versioned. It binds:

- repository, branch, ruleset ID/name, target and enforcement;
- deletion, non-fast-forward, PR, stale-review and thread-resolution rules;
- the exact required status-check names;
- ordered review domains, each with one or more CODEOWNERS patterns and owners;
- review enforcement state, approval count, CODEOWNER flag and blocker.

Only two review states are valid:

- `blocked`: zero approvals, CODEOWNER review disabled, one current owner, and a
  non-empty independent-reviewer blocker;
- `enforced`: at least one approval, CODEOWNER review enabled, at least two
  distinct owners, and no blocker.

The current record uses `blocked`. This makes future activation explicit without
pretending that review is enforceable today.

## CODEOWNERS scope

The six required domains use repository-root patterns:

- security: secret/redaction, execution policy/tool-host, provider access and
  Windows security boundaries;
- schema: App API schema, MCP contracts/schemas and public-schema policy;
- migration: migration ledgers plus native project/platform migration owners;
- release: workflows, release tooling/scripts, native release authority;
- research-Gate: quality-gate contract/report and owning tests;
- authorization: authorization/reviewer records, validator/tests/spec,
  CODEOWNERS, and the Program Ledger.

The initial ordered entries are fixed for this slice (all owned by
`@jxpeng98`):

```text
# security
/.github/
/packages/qiongli-native/crates/qiongli-windows-security/
/packages/qiongli-native/crates/qiongli-config/src/secret.rs
/packages/qiongli-native/crates/qiongli-config/src/redaction.rs
/packages/qiongli-native/crates/qiongli-execution/src/policy.rs
/packages/qiongli-native/crates/qiongli-execution/src/tool_host.rs
/packages/qiongli-native/crates/qiongli-runtime/src/providers/access.rs
/packages/qiongli-native/crates/qiongli-platform/src/grant.rs

# schema
/packages/qiongli-native/apps/qiongli/src/
/packages/qiongli-native/crates/qiongli-runtime/src/contract.rs
/packages/qiongli-app-api/src/
/content/mcp-contracts/v2/
/content/schemas/
/tooling/architecture/public-schema-policy.json
/tooling/scripts/validate_public_schema_policy.py
/tests/test_public_schema_policy.py
/docs/reference/cli.md

# migration
/tooling/migration/
/packages/qiongli-native/crates/qiongli-project/src/migration.rs
/packages/qiongli-native/crates/qiongli-platform/src/legacy_migration.rs
/packages/qiongli-native/apps/qiongli/src/legacy_migration_cli.rs

# release
/.github/workflows/
/tooling/release/
/scripts/release_automation.sh
/scripts/release_preflight.sh
/scripts/release_postflight.sh
/scripts/release_ready.sh
/scripts/release_version.py
/scripts/release_upload_assets.py
/packages/qiongli-native/release/
/packages/qiongli-native/crates/qiongli-platform/src/release_authority.rs
/packages/qiongli-native/crates/qiongli-platform/src/release_candidate.rs
/packages/qiongli-native/crates/qiongli-platform/src/native_release.rs

# research-gate
/content/standards/quality-gate-contract.yaml
/content/templates/quality-gate-report.md
/tests/test_quality_gate_contract.py

# authorization
/.github/CODEOWNERS
/tooling/architecture/authorization-policy-v1.json
/tooling/architecture/authorization-receipt-v1.schema.json
/tooling/architecture/repository-review-policy-v1.json
/tooling/scripts/validate_authorization_policy.py
/tests/test_authorization_policy.py
/.trellis/spec/product/control/authorization-policy-v1.md
/docs/superpowers/roadmaps/qiongli-program-ledger-v1.json
/tooling/scripts/update_program_roadmap.py
/tests/test_program_roadmap.py
```

The validator parses only the deliberately small CODEOWNERS subset used here:
comments, blank lines, and whitespace-separated `pattern owner...` entries.
Each pattern is a literal repository-root file or directory, so it can reuse
canonical path/symlink containment checks. It compares ordered entries to the
policy; no glob engine or general CODEOWNERS parser is added.

## Live ruleset update

Read ruleset `18800504`, construct a full replacement payload preserving every
current field, append only `Evaluation Truth V1` with the GitHub Actions
integration ID, then `PUT` and immediately `GET` the ruleset again. Abort before
mutation if the pre-read differs from the recorded baseline. Do not add a bypass
or change review enforcement.

The task PR must first demonstrate that `Evaluation Truth V1` exists and passes
for its exact head. After the live update, GitHub must report that check as
required for the same PR before merge.

## Compatibility and evidence

- Policy and CODEOWNERS are repository control metadata; no product/public wire
  contract changes.
- Existing native required checks remain exact and strict.
- A failed live precondition leaves GitHub unchanged and the ledger blocked.
- A ruleset update failure is retried only after a fresh read; never apply a
  stale payload.
- Exact-head CI is development evidence, not release authorization.

## Rollback

- Before merge: revert the feature branch; CODEOWNERS and policy never reach
  `2.x`.
- Live setting: restore the pre-read payload only if the added Evaluation Truth
  context causes a confirmed repository-control failure.
- After merge: revert the policy PR. Do not weaken deletion, force-push, PR or
  native-check protection as part of rollback.

## Deferred work

Independent review enforcement requires a user-nominated second GitHub account.
It is deliberately not simulated with an Agent, CI principal, bot approval,
bypass role, or self-review.
