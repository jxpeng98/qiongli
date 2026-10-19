---
id: human-voice-rewriter
stage: J_proofread
description: "Revise stiff, translated or generic scholarly prose into natural English or Chinese while preserving meaning, evidence and author voice."
inputs:
  - type: Manuscript
    description: "Current manuscript text"
  - type: AIDetectionReport
    description: "Flagged passages from ai-fingerprint-scanner"
outputs:
  - type: HumanizedManuscript
    artifact: "proofread/humanized_manuscript.md"
constraints:
  - "Must not alter statistical values, citations, or technical terminology"
  - "Must preserve claim strength and evidence links"
  - "Must change only what improves the requested text; correct passages may remain unchanged"
failure_modes:
  - "Meaning drift from overly aggressive rewriting"
  - "Introducing factual errors during paraphrasing"
  - "Creating new AI patterns by using a single rewriting template"
tools: [filesystem]
tags: [proofread, humanization, rewriting, voice, style]
domain_aware: false
---

# Human-Voice Rewriter Skill

Revise scholarly expression without changing the research or claiming to establish authorship.

## Purpose

Resolve concrete expression problems using the smallest faithful edit. Do not
add invented evidence, personal stance, deliberate mistakes or detector-oriented
changes. Keep required AI disclosure; the legacy task/output names are retained.

## When to Use

- When the user asks to improve scholarly expression or author voice
- When actual language issues were identified in J1 or supplied feedback
- For requests described as humanizer, Harmonizer, naturalizing or removing
  translationese; choose faithful expression, never detector evasion
- Use J4 for grammar-only corrections and source comparison for attribution issues
- When an original and edited passage are supplied for meaning review, compare
  them directly; do not start another language rewrite

## Related Task IDs

- `J2` (human-voice rewrite)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/humanized_manuscript.md`

## Inputs

- Requested passage or manuscript; any existing J1 findings or supplied feedback
- For meaning review, the original and edited versions at the same scope;
  request a missing version rather than claim to have compared it
- A direct edit needs only the relevant text and constraints; return it in chat
  without starting J1 or requiring a project. Formal J2 runs retain the declared
  inputs, full manuscript artifact and applicable gates below.
- Ask only when missing text or context prevents a faithful edit. For a formal
  project run, record required missing artifacts in `context/gap_notes.md`
  through the authorized write path; a direct edit needs no gap-note file.

## Process

### Step 1: Set Scope and Voice

Prioritize actual meaning/readability problems and the user's selected passages.
Verify supplied flags rather than treating their severity as a command to rewrite.
Correct text may remain unchanged. Follow the user's voice sample, then the
draft's voice, then disciplinary and venue conventions. Preserve the source's
English variety or Chinese script unless a change is requested. Default to
sentence and within-paragraph edits; restructure sections only within requested
scope. For learning-oriented requests, explain targeted edits instead of silently
replacing the author's reasoning.

### Step 2: Edit Language at the Requested Depth

Read `references/scholarly-voice.md` for the output language. Preserve meaning
first, make it idiomatic second, polish only where useful (信、达、雅). Identify
the paragraph's point and how each sentence supports, qualifies or develops it.
Clarify referents and information order before adding transitions. A smoother
sentence must not introduce a cause, contrast or premise absent from the source.
Flag ambiguity rather than inventing a bridge. Keep necessary uncertainty,
technical terms and author stance. No word blacklist or rhythm quota applies.
Follow its language-editor responsibilities: the smallest useful edit may
reorder, combine or split sentences within scope. Do not preserve awkward
syntax just to minimize the number of changed words. Remove empty qualifications
without changing meaningful uncertainty. In a meaning-review-only request,
skip this editing pass and compare the supplied versions in Step 4.

### Step 3: Read the Edited Passage Continuously

Check whether each sentence develops the paragraph's point with natural phrasing
and clear referents. Repair concrete expression problems within the requested
scope; do not add a missing scientific premise to make the text flow.

### Step 4: Review Meaning Against the Original

Use the meaning-review responsibilities in `references/scholarly-voice.md`.
For every rewritten passage, compare:

| Integrity Check | ✓/✗ |
|----------------|-----|
| Statistical values unchanged | |
| Citations remain correctly attributed | |
| Technical terminology preserved | |
| Negation, quantifiers, units and population/time scope preserved | |
| Claim strength not weakened or inflated | |
| Causal language matches original intent | |
| No evidence omitted or added | |

Report actual drift with original/edited spans and its consequence; repair the
affected wording and recheck that change. Leave sound edits intact, including
direct statements of observed results. No drift is a valid finding. Do not
restore deleted boilerplate or add uncertainty absent from the original meaning.
This comparison preserves meaning; source-support verification remains a separate
task. Stop when the requested issues are resolved. Use an independent reviewer
only when required and actually available through the active Host; follow
`skills/Z_cross_cutting/model-collaborator.md` and report unavailable requirements.

### Step 5: Return the Requested Output

Return only the revision for a direct polish, adding a brief note for a material
uncertainty or when requested. A meaning-review-only request returns findings
and any minimal repairs, not another full rewrite. For a formal J2 run, integrate
changes into the full manuscript and record them in its change log.

## Output Contract

- `HumanizedManuscript`: write `RESEARCH/[topic]/proofread/humanized_manuscript.md`.
- Preserve the distinction between finding, interpretation and implication;
  do not impose new headings on a bounded passage or fixed manuscript structure.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Expression-only edits retain claim IDs, source links and the existing evidence
  ledger. Do not rebuild the Research Graph or revise claims as a side effect of
  polishing. If a substantive claim change is separately requested, follow
  `references/evidence-ledger-contract.md` and
  `references/academic-graph-continuity.md` within its approved scope.
- For that substantive change, supported claims need source pointers; unsupported
  central claims become `gap_note` rows under the evidence-ledger contract.
- Apply `references/citation-risk-policy.md` when citation risk is material.
  Report it in chat for a direct edit; formal project work records it in
  `proofread/citation-risk-report.md` through the authorized write path.

## Quality Bar

The humanized manuscript is **ready** when:

- [ ] All substantive issues in the requested scope are resolved or reported as gaps
- [ ] Every edit improves a demonstrated issue; no strategy quota applies
- [ ] Every edited passage compared with the original for meaning drift
- [ ] Meaning, author voice and required disclosure are preserved
- [ ] Formal J2 output includes the full manuscript; direct edits respect requested scope
- [ ] Formal J2 change log records rewrites; direct polish has only the requested output

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Using one rewrite template for all passages | Ignores the actual language issue | Choose the smallest edit that helps this passage |
| Over-casualizing academic prose | Sounds unprofessional | Keep formality level appropriate to venue |
| Changing "significant" to a synonym without context | May alter statistical meaning | Distinguish statistical "significant" from colloquial use |
| Rewriting Methods section too aggressively | Methods should be precise and reproducible | Lighter touch in Methods — focus on transitions, not procedures |
| Not verifying evidence preservation | Meaning drift goes undetected | Mandatory diff-check per passage |

## Output Template

```markdown
---
task_id: J2
template_type: humanized_manuscript
topic: <topic>
primary_artifact: proofread/humanized_manuscript.md
---

# Humanized Manuscript

[Full revised manuscript text with all J2 rewrites integrated]

---

## J2 Change Log

| # | Section | Original Excerpt | Rewritten Excerpt | Strategy Used | Accuracy Check |
|---|---------|-----------------|-------------------|---------------|---------------|
| 1 | Results ¶2 | "We conducted an examination of the association." | "We examined the association." | Replace a noun-heavy phrase; retain association | Same claim and scope |
```
