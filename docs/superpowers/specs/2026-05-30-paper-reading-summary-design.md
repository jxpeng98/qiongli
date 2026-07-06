# Paper Reading Summary Design

## Goal

Strengthen Qiongli B2 `/paper-read` so it produces both trustworthy single-paper reading notes and project-level paper reading summaries that can be reused by gap analysis, related-work writing, synthesis, and manuscript drafting.

## User-Approved Scope

Use the combined approach:

1. Enhance the existing `/paper-read` workflow rather than introducing a new top-level task ID.
2. Keep B2 as the canonical task for targeted paper reading.
3. Add explicit project-level summary artifacts for organizing multiple paper notes.
4. Treat truthfulness and boundary control as hard requirements: summaries must be evidence-grounded, uncertainty must be visible, and missing evidence must never be filled in by invention.

## Artifact Contract

B2 should continue to produce:

- `notes/`
- `bibliography.bib`
- `retrieval_manifest.csv`

B2 should additionally produce or update:

- `literature/paper_reading_summary.md`
- `literature/paper_reading_matrix.md`

The single-paper note remains `RESEARCH/[topic]/notes/[citekey].md`. The project-level summary and matrix live under `RESEARCH/[topic]/literature/` because they organize the reading corpus rather than a single paper.

## Workflow Behavior

`/paper-read` should run two layers:

1. Single-paper deep reading:
   - Retrieve or identify full text where possible.
   - Normalize metadata and citekey.
   - Write a structured paper note.
   - Record retrieval status and bibliography state.

2. Project-level summary integration:
   - Update or create `literature/paper_reading_matrix.md`.
   - Update or create `literature/paper_reading_summary.md`.
   - Integrate the current paper into themes, methods, datasets, findings, limitations, contradictions, gaps, and reusable writing points.
   - Preserve existing human-written content. If an update cannot be merged safely, append a clearly marked dated entry instead of replacing prior prose.

## Truthfulness And Boundary Controls

All B2 summary outputs must follow these rules:

- Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications.
- Each central summary claim must be traceable to a note, source section, quote, table, abstract, or metadata field.
- If only metadata or abstract is available, label the note and summary entry with `evidence_limit: abstract_only`.
- If a full-text claim cannot be verified, write it as a gap note or uncertainty, not as fact.
- Separate author claims, agent interpretation, and project relevance.
- Mark inference strength explicitly: `direct_evidence`, `reasonable_inference`, or `unsupported_gap`.
- Keep strong synthesis claims out of B2 unless multiple notes support them; B2 may organize evidence, but it should not overstate review-grade coverage.

## Template Changes

`templates/paper-note.md` should add explicit fields for:

- evidence limit and retrieval status
- source anchors for key claims
- author-stated contribution
- agent interpretation
- project relevance
- reusable citation points
- uncertainty and missing-information notes

Create `templates/paper-reading-summary.md` for project-level narrative organization:

- corpus overview
- theme clusters
- method and data patterns
- stable findings
- contradictions and contested claims
- research gaps
- implications for the current project
- writing-ready citation points
- uncertainty register

Create `templates/paper-reading-matrix.md` for compact cross-paper comparison:

- citation/citekey
- evidence limit
- theory/framework
- method/identification
- dataset/source
- main finding
- limitations
- project relevance
- claim anchors

## Contract And Reference Updates

Update the canonical contract and generated references so B2 declares the two new literature artifacts.

Primary source files:

- `standards/research-workflow-contract.yaml`
- `qiongli-workflow/workflows/paper-read.md`
- `qiongli-workflow/references/stage-B-literature.md`
- `qiongli-workflow/references/workflow-contract.md` generated from the YAML contract
- `templates/paper-note.md`
- `templates/paper-reading-summary.md`
- `templates/paper-reading-matrix.md`

Generated or mirrored payloads should be refreshed by existing sync scripts after source edits.

## Testing And Verification

Verification should include:

- A failing-first check that B2 contract outputs include the new summary and matrix artifacts.
- A failing-first check that the paper note template exposes evidence-limit and anti-hallucination fields.
- A failing-first check that the new summary template requires source anchors and uncertainty handling.
- `python3 scripts/validate_research_standard.py --strict`
- sync/package checks needed to keep payload copies consistent.

## Out Of Scope

- No new top-level task ID.
- No new provider implementation.
- No automatic PDF parsing engine.
- No claim that B2 produces systematic-review-grade synthesis.
- No fabricated examples using real paper details.
