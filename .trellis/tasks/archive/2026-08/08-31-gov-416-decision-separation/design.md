# Design: GOV-416 decision separation

## Boundary

Extend the existing authorization spine only:

```text
repository.merge receipt
  -> exact merged integration commit
publication.publish-release receipt
  -> exact published assets and channels
independent public verification
publication.announce-release receipt
  -> exact announcement content and channels
```

Each arrow is a prerequisite, never transferred authority. No product, release
automation, credential, or receipt-storage path changes.

## Canonical Owners

- `tooling/architecture/authorization-policy-v1.json` owns the ordered action
  inventory and non-transitive rules.
- `tooling/architecture/authorization-receipt-v1.schema.json` owns the closed
  action enum for one immutable decision per receipt.
- `tooling/scripts/validate_authorization_policy.py` remains the single
  standard-library validator.
- `tests/test_authorization_policy.py` owns focused fail-closed mutations.
- `.github/delivery-checklists.md` and `.github/pull_request_template.md` own
  operator confirmation; the product-control spec owns maintenance guidance.

## Announcement Action

Add `publication.announce-release` after `publication.publish-release`:

- default rule: `verified-publication-before-announcement`;
- executors: Maintainer and Agent/CI principal;
- authorizer: existing Release Approver, `all-of`;
- bindings: object scope, exact source revision, plan digest, artifact digests,
  channels, constraints, and expiry;
- evidence: publication receipt, independent public verification,
  announcement-content digest, and announcement approval.

The plan digest identifies the exact announcement text/claim set. Artifact
digests identify the already-public verified bytes. The action uses existing
receipt fields; no schema field or second receipt format is needed.

## Closed Separation Rules

Append three ordered negative transitions:

1. `repository.merge` does not authorize `publication.announce-release`;
2. `publication.publish-release` does not authorize
   `publication.announce-release`;
3. `ci-green` does not authorize `publication.announce-release`.

The existing rule already separates merge from publication. Because each
receipt has exactly one scalar action, announcement authority cannot substitute
for merge or publication authority either.

## Validation

Extend the existing expected action/rule/authorizer/default/evidence constants.
Announcement gets one policy guard requiring plan digest, artifact digests, and
channels together. `validate_receipt` reuses each action's declared digest
bindings: a required plan digest must be non-null and required artifact digests
must be non-empty. The existing schema equality check automatically keeps the
action enum synchronized.

Add one mutation table covering missing action/schema enum, wrong authorizer,
missing policy/receipt plan or artifact binding, missing channel/evidence,
removed negative transition, and weakened delivery wording. Reuse existing
inventory, receipt, and delivery tests.

## Compatibility and Rollback

- Additive governance action; current receipt fields and prior receipts remain
  unchanged and valid only for their recorded action.
- No runtime/public product schema or persisted data changes.
- Rollback reverts the policy/schema/validator/test/spec/delivery diff together.

## Delivery Strategy

Freeze one implementation PR and run the full protected Slice once. Merge while
GOV-416 remains active. Then record the implementation SHA and exact-head
Evaluation Truth run in an allowlisted Ledger/Trellis closeout PR, archive the
task, and use the lightweight CI path.

## Deferred

A distinct announcement-approver role, receipt database, and automated
announcement publisher require separate product/security authority and are not
needed to prove three action-bound decisions.
