# Audit roadmap executability and credibility

## Goal

Determine whether the current Qiongli 2 master roadmap is a trustworthy and
actionable program authority after EVAL-407: its present-state claims must match
repository and accepted evidence, its dependencies and gates must support a
real execution sequence, and its next tasks must be sufficiently bounded to
enter Trellis planning without inventing missing decisions.

## Confirmed Facts

- The audit snapshot is commit
  `255baa3ec430efdc837748a6676b547163ba4416` on branch
  `fix/alpha3-codex-claude-host-qualification`; the roadmap targets `2.x`.
- EVAL-401 through EVAL-407 are checked and have archived Trellis work and
  implementation commits. EVAL-408 through EVAL-411 remain unchecked.
- M0 internal first-use work is closed while target-native, live-Host, update,
  trust, publication, and public-observation evidence remains open.
- The roadmap has a useful milestone dependency graph and explicit entry/exit
  gates, but several present-tense statements predate recent task closure.
- Initial inspection found three concrete drift candidates:
  - Sections 8 and 9 refer to a now-archived "current" qualification task and
    direct the next task to EVAL-401—405.
  - Section 3.3 still reports the empty-eval/YAML validation gap as open while
    Section 10 marks EVAL-401—407 complete.
  - Section 8.1 says detailed state lives in a machine-readable ledger while
    `GOV-401` still requires creating that ledger.
- The roadmap claims 32 GitHub Epic Issues cover 232 task IDs exactly once;
  this is a current external claim and needs independent verification rather
  than trust in prose.

## Requirements

### R1 — Audit authority and freshness

- Check the roadmap's status header, baseline, current-task language, branch,
  release state, acceptance-ledger relationship, and time-based guidance
  against the current repository and authoritative records.
- Classify every mismatch as stale wording, factual contradiction, missing
  evidence, or intentionally deferred state.

### R2 — Audit status credibility

- Trace checked M0/M1 claims to the narrowest owner: archived Trellis task,
  implementation commit, executable test/spec, acceptance receipt, or external
  run/PR where applicable.
- Do not treat a checkbox, merged commit, historical receipt, or local test as
  stronger evidence than its owning gate permits.
- Mark external or permission-bound claims `unverified` when they cannot be
  independently inspected; do not convert absence of access into failure.

### R3 — Audit executability

- Evaluate each milestone at program level and each remaining M1 item at task
  level for: concrete deliverable, owner/authority, dependencies, entry state,
  executable validation, exit condition, and blocker handling.
- Distinguish `ready now`, `ready after bounded planning`, `dependency-blocked`,
  and `aspirational/not yet executable`.
- Identify the smallest credible next Trellis task and the decisions/evidence it
  would still need before implementation.

### R4 — Audit structural integrity

- Verify task-ID uniqueness and stated counts, checkbox totals, duplicate or
  conflicting sections, local link targets, milestone ordering, and cross-
  references such as the 90-day plan and risk register.
- Verify the GitHub Project/Epic mapping and other live external identities with
  current GitHub evidence where access permits.

### R5 — Produce an evidence-backed assessment

- Record findings under task research with file/line anchors, severity,
  evidence, impact, and a minimal corrective recommendation.
- Provide separate executability and credibility ratings with an explicit
  rubric; do not collapse them into one unexplained score.
- End with a prioritized repair sequence and a clear recommendation for the
  next task, while leaving the roadmap and product code unchanged.

## Acceptance Criteria

- [x] Every roadmap milestone M0—M7 has an executability classification and a
      stated basis.
- [x] Every checked M0/M1 item is sampled or traced to its evidence owner, with
      exact gaps called out rather than inferred away.
- [x] Every remaining M1 item is classified as ready, planning-needed,
      dependency-blocked, or not yet executable.
- [x] Task-ID count/uniqueness, checkbox totals, local links, duplicate sections,
      stale temporal language, and the 32-Epic/232-ID claim are checked.
- [x] Findings include file/line anchors, severity, evidence, impact, and the
      smallest corrective action.
- [x] The final report gives separate transparent ratings for executability and
      credibility, plus confidence and verification limits.
- [x] The audit names one recommended next Trellis task and explains why it is
      the next safe unit of work.
- [x] No roadmap checkbox, product file, release state, remote object, or public
      claim is changed by this task.

## Out of Scope

- Editing or regenerating the roadmap, program ledger, ADRs, issues, milestones,
  GitHub Project, acceptance ledger, or product code.
- Re-running full release qualification, live Host, target-native, package,
  update, trust, or publication journeys.
- Treating long-horizon estimates as delivery commitments or decomposing all
  232 IDs into Trellis tasks.
- Implementing EVAL-408, GOV-401, or any other roadmap item during the audit.

## Risks and Limits

- Some GitHub Project fields or protected evidence may be inaccessible; those
  claims will remain explicitly unverified.
- Historical receipts can prove only their exact source/package identity.
- Long-horizon M2—M7 items can be assessed for dependency coherence and task
  shape, not implementation feasibility at code-line precision before their
  entry gates close.
