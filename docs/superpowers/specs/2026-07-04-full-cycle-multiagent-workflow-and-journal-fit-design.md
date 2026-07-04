# Full-Cycle Multi-Agent Workflow And Journal Fit Design

## Goal

Make Qiongli support a strict end-to-end academic paper workflow that can move
from topic selection to broad evidence search, data and methods, manuscript
writing, review, strong judging, journal recommendation, and revision feedback
without losing stage state or drifting away from locked decisions.

This becomes the next priority roadmap slice before additional subject
expansion work. The reason is simple: subject expansion, feedback-aware routing,
and marketplace readiness should be validated against a full-cycle harness
rather than only against isolated stage tests.

The first implementation should be a thin vertical slice. It should prove the
control surface, evidence handoff, judge gates, and reverse journal-fit
recommendation path. It does not need to generate a complete publishable paper
in one run.

## Current Context

The repository already has most of the building blocks:

- The canonical workflow contract defines stages `A`, `B`, `C`, `D`, `E`,
  `F`, `G`, `J`, `H`, `I`, and `K`.
- `/paper` routes users to individual task IDs such as `A1`, `B1`, `C1`,
  `F3`, `G3`, `J4`, `H3`, and `H4`.
- Stage handoff rules require `context/stage_handoff.md` with completed
  artifacts, decisions, unresolved questions, evidence dependencies, and
  revisit triggers.
- `context_package` already distributes boundary review and writing harness
  data to multiple agents.
- `qiongli_task_run` is preview-first and can run local agents only when
  explicitly requested.
- Literature provider work now separates discovery, full-text candidates,
  Zotero attachment verification, and evidence limits.
- `venue-analyzer` supports target-venue-first analysis early in Stage A.
- Stage H already contains submission packaging, peer-review simulation, and
  fatal flaw detection.
- Venue profiles exist for core and subject-specific venues.

The gap is orchestration quality. Qiongli can perform many pieces, but it does
not yet have a release-grade full-cycle harness that proves stage artifacts,
agent roles, judge decisions, and journal recommendations remain aligned across
multiple rounds.

## Product Decision

Create a new full-cycle workflow line and merge it into the existing roadmap as
the next priority before the remaining subject expansion stages.

The design has two linked capabilities:

1. Full-cycle multi-agent workflow harness
2. Reverse journal-fit recommendation from an existing manuscript

The full-cycle workflow owns the long-running state machine. Reverse journal fit
is a required checkpoint near submission, but it can also be invoked directly
for an already written manuscript.

## Non-Goals

- Do not replace `/paper` as the single-task router.
- Do not make one command silently run a complete paper project by default.
- Do not launch real local agents unless `run_agents=true` and the existing
  runtime safety checks pass.
- Do not claim exhaustive literature coverage only because metadata search ran.
- Do not recommend a journal from title or abstract alone when manuscript,
  methods, evidence, or claim maps are missing.
- Do not recommend a top venue based only on impact, topical salience, or user
  preference.
- Do not bypass the existing workflow contract, stage handoff contract, or
  claim-evidence ledger.
- Do not fully implement new subject activation in this slice.

## Product Model

### Existing single-task path

Users can still run focused tasks:

```text
/paper topic
/lit-review topic
/paper-write topic
/submission-prep topic
```

Those commands remain useful for local work.

### New full-cycle path

The new path plans and checks a whole paper lifecycle:

```text
/paper-lifecycle topic
```

The first version should support preview mode as the default. It produces a
lifecycle plan, stage gate report, required artifact list, and next-action
recommendations without launching agents.

Optional local-agent execution remains explicit:

```json
{
  "run_agents": true,
  "execution_mode": "triad",
  "controller": "codex",
  "primary": "claude",
  "reviewer": "antigravity"
}
```

### Direct reverse journal-fit path

Users with an existing manuscript can ask:

```text
Recommend the best journal for this manuscript.
```

The system should route this to reverse journal fit, not to early-stage
venue-analyzer, unless the user is still choosing a target before writing.

## Full-Cycle State Machine

The full-cycle workflow is a state machine over existing stages. It should not
invent new academic stages when an existing stage can hold the work.

Required lifecycle checkpoints:

| Checkpoint | Main stages | Required proof |
|---|---|---|
| Idea lock | `A1`, `A2`, `A4`, `A5` | research question, contribution, boundary review, initial venue assumptions |
| Evidence base | `B1`, `B2`, `B3`, `B6` | search plan, search log, dedup log, retrieval manifest, literature map |
| Design and data | `C1`, `C3`, `C4`, optional `I3-I8` | study design, variable or construct spec, data plan, analysis or reproducibility status |
| Manuscript build | `F1-F6` | outline, draft, claim-evidence map, figures or tables plan |
| Compliance and proofread | `G1-G4`, `J1-J4` | reporting checklist, cross-section integrity, tone/proofread reports |
| Strong judge | `H3`, `H4` | peer review simulation and fatal flaw analysis |
| Journal fit | new `H5` | ranked journal fit report with evidence-based recommendation |
| Feedback loop | `H2`, `H2_5` | response matrix, revision plan, reviewer empathy check |

The state machine should expose these statuses:

- `not_started`
- `blocked_missing_artifact`
- `blocked_unresolved_boundary`
- `ready_for_agent`
- `ready_for_human_review`
- `passed`
- `reopened_by_revisit_trigger`

## Stage Handoff And Drift Control

Every stage transition must read and update:

- `context/research_state.md`
- `context/decision_log.md`
- `context/boundary_review.md`
- `context/stage_handoff.md`
- `evidence/claim-evidence-ledger.csv`

The full-cycle harness must block or warn when:

- a downstream task changes the research question without a decision log entry,
- a manuscript claim has no evidence ledger row,
- a methods or data claim exceeds the stage `C/I` evidence status,
- a journal recommendation ignores current manuscript limits,
- a strong judge reports a fatal flaw but the lifecycle status still says
  submission-ready,
- a revision promise is made without a source artifact or feasibility note,
- an agent output omits the locked non-goals or claim-strength boundary.

## Multi-Agent Roles

The first full-cycle workflow should support these logical roles:

- `controller`: builds the lifecycle plan, assigns tasks, and resolves stage
  status.
- `evidence_agent`: checks literature, full-text status, Zotero verification,
  and evidence ledger coverage.
- `methods_agent`: checks design, data, analysis plan, and reproducibility
  commitments.
- `writer_agent`: drafts or revises manuscript text under the writing harness.
- `review_agent`: reviews cross-section integrity, compliance, and reviewer
  risks.
- `strong_judge`: cannot draft text; it can only block, pass, or request
  revision with evidence-backed reasons.
- `journal_fit_agent`: ranks journals from manuscript and venue-profile
  evidence.

These are roles, not mandatory separate model processes. In solo mode, Qiongli
can execute them sequentially with role gates. In duo or triad mode, the context
package must make each role's allowed actions explicit.

## Strong Judge Contract

The strong judge is a gate, not a coauthor.

Allowed outputs:

- `pass`
- `revise`
- `block_submission`
- `reopen_stage`

Required fields:

- decision
- severity
- affected claim or artifact
- evidence basis
- missing artifact if any
- required revision
- stage to reopen if blocked
- whether journal recommendation remains valid

The strong judge must not:

- rewrite the manuscript directly,
- invent reviewer comments,
- recommend a venue without reading the journal fit report,
- override a locked decision without a revisit trigger.

## Reverse Journal-Fit Recommendation

### Problem

Current venue support is mostly target-first. A user can name a target venue,
and Qiongli can analyze fit and formatting implications. The missing path is
manuscript-first: read an existing draft and recommend the best venues.

### New task

Add a canonical Stage H task:

```text
H5: Reverse journal-fit recommendation
```

Primary output:

```text
submission/journal_fit_recommendation.md
```

Optional machine-readable output:

```text
submission/journal_fit_recommendation.json
```

### Required inputs

The recommender should require enough evidence to avoid shallow ranking:

- manuscript draft or structured manuscript sections,
- title and abstract if available,
- research question and contribution statement,
- methods, data, or evidence design summary,
- claim-evidence map,
- reporting or compliance status if available,
- current limitations or fatal flaw report if available,
- candidate subject or discipline,
- venue profile catalog.

If the manuscript is incomplete, the recommender should return a blocked or
partial report. It may suggest what to collect next, but it must not claim a
best journal.

### Ranking dimensions

Each candidate journal should be scored and explained on:

- scope fit,
- contribution fit,
- method and evidence fit,
- article type fit,
- audience fit,
- reporting and data-policy fit,
- manuscript maturity,
- reviewer-risk fit,
- desk-reject risk,
- required revision before submission,
- stretch or safe positioning.

### Recommendation classes

The ranked report should include:

- `primary`: best realistic fit,
- `stretch`: attractive but high-risk venue,
- `safe`: good fit with lower positioning risk,
- `fallback`: useful if top options are blocked,
- `do_not_submit`: venues where current manuscript is a poor fit.

The recommender must explain why a higher-status journal is not primary when
the manuscript evidence does not support it.

## Harness Modes

### Preview harness

Default mode. It reads fixtures and project artifacts, then reports stage and
journal-fit gate status. It must not call external providers or local agents.

### Provider-connected harness

Optional mode. It can call literature provider tools to verify search plan,
coverage diagnostics, and full-text candidate status. It still must not launch
writing agents unless explicitly requested.

### Local-agent harness

Opt-in mode. It can run a bounded lifecycle slice with local agents after the
same safety model as `qiongli_task_run`.

### Release harness

Release checks should start with deterministic preview fixtures. Provider and
local-agent variants should be maintainer opt-in until they are reliable on CI
or a controlled runner.

## Harness Report Schema

The full-cycle harness should produce a JSON report:

```json
{
  "schema_version": "1.0",
  "mode": "preview",
  "topic": "demo-paper",
  "paper_type": "empirical",
  "lifecycle_status": "blocked_missing_artifact",
  "stage_gates": [
    {
      "stage": "B",
      "status": "passed",
      "required_artifacts": ["search_strategy.md", "search_log.md"],
      "missing_artifacts": [],
      "warnings": []
    }
  ],
  "drift_checks": {
    "locked_question_preserved": true,
    "claim_evidence_coverage": "partial",
    "unresolved_judge_blocks": 1
  },
  "journal_fit": {
    "status": "partial",
    "primary": null,
    "blocking_reasons": ["missing claim-evidence map"]
  },
  "recommended_next_tasks": ["F4", "H4", "H5"]
}
```

Markdown reports should be concise and point to the exact missing artifact or
stage to reopen.

## Fixture Strategy

The first implementation should add deterministic fixtures for:

- clean full-cycle preview path,
- missing literature coverage,
- manuscript drift from locked research question,
- unsupported claim in manuscript,
- fatal flaw unresolved before submission,
- journal overreach where a top venue is not justified,
- incomplete manuscript where journal fit must block,
- revision feedback reopening an earlier stage.

Fixtures should use small text files and static venue profiles. They should not
depend on external provider calls or model output.

## Roadmap Integration

The adaptive subject roadmap should be reconciled as follows:

1. Mark the local-agent smoke and subject-gate foundations as at least
   partially implemented on `dev`.
2. Insert the full-cycle workflow harness as the next priority stage.
3. Treat reverse journal fit as a required subfeature of that stage.
4. Resume subject expansion after the full-cycle harness can test stage
   handoff, drift prevention, and journal-fit gates.
5. Keep feedback-aware subject routing and marketplace/read-only fallback as
   downstream work, but make them report into the full-cycle harness once they
   are implemented.

## Success Criteria

- A preview full-cycle harness can evaluate a sample project without launching
  agents.
- The harness blocks when required stage artifacts are missing.
- The harness detects manuscript drift from a locked research question.
- The harness reports unsupported manuscript claims from the claim-evidence
  ledger.
- The reverse journal-fit recommender ranks at least three candidate journals
  from venue profile evidence.
- The recommender blocks best-journal claims when manuscript evidence is too
  thin.
- Strong judge findings can reopen a specific earlier stage.
- `/paper` remains backward compatible as a single-task router.
- Generated docs and workflow packages expose the new path consistently.

## Risks And Mitigations

- Scope creep: start with preview harness and one empirical fixture.
- False confidence: block best-journal recommendation when required manuscript
  evidence is absent.
- Agent drift: use context package role constraints and judge gates.
- Overly rigid workflow: allow stage reopen with explicit revisit triggers.
- Provider instability: keep provider-connected harness optional at first.
- Subject expansion coupling: keep subject-specific logic behind existing
  subject contracts and venue profiles.

## Rollout

1. Documentation and workflow contract update.
2. Deterministic lifecycle harness preview.
3. Reverse journal-fit recommender over local venue profiles.
4. MCP/CLI exposure for preview-only calls.
5. Optional local-agent lifecycle slice.
6. Release-readiness integration after preview fixtures stabilize.
