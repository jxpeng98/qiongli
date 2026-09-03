# Simplify Qiongli 2.x delivery flow

## Goal

Keep Qiongli 2.x lightweight by separating daily development, pull-request
integration, cross-platform builds, and releases into independently triggered
lanes while one master roadmap continues to own product direction and order.

## Background

- The existing Native CI already separates ready-PR source checks from explicit
  `workflow_dispatch` package and release-candidate work.
- The current Trellis workflow nevertheless requires a Slice-tier final check
  for every task, and ordinary documentation/process pull requests conservatively
  trigger the three-platform Rust matrix.
- The 237-row program ledger is useful evidence state, but it should not compete
  with the master roadmap as a second planning surface.
- Alpha 5 is now an unpublished internal candidate at
  `842f6bb7136fc03551b7a1acf3b612daa3dc6953`; its successful Native CI and
  candidate runs do not authorize publication.

## Requirements

### R1 — One authority chain

- The master roadmap owns direction, milestone order, and the short
  `NOW / NEXT / LATER` horizon.
- The program ledger owns task state and exact evidence only; its generated
  index is a detailed status view, not a competing roadmap.
- One active Trellis task selects a bounded roadmap work package and owns its
  implementation scope.

### R2 — Independent delivery lanes

- Daily development defaults to the smallest Focused checks for changed
  behavior and does not require cross-platform builds or release receipts.
- A ready pull request owns exact-head integration checks and review status.
- Cross-platform builds prove only the named target/source; they do not grant
  merge, release, or publication authority.
- Release qualification runs only for an explicitly selected candidate and
  retains full exact-source, three-platform, trust, rollback, and public-boundary
  controls.
- Security, authorization, schema, path ownership, and data-loss risks keep
  their earliest applicable negative checks.

### R3 — Lightweight Agent ownership

- Small work remains in the main Agent.
- A planned medium or large task uses the main Agent as supervisor/orchestrator,
  one Implement Agent, and one Check Agent through existing Trellis roles.
- Durable `trellis channel` coordination remains opt-in for genuine multi-turn
  peer communication; no fourth Orchestrator role or message bus is added.

### R4 — Make the policy executable and easy to find

- Reduce contributor and delivery instructions to one entrypoint and clear
  links to each lane's existing owner.
- Let pure non-runtime documentation, Trellis process, and evidence pull
  requests preserve required context names while skipping the native build
  matrix.
- Keep mixed, unknown, runtime, workflow, action, fixture, and empty diffs on
  the fail-safe full matrix path.
- Preserve historical plans and receipts as read-only evidence instead of
  deleting or rewriting them.

## Acceptance Criteria

- [x] CONTRIBUTING presents one master roadmap and four delivery lanes without
      the obsolete migration-inventory wall of text.
- [x] Trellis final checks default to task-scope Focused validation; Slice,
      Build, and Acceptance run only when their lane is explicitly entered.
- [x] Future complex Trellis tasks can use the existing main -> Implement ->
      Check ownership model, while small work remains inline.
- [x] The native change-boundary test proves pure process/docs changes skip the
      Rust matrix and mixed/runtime changes still require it.
- [x] Roadmap docs expose one current `NOW / NEXT / LATER` horizon and clearly
      demote the ledger/index to evidence state.
- [x] Alpha 5 is recorded as an internal candidate with no public-release claim.
- [x] Focused policy, roadmap, task, and release-note validation passes.

## Out of Scope

- Removing product tests, security checks, release signing, or data-loss guards.
- Changing required GitHub context names, branch protection, or release workflow
  publication behavior.
- Deleting the program ledger, historical plans, or accepted receipts.
- Product features, a new CI system, a new Agent type, or a persistent Agent bus.
- Committing, pushing, merging, tagging, publishing, or announcing without the
  separate authority required for those actions.

## Key Decisions

- Separate responsibility and trigger boundaries, not infrastructure.
- Reuse the existing Native CI path classifier and Trellis Agent roles.
- Keep three verification meanings (`Focused`, `Slice`, `Acceptance`); treat
  cross-platform Build as an independently triggered activity, not a fourth
  evidence tier.
