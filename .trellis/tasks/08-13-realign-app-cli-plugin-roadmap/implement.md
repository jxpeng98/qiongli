# Implementation Plan

## 1. Apply The Roadmap Realignment

- [x] Start only after explicit approval of the latest planning summary.
- [x] Update the master-roadmap status, baseline/gaps, dependency sequence,
      M0/M1 current-task language, and first-90-days projection.
- [x] Correct stale 232 references to the existing 233-ID inventory without
      adding a Task ID or changing unrelated milestone checklists.
- [x] Update `.trellis/spec/product/control/index.md` with the same immediate
      sequence and evidence boundary.

## 2. Verify Planning Authority

- [x] Confirm that the roadmap still contains exactly 233 unique backticked
      Task IDs.
- [x] Confirm activation is named before Plugin quality everywhere that names
      the current/next execution lane.
- [x] Confirm M0 release qualification, EVAL-409, and M2-M7 remain open where
      their evidence is absent.
- [x] Confirm local EVAL-401 through EVAL-407 work is not described as accepted
      on `origin/2.x`.

Checks:

```bash
rg -o '`[A-Z]+-[0-9]+`' \
  docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md \
  | sort -u | wc -l
rg -n 'activation|Plugin quality|EVAL-409|origin/2.x|233' \
  docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md \
  .trellis/spec/product/control/index.md
git diff --check
```

## 3. Transition To P0 Only

- [x] Review the final diff against the parent PRD and research audit.
- [ ] Run the Trellis quality/spec closeout for this documentation task and
      archive it after its change is committed.
- [ ] Start `.trellis/tasks/08-13-close-app-cli-plugin-activation` as the only
      active implementation task.
- [x] Leave `.trellis/tasks/08-13-make-plugin-quality-executable` in planning
      until P0 is accepted or explicitly deferred with evidence.

## Quality Evidence

- The before/after master-roadmap Task-ID sets are identical and contain 233
  unique IDs.
- Priority searches show activation before `EVAL-409` Plugin quality and the
  remaining M1 lane in the dependency graph, M0/M1 boundaries, and 90-day plan.
- All three Trellis task manifests validate in inline mode.
- `git diff --check` passes; no product or package input changed, so code,
  packaged-App, and live-Host tests are not required for this documentation
  slice.

## Review Focus

- The roadmap changed order, not historical evidence.
- No new backlog, architecture layer, or release claim was introduced.
- The child dependency is explicit rather than inferred from directory order.

## Rollback Point

Revert the documentation commit and clear the P0 active pointer if the priority
decision is withdrawn before product implementation begins.
