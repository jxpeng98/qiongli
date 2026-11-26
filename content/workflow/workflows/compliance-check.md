---
description: "Check reporting requirements, research integrity, and writing style before submission."
---

# Compliance Check Workflow (G-Stage)

Check the part of the manuscript the user identifies. For a question about a
supplied passage, return the assessment or requested wording in chat. Describe
the sections actually checked. A passage can establish a local inconsistency
even when whole-manuscript reporting completeness remains unknown.

## Request

$ARGUMENTS

## Select the check

Use the relevant section of `references/stage-G-compliance.md`. Reuse the study
design, manuscript version and any checklist already supplied. If more than one
check fits, use the user's desired result to decide which are needed.

| Task | What to check | Formal project output |
|---|---|---|
| G1 | Applicable reporting items against manuscript locations with `reporting-checker` | `RESEARCH/[topic]/reporting_checklist.md` |
| G2 | Applicable PRISMA items and counts against the review records with `prisma-checker` | `RESEARCH/[topic]/prisma_checklist.md` |
| G3 | Research question, methods, results and claims across the supplied sections with `manuscript-architect` | `RESEARCH/[topic]/manuscript/claims_evidence_map.md` |
| G4 | Actual overclaiming, qualifications and tone in context with `tone-normalizer` | `RESEARCH/[topic]/compliance/tone_normalization.md` |

## Formal checkpoint

Reuse the selected project and its locked boundary in `context/boundary_review.md`.
Use `boundary-interviewer` for an unresolved or contradicted choice that affects
checklist fit, claim strength or disclosure. For sign-off, verify each required
item against the relevant manuscript and records. Mark missing evidence and
complete the checks it does not affect. Save requested project outputs through
preview, explicit approval and revision-checked apply.
