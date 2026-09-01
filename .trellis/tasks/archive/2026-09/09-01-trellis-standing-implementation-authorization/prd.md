# Allow scoped standing implementation authorization

## Goal

Let an explicit, scoped standing authorization cover Trellis planning and
implementation for a named goal or task chain, so each converged plan can move
directly into execution without a redundant second approval message.

## Background

- The current goal explicitly authorizes creating plans and executing them while
  continuing the 2.x roadmap.
- `.agents/skills/trellis-brainstorm/SKILL.md:8-14` and `:58-61` currently require
  a user response after the initial request and force every final planning summary
  to end the turn.
- `.trellis/workflow.md:150-152`, `:192-210`, and `:293` repeat the separate-
  approval rule in the canonical workflow and live planning breadcrumbs.
- `.agents/skills/trellis-continue/SKILL.md:35` and
  `.codex/hooks/session-start.py:280-287` still tell resumed sessions to ask for
  another start review.
- Planning artifacts, requirement convergence, unresolved user decisions, and
  `task.py start` remain useful gates; only the duplicate message round-trip is
  unnecessary when the user has already granted explicit standing authority.

## Requirements

- R1. Treat an explicit authorization to plan and execute a named goal, task, or
  task chain as scoped standing implementation authorization.
- R2. After requirement convergence and final plan review, allow `task.py start`
  in the same turn when that standing authorization covers the final plan.
- R3. Preserve the existing subsequent-message approval path when no standing
  authorization exists.
- R4. Never infer standing authorization from a generic request to build, fix,
  continue, or create a task; the user must explicitly authorize execution.
- R5. Standing authorization must not resolve user-owned product, scope, UX,
  compatibility, risk, or acceptance decisions, and must not cover a material
  scope/risk change or an external/destructive action outside its stated scope.
- R6. Preserve task creation consent, planning artifacts, convergence review,
  validation, and `task.py start`; the change removes only redundant approval.
- R7. Keep the canonical workflow, brainstorm skill, continue skill, and Codex
  session-start hint aligned.
- R8. Add one stdlib-only regression check for the policy alignment.

## In Scope

- Local Trellis workflow and Codex planning/resume guidance.
- A narrow repository test that detects contradictory approval guidance.
- PR creation and merge into `2.x`, as already authorized for this goal.

## Out of Scope

- New task states, configuration keys, authorization databases, or parsers.
- Relaxing release-environment protection, credential, destructive-action,
  security, or external-publication authorization boundaries.
- Changing product code or the 2.x roadmap itself.
- Making the authorization portable to unrelated goals, repositories, or threads.

## Acceptance Criteria

- [x] An explicit scoped standing authorization can move a converged final plan
  through `task.py start` without waiting for another user message.
- [x] A task without standing authorization still requires explicit approval of
  its latest final planning summary.
- [x] Any unresolved user-owned decision or uncovered material scope/risk change
  still pauses for fresh user input.
- [x] `.trellis/workflow.md`, `trellis-brainstorm`, `trellis-continue`, and the
  Codex session-start hint describe the same two authorization paths.
- [x] A stdlib `unittest` fails if those four surfaces drift back to conflicting
  policy wording.
- [x] The focused test and Python syntax check pass before the PR is merged to
  `2.x`.

## Key Decision

Use the existing conversation/task artifacts as authorization evidence. Do not add
a config switch or persistent authorization subsystem; a standing authorization is
valid only when its scope is explicit and visible in the current goal or task PRD.

## Risks and Deferred Items

- Broad wording could be over-applied. The policy therefore requires explicit
  execution language, a named scope, no unresolved user-owned decisions, and a
  fresh review when scope or risk materially changes.
- Cross-thread/global standing authorization is deferred; it would require a
  separate persistence and revocation design.

## Open Questions

None.
