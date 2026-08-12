# Audit Execution Plan

## 1. Freeze and inventory the audit snapshot

- [x] Record branch, HEAD, dirty state, target roadmap/ledger/spec identities,
      and current/archived Trellis task state.
- [x] Count task IDs and checkbox states; detect duplicate IDs/headings, broken
      local links, stale "current/next" wording, and inconsistent status terms.

## 2. Verify authority and completed-state claims

- [x] Trace M0 checked items to the acceptance ledger and exact-source limits.
- [x] Trace EVAL-401—407 to archived tasks, commits, tests, specs, and roadmap
      state; identify claims that were not refreshed after completion.
- [x] Inspect the live GitHub Project, Epic coverage, referenced PR/runs, branch,
      tag, and release claims where current access permits.

## 3. Score executability

- [x] Classify M0—M7 using the explicit rubric and current entry gates.
- [x] Audit every remaining M1 item at task level, including dependencies,
      validation shape, owner, blocker, and minimum decomposition.
- [x] Test whether the roadmap's dependency graph, exit gates, 90-day sequence,
      risk mitigations, and current-next-task wording agree.

## 4. Score credibility and synthesize findings

- [x] Classify material claims as supported, partially supported, stale,
      contradictory, or unverified with file/line evidence.
- [x] Separate roadmap design quality from current status reliability.
- [x] Produce prioritized P0/P1/P2 corrections and name the smallest safe next
      Trellis task without making those changes.

## 5. Validate and close the audit

- [x] Re-run structural checks and `git diff --check` on task artifacts.
- [x] Confirm no files outside the audit task changed.
- [x] Review every PRD acceptance criterion against `research/roadmap-audit.md`.
- [x] Run Trellis check and record that no product spec update is required for
      this read-only audit.
- [ ] Obtain commit confirmation, archive the task, and record the session; do
      not push.

## Validation Commands

```bash
rg -n "^- \[[ x]\] `[A-Z]+-[0-9]+`" \
  docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md
git diff --check
git status --short
```

## Risky Boundaries

- Do not mutate GitHub Project/Issues while inspecting them.
- Do not infer publication or release qualification from a successful local or
  CI check.
- Do not mark remote evidence failed solely because credentials or permissions
  prevent inspection.
- Do not let the audit itself become GOV-401 or a roadmap rewrite.
