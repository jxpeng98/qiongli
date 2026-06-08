---
description: 深度阅读并分析单篇学术论文，生成结构化笔记
---

# Deep Paper Reading & Analysis

Conduct in-depth reading and structured analysis of an academic paper.

Canonical Task ID (from the globally installed `qiongli-workflow` skill):
- `B2` targeted paper reading

## Paper

$ARGUMENTS

## Workflow

### Step 0: Project Context Selection

**Required**: Determine which research project this paper belongs to.

Ask the user:
> "Which research project folder should this paper be saved to?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`
> - Standalone: `RESEARCH/standalone/`

Set `[topic]` variable based on user selection.

Ensure the target directory structure exists:
```
RESEARCH/[topic]/
├── literature/
├── notes/
├── retrieval_manifest.csv
└── bibliography.bib
```

### Step 1: Paper Retrieval

Attempt to access the paper through the MCP/provider stack:
1. Direct URL or user-supplied local file when provided.
2. `metadata-registry` DOI/title metadata lookup using configured Crossref/OpenAlex overlays when available.
3. `scholarly-search` title/identifier lookup using configured Semantic Scholar, OpenAlex, arXiv, or platform-native provider paths.
4. `fulltext-retrieval` retrieval planning for OA PDF, preprint, or abstract-only access.

Record provider capability mode as `provider_connected` or `strategy_only`. If no MCP/provider or platform-native search is available, use user-supplied metadata only and keep that evidence boundary visible.

If full text unavailable, work with abstract and metadata only. Mark the note and project-level summary entry with `evidence_limit: abstract_only` or `evidence_limit: metadata_only`.

### Step 1.5: Truthfulness Boundary

Apply this boundary before writing any note, summary, matrix row, or BibTeX-adjacent claim:

- Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications.
- Every central claim must have a source anchor: paper section, page, table, quote, abstract, metadata field, or existing note anchor.
- Separate author claims, extracted facts, agent interpretation, and project relevance.
- Label inference strength as `direct_evidence`, `reasonable_inference`, or `unsupported_gap`.
- If evidence is missing, write an `unsupported_gap` entry or uncertainty note instead of completing the field.
- B2 may organize targeted reading evidence, but it must not claim systematic-review-grade coverage.

### Step 2: Metadata Extraction

Extract bibliographic information:
- Title
- Authors (with affiliations if available)
- Publication venue (journal/conference)
- Year
- Volume/Issue/Pages
- DOI
- Keywords
- Abstract

### Step 3: Deep Reading Analysis

Use the **paper-extractor** skill to analyze:

#### Research Problem
- What problem does this paper address?
- Why is this problem important?
- What is the research gap being filled?

#### Research Questions/Objectives
- What are the explicit RQs or hypotheses?
- What are the research objectives?

#### Theoretical Framework
- What theories/frameworks guide the research?
- How are key concepts defined?
- What is the conceptual model (if any)?

#### Methodology
- **Research Design**: Qualitative/Quantitative/Mixed?
- **Sample/Data**: Who/what was studied? Sample size?
- **Data Collection**: How was data gathered?
- **Data Analysis**: What analytical methods were used?
- **Validity/Reliability**: How was rigor ensured?

#### Key Findings
- What are the main results?
- What patterns/themes emerged?
- What are the effect sizes/statistics (if quantitative)?

#### Contributions
- What new knowledge does this paper add?
- What are the theoretical contributions?
- What are the practical implications?

#### Limitations
- What limitations do the authors acknowledge?
- What limitations do you identify?

#### Future Research
- What do authors suggest for future work?
- What questions remain unanswered?

### Step 4: Critical Evaluation

Apply the **quality-assessor** skill:
- Assign A-E evidence rating
- Evaluate argument strength
- Assess methodological rigor
- Identify potential biases

### Step 5: Generate Outputs

**Paper Note** (Markdown):
Create structured note using `templates/paper-note.md` → Save to `RESEARCH/[topic]/notes/[citekey].md`

Use the **metadata-enricher** skill to:
1. Normalize DOI and metadata through `metadata-registry` using Crossref/OpenAlex overlays when configured.
2. Generate a consistent citekey from normalized metadata or clearly marked user-supplied metadata.
3. Record `strategy_only` when metadata is abstract-only, metadata-only, or manually supplied.

Use the **fulltext-fetcher** skill to:
1. Route full-text planning through `fulltext-retrieval`.
2. Attempt OA/preprint retrieval only through configured resolver overlays or user-provided files.
3. Document retrieval status, version read, and evidence limit in `retrieval_manifest.csv`.

**BibTeX Entry**:
Generate properly formatted BibTeX → Append to `RESEARCH/[topic]/bibliography.bib`

### Step 6: Project-Level Reading Summary

Update or create these B2 summary artifacts:

1. `RESEARCH/[topic]/literature/paper_reading_matrix.md` using `templates/paper-reading-matrix.md`
2. `RESEARCH/[topic]/literature/paper_reading_summary.md` using `templates/paper-reading-summary.md`

For the current paper, add or update:
- citation/citekey
- `evidence_limit`
- retrieval status and version read
- theory/framework
- method or identification strategy
- dataset/source
- main finding
- limitations
- project relevance
- `source_anchor`
- inference strength (`direct_evidence`, `reasonable_inference`, `unsupported_gap`)

When merging into existing summary files:
- Preserve existing human-written notes.
- Do not overwrite prior synthesis prose unless the replacement is strictly better grounded and all source anchors are retained.
- If a safe merge is unclear, append a dated entry under the relevant section.
- Put unsupported or under-specified material in the uncertainty register.

## Output Format

The paper note should follow this structure:

```markdown
# [Paper Title]

## Metadata
- **Authors**:
- **Year**:
- **Venue**:
- **DOI**:
- **Evidence Rating**: [ ] A [ ] B [ ] C [ ] D [ ] E
- **Evidence Limit**: full_text / abstract_only / metadata_only / unavailable
- **Retrieval Status**: retrieved_oa / retrieved_preprint / abstract_only / not_retrieved:<reason>

## Source Anchors
| Claim ID | Claim Type | Source Anchor | Inference Strength |
|---|---|---|---|
| C1 | author_claim / extracted_fact / interpretation / project_relevance | section/table/page/abstract/metadata | direct_evidence / reasonable_inference / unsupported_gap |

## Quick Summary
[2-3 sentence summary]

## Research Problem
[Problem statement and significance]

## Research Questions
1. RQ1: ...
2. RQ2: ...

## Theoretical Framework
[Theories and key concepts]

## Methodology
| Aspect | Description |
|--------|-------------|
| Design | |
| Sample | |
| Data Collection | |
| Analysis | |

## Key Findings
- Finding 1
- Finding 2
- ...

## Contributions
- Theoretical:
- Practical:

## Limitations
-

## Future Research
-

## My Notes
[Personal reflections, connections to other work, questions]

## BibTeX
```bibtex
@article{...}
```
```

Begin deep paper analysis now.
