---
id: tone-normalizer
stage: G_compliance
description: "Review academic tone against the evidence, removing empty qualifiers and repetition while preserving warranted uncertainty and author voice."
inputs:
  - type: Manuscript
    description: "Draft text requiring tone normalization"
outputs:
  - type: ToneNormalization
    artifact: "compliance/tone_normalization.md"
constraints:
  - "Must judge claim strength from meaning and evidence, not hedge counts or word lists"
  - "Must preserve technical precision while improving readability"
  - "Must preserve consequential uncertainty, scope and limitations"
failure_modes:
  - "Over-cutting removes necessary nuance"
  - "Domain-specific hedging conventions not respected"
  - "Author voice completely erased (reads like a machine)"
tools: [filesystem]
tags: [compliance, tone, academic-writing, editing, conciseness]
domain_aware: false
---

# Tone Normalizer Skill

Review whether the prose states supported claims clearly and qualifies actual uncertainty.

## Purpose

Remove empty framing and repeated qualifications without weakening supported
findings. G4 reviews tone and evidence calibration; J2 edits language and checks
the revision against its source. Use `references/scholarly-voice.md` for their
shared editing and meaning-review responsibilities, including bilingual examples.

## Related Task IDs

- `G4` (tone normalization)

## Output (contract path)

- `RESEARCH/[topic]/compliance/tone_normalization.md`

## When to Use

- After manuscript draft is complete (but before final submission)
- After AI-assisted writing (AI text tends to be wordy and pattern-heavy)
- When reviewer feedback says "needs tightening" or "writing is unclear"
- Between R&R rounds to polish revised sections

## Inputs

- `Manuscript`: Draft text requiring tone normalization
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Locate Actual Tone Problems

Inspect the paragraph's claim, supporting material and role in the argument.
Flag a passage only when its meaning or expression causes a concrete problem:

| Issue | Review decision |
|-------|-----------------|
| Unsupported strength or scope | Identify the inference or generalization the evidence cannot support; adding "may" alone does not repair it |
| Empty or repeated qualifications | Remove them when the remaining wording retains the actual uncertainty and scope |
| Vague praise or significance | State the supported criterion or result; a flattering synonym adds no evidence |
| Repetition or awkward framing | Clarify information order and referents; retain logical connectors and needed explanation |

Direct statements of observed results, performed methods or established facts
need no hedge merely to sound academic. Speculation and uncertain inferences
retain meaningful qualifications. Judge neither certainty nor authorship from
particular words, sentence patterns or a count of modal verbs.

### Step 2: Apply Calibrated Rewrites

For each flagged passage, produce a rewrite that:

1. **Preserves meaning in expression edits** — identify any substantive correction separately with its evidence
2. **Calibrates confidence** — match claim strength to evidence strength
3. **Improves flow** — remove repetition or expand needed explanation; no word-reduction target applies
4. **Maintains voice** — don't make everything sound identical

#### Confidence Calibration Guide

| Claim and evidence | Appropriate treatment |
|--------------------|-----------------------|
| Observed sample result or performed method | State it directly, retaining population, units, time and uncertainty estimates |
| Association | Name the association and its scope; do not turn it into a causal effect |
| Causal inference | Check identification assumptions and actual evidence; a design label alone is insufficient |
| Participant account or case observation | Attribute it to the participants or case without making a prevalence claim |
| Untested explanation or extrapolation | Preserve its provisional status and relevant limits |

G4 may propose an evidence-backed claim correction, identifying the changed
meaning and source basis in its log. In a language-only request, report an
unsupported claim separately for substantive review instead of silently changing
it. Missing support is not supplied by a more cautious verb. Preserve consequential
limitations in the manuscript; keep tool-status and missing-input notes in the
existing work record. Avoid repeating the same limitation where it adds nothing,
but retain it wherever a passage would otherwise mislead, including an abstract.

### Step 3: Check Readability and Meaning

| Rule | Check | Action |
|------|-------|--------|
| Paragraph continuity | Can the reader follow the point and its development? | Repair information order without inventing a premise |
| Define acronyms once | First use fully spelled out; subsequent uses abbreviated | Find duplicated definitions |
| Active voice where possible | "We measured X" not "X was measured by us" | Rewrite passive constructions (unless convention demands passive) |
| Consistent terminology | Same concept = same word throughout | Make a terminology table |
| Numbers | Do quantities and formatting retain their meaning? | Preserve values and units; apply supplied venue rules |
| Citation support | Does the cited source support this claim? | Report missing support; use only inspected sources for proposed repairs |

Verify proposed expression edits against the original using the meaning-review
procedure in `references/scholarly-voice.md`. A request to check a polished
version authorizes that comparison, not another whole-text tone rewrite.

### Step 4: Generate the Tone Normalization Log

Record every change for author review:

| # | Location | Original | Issue | Rewrite | Category |
|---|----------|----------|-------|---------|----------|
| 1 | Results ¶1 | "It is important to note that we surveyed 120 firms." | Empty framing | "We surveyed 120 firms." | Expression; same sample |

## Output Contract

- `ToneNormalization`: write `RESEARCH/[topic]/compliance/tone_normalization.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

The tone normalization is **ready** when:

- [ ] Findings identify actual problems in the requested scope; zero findings is valid
- [ ] Expression edits preserve meaning; substantive proposals identify the changed claim and evidence
- [ ] Confidence language calibrated to evidence strength
- [ ] Necessary uncertainty and limitations remain; empty qualifiers and repetition are resolved
- [ ] Paragraph flow improves without a shortening quota or word blacklist
- [ ] Author voice preserved (not robotified)
- [ ] Log compiled with original → rewrite pairs for author review

## Minimal Output Format

```markdown
# Tone Normalization Log

## Summary
- Passages flagged: [n]
- Reviewed scope: [sections/passages]
- Categories: Claim calibration [n], Empty qualifiers [n], Repetition/flow [n]

## Changes

| # | Location | Original | Issue | Rewrite | Category |
|---|----------|----------|-------|---------|----------|

## Terminology Consistency
| Term used | Inconsistent variants found | Standardized to |
|-----------|---------------------------|-----------------|

## Confidence Calibration Issues
| Location | Claim strength | Evidence strength | Adjustment |
|----------|---------------|-------------------|------------|
```

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 过度删减 | 删减了必要的 hedging | 保留 evidence-based hedging（如 may, suggest） |
| 中文论文英文味 | 直译导致语体不对 | 区分中英文学术 register |
| 去掉所有 transition | 段落间缺少衔接 | 去的是 filler，留 logical connector |
| 为每个主张加限定词 | 已观察到的结果也显得犹疑 | 根据证据表达真实不确定性，不按句子分配限定词 |
| 复核时重新改写全文 | 润色成果被模板语气覆盖 | 指出具体含义偏移，只修正受影响的句子 |
| 忽略学科惯例 | 某些学科允许主动语态 | 核查目标期刊的 style preference |
