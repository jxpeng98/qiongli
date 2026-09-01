# GOV-418 self-authorization negative policy

## Goal

Complete roadmap item `GOV-418` with a focused policy-as-code negative matrix
that proves Agent/CI execution, CI success, and weakened authorization bindings
cannot become authority for a privileged action. Reuse the existing v1 policy,
receipt, validator, and test owners; do not create a second authorization
system.

## Background

- `GOV-410` through `GOV-412` already established the closed authorization
  policy, receipt schema, standard-library validator, and Evaluation Truth gate.
- The current validator already rejects Agent/CI authorizers, non-human signal
  authorizers, missing action bindings, and weakened `ci-green` negative
  transitions.
- Current tests sample one action and generic transition mutations. They do not
  explicitly prove those controls across all twelve privileged actions and all
  three `ci-green` boundaries.
- `GOV-413` remains blocked by the independent-reviewer constraint. `SEC-820`
  separately owns broader runtime attacks such as replay, stale approval,
  destination substitution, confused deputy, and role escalation.

## Requirements

- Add one table-driven negative matrix covering every v1 action.
- Prove `agent-ci-principal` and `ci-green` cannot be inserted into any action's
  authorizer set.
- Prove an Agent/CI executor receipt cannot name Agent/CI as the authorizer,
  while a correctly human-authorized baseline for the same action remains valid.
- For every action, remove each declared binding one at a time and prove the
  current validator rejects the resulting scope-widening policy.
- Prove each `ci-green` rule for merge, release publication, and public
  announcement fails closed when removed or converted into a positive grant.
- Keep the existing policy, receipt schema, runtime behavior, dependencies, and
  public product contracts unchanged unless the tests reveal a real enforcement
  gap at their current shared owner.
- Record `GOV-418` as accepted only from exact implementation-head CI evidence.

## Out of Scope

- A runtime authorization service, bearer credential, signing system, receipt
  database, or new public App/CLI/MCP schema.
- The broader `SEC-820` replay, stale-input, destination-substitution,
  confused-deputy, and role-escalation test suite.
- Resolving `GOV-413`, changing GitHub reviewer staffing, or publishing a
  release in this task.

## Acceptance Criteria

- [ ] Every v1 action rejects Agent/CI and `ci-green` as authorizer entries.
- [ ] Every Agent/CI-executable action has a valid human-authorized receipt
      baseline and rejects the same receipt when Agent/CI self-authorizes.
- [ ] Removing any declared action binding is rejected, including scope,
      revision, digest, constraints, expiry, and action-specific bindings.
- [ ] Removing or reversing any `ci-green` non-authorization rule for merge,
      publication, or announcement is rejected.
- [ ] The canonical policy, receipt example, and focused authorization tests
      remain green through Evaluation Truth.
- [ ] No policy inventory, receipt schema, runtime, package, Host, release, or
      user-data behavior changes.
- [ ] The Program Ledger and generated index truthfully bind `GOV-418`
      acceptance to the implementation commit and exact-head CI run.

## Decisions

- Use the existing validator as the enforcement owner; this task adds exhaustive
  negative proof rather than duplicating rules in a new policy section.
- Treat removal of a declared binding as the policy-as-code form of scope
  widening. Runtime consumers still verify exact current scope and inputs as
  required by the existing v1 spec.
- Use one implementation PR and one evidence-only closeout PR, matching the
  accepted GOV delivery pattern.

## Open Questions

None.

## Notes

- The shortest valid change is expected to touch the focused test, the
  product-control spec, Trellis task evidence, and the Program Ledger/index.
