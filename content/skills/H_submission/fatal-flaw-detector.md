---
id: fatal-flaw-detector
stage: H_submission
description: "Identify evidenced submission blockers and material risks with proportionate remedies."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
outputs:
  - type: FatalFlawAnalysis
    artifact: "revision/fatal_flaw_analysis.md"
constraints:
  - "Must categorize flaws by severity (fatal/major/minor)"
  - "Must propose specific remediation for each flaw"
  - "Must test from the reviewer's perspective, not the author's"
failure_modes:
  - "False positive on methodological choices that are defensible"
  - "Missing domain-specific fatal flaws"
  - "Focusing on writing issues while missing structural problems"
tools: [filesystem]
tags: [submission, fatal-flaw, desk-reject, quality-gate, pre-submission]
domain_aware: true
---

# Fatal Flaw Detector Skill

## Purpose

Identify demonstrated blockers and material risks before submission. Explain
whether they concern scientific validity, missing reporting or fit to the chosen
venue, and what would resolve them. Do not predict the editor's decision.

## Related Task IDs

- `H4` (fatal flaw analysis)

## Output (contract path)

- `RESEARCH/[topic]/revision/fatal_flaw_analysis.md`

## When to Use

- The user requests a submission-blocker check or formal H4 analysis.
- A substantive revision needs targeted verification of earlier issues.
- A readiness assessment needs to distinguish repairable presentation from a
  central claim that the research cannot support.

## Inputs

Use the current manuscript, relevant evidence/supplements and supplied target
requirements. Identify what was inspected. Without a target, assess validity and
reporting within scope, but leave venue eligibility open. Missing facts require
clarification, not assumed ethical violations or a fabricated gap report.

## Process

Follow `references/stage-H-submission.md` for severity, applicable sources, review
permission and formal write boundaries. Use its criteria for the actual article
type and design; no minimum flaw count or universal rejection thresholds.

1. Check the central claims against the design, analysis or argument and available
   evidence. Trace contradictions through the manuscript and supplements before
   declaring a finding. A concern about an assumption is not proof it fails.
2. Check the chosen venue's eligibility and applicable requirements. Separate a
   venue mismatch from scientific invalidity. Flexible initial formatting,
   nonapplicable declarations and restricted data must not become automatic flaws.
3. Test the strongest supported objections. Consider defensible methodological
   choices and counterevidence. Distinguish missing reporting from work demonstrably
   not done; neither missing data access nor an unexplained omission proves misconduct.
4. Classify actual findings as `fatal`, `major` or `minor` using the shared severity
   definitions. State the affected claim or submission requirement and consequence.
   Unknown checks remain unresolved rather than automatically fatal.
5. Propose the smallest adequate remedy. A wording fix is sufficient only if the
   retained claim is supported; narrowing a claim may require changes throughout
   the paper. New experiments, reanalysis, approval or target changes remain
   explicit work/decisions, not silently completed tasks.

Use only relevant checks, for example:

| Concern | Evidence needed before classifying it |
|---|---|
| Claim/design mismatch | Exact claim plus the design's justified inferential limits |
| Unsupported result | Result/analysis/source discrepancy and effect on the conclusion |
| Missing methodological detail | What cannot be evaluated, where it is absent, and what would clarify it |
| Ethics or access obligation | Applicable obligation and actual status; no approval number invented from a template |
| Out-of-scope article | Official scope/article-type rule and the manuscript's actual contribution |
| Incomplete submission | Applicable stage requirement plus the missing file or unresolved fact |
| Reproducibility concern | Needed evidence/steps and permitted access conditions; public release is not always required |

## Output Contract

Formal H4 writes `RESEARCH/[topic]/revision/fatal_flaw_analysis.md`. A focused
answer may stay in chat. Include inspected scope, sources, actual findings and
unresolved checks. Follow the shared contract for central claim-ledger and material
citation-risk updates; apply `references/academic-output-rubric.md`.

| Issue ID | Kind | Location and evidence | Criterion / consequence | Severity | Remedy / new work | Uncertainty |
|---|---|---|---|---|---|---|

Use a scoped recommendation: `Submit`, `Fix-then-submit` or `Do-not-submit`, only
when the necessary checks support it. Otherwise state that readiness is unresolved.
`Submit` means no blocker found within the completed checks, not guaranteed
acceptance or permission to submit. Retain previous IDs and existing BLOCK findings.
Zero findings is valid; never invent flaws, citations, results or reviewer comments.

## Quality Bar

- [ ] Relevant validity, reporting and venue constraints checked within stated scope.
- [ ] Each finding has a source, consequence and justified severity.
- [ ] Fatal/major findings have specific remedies or explicit new-work decisions.
- [ ] Unavailable evidence and unresolved checks are distinct from demonstrated flaws.
- [ ] Readiness preserves formal gates and independence requirements.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Call a long introduction fatal | Check its actual effect and the applicable requirement |
| Require power or robustness for every design | Use the method's justified evidence standard |
| Infer no ethics approval from an absent statement | Request status; do not invent approval or misconduct |
| List imagined objections to meet a quota | Report only supported risks and open questions |
| Repair an invalid causal claim with a hedge | Check whether the revised claim and whole argument are supportable |
