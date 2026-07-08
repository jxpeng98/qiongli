---
id: supervisor-feedback-integrator
stage: M_dissertation
description: "Map supervisor feedback to dissertation chapters, claims, required actions, evidence needs, priorities, and revision status."
inputs:
  - type: DissertationChapterMap
    description: "Current chapter map and status"
  - type: ReviewComments
    description: "Supervisor feedback or committee comments supplied by the user"
outputs:
  - type: DissertationFeedbackLog
    artifact: "dissertation/supervisor_feedback_log.md"
constraints:
  - "Must preserve supervisor meaning without inventing approval"
  - "Must separate feedback interpretation from revision commitment"
  - "Must link each action to affected chapter or claim"
failure_modes:
  - "Turns supervisor comments into generic advice"
  - "Claims feedback has been resolved without evidence"
  - "Invents supervisor intent"
tools: [filesystem]
tags: [dissertation, supervisor-feedback, revision, feedback-log]
domain_aware: true
---

# Supervisor Feedback Integrator Skill

Turn supplied supervisor or committee feedback into a traceable dissertation revision log.

## Purpose

Write `dissertation/supervisor_feedback_log.md` and support `dissertation/revision_plan.md` without inventing feedback or implying approval.

## When to Use

- When the user supplies supervisor comments.
- Before revising chapters based on feedback.
- When feedback needs prioritization and evidence mapping.

## Inputs

- `DissertationChapterMap`
- `ReviewComments`
- Optional draft chapters and current revision plan.

## Process

1. Extract each feedback item.
2. Link it to affected chapter, section, claim, method, or evidence.
3. Assign required action, priority, evidence needed, and status.
4. Record ambiguity and questions for the supervisor.

## Output Contract

Write `RESEARCH/[topic]/dissertation/supervisor_feedback_log.md`; update `RESEARCH/[topic]/dissertation/revision_plan.md` when the revision sequence is clear.

## Quality Bar

- [ ] Feedback meaning is preserved.
- [ ] Each action has an affected artifact.
- [ ] Ambiguous comments remain marked.
- [ ] No supervisor approval is implied.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Inventing intent | Misrepresents supervisor | Mark ambiguity |
| No artifact link | Hard to execute | Attach each item to chapter/claim |
| Premature "resolved" | False status | Require evidence of revision |
