# Project instructions

## Keep the project aligned

- The [master roadmap](docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md)
  owns direction, dependencies and the current execution horizon.
- The [program ledger](docs/superpowers/roadmaps/qiongli-program-ledger-v1.json)
  owns status and accepted evidence; its
  [current index](docs/superpowers/roadmaps/qiongli-current-program-index.md)
  is generated. Never infer acceptance from a checkbox, passing local test or merge.
- Read the current bounded plan linked from the roadmap and only the relevant
  package/layer specs under `.trellis/spec/` before changing their contracts.
  Accepted ADRs own architecture; supersede them instead of rewriting history.
- Existing `.trellis/tasks/`, `.trellis/workspace/` and `.trellis/spec/` preserve
  plans, decisions and evidence. They are ordinary project knowledge, not a
  mandatory task engine. Do not reinstall Trellis skills, hooks or agents unless
  the user asks.

## Deliver small, complete increments

1. Inspect the working diff and trace the behavior to its existing owner.
2. State the user outcome and the smallest useful change. Reuse an existing plan;
   create one short plan only when the scope needs it. Routine fixes need no new
   task directory, PRD, context manifest, phase approval or journal entry.
3. Implement within the user's requested scope and run the closest meaningful
   checks. Continue through routine fixes without asking for approval again.
4. Review the final diff; update contracts when behavior changes and record what
   passed, what remains unverified and the next increment in the existing plan.

The main Agent may implement and check directly. Delegate only bounded independent
work with clear file ownership when it helps; no mandatory role chain or channel.
Parallel work shares a stable interface and one integration outcome. A blocked
external validation lane does not stop independent offline development.

## Preserve the product boundaries

- Follow [CONTRIBUTING.md](CONTRIBUTING.md) for source ownership and daily, PR,
  build and release checks. Run affected checks while editing; use required CI
  when preparing integration, and qualify packages at a named candidate boundary.
- Keep input validation, permission checks, compatibility and data-loss negative
  cases. Qiongli project writes still use existing preview/approval/CAS owners.
- Preserve unrelated working changes and user data. Material scope changes or
  external/destructive actions need authority for that action; ordinary coding
  authorization does not imply publication or access to private research data.
- Commit, push, merge and publish only within the user's authorization. Never
  auto-commit bookkeeping or start release work because a local check passed.
