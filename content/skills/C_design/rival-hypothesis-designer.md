---
id: rival-hypothesis-designer
stage: C_design
description: "Construct competing theories and rival explanations to strengthen design by pre-empting reviewer objections."
inputs:
  - type: DesignSpec
    description: "Study design and identification strategy"
  - type: HypothesisSet
    description: "Primary hypotheses or propositions"
outputs:
  - type: RivalHypotheses
    artifact: "design/rival_hypotheses.md"
constraints:
  - "Must ground consequential rivals in theory or available evidence"
  - "Must state possible distinguishing evidence or explain why a rival remains unresolved"
  - "Must link feasible controls or tests to the claim and disclose their assumptions"
failure_modes:
  - "Rivals are straw men (no one would actually propose them)"
  - "Unresolvable rivals are hidden or described as ruled out"
  - "Only statistical threats listed; no substantive theoretical alternatives"
tools: [filesystem]
tags: [design, rival-hypotheses, threats-to-validity, competing-theories]
domain_aware: false
---

# Rival Hypothesis Designer Skill

Identify credible competing explanations and what the available evidence can distinguish.
Follow the **Design judgment contract** in `references/stage-C-design.md`; preserve
explicit protocol requirements without adding rivals to meet a default count.
A narrow chat request does not imply a formal C1_5 write or downstream execution.

## Purpose

Construct competing theories and rival explanations to strengthen design by pre-empting reviewer objections.

## Related Task IDs

- `C1_5` (rival hypothesis design)

## Output (contract path)

- `RESEARCH/[topic]/design/rival_hypotheses.md`

## When to Use

- After hypotheses (A1_5) and study design (C1) are drafted
- Before locking the analysis plan (C3) — rivals may require additional data or controls
- When the identification strategy has known weaknesses
- For qualitative research: before data collection, to plan disconfirming-case search

## Inputs

- `DesignSpec`: Study design and identification strategy
- `HypothesisSet`: Primary hypotheses or propositions
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Identify Rival Sources

Consider sources relevant to the focal claim; the categories below are prompts,
not a quota. Record consequential unresolved rivals even when no test is feasible:

| Source | What to Ask | Example |
|--------|-------------|---------|
| **Alternative theories** | What other theoretical framework predicts the same pattern? | SDT vs expectancy theory both predict motivation effects |
| **Confounders** | What unmeasured variable produces a spurious association? | Firm size drives both digital adoption and performance |
| **Reverse causality** | Could the DV cause the IV instead? | Productive workers choose remote work (not reverse) |
| **Methodological artifacts** | Could the pattern be an artifact of measurement, timing, or selection? | Common-method bias in single-source surveys |

For qualitative research, add:
| Source | What to Ask | Example |
|--------|-------------|---------|
| **Alternative framings** | What other interpretive lens would organize the same data differently? | Power lens vs sensemaking lens on the same governance episodes |
| **Informant bias** | Do participants have reasons to present a particular narrative? | Managers emphasize their rationality; employees emphasize constraints |

### Step 2: Construct Each Rival in Detail

For each grounded rival, specify these elements where supported. State unavailable
evidence or an unresolved limitation instead of inventing a discriminating test:

```
Rival R[n]: [name]

Mechanism:       Why this rival would produce the same observed pattern
Observable       What data pattern would look different IF this rival
Implication:     is the true explanation (not the focal hypothesis)
Design Control:  How the study design already addresses this
                 (control variable, fixed effect, matching, case selection)
Empirical Test:  What additional test could distinguish focal from rival
                 (interaction term, instrumental variable, disconfirming case)
```

### Step 3: Build the Rival Comparison Matrix

| ID | Rival | Mechanism | Observable Difference | Design Control | Empirical Test | Status |
|----|-------|-----------|----------------------|----------------|----------------|--------|

Use stable rival IDs. Explain any assumptions needed to distinguish the focal
claim from each rival. A pre-trend test, lagged variable or added control alone
does not rule out an alternative explanation.

### Step 4: Assess Rival Quality

Evaluate each rival against these criteria:

| Criterion | Question | Good Rival | Bad Rival |
|-----------|----------|-----------|-----------|
| **Plausible** | Would a reasonable reviewer propose this? | "Selection effects are common in observational studies" | "Aliens influenced the data" |
| **Informative** | What could distinguish them under stated assumptions? | Feasible evidence or a clearly unresolved limitation | A test is claimed decisive without supporting assumptions |
| **Non-trivial** | Does it challenge the core claim, not a peripheral detail? | Challenges the identification strategy | Questions a control variable's coding |
| **Grounded** | Is there theory or evidence behind the rival? | Cites prior work showing this rival matters | Pure speculation |

> **Anti-pattern**: Listing only statistical threats (heteroskedasticity, non-normality) as "rivals." These are estimation concerns, not rival explanations. Rivals must offer a **different substantive explanation** for the pattern.

### Step 5: Feed Downstream

For the agreed formal scope, propose these links and retain existing write
approval/CAS. Do not silently amend an approved design, run tests or collect data.

- Each rival with a planned test → add to `C3` analysis plan and `C3_5` robustness plan
- Each rival with a design control → verify the control is in `C1` study design
- Each rival without a test → acknowledge explicitly as a limitation in `F3` discussion

## Output Contract

- For formal C1_5, `RivalHypotheses`: write `RESEARCH/[topic]/design/rival_hypotheses.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

The rival hypothesis set is **ready** when:

- [ ] Rivals are substantive, consequential and grounded; no filler to meet a count
- [ ] Mechanisms, possible distinguishing evidence and assumptions are explicit
- [ ] Infeasible tests and unresolved rivals remain visible with consequences for claims
- [ ] Agreed tests/controls have downstream links; chat proposals do not claim those files were updated

## Minimal Output Format

```markdown
# Rival Hypotheses

## Focal Claim
[Primary hypothesis or proposition being defended]

## Rival Comparison Matrix

| ID | Rival | Mechanism | Observable Difference | Design Control | Empirical Test | Status |
|----|-------|-----------|----------------------|----------------|----------------|--------|

## Detailed Rival Specifications

### R1: [Name]
- **Mechanism**: ...
- **Observable implication**: ...
- **Design control**: ...
- **Empirical test**: ...

## Assessment
- Rivals addressed by design: [IDs and supporting assumptions]
- Rivals testable empirically: [IDs and available evidence]
- Rivals acknowledged as limitations: [IDs and claim consequences]

## Downstream Links
- Analysis plan additions: [tests to add to C3]
- Robustness plan additions: [checks to add to C3_5]
- Planned limitations: [rivals that cannot be resolved]
```

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 只考虑稻草人替代 | 竞争假设太弱不构成威胁 | 找该理论的 strongest advocate 论文 |
| 未连接到测试 | 列出竞争解释但不设计区分检验 | 说明可行的区分证据；无法区分时保留解释边界 |
| 遗漏内生性 | 反向因果或遗漏变量 | 系统检查 endogeneity threats |
| 只考虑理论竞争 | 忽视方法论替代（不同估计方法） | 区分实质竞争解释与估计方法诊断 |
| 数量太多 | 无法全部回应 | 按证据、后果与可行性排序，不凑数量 |
