# Design: GOV-415 exact-head and history policy

## Boundary

Extend the existing repository-governance spine only:

```text
master roadmap history rules
  -> repository-review-policy-v1.json history_policy
  -> existing authorization validator + focused mutation table
  -> Evaluation Truth V1
  -> delivery checklist and PR confirmation wording
```

No Git history, GitHub setting, product input, package, workflow, or release
artifact is mutated by the implementation.

## Canonical Owners

- `tooling/architecture/repository-review-policy-v1.json` owns one closed,
  machine-readable history policy beside the existing `2.x` ruleset snapshot.
- `.github/delivery-checklists.md` owns operator guidance.
- `.github/pull_request_template.md` owns the current-head confirmation field.
- `tooling/scripts/validate_authorization_policy.py` remains the shared standard-
  library validator; no new public validation entry point is added.
- `tests/test_authorization_policy.py` owns the smallest fail-closed mutation
  table.
- `.trellis/spec/product/control/authorization-policy-v1.md` records the
  maintenance contract.

## Policy Shape

Add one top-level `history_policy` object to the existing closed review record.
The validator owns its exact v1 keys and ordered arrays.

### `exact_head`

- identity: full lowercase commit SHA;
- head changes: new commit, amend, rebase, merge, history rewrite;
- ordered evidence rules:
  - exact-head CI and review: any head change, current head only;
  - authorization: invalidate when revision, scope, plan, or artifact digest
    binding changes;
  - package: invalidate only when a bound package input/digest changes;
  - release: invalidate when source, version, target, digest, metadata,
    destination, channel, or claim changes and never transfer to a replacement.

### `protected_refs`

- patterns: `refs/heads/2.x`, `refs/heads/release/*`, `refs/tags/*`;
- update path: protected PR or separately authorized release workflow only;
- force push and history rewrite: forbidden;
- a head owning accepted evidence is never eligible for rewrite.

### `feature_branch_rewrite`

- eligibility: unprotected + unpublished + no accepted evidence;
- mode: exceptional;
- authority: owner approval;
- review: before review or explicit reviewer notice;
- push mode: `--force-with-lease` only;
- replaced receipts: invalidate all;
- preferred alternative: ordinary follow-up commit.

Use bounded policy tokens rather than executable shell fragments except for the
two operator-visible Git options. This is a contract, not a Git command runner.

## Validation

The existing review validator adds `history_policy` to the closed root and
compares the nested object to one expected v1 value. Ordered lists therefore
reject omission, duplication, or reordering; closed-object equality rejects
unknown or weakened fields. Existing live-ruleset validation remains unchanged.

One mutation table in `tests/test_authorization_policy.py` changes representative
leverage points: head events/evidence reuse, protected patterns and force policy,
feature eligibility, owner/notice, push mode, receipt invalidation, and preferred
follow-up. The existing delivery-document test mutates the new literal markers.

## Compatibility and Rollback

- Governance metadata only; no public/runtime schema or persisted user data.
- Existing ruleset snapshot and GitHub remote remain read-only.
- Revert the JSON, checklist/template, validator/test, and spec together. No data
  migration or ref repair exists because the task never changes Git history.

## Delivery Strategy

Freeze one source-affecting implementation PR and run the protected native matrix
once. Merge with GOV-415 still active. Then create one separate Trellis/Ledger
closeout PR containing only allowlisted evidence files; record the exact
implementation SHA and Evaluation Truth run, archive/journal the task, and use
the lightweight evidence-only contexts. This avoids GOV-414's duplicate full
matrix while retaining exact-head truth.

## Deferred

Universal hosted branch/tag enforcement requires a separately authorized GitHub
ruleset change. GOV-415 records the normative contract and the current gap; it
does not present policy-as-code as remote enforcement.
