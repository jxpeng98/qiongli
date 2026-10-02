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

## Native Save and Resume

For a registered project with a pending, reviewable capture, the CLI supports
`project capture consolidate preview --project-id <prj_id> --capture-id <cap_id>
--stage-handoff-file <absolute-draft.md>`. Supply a new handoff entry with the
sections above, stable IDs, source anchors and explicit evidence limits. Do not
pass the whole existing handoff back as the new entry or infer its contents from
the capture summary. The native owner checks bounded UTF-8 data, not scholarly
completeness; review the actual draft before approval.

Review the returned `stageHandoffContent` and artifact deltas. The complete
result preserves existing bytes and appends a capture-marked entry. Apply through
the same `project capture consolidate apply` command with the same draft,
`--reviewed-at-unix`, `--expected-plan-digest`, and the existing academic-review
and filesystem-write approvals. These flags express actual authorization; a
Skill, file or receipt cannot supply it. A changed draft, project revision or
registered semantic input requires a fresh review. Never retry by dropping the
handoff flag or restoring stale source bytes. An already consolidated capture
cannot be reused to append a handoff; create a reviewed capture at the current
revision for genuinely new work.

After restart, read the current registered `research_state` and `stage_handoff`
through the existing revision-bound readers. Read the latest available stage
summary separately, checking its own source bytes and dependencies.
The last capture-marked handoff is the latest appended entry, not permission to
ignore earlier unresolved dependencies. A changed source reopens affected
conclusions. Non-registered sources and stage summaries still require their own
source-byte checks; native revision checks do not cover arbitrary attachments.

This save records a handoff together with ordinary capture consolidation. It
does not advance the stage, create a stage-summary document, rebuild the Graph,
or establish research-quality acceptance. The optional handoff is a CLI path;
there is no new MCP save tool. If the available Host lacks this CLI capability,
report that limitation and retain the candidate for review without bypassing the
project write owner. Saving alone never authorizes deletion.
