---
description: "Draft or revise a research paragraph, section, or proposal while preserving its evidence and claim limits."
---

# Academic Writing

Draft or revise the requested section using the supplied evidence and constraints.
Canonical Task ID: `F2` single-section writing. A direct paragraph edit can stay
in chat; it does not require a project or claim completion of a formal F2 run.

## Request

$ARGUMENTS

## Scope and inputs

Reuse the requested section, audience, word limit, citation style and available
material. Ask only when missing information changes a central claim or prevents
the requested result. A missing optional field does not start an interview.

For coursework or dissertation requests, preserve the rubric, evidence and
integrity boundaries. Read `/coursework` or `/dissertation` only for unresolved
requirements or an explicitly requested L/M task. Use `/paper-write` for a full
manuscript and `proposal-writer` for a research proposal or 开题报告.

## Writing contract

Read the shared Writing Harness Contract in `references/stage-F-writing.md`.
It owns scope, Story Spine, drafting granularity, review and completion. Use
`manuscript-architect` when the task needs manuscript structure or a formal
claim-evidence map; its full-paper outputs do not become paragraph prerequisites.
During drafting and after substantive edits, follow its manuscript-first trace
route in `references/evidence-verification.md`. Detect claims in the actual text,
including added explanations, background and notes; inspect matching original
evidence and recheck the final wording. Check paragraph depth and continuity
separately from source support. Existing citations or a completed note are not
enough to certify the new prose. Apply the shared readability check to every
section: the final text must explain its evidence and connections in clear,
concise language, preserving necessary detail and technical precision.

Select evidence checks for the section's purpose:

| Section | Evidence boundary |
|---|---|
| Title / abstract | Represent the supplied work and findings; follow the requested venue or word limit |
| Introduction | Support source-dependent background and positioning; no invented citations or novelty claims |
| Literature review / related work | Apply the shared contract's literature-review guidance: compare inspected evidence and explain its consequences for this paper's question or choices |
| Methods | Describe reported procedures; leave missing sampling, measurement and ethics details unknown |
| Results | Preserve supplied numbers and uncertainty; do not infer significance, causality or an unreported analysis |
| Discussion / limitations | Separate supported interpretation, alternatives and hypotheses; keep inferential limits visible |
| Proposal / opening report | Describe planned work as planned; identify missing evidence, access or institutional requirements |

Use `boundary-interviewer` for a consequential unresolved claim, purpose or
evidence threshold. Reuse settled decisions; a narrower supported draft can
proceed while an unsupported extension remains a gap. In project work, propose
changed central-claim decisions for `context/decision_log.md` or
`academic-context-maintainer` through the existing write owner.

## Completion

Return the requested text in the requested format. Add only necessary evidence
gaps or consequential editorial notes; do not attach a generic outline, citation
table, improvement checklist or extra title candidates to every response.
Apply the shared Writing Harness's length check to the final prose; source-anchor
citations do not make an undersized paragraph satisfy its requested length.
Use citation placeholders only when a needed source is absent, clearly marking
them as unresolved. Preserve the author's meaning, claim strength and voice.

For formal F2 persistence, update the selected section of
`manuscript/manuscript.md` using the canonical contract and preview/approval/CAS.
Retain surrounding user text and stable claim IDs. A proposed draft is not a
verified write or a submission-ready manuscript.
