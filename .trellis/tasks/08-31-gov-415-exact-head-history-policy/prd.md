# Implement GOV-415 exact-head and history policy

## Goal

Make source-bound evidence invalidation and Git history-rewrite limits explicit,
version-controlled, and machine-verifiable so stale green checks cannot survive a
source change and the repository contract fails closed against protected-ref
force pushes.

## Background and Confirmed Facts

- The Program Ledger defines GOV-415 as: "Define exact-head evidence
  invalidation, feature-branch history-rewrite limits and an absolute
  no-force-push rule for protected branches, release branches and tags."
- The master roadmap already owns the policy intent: direct protected-ref pushes
  are prohibited, every new PR head invalidates stale evidence, protected refs
  and accepted-evidence heads are never force-pushed, and an exceptional
  feature rewrite requires owner approval, reviewer notice,
  `--force-with-lease`, and receipt invalidation
  (`docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md:1064`,
  `:1094`, `:1115`).
- The accepted invalidation guide distinguishes always-current exact-head CI
  from input-bound package/release evidence so unchanged product inputs do not
  trigger unnecessary rebuilds
  (`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md:155`).
- GOV-413 and GOV-414 established the existing owners:
  `repository-review-policy-v1.json`, `authorization-policy-v1.md`, the delivery
  checklist/PR template, one standard-library validator, one focused test module,
  and Evaluation Truth. This task extends those owners rather than adding a
  second policy file or workflow.
- Read-only GitHub API inspection on 2026-08-31 shows active no-bypass
  `non_fast_forward` protection for `2.x` and `release/1.x-python`; no tag-target
  ruleset or general `release/*` ruleset exists. GOV-415 says **define**, while
  GOV-413 owns hosted branch/reviewer configuration, so this task must not claim
  universal remote enforcement or mutate repository settings.
- Repository search found no executable `git push --force` or
  `--force-with-lease` path; the latter appears only in the owning roadmap text.

## Requirements

### R1 — Exact-head and input-bound evidence

- Bind exact-head identity to one full commit SHA and treat new commits, amend,
  rebase, merge, and history rewrite as head-change events.
- Any head change invalidates exact-head CI and review evidence; authorization
  receipts remain reusable only while every revision/scope/digest binding is
  current.
- Package evidence is invalidated only by a changed bound package input or
  digest. Release evidence is invalidated by changed source, version, target,
  artifact digest, update metadata, destination, channel, or claim and never
  transfers to a replacement candidate.
- Reuse is permitted only when the owning receipt's complete binding set remains
  identical; uncertainty fails closed without automatically rerunning unrelated
  package or release work.

### R2 — Protected-ref immutability

- Cover `refs/heads/2.x`, `refs/heads/release/*`, and `refs/tags/*` as protected
  ref classes.
- Updates use only the protected PR or separately authorized release path.
- Plain or lease-protected force push and every history rewrite are forbidden for
  those refs and for any head that owns accepted evidence. There is no emergency
  bypass inside GOV-415.

### R3 — Exceptional feature-branch rewrite

- Limit rewrite eligibility to an unprotected, unpublished working branch with
  no accepted evidence.
- Require explicit owner approval and either completion before review or an
  explicit reviewer notice.
- Permit only `--force-with-lease`; plain `--force` is never permitted.
- Invalidate every receipt bound to replaced commits and prefer an ordinary
  follow-up commit.

### R4 — One executable governance owner

- Extend the closed `repository-review-policy-v1.json` record with one bounded
  history-policy object; add no second schema, CLI, hook, dependency, or service.
- Keep the canonical checklist and PR template consistent with that object.
- Extend `validate_authorization_policy.py` and its existing mutation-based test
  module so missing, reordered, unknown, or weakened history rules fail closed.
- Keep Evaluation Truth as the only CI owner and preserve every existing public
  validation function.

### R5 — Truthful delivery

- Mark only GOV-415 active during implementation and accepted only from a full
  implementation SHA plus exact-head Evaluation Truth run.
- Keep hosted GitHub settings read-only and report the current enforcement gap
  explicitly; do not claim tag or general release-branch remote protection.
- Preserve green evidence, merge authorization, release authorization, and
  publication authorization as separate decisions.

## Acceptance Criteria

- [ ] The existing repository-review JSON contains one closed history policy
      with the required head events, evidence rules, protected ref classes, and
      feature rewrite constraints.
- [ ] The delivery checklist and PR template state that every head change
      replaces stale exact-head evidence, plain `--force` is forbidden, and the
      only feature exception is bounded `--force-with-lease`.
- [ ] The existing validator accepts the repository state and rejects a missing
      history object, weakened invalidation, narrowed protected refs, permitted
      protected force push, plain feature `--force`, missing authority/notice,
      or retained replaced-commit receipt.
- [ ] `python tooling/scripts/validate_authorization_policy.py` and
      `python -m unittest tests.test_authorization_policy
      tests.test_program_roadmap -v` pass, and Evaluation Truth retains those
      existing commands.
- [ ] A read-only hosted-settings audit remains truthful: no GitHub ruleset is
      changed and no unimplemented enforcement is claimed.
- [ ] No product, package, release asset, tag, branch history, user data,
      dependency, hook, service, or workflow is created or mutated.
- [ ] One frozen implementation head passes protected Linux, macOS, Windows,
      native-boundary, and Evaluation Truth contexts once before merge.
- [ ] A separate evidence-only closeout records the implementation SHA/run,
      accepts GOV-415, archives the task, and avoids a second full native matrix.

## Out of Scope

- Creating/updating GitHub branch or tag rulesets, collaborator permissions,
  CODEOWNER enforcement, bypass actors, or protected-branch approval counts.
- Rewriting any real branch, replacing any tag, replaying a receipt, or creating
  a release candidate/publication.
- Client-side Git hooks, server hooks, a new command wrapper, a general Git ref
  policy engine, or a new JSON Schema dependency.
- GOV-416 merge/release/announcement decision separation, GOV-417 exceptional
  authorization paths, and GOV-418 self-authorization enforcement.

## Risks and Deferred Enforcement

- Checked-in validation prevents policy drift but cannot by itself stop a GitHub
  administrator from changing an unprotected remote ref. The existing live
  enforcement gap remains explicit and belongs to a separately authorized
  hosted-settings task under GOV-413 ownership.
- Over-invalidating input-bound evidence would waste the cross-platform/package
  work GOV-414 was designed to reduce. The contract therefore distinguishes
  exact-head evidence from receipt-bound package/release evidence.
