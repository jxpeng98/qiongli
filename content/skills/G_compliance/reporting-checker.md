---
id: reporting-checker
stage: G_compliance
description: "Validate reporting guideline completeness for target study type (CONSORT, STROBE, COREQ, SRQR, TRIPOD, etc.)."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
  - type: DesignSpec
    description: "Study design for guideline selection"
outputs:
  - type: ReportingChecklist
    artifact: "reporting_checklist.md"
constraints:
  - "Must select appropriate guideline based on study design"
  - "Must reference specific manuscript sections for each item"
  - "Must distinguish must-fix from nice-to-have gaps"
failure_modes:
  - "No standard guideline exists for the study type"
  - "Multiple guidelines applicable with conflicting requirements"
  - "Checklist items marked 'present' but content is actually insufficient"
tools: [filesystem, reporting-guidelines]
tags: [compliance, reporting, CONSORT, STROBE, COREQ, SRQR, TRIPOD, guidelines]
domain_aware: true
---

# Reporting Checker Skill

Check whether a manuscript is complete and aligned with the appropriate reporting guideline, producing a structured "what's missing" action list with specific manuscript locations.

## Purpose

Validate reporting guideline completeness for target study type (CONSORT, STROBE, COREQ, SRQR, TRIPOD, etc.).

## Related Task IDs

- `G1` (reporting completeness)

## Output (contract path)

- `RESEARCH/[topic]/reporting_checklist.md`

## When to Use

- Before submission (final quality assurance pass)
- Before sharing a preprint / conference camera-ready
- When converting working notes into a paper draft
- After major revisions (verify nothing was dropped)

## Inputs

- `Manuscript`: Draft manuscript
- `DesignSpec`: Study design for guideline selection
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

Create or update `RESEARCH/[topic]/quality-gate-report.md` with a Q3 `semantic_checks` entry using `q3_reporting_completeness`. Use structured evidence refs for the semantic check: each `evidence_refs` item must include `artifact`, `anchor`, and `supports`, with optional `claim_id` or `diagnostic_id` when the evidence maps to a specific claim or diagnostic. Evidence must point to `RESEARCH/[topic]/reporting_checklist.md`, relevant submission statements, and any explicit waiver for non-applicable items. If a required reporting item is absent or contradicts methods, ethics, data availability, or analysis artifacts, set Q3 to `BLOCKED` or `FAIL` and record the required action.

### Step 1: Determine Study Design and Select Guideline

Use `references/stage-G-compliance.md` for the shared design-to-guideline mapping,
edition/extension checks and reporting-versus-validity boundary. Read the current
applicable official checklist or the exact version required by the supplied
protocol/venue. Record its title, version, URL/source, checked date and selection
rationale. Do not use a remembered item count or an abbreviated local table as
proof of completeness. If the official checklist cannot be accessed, check the
available supplied requirements and report the remaining verification gap.

For mixed methods, assess both strands and their integration with the applicable
standard; the quantitative/qualitative majority does not decide the whole checklist.
For systematic reviews use `skills/G_compliance/prisma-checker.md`. For scoping
reviews, check the applicable PRISMA-ScR requirements through this item-mapping
process; do not impose the systematic-review workflow or pooling artifacts.
For an uncatalogued discipline, use the
verified venue and method requirements rather than inventing a mandatory guideline.

### Step 2: Apply Checklist Item-by-Item

Preserve the selected checklist's item IDs and wording in the working checklist
when permitted. Map each requirement to an actual manuscript location and evidence.
Use Present / Partial / Missing / Not applicable (reason) / Unverified (needed
source). A mentioned heading is not substantive reporting; a missing statement
is not proof the procedure was never performed.

Reconcile sample flow, methods, outcomes, uncertainty, protocol amendments and
ethics/data/code statements against the source artifacts. Keep source limits and
conflicting requirements visible. Propose specific edits without inventing study
facts or rewriting a result to meet a checklist.

### Step 3: Grade Each Gap

For each ❌ or ⚠️ item, classify:

| Severity | Meaning | Action |
|----------|---------|--------|
| **Must-fix** | Missing applicable requirement or contradiction blocks the stated readiness claim | Resolve or retain the block |
| **Should-fix** | Material reporting improvement supported by the study or guidance | Fix before submission |
| **Nice-to-have** | Best practice but not venue-required | Fix if time allows |

### Step 4: Generate Fix Action List

For each gap, specify exactly what content to add and where:

```
Item #7 (STROBE): Variables — ❌ Missing
Location: Methods § 3.2
Action: Add table with outcome variable definition, measurement,
        exposure/predictor operationalization, and confounder list
Priority: Must-fix
```

## Output Contract

- `ReportingChecklist`: write `RESEARCH/[topic]/reporting_checklist.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

The reporting checklist is **ready** when:

- [ ] Correct guideline selected and justified (with venue confirmation note)
- [ ] Every checklist item mapped to a specific manuscript section or marked N/A with justification
- [ ] All must-fix items have specific action instructions
- [ ] No item marked "present" without verifying the content is sufficient (not just mentioned)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Marking "present" because the word appears | Section exists but content is insufficient | Check substance, not just existence |
| Skipping N/A justification | Reviewer questions why item omitted | Always document why N/A |
| Using wrong guideline for study type | Misalignment wastes effort | Confirm design → guideline mapping with venue |
| Ignoring guideline extensions | STROBE has cohort/case-control/cross-sectional variants | Use the design-specific extension |

## Minimal Output Format

```markdown
# Reporting Checklist

## Guideline: [CONSORT / STROBE / COREQ / SRQR / TRIPOD]
- Study design: ...
- Venue / article type / submission stage: ...
- Guideline version / source URL / checked date: ...
- Justification and source-access limits: ...

## Checklist

| # | Item | Status | Manuscript location | Gap / Action | Priority |
|---|------|--------|-------------------|--------------|-----------|
| 1 | ... | Present/Partial/Missing/N/A/Unverified | § X.Y | ... | Must-fix / Should-fix / N/A |

## Action List (prioritized)

### Must-fix
1. ...

### Should-fix
1. ...
```
