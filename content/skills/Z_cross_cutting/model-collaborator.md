---
id: model-collaborator
stage: Z_cross_cutting
description: "Delegate bounded research work to available subagents or exchange source-bound review and edit proposals with another Host; reconcile results and disagreements."
inputs:
  - type: TaskPacket
    description: "Task specification for multi-agent research execution and cross-review"
outputs:
  - type: CollaborationTrace
    artifact: "logs/model_collab_trace.md"
constraints:
  - "Must ensure independent execution before synthesis"
  - "Must document agent disagreements explicitly"
failure_modes:
  - "Agent unavailable for scheduled collaboration"
  - "Output format incompatible across agents"
tools: [filesystem]
tags: [cross-cutting, multi-agent, collaboration, independent-review]
domain_aware: false
---

# Model Collaborator Skill

## Purpose

Coordinate bounded independent review of literature, writing, qualitative coding,
statistics, analysis code or rebuttals. Match roles to the actual available
capabilities and evidence needs, not to model or Host brand stereotypes.

## When to Use

Use for requested independent review or a research task whose gate requires it.
Also use when the user requests parallel work, another Agent's review, or a
cross-Host edit handoff. Delegate through actual available Host tools for these
requests when a bounded independent task is useful; a named role alone cannot
execute work. Do not install a runtime or change models to create a collaborator.
A quick edit or ordinary reading task normally needs one agent. Multiple roles
in one conversation are useful self-review but are not independent execution.

## Inputs

- `TaskPacket`: objective, Task ID, source artifacts, constraints and output path.
- Available Host/tool capabilities and any required reviewer independence.
- If sources, permissions or an independent reviewer are missing, record a gap note
  and complete only the supported portion. Do not invent a second review.

## Process

1. Choose the smallest useful arrangement: independent parallel reviews for
   screening or competing interpretations; draft then review for a manuscript
   or code change; one-agent self-review when independence is not required.
2. Use the user's current model configuration. Do not install another runtime,
   request API keys or replace the configured model to fill a role. Load
   `references/platform-routing.md` for native 2.x execution and recovery.
3. For registered project orchestration, use visible `qiongli_orchestrator_route`
   and follow the returned Full MCP sequence: select project/revision, run
   `qiongli_orchestration_doctor`, start, read evidence, submit the bounded
   candidate, then obtain the next handoff. A Lite preview cannot execute a run.
   The coordinator owns this sequence. A delegated participant returns its
   proposal instead of starting or advancing another run. Before submission,
   the coordinator checks it against authenticated reads from its own MCP
   process; a child's file hash is not an authenticated MCP evidence reference.
4. If authorized native subagents exist, give each a bounded packet with the
   same source revision, question, evidence anchors, output contract and allowed
   actions. Independent first-pass reviewers should not receive the other's
   verdict. A draft-review chain necessarily exposes the draft: describe that
   dependence honestly. Keep candidates isolated; do not permit concurrent
   canonical project writes.
   Use the Host's actual spawn/delegate tool, record its returned task identity,
   and collect the result through its wait/read tools. A queued or running task
   is not completed. Respect the Host's concurrency limit; cancel only the
   delegated work that is no longer needed. Parallel edits use isolated candidate
   files or supported worktrees, with explicit file ownership and an integrator.
5. Compare findings against sources, diagnostic results and the research method.
   Record conflicting claims, evidence for each and the resolution or remaining
   blocker. Majority agreement and high confidence are not evidence; do not
   discard a supported minority objection.
6. Return one synthesis plus source-bound candidate(s). Preserve actual run,
   revision, generation, document/handoff digests and evidence references when
   supplied by Qiongli. Candidate submission does not approve artifact apply.
   If the independent lane is unavailable, report it explicitly and offer a
   bounded self-review without marking its independent-review gate as passed.

### Choose the collaboration strategy

| Need | Arrangement | Completion evidence |
|---|---|---|
| Independent first opinions | Give separate reviewers the same sources and question, without each other's verdict | Separately executed outputs with matching source identities |
| Improve a draft or implementation | Author proposes; reviewer examines the exact candidate; integrator resolves findings | Reviewed candidate bytes and recorded findings/resolutions |
| Independent components | Delegate non-overlapping source groups or candidate files | Each requested result collected, dependencies and overlaps checked |
| Work with another Host | Exchange a bounded handoff and review packet through authorized tools or user transfer | Actual returned result, participant and source/candidate bindings |

No arrangement implies a fixed number of agents or discussion rounds. Continue
when evidence, disagreements or the explicit protocol require it; otherwise
integrate the supported result. A timeout, unavailable peer or cancelled task
leaves a visible gap, not a successful vote. Do not restart an endless debate.

### Cross-Host review and editing

Use `templates/agent-handoff.md` and `templates/agent-review-packet.md`. Include
the task/outcome, source paths and locators, observed revision or file digests,
exact candidate when reviewing a draft, unresolved questions, allowed actions and
return destination. Share only the authorized subset of research content; do not
forward a whole conversation, credentials, approval tokens or unrelated files.

The current native Full MCP run is bound to its Host descriptor. Another Codex
session or Claude Host is not automatically entitled to its active handoff,
authenticated evidence references or in-memory approvals. Do not edit those fields
or impersonate the originating Host to bypass a binding mismatch. For this lane,
keep one coordinator on the originating run; the other Host returns a review or
edit proposal for the supplied source snapshot. A portable packet is not a native
handoff token and does not grant tool access. Automatic cross-Host task claims and
simultaneous canonical writes remain a separate capability.

Use an available, authorized communication tool when present. Otherwise prepare
the exact packet for the user to transfer and report `awaiting external review`;
do not claim the other Host was contacted. When a result returns, verify the task,
source revision/digests and candidate identity against current authorized evidence.
A stale or mismatched response needs reconciliation or another review; never apply
its patch blindly. The coordinator records accepted/rejected findings and applies
only the approved integrated candidate through the existing write owner.

### Carry research state forward

The handoff carries stable claim/decision IDs, citekeys, source anchors, unresolved
method limits and prior summary links. An external suggestion does not turn a
proposed Graph relation into reviewed support. Use `references/academic-graph-continuity.md`
for changed canonical records and `references/stage-consolidation.md` for a stage
close. Preserve old summaries and update the existing history, not a parallel chat
archive. Hook reminders can prompt this check after resume or compaction; they
cannot establish current revision, source coverage, completed review or approval.

## Output Contract

Write `RESEARCH/[topic]/logs/model_collab_trace.md` through the applicable project
write owner. Until persistence is approved, present it as a proposed trace.

The trace records task and sources, actual participants/capabilities, independence
or dependence, findings with source anchors, disagreements, resolutions and
remaining gaps. Use `templates/duo-review-report.md` when its structured review
is needed. Describe concise decision rationale, not hidden reasoning traces.
Do not invent session IDs, evidence hashes, success flags, citations, data,
sample sizes, statistical results or reviewer comments. Separate finding,
interpretation and implication; apply `references/academic-output-rubric.md`.

## Quality Bar

- Every claimed independent review has an actual separately executed output.
- Reviewers worked on the declared source revision and did not share first-pass verdicts.
- Conflicts are resolved using evidence or carried forward as explicit blockers.
- Candidate and approval bindings are preserved; no review result authorizes a write.
- Missing tools or reviewers produce a truthful partial result, not a simulated success.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Assign expertise from a model name | Use the actual task, tools and observed capability |
| Treat sequential personas as independent reviewers | Label the work self-review and keep the unmet gate open |
| Merge by majority or confidence | Resolve against source evidence and methodological validity |
| Reuse stale context after a handoff | Re-read authorized evidence at the current revision |
| Let reviewers overwrite one another | Keep bounded candidates separate until approved integration |
