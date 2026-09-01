# GOV-417 authorization exception lifecycle

## Goal

Complete roadmap item `GOV-417` by defining a fail-closed authorization
exception lifecycle for denial, expiry, revocation, emergency hotfix, and
post-incident reconciliation. Reuse the existing governance and
authorization-policy owners so operators and automation have one auditable
contract rather than a parallel approval system.

## Background

- `GOV-413` remains blocked by the single-maintainer independent-review
  constraint and is not resolved by this task.
- `GOV-416` already separates change approval from privileged execution and
  requires action-bound authorization evidence.
- `GOV-418` separately owns negative policy tests proving that an Agent, CI
  job, or successful check cannot approve itself or widen granted scope.
- The canonical owners already exist: the v1 authorization policy, receipt
  schema, stdlib validator, focused policy test module, product-control spec,
  and delivery checklist.
- The v1 receipt schema already carries finite expiry, decision values for
  approval/denial/revocation/expiry, reason codes, constraints, and evidence
  references, so this task does not need a parallel receipt format.

## Requirements

- Define explicit states and transitions for authorization denial, expiry,
  revocation, emergency hotfix use, and post-incident reconciliation.
- Fail closed when required authorization evidence is absent, stale, revoked,
  mismatched, or incomplete.
- Preserve the existing separation between product/change review and
  action-bound privileged execution.
- Make every emergency path bounded, attributable, time-limited, and followed
  by auditable reconciliation; an emergency path must not become a standing
  bypass.
- Reuse existing policy-as-code and validation owners; add no new service,
  approval subsystem, or speculative abstraction.
- Leave `GOV-413`, `GOV-418`, and unrelated roadmap states unchanged unless
  their own acceptance evidence is produced.

## Out of Scope

- Adding an independent human reviewer or claiming `GOV-413` complete.
- Implementing the broader `GOV-418` adversarial self-approval test matrix.
- Publishing a release, changing product behavior, or adding a new runtime
  authorization service.

## Acceptance Criteria

- [ ] One canonical contract defines all five lifecycle paths and their
      allowed/forbidden transitions.
- [ ] Existing validation enforces the closed lifecycle inventory and rejects
      missing, reordered, unknown, or weakened lifecycle definitions.
- [ ] The lifecycle reuses the v1 receipt decisions and bindings without a new
      service, schema version, role, action, or dependency.
- [ ] One focused mutation table covers the lifecycle contract without
      duplicating the broader `GOV-418` adversarial matrix.
- [ ] Documentation explains operator evidence, emergency limits, and the
      required post-incident reconciliation record.
- [ ] Existing authorization separation and unrelated roadmap states remain
      unchanged.
- [ ] The generated program index remains current and task evidence is
      sufficient to review a truthful `GOV-417` transition.

## Decisions

- Emergency hotfixes are limited to the `repository` plane and continue through
  the existing protected PR path, required checks, and no-force rules.
- Emergency status grants no research, restricted-data, release publication,
  or public-announcement authority.
- The current v1 receipt schema remains unchanged; lifecycle meaning is added
  to the existing policy, validator, operator checklist, tests, and spec.
- Post-incident reconciliation is evidence, not retroactive authorization, and
  must complete before another emergency-hotfix authorization is accepted.

## Open Questions

None.
