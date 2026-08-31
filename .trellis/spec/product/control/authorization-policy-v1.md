# Authorization, Receipt, and Repository Review V1

## 1. Scope / Trigger

Use this contract when changing authorization roles, sensitive actions,
non-transitive authority, redacted decision receipts, CODEOWNERS, or the
protected `2.x` review ruleset. It is governance policy-as-code; it does not
grant runtime authority and a receipt is never a bearer credential.

## 2. Signatures

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy -v
```

Canonical artifacts:

- `tooling/architecture/authorization-policy-v1.json`;
- `tooling/architecture/authorization-receipt-v1.schema.json`;
- `tooling/architecture/repository-review-policy-v1.json`;
- `.github/CODEOWNERS`;
- `.github/delivery-checklists.md`;
- `.github/pull_request_template.md`.

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
- Repository review policy has exactly six ordered sensitive domains: security,
  schema, migration, release, research-Gate, and authorization. Every v1 path is
  literal, repository-rooted, present, symlink-free, and owned by `@jxpeng98`.
- Ruleset `18800504` targets only `2.x`, has no bypass, blocks deletion and
  non-fast-forward changes, requires PR/thread resolution, and keeps the native
  cross-platform checks plus `Evaluation Truth V1` strict and current.
- Review state is `blocked` while only one eligible human exists: required
  approvals remain zero and CODEOWNER approval remains disabled. `enforced`
  requires at least two distinct owners on every path, one approval, CODEOWNER
  review, exact-head non-stale evidence, and no blocker.
- The delivery checklist has exactly four stages: pre-commit, pre-push, pull
  request, and release. Every checklist item declares Machine or Human/authority
  evidence, selects Focused/Slice/Acceptance proportionately, and keeps commit,
  push, merge, and publication authority non-transitive.
- The default PR template links the canonical checklist and records bounded
  scope/non-goals, affected boundaries, exact-head tests, compatibility,
  migration/rollback, risks/follow-ups, and required reviewers. A new push
  invalidates stale evidence.

## 4. Validation & Error Matrix

- missing, duplicate, reordered, or unknown plane/role/action -> fail;
- unknown action references, Agent/CI authorizer, or weakened binding -> fail;
- missing, duplicate, reordered, unknown, or positive authority transition ->
  fail;
- non-canonical, missing, linked, or non-file repository evidence -> fail;
- changed Draft, open receipt, unknown field/value, missing digest/expiry, or
  unsafe evidence reference -> fail;
- invalid or non-redacted synthetic example -> fail;
- missing/reordered review domain or owned path, malformed CODEOWNERS line,
  unknown owner, missing file/directory, symlink, glob, or policy drift -> fail;
- removed branch rule/check, bypass actor, widened branch target, or false
  blocked/enforced review state -> fail.
- missing/reordered delivery stage, unlabeled checklist item, required command
  or authorization warning, PR field, or canonical checklist link -> fail.

## 5. Good / Base / Bad Cases

- Good: a bounded decision names one permitted action, exact object/revision,
  appropriate actor and authorizer roles, digest, constraints, expiry, and
  redacted evidence.
- Base: policy and schema remain unchanged and Evaluation Truth validates them
  without creating any receipt or changing product state.
- Bad: green CI, a prior edit/commit/merge, or an old receipt is treated as
  authorization for a later action or changed input.
- Repository good: CODEOWNERS exactly matches the checked-in review policy and
  the live ruleset adds Evaluation Truth without weakening an existing rule.
- Repository base: one maintainer receives path ownership routing, while the
  ledger truthfully retains the independent-reviewer blocker.
- Repository bad: CI/Agent approval, self-review, bypass, or an impossible
  one-person CODEOWNER requirement is presented as independent authorization.

## 6. Tests Required

- Mutate each closed inventory and non-transitive rule set; assert fail-closed
  validation.
- Mutate role/action references and Agent/CI authorization; assert rejection.
- Weaken the closed schema, digest, expiry, bounds, or enums; assert rejection.
- Mutate the example with missing/unknown fields, stale expiry, absent digests,
  unknown action, or absolute evidence path; assert rejection.
- Mutate domains, paths, owners, branch rules, required checks, bypass actors,
  CODEOWNERS text, and review-state coherence; assert rejection.
- Remove checklist stages, evidence labels, required commands/warnings, PR
  fields, and the canonical link; assert rejection.
- Run the validator and focused tests in Evaluation Truth.

## 7. Wrong vs Correct

Wrong: infer merge or publication authority from a successful check, replay a
receipt against changed input, or enable self-blocking review and call it
independent approval.

Correct: obtain a separate scoped decision for the exact next action, retain a
redacted receipt, verify every binding, route sensitive paths to CODEOWNERS, and
leave review blocked until another eligible human can approve exact-head work.
