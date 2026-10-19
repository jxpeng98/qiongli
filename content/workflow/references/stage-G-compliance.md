# Stage G — Polishing & Compliance (G1–G4)

This stage is “submission hardening”: reporting completeness, PRISMA checks, cross-section consistency, and tone calibration.

## Canonical outputs (contract paths)

- `G1` → `reporting_checklist.md`
- `G2` → `prisma_checklist.md`
- `G3` → `manuscript/claims_evidence_map.md` (updated)
- `G4` → `compliance/tone_normalization.md`

## Quality gate focus

- `Q2` (claim-evidence traceability): enforced via `G3`.
- `Q3` (reporting completeness): enforced via `G1` and `G2`.
- Semantic gate report: update `quality-gate-report.md` with `q3_reporting_completeness`; evidence must anchor required checklist items, disclosures, submission statements, and explicit waivers.

---

## G1 — Reporting Completeness (non-PRISMA)

Select by study design, article type, venue and applicable edition/extension.
Use the [EQUATOR library](https://www.equator-network.org/library/) for health
research and [APA JARS](https://www.apa.org/pubs/journals/resources/apa-style-jars.html)
where relevant to behavioral/education research; computing and humanities use
their method and verified venue requirements. A reporting checklist is not a
study-quality score or evidence of ethics approval.

For each required item record its source/version, manuscript location, evidence
and status: addressed, missing, not applicable with reason, or unverified.
Distinguish absent reporting from a demonstrated methodological flaw. Do not
manufacture an exemption, sample size, registration or result to fill a row.

**Definition of done**
- A guideline is selected and declared (by design type):
  - RCT report → CONSORT; trial protocol → SPIRIT (2025 statements available;
    verify the applicable edition and extension)
  - Observational → STROBE
  - Diagnostic accuracy → STARD; clinical prediction model → TRIPOD+AI as applicable
  - Qualitative → COREQ / SRQR
  - Animal research → ARRIVE; case report → CARE; economic evaluation → CHEERS
- Checklist items are mapped to manuscript locations
- Missing items are converted into concrete edit tasks

Write into: `reporting_checklist.md`.

---

## G2 — PRISMA Compliance (Systematic Review)

**Definition of done**
- PRISMA 2020 checklist is completed with manuscript locations
- Select an applicable extension for the review type, such as PRISMA-ScR for
  scoping reviews; do not require meta-analysis merely to complete the checklist
- Counts reconcile across `search_log.md`, `screening/`, `extraction_table.md`, and `synthesis.md`
- Reconcile records, sought/retrieved reports and included studies separately;
  inaccessible reports and duplicate reports are not interchangeable exclusions
- Deviations from protocol are documented (if any)

Write into: `prisma_checklist.md`.

---

## G3 — Cross-section Integrity Check

**Definition of done**
- Abstract claims appear in the claim–evidence map and are supported
- RQs ↔ Methods ↔ Results alignment holds (no “method drift”)
- Discussion claims are calibrated to evidence quality

Write/update: `manuscript/claims_evidence_map.md`.

---

## G4 — Tone & Style Normalization

Use `skills/G_compliance/tone-normalizer.md` to review actual overclaiming,
empty qualifications, repetition and unclear expression. Follow
`references/scholarly-voice.md` for language editing and meaning review.

**Definition of done**
- Claim strength and scope match available evidence; substantive corrections
  identify the changed claim and source basis
- Observed results can be stated directly; necessary uncertainty and limitations
  remain without a hedge quota or banned-word list
- Paragraphs flow clearly; shortening is not a required outcome
- A polished candidate receives a meaning comparison with its original, with
  repairs limited to actual drift rather than another general tone rewrite

Write into: `compliance/tone_normalization.md` with:
- specific findings in the reviewed scope, which may be empty
- suggested rewrites
- global style rules adopted (tense, voice, acronym policy)
