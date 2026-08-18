# Authorization Policy and Receipt V1

## 1. Scope / Trigger

Use this contract when changing authorization roles, sensitive actions,
non-transitive authority, or redacted decision receipts. It is governance
policy-as-code; it does not grant runtime authority and a receipt is never a
bearer credential.

## 2. Signatures

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy -v
```

Canonical artifacts:

- `tooling/architecture/authorization-policy-v1.json`;
- `tooling/architecture/authorization-receipt-v1.schema.json`.

## 3. Contracts

- The policy has exactly three independent planes: research, repository, and
  publication; authority never flows between them implicitly.
- The eight v1 roles and eleven v1 actions are closed, ordered inventories.
- Each action binds its executor, human authorizer rule, object scope, exact
  revision, plan or artifact digest, constraints, expiry, and evidence.
- Agent/CI may execute an already-authorized mechanical action and emit
  evidence, but cannot authorize, self-review, or widen scope.
- Preview, edit, stage, commit, push, PR, merge, CI success, and publication
  remain explicitly non-transitive as encoded by the policy.
- The Draft 2020-12 receipt is closed, finite, redacted, and immutable evidence
  of one decision. A consumer must still verify current scope, revision,
  digest, decision, constraints, and expiry before acting.
- A later App, CLI, or MCP receipt surface must use ADR 0216's Rust-owned public
  schema and compatibility path; this JSON Schema is not a product wire owner.

## 4. Validation & Error Matrix

- missing, duplicate, reordered, or unknown plane/role/action -> fail;
- unknown action references, Agent/CI authorizer, or weakened binding -> fail;
- missing, duplicate, reordered, unknown, or positive authority transition ->
  fail;
- non-canonical, missing, linked, or non-file repository evidence -> fail;
- changed Draft, open receipt, unknown field/value, missing digest/expiry, or
  unsafe evidence reference -> fail;
- invalid or non-redacted synthetic example -> fail.

## 5. Good / Base / Bad Cases

- Good: a bounded decision names one permitted action, exact object/revision,
  appropriate actor and authorizer roles, digest, constraints, expiry, and
  redacted evidence.
- Base: policy and schema remain unchanged and Evaluation Truth validates them
  without creating any receipt or changing product state.
- Bad: green CI, a prior edit/commit/merge, or an old receipt is treated as
  authorization for a later action or changed input.

## 6. Tests Required

- Mutate each closed inventory and non-transitive rule set; assert fail-closed
  validation.
- Mutate role/action references and Agent/CI authorization; assert rejection.
- Weaken the closed schema, digest, expiry, bounds, or enums; assert rejection.
- Mutate the example with missing/unknown fields, stale expiry, absent digests,
  unknown action, or absolute evidence path; assert rejection.
- Run the validator and focused tests in Evaluation Truth.

## 7. Wrong vs Correct

Wrong: infer merge or publication authority from a successful check or replay a
receipt against a different revision, digest, destination, or object.

Correct: obtain a separate scoped decision for the exact next action, retain a
redacted receipt as evidence, and verify every binding again at use time.
