# Design: authorization policy and receipts

## Boundary

Before this slice changes governance policy, existing product owners are
verified end to end:

```text
private WorkflowVariantStore edit
  -> managed Skills and Codex/Claude Plugin reconcile
  -> exact variant receipt and installed Skill bytes
  -> fresh Plugin/Skill/Full-MCP Ready observation
  -> reset and canonical recovery
```

This is a gate, not a second implementation. A passing current owner is left
unchanged; a failing owner is repaired before the policy work continues.

This slice publishes governance policy-as-code only:

```text
master roadmap Section 19.1–19.4
  -> authorization-policy-v1.json
  -> authorization-receipt-v1.schema.json
  -> standard-library validator + focused tests
  -> Evaluation Truth
```

It does not add a runtime decision service or change
`NativePublicationAuthorizationV1`. A later public App, MCP, or CLI consumer
must adopt ADR 0216's Rust-generated schema path in its own compatibility change.

## Canonical artifacts

- `tooling/architecture/authorization-policy-v1.json` owns the closed plane,
  role, action, binding/evidence, and non-transitive inventories.
- `tooling/architecture/authorization-receipt-v1.schema.json` owns the
  evidence-only receipt shape and contains one synthetic example.
- `tooling/scripts/validate_authorization_policy.py` checks cross-file semantic
  invariants with the Python standard library.
- `tests/test_authorization_policy.py` mutates loaded records to prove that the
  validator fails closed.
- `.trellis/spec/product/control/authorization-policy-v1.md` records the
  executable maintenance contract.

No new ADR is required: the master roadmap already owns this requested policy,
and this slice does not replace an accepted architecture decision.

## Policy shape

The root is closed and versioned. Ordered arrays contain:

- planes: `research`, `repository`, `publication`;
- the eight roles named in Section 19.3;
- actions covering the three `GOV-410` planes and repository delivery steps;
- non-transitive rules expressed only as `source` and `does_not_authorize`.

Each action names allowed executor roles, required authorizer roles, required
bindings, required evidence, and a default rule. `agent-ci-principal` may be an
executor but is rejected from every authorizer list. Authorizer roles are roles,
not identity federation or reusable credentials.

The validator owns the exact v1 inventory. Adding an action, role, or transition
requires an explicit schema-version/policy review rather than being silently
accepted as an unknown extension.

## Receipt shape

The schema uses Draft 2020-12, `additionalProperties: false`, closed enums from
the policy, bounded arrays/strings, SHA-256 patterns, and an RFC 3339 issue/expiry
pair. It requires:

- one opaque decision ID and exact action/object scope;
- actor and authorizer roles;
- one exact project/source revision;
- at least one plan digest or artifact digest;
- classification, decision, constraint codes, reason code, timestamps, and
  evidence references.

The included example is synthetic, contains repository-relative or opaque
identifiers only, and demonstrates an approved repository commit. The validator
checks both the schema structure and example semantics without implementing a
general JSON Schema engine.

## Validation

Reuse `is_canonical_repository_path` from the ADR validator for repository
evidence references. The new validator performs only v1-specific checks:

1. exact root/object keys and schema versions;
2. exact ordered inventories and unique IDs;
3. all cross-references resolve to declared roles/actions/planes;
4. Agent/CI is never an authorizer;
5. required non-transitive pairs exist exactly once and no rule claims a
   positive implication;
6. the receipt schema has the expected draft, closed fields, enums, required
   digest/expiry controls, and one valid redacted example;
7. referenced repository evidence resolves to existing, canonical, non-symlink
   regular files.

Focused tests use deep-copy mutation of the repository records, matching the
existing public-schema-policy pattern. No dependency or generic framework is
added.

## Compatibility and rollout

- Current product-spine tests and the packaged vertical receipt establish the
  preflight. A new package run is required only if a product/package input is
  changed while repairing a discovered gap.
- Version 1 is new governance metadata and changes no product wire contract.
- Existing specialized approval tokens and release authorization records remain
  authoritative for their current runtime actions.
- The policy is first marked active in the Program Ledger, then enforced in
  Evaluation Truth, then accepted only from exact-head CI evidence.
- A revert removes the new policy gate and spec without touching product state,
  Host state, repository history, packages, releases, or research data.

## Deferred work

- `GOV-413`–`GOV-418` operational enforcement and exceptional paths.
- Rust-generated public receipt consumers under ADR 0216.
- General receipt persistence, signing, revocation storage, and UI review.
