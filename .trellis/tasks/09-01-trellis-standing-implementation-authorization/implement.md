# Implementation Plan

- [x] After final review under the current Trellis rule, run `task.py start` and
      create `feat/trellis-standing-authorization` from the current `2.x` head.
- [x] Update `.trellis/workflow.md` so Phase 1, both planning breadcrumbs, the
      guardrail, activation step, and completion criteria accept fresh approval or
      explicit scoped standing authorization.
- [x] Update `.agents/skills/trellis-brainstorm/SKILL.md` so a covered converged
      plan may start in the same turn while unresolved or materially changed scope
      still pauses for the user.
- [x] Align `.agents/skills/trellis-continue/SKILL.md` and
      `.codex/hooks/session-start.py` with the same two-path rule.
- [x] Add `tests/test_trellis_standing_authorization_policy.py` as the single
      stdlib-only policy drift check.
- [x] Run:
      `python3 -m unittest tests.test_trellis_standing_authorization_policy -v`;
      `python3 -m py_compile .codex/hooks/session-start.py`;
      `python3 ./.trellis/scripts/task.py validate 09-01-trellis-standing-implementation-authorization`.
- [ ] Review the focused diff, commit with Conventional Commits, create a PR to
      `2.x`, verify its required checks, and merge it.
- [ ] Close and archive the Trellis task, then continue to the Alpha 5 release
      freeze without requesting another implementation approval inside the
      already-authorized roadmap goal.
