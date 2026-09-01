# Design: GOV-417 authorization exception lifecycle

## Boundary

Extend the existing authorization spine only:

```text
denied / expired / revoked receipt -> action blocked -> new authorization
incident -> new repository-only approval -> protected PR/check flow
         -> completed or aborted hotfix -> reconciliation evidence
```

An emergency is a bounded reason for a new authorization, not a bypass or a
way to revive an invalid receipt. No runtime service, receipt store, role,
action, dependency, or schema version is added.

## Canonical Owners

- `tooling/architecture/authorization-policy-v1.json` owns one ordered,
  closed `authorization_lifecycle` inventory.
- `tooling/architecture/authorization-receipt-v1.schema.json` remains unchanged;
  its existing decisions, expiry, reason, constraints, and evidence references
  carry the required receipt data.
- `tooling/scripts/validate_authorization_policy.py` remains the single
  standard-library validator.
- `tests/test_authorization_policy.py` owns one focused mutation table.
- `.github/delivery-checklists.md` owns operator steps, while
  `.trellis/spec/product/control/authorization-policy-v1.md` owns maintenance
  guidance.

## Closed Lifecycle Contract

Add one root array whose five entries share these exact fields: `id`, `scope`,
`trigger`, `effect`, `next_step`, `required_evidence`, and
`forbidden_transition`. The validator compares the full ordered value with one
expected constant, matching the repository's existing closed-policy pattern.

1. `denial`
   - Scope: all actions.
   - Trigger/effect: an immutable `denied` receipt blocks execution.
   - Next step: create a new authorization; never mutate or revive the receipt.
   - Evidence: the receipt, bounded reason code, and safe next action without
     leaking policy or classified content.
2. `expiry`
   - Scope: all actions.
   - Trigger/effect: current time at or after `expires_at` blocks execution with
     no grace or replay.
   - Next step: create a new authorization with current bindings.
   - Evidence: the original receipt and expiry timestamp.
3. `revocation`
   - Scope: all actions.
   - Trigger/effect: a separate immutable `revoked` receipt blocks future use;
     in-flight work reaches a safe cancelled/blocked state without publishing
     partial canonical state.
   - Next step: create a new authorization.
   - Evidence: reference the original receipt and match its action, scope,
     revision, plan/artifact digests, plus a bounded reason code.
4. `emergency-hotfix`
   - Scope: repository plane only.
   - Trigger/effect: a declared incident may receive a new finite `approved`
     receipt, but work still uses the existing protected PR and required checks.
   - Next step: post-incident reconciliation is mandatory after completion or
     abort.
   - Evidence: incident reference, named human decision, minimum scope, exact
     head, new receipt, required checks, rollback plan, finite expiry, and full
     audit trail.
   - Forbidden: force push, check/review bypass, research-data action, release
     publication, or public announcement.
5. `post-incident-reconciliation`
   - Scope: repository plane only.
   - Trigger/effect: a completed or aborted emergency produces audit evidence;
     the next emergency remains blocked until this evidence is complete.
   - Next step: return to the normal authorization workflow.
   - Evidence: incident reference, exact action and impact, verification result,
     rollback status, follow-up owner/review, and truthful reviewer/blocker
     status.
   - Forbidden: retroactive authorization or scope widening.

## Receipt Semantics

- Every receipt stays immutable and action-bound. Denial, expiry, or revocation
  cannot be converted into approval.
- A replacement request receives a new `authorization_id` and re-verifies all
  scope, revision, digest, constraint, and expiry bindings.
- A revocation receipt references its subject through existing `evidence_refs`;
  no `supersedes` field is added.
- Reconciliation is an audit record, not a fifth receipt decision. It grants no
  authority and cannot cure an unauthorized action.

## Validation and Test

Add `authorization_lifecycle` to the exact root keys and add one
`EXPECTED_AUTHORIZATION_LIFECYCLE` constant. One equality guard rejects a
missing, reordered, unknown, broadened, or weakened path. Add one test mutation
table covering those cases plus required delivery wording. Existing receipt
schema and receipt-validation tests continue to prove the finite decision set
and timestamp shape.

## Operator Contract

Add a `### Authorization exceptions` subsection without changing the four
top-level delivery stages. It states that invalid receipts block work, the
emergency path is repository-only and preserves checks/no-force rules, and
reconciliation cannot retroactively authorize. Add these statements to the
existing required-marker list so wording cannot silently weaken.

The PR template does not change: it already records exact-head tests, rollback,
authorization impact, and truthful reviewer blockers.

## Compatibility and Rollback

- Governance-only additive contract; product behavior, persisted data, prior
  receipts, and public schemas remain unchanged.
- The exact validator constant intentionally makes policy weakening a reviewed
  code change rather than accepting arbitrary lifecycle text.
- Rollback reverts the policy, validator, test, spec, and checklist diff
  together; no migration or data repair is required.

## Delivery Strategy

Use one implementation PR and run the protected Slice once after its head is
frozen. After merge, record the implementation SHA and exact-head Evaluation
Truth run in an evidence-only closeout PR, archive the task, and use the
lightweight CI path.

## Deferred

Runtime authorization enforcement, a receipt database, public release
withdrawal/replacement, independent-reviewer staffing, and the GOV-418
self-approval matrix remain with their existing owners.
