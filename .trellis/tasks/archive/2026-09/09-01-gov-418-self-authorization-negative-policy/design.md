# Design: GOV-418 self-authorization negative policy

## Boundary

The v1 authorization contract already owns the required behavior:

```text
authorization-policy-v1.json
  -> validate_authorization_policy.py
  -> tests/test_authorization_policy.py
  -> Evaluation Truth
```

This task strengthens proof at the existing test owner. It does not add a new
policy structure, validator entry point, runtime authorization engine, schema,
role, action, or dependency.

## Existing Guards Reused

- `EXPECTED_AUTHORIZERS` fixes the permitted human authorizers for all actions.
- The Agent/CI-specific guard rejects `agent-ci-principal` from authorizer lists
  and receipt authorizers.
- `COMMON_BINDINGS`, digest requirements, and the existing action-specific
  binding checks reject weakened scope.
- `EXPECTED_RULES` fixes `ci-green` as a negative signal for merge, release
  publication, and public announcement.

These are already the shared enforcement owners. Duplicating them in another
JSON section or framework would add drift without increasing protection.

## Negative Matrix

Add one focused matrix in `tests/test_authorization_policy.py`:

1. For each of the twelve actions, insert `agent-ci-principal` and `ci-green`
   into its authorizer list and require validation failure.
2. For each action, build the smallest valid Agent/CI-executed receipt using one
   permitted human authorizer, then replace that authorizer with Agent/CI and
   require receipt validation failure.
3. For each action, remove every declared binding one at a time and require
   policy validation failure. Existing common and action-specific guards decide
   the exact error; the test asserts fail-closed behavior, not duplicate logic.
4. For each privileged `ci-green` target, remove its negative rule and convert
   it to a positive `authorizes` rule in separate mutations; both must fail.

The valid baseline assertion prevents a negative test from passing only because
its fixture was already invalid.

## Documentation and Evidence

Update `.trellis/spec/product/control/authorization-policy-v1.md` so future
changes must preserve the exhaustive matrix. Mark only `GOV-418` active during
implementation and accepted only after exact-head Evaluation Truth succeeds.

No policy/schema file is changed unless the matrix exposes a real validator
gap. No ADR is required because this implements an existing roadmap contract
without changing architecture or a public boundary.

## Compatibility and Rollback

- Test/spec/ledger-only unless a current guard fails; no migration or package
  rebuild is required by the local change boundary.
- GitHub's protected PR matrix still supplies Linux, macOS, Windows, Native, and
  Evaluation Truth evidence for the implementation head.
- Rollback reverts the focused test/spec/evidence diff and changes no product or
  external state.

## Deferred

`SEC-820` retains runtime replay, stale approval, destination substitution,
confused deputy, and broader role-escalation adversarial testing.
