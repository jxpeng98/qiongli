---
description: 投稿前打包（reporting checklist + cover letter + submission checklist + statements）
---

# Submission Preparation

Assemble a submission-ready package for a target venue.

Canonical Task ID (from the globally installed `qiongli-workflow` skill):
- `H1` submission package

## Target

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Follow `references/stage-H-submission.md` for applicable venue requirements,
source verification, author confirmations and formal write boundaries. This workflow
packages files; a request only to adapt to a chosen target uses A5, while a request
to recommend venues from a draft uses H5. Do not launch H1 for either by default.

Reuse the selected `RESEARCH/[topic]/` folder. Ask for a destination only when
saving is requested and none is known. A narrow answer can stay in chat.
Formal H1 retains this structure through the existing preview/approval/CAS owner;
mark nonapplicable items with reasons and unresolved items as pending:
```
RESEARCH/[topic]/submission/
├── cover_letter.md
├── submission_checklist.md
├── title_page.md
├── highlights.md
├── suggested_reviewers.md
├── author_contributions_credit.md
├── funding_statement.md
├── coi_statement.md
├── data_availability.md
├── ai_disclosure.md
└── supplementary_inventory.md
```

For formal H1, account for applicable reporting obligations at project root:
```
RESEARCH/[topic]/reporting_checklist.md
```

### Step 1: Collect Submission Requirements

Reuse the manuscript, target, article type and submission stage. Verify applicable
official instructions and exceptions; record source URLs/titles and checked dates
in the submission checklist. Ask only for missing facts that change the package,
such as author approval or data access status. Do not ask the user to reconstruct
public requirements that can be checked with available authorized tools.

### Academic Boundary Review Trigger (MVP)

Use `boundary-interviewer` before submission or rebuttal work when unresolved boundaries remain around journal fit, submission readiness, missing statements, data/code availability, authorship declarations, reviewer-sensitive weaknesses, impossible reviewer requests, response commitments, claim strength, or evidence threshold.

The boundary pass must ask one academic question at a time. The answer must state which submission or revision artifact changes, what promise can truthfully be made, and whether the decision belongs in `context/decision_log.md` or `context/stage_handoff.md`.

### Step 2: Reporting Compliance Check

Use **reporting-checker** for the applicable design and formal Q3 gate:
- `RESEARCH/[topic]/reporting_checklist.md` (use `templates/reporting-checklist.md`)

If this is a systematic review, run **prisma-checker** instead (or in addition).

### Step 3: Package Submission Artifacts

Use **submission-packager** to draft:
- `RESEARCH/[topic]/submission/cover_letter.md` (use `templates/cover-letter.md`)
- `RESEARCH/[topic]/submission/submission_checklist.md` (use `templates/submission-checklist.md`)
- `RESEARCH/[topic]/submission/title_page.md` (use `templates/title-page.md`)
- `RESEARCH/[topic]/submission/highlights.md` (use `templates/highlights.md`)
- `RESEARCH/[topic]/submission/suggested_reviewers.md` (use `templates/suggested-reviewers.md`)
- `RESEARCH/[topic]/submission/author_contributions_credit.md` (use `templates/author-contributions-credit.md`)
- `RESEARCH/[topic]/submission/funding_statement.md` (use `templates/funding-statement.md`)
- `RESEARCH/[topic]/submission/coi_statement.md` (use `templates/coi-statement.md`)
- `RESEARCH/[topic]/submission/data_availability.md` (use `templates/data-availability.md`)
- `RESEARCH/[topic]/submission/ai_disclosure.md` (use `templates/ai-disclosure.md`)
- `RESEARCH/[topic]/submission/supplementary_inventory.md` (use `templates/supplementary-inventory.md`)

Begin submission preparation now.
