---
id: submission-packager
stage: H_submission
description: "Assemble submission-ready package: cover letter, compliance files, title page, disclosures, and supplementary materials."
inputs:
  - type: Manuscript
    description: "Finalized manuscript"
  - type: VenueAnalysis
    description: "Target venue requirements"
    required: false
outputs:
  - type: SubmissionPackage
    artifact: "submission/cover_letter.md"
constraints:
  - "Must include all venue-required items"
  - "Must generate CRediT author contributions when required"
  - "Must verify anonymization for double-blind venues"
failure_modes:
  - "Missing co-author information for disclosures"
  - "Venue requirements changed since analysis"
  - "Supplementary materials reference missing files"
tools: [filesystem, submission-kit]
tags: [submission, cover-letter, checklist, disclosures, supplementary]
domain_aware: true
---

# Submission Packager Skill

## Purpose

Prepare the selected venue's submission files and identify what still needs author
confirmation. A completed template is not proof of compliance or submission.

## Related Task IDs

- `H1` (submission packaging)

## Output (contract paths)

- `RESEARCH/[topic]/submission/cover_letter.md`
- `RESEARCH/[topic]/submission/submission_checklist.md`
- `RESEARCH/[topic]/submission/title_page.md`
- Remaining H1 paths in `references/stage-H-submission.md`

## When to Use

- The user requests a package for a near-final manuscript.
- A revision needs its applicable files refreshed.
- For journal choice or manuscript adaptation alone, use H5 or A5 without
  automatically creating a submission package.

## Inputs

Reuse the current manuscript, confirmed target/type/stage, venue requirements and
verified author information. Ask for only missing decision-relevant facts. Retain
pending declarations rather than assuming coauthor approval, ethics status, funding,
conflicts, AI use or data access. A narrow answer can stay in chat.

## Process

Follow `references/stage-H-submission.md` for source currency, applicability,
truthful author commitments, formal write boundaries and readiness gates.

1. Verify the applicable requirements and record their sources/checked dates in
   `templates/submission-checklist.md`. Check initial versus revision rules and
   article-type exceptions before imposing formatting. Reuse A5 evidence after
   checking whether the target, rules or manuscript have changed.
2. Account for Q3 and applicable reporting obligations (G1/G2). Use the relevant
   design checklist; a theoretical article does not automatically need an empirical
   checklist. A draft with unresolved required items remains pending.
3. Draft the cover letter using `templates/cover-letter.md`: connect the actual
   contribution to this venue's readers without repeating the abstract or adding
   unsupported novelty. Include author approval, exclusive submission and other
   assurances only when confirmed. Do not invent an editor's name.
4. Prepare the applicable title page, highlights, contributions, funding, COI,
   data/code availability, AI disclosure and supplementary inventory using the
   existing templates. Their example facts, limits and checkmarks are placeholders.
   Replace them with verified facts or explicit pending/nonapplicable status;
   never claim an unavailable repository, approval or file exists.
5. Check anonymity under the actual venue instructions. Preserve a source manuscript
   and propose an appropriate submission copy through the existing write owner.
   Self-citation handling varies: do not remove relevant citations or replace them
   with blinded placeholders unless required. Review identifying text, metadata,
   tracked changes and repository links without automatically accepting changes,
   deleting originals or promising future public data access.
6. If reviewer suggestions are requested, verify identity, relevant expertise,
   contact source and conflicts under that venue's rules. Number and conflict
   windows are venue-specific. Do not invent contacts, assume no conflicts, cite
   someone merely to secure a review, or contact candidates automatically.
7. Verify the resulting files and cross-references where tools permit. Distinguish
   prepared, checked, pending and not applicable; a Markdown draft is not a checked
   PDF, uploaded supplement or completed portal step. Return remaining actions.

## Output Contract

Formal H1 keeps the canonical files listed in the shared Stage H contract; mark
nonapplicable items with a reason. Use `submission/submission_checklist.md` to
record the applicable requirements, checked evidence and remaining gaps. Include:

| File / declaration | Required at this stage? | Path / source | Status | Remaining author action |
|---|---|---|---|---|

A package is ready only when applicable requirements and formal gates are actually
satisfied. Preserve source integrity under the shared evidence/citation contracts
and apply `references/academic-output-rubric.md`. Do not invent author facts,
reviewer comments, scientific results or completed checks. Packaging does not
submit the paper, contact anyone or authorize deletion.

## Quality Bar

- [ ] Requirements identify the exact venue, type, stage and current source basis.
- [ ] Cover letter and manuscript claims agree; assurances are confirmed.
- [ ] Required declarations, reporting and supplemental files are accounted for.
- [ ] Anonymity and output format were checked where applicable and accessible.
- [ ] Unresolved facts, unchecked outputs and portal actions remain visible.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Copy example approvals or checkmarks | Use confirmed facts and actual validation status |
| Enforce final format on initial submission | Check stage-specific rules |
| Erase identifying citations automatically | Follow the venue's anonymity policy and preserve originals |
| Treat reviewer suggestions as an outreach task | Prepare verified suggestions only |
| Declare ready because all files exist | Check applicable content, gates and unresolved facts |
