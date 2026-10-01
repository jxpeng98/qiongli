# Stage Handoff Contract

Stage handoffs preserve what a downstream stage inherits and what remains uncertain.

## Canonical Path

- `RESEARCH/[topic]/context/stage_handoff.md`

## Required Sections

- `Completed Artifacts`
- `Decision Summary`
- `Resolved Grill Decisions`
- `Unresolved Questions`
- `Open Grill Issues`
- `Evidence Dependencies`
- `Assumptions Passed Forward`
- `Risks For Next Stage`
- `Revisit Triggers`
- `Recommended Next Tasks`

## Rules

- Link every completed artifact using a concrete `RESEARCH/[topic]/...` path or project-relative path.
- Record unresolved questions explicitly; write `None` only when the stage has been checked.
- Record `Resolved Grill Decisions` when a boundary interview, stage-aware grill, or self-critique loop closed a question that affects downstream scope, claim strength, methods, evidence thresholds, code, submission, or presentation.
- Record `Open Grill Issues` when a light automatic grill or deep grill found a risk that cannot be resolved in the current stage.
- Evidence dependencies should point to the evidence ledger, bibliography, analysis output, or gap note.
- `Revisit Triggers` must state what new evidence, user decision, reviewer comment, diagnostic failure, or analysis result would reopen a resolved decision.
- Do not treat a stage as ready for downstream work when inherited assumptions are hidden.
- Carry the three-question decisions from `references/academic-output-rubric.md`
  into existing sections: decision/claim IDs and rationale in `Decision Summary`,
  current source/output anchors in `Evidence Dependencies`, and missing premises
  in `Unresolved Questions`. Under `Recommended Next Tasks`, distinguish what can
  proceed from the action awaiting a named input/check. Use `Revisit Triggers` for
  the evidence change that would reopen the decision. No extra handoff schema is
  needed; preserve earlier entries and completed work.
- Before resuming a dependent task, compare its source/result revision with the
  basis of the inherited decision. A changed input requires rechecking affected
  conclusions and naming stale downstream artifacts; unrelated work can continue.
- When a saved stage consolidation is requested, follow
  `references/stage-consolidation.md` and link its versioned document from the
  handoff. Preserve completed work, earlier summaries and source dependencies;
  stage handoff never authorizes deletion. File selection and removal belong to
  the user alone.
