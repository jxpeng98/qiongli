# Implement GOV-416 decision separation

## Goal

Make merge, release publication, and public announcement three explicit,
non-transitive authorization actions so one decision or receipt cannot be
replayed as authority for either later action.

## Background and Confirmed Facts

- The Program Ledger defines GOV-416 as keeping merge authorization, release
  authorization, and public-announcement authorization separate.
- The existing authorization policy already owns `repository.merge` and
  `publication.publish-release`; the receipt schema binds each receipt to one
  scalar `action`. The missing action is public announcement.
- The master roadmap requires public verification before announcement and binds
  announcement content to version, targets, claims/non-claims, upgrade and
  rollback guidance, known issues, and verification links
  (`docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md:1031`,
  `:1156`, `:1168`).
- The existing `release-approver` role owns release claims and announcement
  authority. GOV-416 requires separate decisions, not a second human role.
- The release checklist already orders merge, publication, verification, and
  announcement, but it does not require a distinct announcement decision or
  receipt.
- Native release envelopes and keys authenticate product artifacts; they do not
  represent human merge/publication/announcement decisions and are outside this
  governance task.

## Requirements

### R1 — Three action-bound decisions

- Preserve `repository.merge` and `publication.publish-release` unchanged.
- Add `publication.announce-release` as the twelfth closed authorization action.
- Require the existing `release-approver` role for announcement authorization;
  Maintainer or Agent/CI may only execute an already-authorized mechanical step.
- Bind announcement authority to exact object/source revision, announcement-plan
  digest, verified public artifact digests, channels, constraints, and expiry.
- Require the publication receipt, independent public-verification evidence,
  announcement-content digest, and a named announcement approval.

### R2 — Non-transitive receipts

- Repository merge must authorize neither release publication nor announcement.
- Release publication must not authorize announcement.
- Green CI must authorize none of merge, release publication, or announcement.
- Each receipt remains immutable evidence of exactly one action; a receipt whose
  `action` differs from the requested action cannot substitute for it.

### R3 — Operator-visible separation

- Keep the release checklist order: merge integration, release decision,
  publication, independent public verification, distinct announcement decision,
  then announcement.
- State that the announcement receipt binds verified public bytes, channels, and
  the exact announcement content/claims; publication authorization does not
  transfer.
- Update the PR confirmation so green checks are not represented as merge,
  release, or announcement authorization.

### R4 — One existing executable owner

- Extend the existing policy JSON, receipt schema enum, standard-library
  validator, focused test module, delivery documents, and product-control spec.
- Make receipt validation enforce every action's declared plan-digest and
  artifact-digests bindings instead of accepting only one generic digest.
- Keep the schema identity at V1: this is an additive governance action and all
  current receipt fields remain unchanged.
- Add no new policy file, receipt store, command, role, dependency, workflow,
  service, or product/runtime schema.

### R5 — Truthful delivery

- Mark only GOV-416 active during implementation and accepted only from a frozen
  implementation SHA plus exact-head Evaluation Truth evidence.
- Run one protected Linux/macOS/Windows Slice for the implementation PR, then use
  a separate allowlisted evidence-only closeout PR to avoid a second full matrix.

## Acceptance Criteria

- [ ] The policy and receipt schema contain twelve ordered actions including
      `publication.announce-release`.
- [ ] Announcement requires the release approver, announcement-plan digest,
      verified artifact digests, channels, expiry, and named evidence.
- [ ] Closed negative transitions prove merge, publication, and CI do not grant
      announcement authority.
- [ ] The release checklist requires independent public verification and a
      distinct announcement decision/receipt before announcement.
- [ ] The PR template states that green checks grant no merge, release, or
      announcement authority.
- [ ] The validator accepts the repository and rejects a missing announcement
      action, wrong authorizer, weakened bindings/evidence, missing transition,
      announcement receipt digest, or schema/template drift.
- [ ] `python3 tooling/scripts/validate_authorization_policy.py` and
      `python3 -m unittest tests.test_authorization_policy
      tests.test_program_roadmap -v` pass.
- [ ] No release, tag, asset, announcement, product code, dependency, workflow,
      service, GitHub setting, or user data is created or changed.
- [ ] One frozen implementation head passes protected Linux, macOS, Windows,
      Native boundary, and Evaluation Truth contexts before merge.
- [ ] An evidence-only closeout records acceptance, archives the task, and leaves
      local `2.x` clean without a second full Native matrix.

## Out of Scope

- Executing or automating merge, publication, verification, or announcement.
- Adding an independent announcement-approver role or requiring different people
  for release and announcement decisions.
- Changing native release signatures, artifact envelopes, credentials, package
  channels, release scripts, or hosted protection settings.
- GOV-417 denial/revocation/emergency paths and GOV-418 self-authorization tests
  beyond the focused GOV-416 negative cases.

## Risks and Deferred Items

- The policy proves separate decisions but does not create a receipt database or
  grant real authority; execution remains controlled by existing human and
  protected-environment processes.
- Existing receipts remain valid for their recorded action. Any changed source,
  digest, channel, or claim still follows GOV-415 invalidation rules.
- A separate announcement-approver role is deferred until policy or staffing
  requires a different human authority; adding it now would not improve action
  separation.
