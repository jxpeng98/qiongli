# Design: Scoped standing implementation authorization

## Boundary

Change the existing textual Trellis policy at its four current decision surfaces:

1. `.trellis/workflow.md` remains the canonical phase and breadcrumb owner.
2. `.agents/skills/trellis-brainstorm/SKILL.md` owns planning behavior.
3. `.agents/skills/trellis-continue/SKILL.md` owns resume routing.
4. `.codex/hooks/session-start.py` owns the Codex session-start next-action hint.

No runtime task state or new configuration is needed. `task.py start` continues to
be the only planning-to-execution state transition.

## Authorization Contract

A final plan may enter implementation through either path:

- **Fresh approval:** after the latest final planning summary, the user explicitly
  approves implementation in a subsequent message.
- **Scoped standing authorization:** before final review, the user explicitly
  authorizes both planning and execution for a named goal, task, or task chain;
  the converged plan stays within that scope and has no unresolved user-owned
  decision.

Both paths require the same planning artifacts, convergence pass, final summary,
validation, and `task.py start`. Standing authorization only removes the forced
turn boundary between final summary and task start.

Fresh input remains required when the plan introduces an uncovered material change
to product scope, UX, compatibility, acceptance, or risk, or requires an external,
destructive, security-sensitive, credential, or publication action not explicitly
covered by the authorization.

## Compatibility

- Generic requests such as "implement", "continue", and task-creation consent do
  not become standing authorization.
- Existing users who do not grant standing authorization see the current review
  and subsequent-approval flow.
- The policy is local to this repository and may be flagged as a deliberate local
  modification during a future `trellis update`.

## Regression Check

Add one `unittest` module under `tests/` that reads the four policy surfaces and
asserts that each contains the shared standing-authorization marker. It also
asserts that the brainstorm skill no longer contains the unconditional
subsequent-message-only rule. This uses only `pathlib` and `unittest`.

## Rollback

Revert the workflow-policy commit. No data migration or task-state repair is
required.
