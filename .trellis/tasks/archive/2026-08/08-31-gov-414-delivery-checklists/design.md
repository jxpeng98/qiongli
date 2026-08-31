# Design: GOV-414 repository delivery checklists

## Boundary

This slice exposes policy that already exists; it does not create another
delivery engine:

```text
master roadmap Sections 19.5-19.7
  + authorization-policy-v1.json
  + release-branch-policy.md verification tiers
  -> .github/delivery-checklists.md
  -> .github/pull_request_template.md
  -> existing authorization validator + focused tests
  -> existing Evaluation Truth V1
```

The checklists select and record evidence. Existing tests, Native CI, protected
branch rules, and release scripts continue to produce or enforce that evidence.

## Canonical artifacts

- `.github/delivery-checklists.md` owns the pre-commit, pre-push, PR, and release
  operator sequence.
- `.github/pull_request_template.md` owns the default PR evidence fields and
  links the canonical checklist instead of copying it.
- `tooling/scripts/validate_authorization_policy.py` remains the single
  governance validator. It checks only stable literal markers needed to prove
  the four stages and PR contract remain present.
- `tests/test_authorization_policy.py` owns one valid-state assertion and one
  compact mutation table for checklist/template drift.
- The existing authorization and repository-review JSON evidence lists identify
  the two new canonical files. No new policy record is needed.

Both new files live under `.github/`, so the current `/.github/ @jxpeng98`
CODEOWNERS entry already covers them. No CODEOWNERS expansion or second-language
documentation mirror is required for this repository workflow surface.

## Checklist contract

Every stage uses Markdown task items with an explicit evidence class:

- **Machine** — a repository command, exact revision, required check, digest, or
  existing release gate can falsify the claim.
- **Human** — scope, secret/restricted-data review, compatibility judgment,
  reviewer decision, release approval, or another authority boundary cannot be
  inferred from automation.

The stages remain non-transitive:

```text
edit != commit != push != PR/merge != release/publication
```

Pre-commit verifies explicit staged paths, staged diff, whitespace, and the
smallest behavior check. Pre-push verifies the intended non-protected branch,
base/diff/checkpoint, and current focused evidence. PR records the roadmap's
minimum fields and exact-head protected checks. Release starts again from the
integration commit, uses the existing release wrappers and explicit candidate
dispatch, and requires separate asset-bound approval.

## Fast verification flow

```text
editing -> Focused checks only
draft PR -> no native matrix expansion
ready source PR -> one exact-head Linux/macOS/Windows Slice
explicit release candidate -> Acceptance gates
```

The checklist does not prescribe cargo-xwin or a Windows guest for every push.
Those remain optional early feedback documented by the release branch policy;
the protected Windows context remains Slice authority.

## Validation

The existing validator loads the two Markdown files and checks an exact small
inventory of headings, evidence labels, canonical commands/links, PR fields, and
the non-authorization warning. Tests remove one marker at a time and require a
failure. This is intentionally a literal contract, not a Markdown parser or a
generic checklist schema.

Evaluation Truth already runs both the validator command and test module, so no
workflow edit is required.

## Compatibility, rollout, and rollback

- The change adds repository guidance only; no App, CLI, MCP, Plugin/Skill,
  package, release, or persisted-data shape changes.
- `GOV-414` moves to `active` when implementation starts. It moves to
  `accepted` only after exact implementation-head CI evidence exists.
- Reverting the checklist/template, their evidence-list entries, validator
  markers, tests, and spec update fully rolls back the slice without migrating
  product or external state.

## Deferred work

- Automatic local hook installation or policy execution.
- Changes to CI classification, required checks, review enforcement, release
  authorization, exceptional paths, or self-authorization controls owned by
  later GOV items.
