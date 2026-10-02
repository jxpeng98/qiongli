---
id: self-critique
stage: Z_cross_cutting
description: "Review research artifacts for concrete claim, evidence and method defects; revise affected work and stop when the applicable checks are satisfied."
inputs:
  - type: AnyArtifact
    description: "Any output requiring quality assurance"
outputs:
  - type: CritiqueLog
    artifact: "review/self_critique_log.md"
constraints:
  - "Must check the applicable evidence and output contract without inventing critique issues"
  - "Must preserve unresolved blockers and stop when the applicable review contract is satisfied or progress needs unavailable evidence"
failure_modes:
  - "Circular critique without convergence"
  - "Inability to identify own systematic biases"
tools: [filesystem]
tags: [cross-cutting, critique, red-team, quality-assurance, Socratic]
domain_aware: false
---

# Self-Critique Skill

Review the requested research output for substantive defects without expanding
the assignment into an unrequested debate or full-stage audit.

## Purpose

Check claim-evidence alignment, methodological validity and the requested output
contract. Use adversarial review when requested or required; routine quality
checking does not require a reviewer persona or additional model.

## Inputs

- `AnyArtifact`: Any output requiring quality assurance
- If a required input is missing, report the affected check as incomplete. For project work, propose the gap for `context/gap_notes.md` through the existing write owner; a direct review can report it in chat.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

1. Identify the requested artifact, relevant stage and applicable review contract.
   Reuse the current draft and prior findings; do not generate a new draft just
   to review an existing one.
2. Check the source evidence and only the relevant stage questions below. They
   are lenses, not a checklist to exhaust or a quota of questions to invent.
   For a claim-to-source audit, use `references/evidence-verification.md` to
   distinguish unsupported inference, contradictory evidence and unverified scope.
3. For an ordinary check, review once. Fix concrete defects and verify the
   affected claims or outputs. Stop when those checks pass; repeat a broader
   review only for changed inputs, new evidence or the formal contract below.
4. Preserve unresolved blockers. If a fix requires unavailable evidence or a
   user decision, report the gap instead of continuing the same debate. A clean
   review may report no findings; missing evidence is not a PASS.
5. In formal runs, keep `review/self_critique_log.md` as the canonical issue
   memory. A direct answer can report its check in chat without creating a log.

## Multi-Round Self-Loop Contract

Review depth follows the requested risk and evidence, not a fixed count attached
to a label such as standard or deep. When a protocol, user instruction or saved
run supplies minimum passes, consecutive passes, revision limits or independent
review requirements, honor them. Do not silently reduce an existing run's limits.

- Review the current artifact against its applicable checks. Revise concrete
  defects, then verify the affected work; do not manufacture a revision to
  justify another round.
- If an explicit minimum remains after a `PASS`, complete the required stability
  review of the same draft. Report only reviews actually performed.
- A required independent review cannot be replaced by role-play or another pass
  in the same conversation. Report unavailable reviewers as an unmet requirement.
- A `BLOCK` verdict remains blocking regardless of confidence. Reaching a review
  or resource limit with unresolved issues means blocked or incomplete.
- Carry unresolved issues forward into the next round.
- Mark each issue as `open`, `partial`, `resolved`, or `superseded`.
- Reuse existing issue IDs when the same problem persists.
- Add new issue IDs only for genuinely new failures introduced by the revision.
- Record the artifact path, section, evidence basis, required fix, and round-to-round status change.

Use `review/self_critique_log.md` as a compact register like:

| Issue ID | Opened In Round | Status | Severity | Artifact / Section | Critique | Required Fix | Resolution Evidence |
|---|---|---|---|---|---|---|---|
| SC-01 | 0 | open | high | `manuscript/manuscript.md` / Discussion | Claim exceeds results evidence | Narrow causal language | Pending |

## Stage-Aware Grill Contract

When a task enters self-critique through a Qiongli grill request, use the same
light automatic grill and deep grill distinction as `boundary-interviewer`.

- Light automatic grill applies when the user is unsure, a stage starts with
  vague scope, a handoff contains open risks, or a claim/method/evidence/code
  decision changes.
- Deep grill applies when the user explicitly asks to be grilled, stress-tested,
  challenged like Reviewer 2, or checked for fatal flaws.
- Every critique question must target the current stage lens, include a
  recommended answer or required fix, and record whether the issue is open,
  partial, resolved, or superseded.
- A deep grill may reopen prior-stage issues only when it records the evidence or
  user decision that triggered the revisit.

## Cross-Stage Grill Memory

Self-critique issues are part of the cross-stage grill memory. For a project review, inspect the relevant existing records before starting a
new critique loop:

- `context/boundary_review.md`
- `context/decision_log.md`
- `context/stage_handoff.md`
- `review/self_critique_log.md`

If a prior issue affects the current artifact, keep the same issue ID and update
its status instead of creating a duplicate. At a formal handoff, open issues that cannot be resolved in
the current stage must be carried through the approved write owner into `context/stage_handoff.md` under `Open
Grill Issues` with a concrete `Revisit Trigger`.

## Stage-specific checks

Use the relevant stage's existing decision and evidence checks; do not maintain
another question bank here or load every stage. Apply the selected subject's
`references/discipline-guidance.md` overlay when it changes the methodological
judgment. These checks do not impose a fixed rival-hypothesis count, numeric
quality score, universal power analysis or one qualitative validity criterion.

| Current artifact | Existing check owner |
|---|---|
| Question, gap, concepts | `references/stage-A-framing.md` |
| Search, screening, paper reading | `references/stage-B-literature.md` |
| Design, estimand, measurement, sample | `references/stage-C-design.md` |
| Consent, participant risk, data access | `references/stage-D-ethics.md` |
| Synthesis, dependence, contradictory findings | `references/stage-E-synthesis.md` |
| Manuscript claims and evidence | `references/stage-F-writing.md` |
| Reporting requirements and integrity | `references/stage-G-compliance.md` |
| Submission and reviewer responses | `references/stage-H-submission.md` |
| Analysis code and reproducibility | `references/stage-I-code.md` |
| Meaning-preserving language edits | `references/stage-J-proofread.md` |
| Talk claims, figures and caveats | `references/stage-K-presentation.md` |
| Coursework against supplied requirements | `references/stage-L-coursework.md` |
| Dissertation chapters and supervisor decisions | `references/stage-M-dissertation.md` |

A capability mapping or reviewer role is routing guidance, not proof that a
review ran. Use the actual Host execution and the applicable review contract.

## Output Contract

- `CritiqueLog`: for formal persistence, update `RESEARCH/[topic]/review/self_critique_log.md` through preview/approval/CAS; a chat review need not create a file.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

- [ ] 已完成当前任务适用的检查；保留协议或已有运行明确配置的复核轮数和独立审查要求
- [ ] 每个 critique 点附带具体修正建议
- [ ] 每轮保留并更新 issue lineage，而不是把 critique 重置
- [ ] Overclaiming 已被识别并降级表述
- [ ] 自相矛盾点已消解或标注为 limitation
- [ ] Critique log 记录了改进前后的对比与 issue 状态迁移

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 走过场 | Critique 只说整体不错 | 说明检查了哪些主张与证据；只报告实际问题，不凑数量 |
| 过度自我批评 | 导致不敢下结论 | 区分 fatal flaw vs. minor improvement |
| 只关注表面 | 挑错别字不挑逻辑 | 按逻辑 → 证据 → 表述优先级 |
| 无 action | 批判完但不修改 | 每条 critique 必须附带 action item |
| Critique 同质化 | 相同输入反复发现同一问题 | 复用 issue ID；缺少新证据或可行修复时说明阻塞并停止 |

## When to Use

- 需要主动提高 red-teaming 强度时
- 产出可能存在浅层推理或过度主张时
- 投稿前做最后一轮逻辑检查时
- 需要 Socratic questioning 来压力测试结论时
