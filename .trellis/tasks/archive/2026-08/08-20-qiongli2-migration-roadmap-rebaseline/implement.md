# Implementation plan: replacement-first 2.0 roadmap

## 1. Start only after approval

- [ ] Review the final PRD/design/plan summary and obtain explicit approval.
- [ ] Run `task.py start` on the current `2.x` base and load the product-control
      guidance required by `trellis-before-dev`.
- [ ] Keep this task limited to roadmap, governance, and verification-flow
      changes. Record product gaps as later bounded tasks.

## 2. Rebase the master roadmap around 1.19 replacement

- [ ] Update the master roadmap's scope, dependency diagram, milestones,
      replacement matrix, first-90-days sequence, risk register, and definition
      of done.
- [ ] Preserve accepted M0/M1 task IDs and exact evidence. Reorder only
      unaccepted work into this sequence:
      - M0: historical Alpha 3 and remaining external qualification;
      - M1: replacement truth and platform baseline;
      - M2: 2.0 beta product replacement and migrated-project dogfood;
      - M3: 2.0 RC, Stable, migration, rollback, and cutover;
      - M4-M7: post-2.0 Kernel, Evidence/Reproducibility, research governance,
        interoperability, collaboration, and institutional expansion.
- [ ] Add only four missing owners: `GOV-320` for replacement truth and tiered
      verification, `PLT-320` for the shared CLI/Plugin/Skills/MCP/Zotero
      vertical, `PLT-321` for App stability on native contracts, and `PLT-322`
      for representative migrated-project Graph v1 acceptance.
- [ ] Reuse existing `REL-902`, `REL-905`, `REL-906`, `REL-913`, and related
      release tasks for migration, support, retirement, install/update, and
      rollback instead of adding duplicate cutover tasks.
- [ ] Keep the existing 16-outcome 1.x parity ledger unchanged as a bounded
      classification contract. Add the wider replacement matrix to the master
      roadmap and state plainly that it supplements, not replaces, that ledger.

## 3. Keep roadmap state deterministic

- [ ] Add the four new rows to Program Ledger v1, order unaccepted rows to
      match the rebased roadmap, and update dependencies without inventing
      acceptance evidence.
- [ ] Keep `GOV-320` active while this task is open; leave product-delivery rows
      proposed or deferred until their own work begins.
- [ ] Update the ledger specification and focused tests from 233 to 237 exact
      roadmap IDs, then regenerate the current program index with the existing
      generator.

Focused checks:

```bash
python3 -m unittest tests.test_program_roadmap
python3 tooling/scripts/update_program_roadmap.py --check
```

## 4. Make the three verification tiers executable guidance

- [ ] Update the product-control spec, `.trellis/workflow.md`, and the existing
      `trellis-check` skill so every task declares Focused, Slice, or Acceptance
      scope.
- [ ] Define task-ending “full-scope” as the complete changed business slice
      and affected packages, not an unrelated full-repository or release run.
- [ ] Require concise success summaries and only the first actionable failure
      plus its smallest reproduction. Do not add a reporter, wrapper, or new
      test runner.
- [ ] Retain early focused checks for security, data loss, schema compatibility,
      authorization, paths, ownership, and other trust boundaries.

## 5. Move package and promotion work to explicit candidate runs

- [ ] In `native-ci.yml`, preserve the four required branch-protection context
      names and all three platforms' Rust source checks.
- [ ] Use the existing `run-frontend-checks` input so portable frontend checks
      run once on Linux while macOS and Windows still build the Desktop and run
      their native Rust checks.
- [ ] Gate desktop package assembly, packaged-product acceptance, Lite candidate
      acceptance, and dependent Community Alpha promotion to explicit
      `workflow_dispatch` runs. Keep Lite runtime compatibility and Evaluation
      Truth automatic.
- [ ] Update the existing branch-policy unittest to fail if required contexts
      drift, portable checks multiply across platforms, or acceptance jobs run
      on ordinary push/PR events.
- [ ] Synchronize the English and Chinese branch-policy documents with the same
      tier and candidate-trigger rules; retain the existing 1.x freeze and
      90-day post-Stable support policy as the sole authority.

Focused check:

```bash
python3 -m unittest tests.test_branch_policy
```

## 6. Verify at the tier this governance slice actually needs

- [ ] Run the two focused unittest modules, deterministic roadmap check, and
      `git diff --check` locally.
- [ ] Freeze the source, then use the existing exact-head Evaluation Truth and
      four required Native CI source contexts as Slice evidence.
- [ ] Do not run package assembly, live-Host, migration/rollback, promotion, or
      full release acceptance for this governance-only task. Those remain
      mandatory for an explicit 2.x candidate.
- [ ] Review the final diff for preserved accepted evidence, synchronized
      derived files, and no claim that missing product behavior was implemented.

Local quality gate:

```bash
python3 -m unittest tests.test_program_roadmap tests.test_branch_policy
python3 tooling/scripts/update_program_roadmap.py --check
git diff --check
```

## Rollback points

- Roadmap and ledger changes revert together, followed by regenerating the
  current index.
- Trellis workflow/spec/skill changes revert together so check semantics do not
  disagree.
- Native CI, branch-policy tests, and bilingual policy text revert together so
  repository behavior and documentation remain aligned.
- No user data, package, release, or live repository setting changes occur in
  this task.
