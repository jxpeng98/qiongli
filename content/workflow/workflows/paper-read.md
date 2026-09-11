---
description: 阅读单篇论文或所给片段，按请求解释发现或生成可追溯的 B2 笔记
---

# Paper Reading

Explain the supplied paper at the requested depth or produce a formal targeted
reading record. Canonical Task ID: `B2` targeted paper reading.

## Paper

$ARGUMENTS

## Scope and access

Reuse the supplied paper and selected project. A question about an excerpt can
be answered in chat without retrieval, a search plan or a project folder. State
what was actually read: full text, abstract, metadata or supplied excerpt.
Do not claim a completed B2 run for a direct answer.

For a formal B2 run or external lookup, follow
`references/literature-provider-routing.md`. Create or update
`qiongli_search_plan` before retrieval; attempt visible
`qiongli_literature_status` to establish capability. If the plan tool is absent,
record the plan in the response or proposed artifact without claiming a call.
Keep these separate in the reading record:

- `provider_capability_mode`: `provider_connected` or `strategy_only`.
- `search_execution_mode`: `hybrid_search`, `provider_connected`, `native_only`
  or `strategy_only`, chosen by the workflow/router from actual tools.
- Source provenance: `user_corpus`, the actual `mcp:<provider>`, or
  `native:codex_web_search` / `native:claude_web_search`.
- `evidence_limit`: `full_text`, `abstract_only`, `metadata_only`, or `manual`
  for user-supplied metadata. An evidence limit does not change execution mode.

Use supplied material first. Retrieve missing metadata or full text only when
needed for this task; do not exhaust providers after the needed evidence is
available. Native search belongs to the active agent, never the MCP server.
Select `strategy_only` only when neither provider nor native search is available.
An abstract-only note must remain visibly limited; it cannot establish unseen
methods, figures, results or systematic-review-grade coverage.

## Evidence and interpretation

Use `paper-extractor` for structured extraction and `quality-assessor` when a
critical appraisal is requested or required by formal B2. Choose the relevant
questions about the problem, design, findings, contribution and limitations;
the paper's type and evidence determine the depth.

Every central claim needs a `source_anchor`: section, page, table, quotation,
abstract, metadata field or existing note anchor. Keep author claims, extracted
facts, agent interpretation and project relevance distinguishable. Label
inference strength as `direct_evidence`, `reasonable_inference` or
`unsupported_gap`. Never invent citations, anchors, sample sizes, measurement
procedures, results or implications to complete a template or rating.

## Formal B2 outputs

Reuse the known destination and existing citekey; ask only if the destination is
ambiguous. Use the templates for their field structure:

- `templates/paper-note.md` → `RESEARCH/[topic]/notes/[citekey].md`.
- Verified bibliographic metadata → `RESEARCH/[topic]/bibliography.bib`; use
  `metadata-enricher` if normalization or citekey reconciliation is needed.
- `templates/paper-reading-matrix.md` →
  `RESEARCH/[topic]/literature/paper_reading_matrix.md`.
- `templates/paper-reading-summary.md` →
  `RESEARCH/[topic]/literature/paper_reading_summary.md`.

Record the evidence limit, retrieval status/version, source anchors and inference
strength alongside findings, method, theory, limitations and project relevance.
Use `fulltext-fetcher` for required retrieval planning and record actual access
in `retrieval_manifest.csv`; do not repeat a completed retrieval.

Preserve human-written notes and prior source anchors. Propose a bounded merge;
when a safe merge is unclear, append a dated entry and keep unresolved material
in the uncertainty register. Registered project persistence requires the existing
preview/approval/CAS owner and a verified result.

For project records, follow `references/academic-graph-continuity.md`: reconcile
paper/claim candidates into the literature map and evidence ledger, reuse
citekeys and disambiguate note-local claim IDs. Do not invent clusters or support
from an abstract. A direct answer needs no graph run. Stop once the requested
answer or formal output contract is satisfied, and report remaining evidence gaps.
