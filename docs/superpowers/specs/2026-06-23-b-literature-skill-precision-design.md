# B Literature Skill Precision Design

## Goal

Upgrade the Stage B literature skills from broad descriptive guidance into
precise, auditable execution contracts. The optimized skills should tell an
agent exactly when a Stage B task is ready, which MCP/provider layer owns each
action, which artifacts must be produced, what blocks review-grade claims, and
how to preserve evidence limits when only metadata, abstracts, or unavailable
full text are present.

The immediate focus is the canonical source under `content/skills/B_literature/`.
Generated package mirrors and release artifacts remain out of scope.

## Current Context

Stage B already has the right structural pieces:

- `content/workflow/references/stage-B-literature.md` defines B1-B6 task
  outputs, provider ownership, search diagnostics, retrieval manifests, and
  truthfulness boundaries.
- `content/skills/B_literature/` contains nine canonical skill cards:
  `academic-searcher`, `concept-extractor`, `paper-screener`,
  `fulltext-fetcher`, `citation-snowballer`, `paper-extractor`,
  `literature-mapper`, `citation-formatter`, and `reference-manager-bridge`.
- `content/skills/registry.yaml` provides discoverability metadata.
- `content/skills-core.md` provides the token-efficient runtime summary.
- `scripts/audit_skill_sections.py --strict` currently passes because every
  skill has the required high-level sections.

The remaining gap is semantic precision. Several B skill cards still mix
workflow contracts with long API reference blocks, direct provider instructions,
generic fallback language, and broad quality advice. That allows an agent to
follow the headings while still bypassing MCP/provider ownership, overclaiming
review-grade coverage, or inventing unsupported extraction details.

## Non-Goals

- Do not redesign the B1-B6 task taxonomy.
- Do not add a new top-level literature skill unless a current skill boundary is
  demonstrably wrong.
- Do not edit generated distribution payloads or release ZIP contents.
- Do not replace the existing Qiongli literature MCPB, Zotero companion, or
  provider tools.
- Do not turn B skills into exhaustive API manuals. Provider-specific mechanics
  belong in the MCP/provider layer or focused references.
- Do not claim systematic-review readiness from single-provider, unresolved, or
  diagnostics-missing search artifacts.

## Recommended Approach

Use a contract-first update:

1. Add a B-stage semantic audit that fails on the current descriptive patterns.
2. Rewrite the nine B skill cards to satisfy the audit and the Stage B playbook.
3. Sync `content/skills-core.md` so default-mode execution no longer carries
   obsolete direct API or Google Scholar fallback behavior.
4. Tighten B-stage `summary` and `when_to_use` entries in
   `content/skills/registry.yaml`.
5. Run structural and semantic validation before any implementation is treated
   as complete.

This keeps the existing Skill Quality Contract intact while adding a stronger
quality gate for the content that currently escapes section-level checks.

## Semantic Audit Design

Add a Stage B focused audit as a separate script:
`scripts/audit_b_literature_skill_precision.py`. Keep it separate from
`audit_skill_sections.py` because the checks are semantic, stage-specific, and
bound to literature-provider, Zotero, screening, and extraction behavior.

The audit should inspect only canonical Stage B markdown files under
`content/skills/B_literature/` and report actionable findings.

Required checks:

- Every B skill names its related Task ID or stage relation.
- Every B skill names concrete `RESEARCH/[topic]/...` artifact paths.
- Provider-facing skills name the MCP/provider owner instead of making direct
  web or API calls the default execution path.
- Search and screening skills define review-grade blockers and triage-only
  fallback behavior.
- Extraction and mapping skills preserve `source_anchor`, `evidence_limit`, and
  unsupported-gap behavior.
- Full-text retrieval uses controlled retrieval statuses and distinguishes the
  built-in planning stub from external resolvers such as Zotero or OA providers.
- Zotero/reference-manager guidance requires status check, dry-run first,
  explicit write, fill-blank default, no overwrite of user-curated fields, and
  import-file fallback.
- Skills avoid long provider API reference sections when those details are not
  the skill's core contract.
- `skills-core.md` B summaries do not instruct direct API fallback as the
  default path.

The first test pass should demonstrate failures against the current files. The
skill rewrites should then make the audit pass.

## Skill Rewrite Contract

### `academic-searcher`

Reframe as the owner of query planning, provider-backed retrieval, dedup-ready
records, and search diagnostics.

Required precision:

- Treat `scholarly-search` as the execution owner for discovery.
- Keep provider names, translated queries, filters, counts, and failures in
  `search_log.md`.
- Write `search_strategy.md`, `search_results.csv`, `dedup_log.csv`, and
  `search_diagnostics.md`.
- Require `search_diagnostics.md` before systematic-review or review-grade
  screening claims.
- Define blockers: fewer than two productive providers for review-grade work,
  unresolved known-item misses, zero-hit required concept blocks, and weak
  screening readiness.
- Make Google Scholar or manual web checks supplemental and logged, not the
  default reproducible pipeline.
- Remove or compress long direct API reference material.

### `concept-extractor`

Make B1_5 a precise pre-search concept contract.

Required precision:

- Decompose the research question into 2-5 concept buckets.
- Record synonyms, near misses, controlled vocabulary candidates, and excluded
  ambiguous terms.
- Draft Boolean blocks that can be pasted into `search_strategy.md`.
- Run or specify a seed recall test when seed papers exist.
- Mark missing seed papers as query gaps instead of broadening silently.

### `paper-screener`

Make screening explicitly dependent on search readiness and PRISMA-compatible
decision logging.

Required precision:

- Carry `record_id`, `query_id`, `source`, `relevance_reason`, and
  `diagnostic_flags` into screening rows.
- Allow triage when diagnostics are unresolved, but block review-grade claims
  until the diagnostics are resolved or a protocol-visible limitation is written.
- Keep title/abstract and full-text decisions separate.
- Require a specific exclusion reason for every excluded record.
- Use controlled full-text status values from the Stage B playbook.

### `fulltext-fetcher`

Align retrieval with the `fulltext-retrieval` provider boundary.

Required precision:

- Produce or update `retrieval_manifest.csv` and `screening/full_text.md`.
- Distinguish built-in planning statuses from actual resolver-backed downloads.
- Treat Zotero, Unpaywall, CORE, arXiv, PMC, and publisher sources as
  provenance values, not as undocumented side effects.
- Record version label, access URL, license, retrieval status, and reason for
  not retrieved.
- Avoid illegal or paywall-bypassing access instructions.

### `citation-snowballer`

Make snowballing a bounded corpus-expansion contract.

Required precision:

- Seed selection must cite included records, missing known items, diagnostic
  gaps, or a named rationale.
- Each round records direction, candidates, decision, match basis, dedup result,
  and saturation status.
- `citation-graph` owns expansion; `search_results.csv` and `dedup_log.csv`
  receive append-ready updates.
- Stop conditions include saturation, irrelevant drift, provider failure, and
  budget/time limits.

### `paper-extractor`

Make extraction source-anchored and evidence-limited.

Required precision:

- One `notes/{citekey}.md` per included paper.
- `extraction_table.md` preserves theory, method or identification, dataset or
  source, main findings, limitations, and quality-relevant fields.
- Every project-level extraction claim carries `source_anchor` and
  `evidence_limit`.
- If only metadata or abstract is available, mark fields as `metadata_only` or
  `abstract_only` and avoid inferring methods, sample sizes, results, or
  limitations from unavailable full text.
- Unsupported fields become `unsupported_gap`, not polished claims.

### `literature-mapper`

Make mapping a defensible taxonomy, not a narrative overview.

Required precision:

- Choose a clustering basis: mechanism, theory, method, context, population, or
  level of analysis.
- Require each cluster to name representative papers, evidence limits, shared
  claims, contradictions, gaps, and contribution implications.
- Prevent chronological paper lists from passing as maps.
- Write `RESEARCH/[topic]/literature/literature_map.md`.

### `citation-formatter`

Make citation formatting a metadata-integrity contract.

Required precision:

- Treat `bibliography.bib` as the canonical export target.
- Normalize DOI, year, venue, author, and citekey fields.
- Define duplicate citekey handling and conflict notes.
- Keep style examples concise; do not turn the skill into a complete citation
  style manual.
- Require missing required metadata to be flagged rather than invented.

### `reference-manager-bridge`

Align the skill with the local Zotero companion and import-file fallback.

Required precision:

- Zotero is a local reference database, not the default discovery provider.
- Search local Zotero only when explicitly requested.
- Local writes require `qiongli_zotero_status`, dry-run upsert, then explicit
  `dry_run: false`.
- Duplicate detection is DOI-first, then stable identifiers, then title/year.
- Default update policy fills blank Zotero fields and preserves user-curated
  fields.
- Companion unavailable means generate `references.json`, `references.ris`,
  `bibliography.bib`, and a `zotero-import-report.md`.
- Crossref verification enriches metadata and flags conflicts; it is not human
  verification.

## Registry And Core Summary Updates

Update only B-stage entries in `content/skills/registry.yaml`:

- Make `when_to_use` trigger-oriented rather than workflow-summary-oriented.
- Include concrete triggers such as unresolved search coverage, PRISMA
  screening, full-text retrieval, Zotero export/import, or citation graph
  expansion.
- Avoid broad strings like "Use when you need to format citations" when the
  relevant trigger is metadata integrity or export readiness.

Update only B-stage summaries in `content/skills-core.md`:

- Replace direct API fallback wording with MCP/provider ownership.
- Preserve token efficiency.
- Include blockers and output contracts only where an agent could otherwise make
  a harmful shortcut.

## Validation Plan

Use a RED-GREEN-REFACTOR loop for the skill work:

1. RED: add audit fixtures or unit tests that fail against current B skill
   content.
2. GREEN: rewrite the minimum B skill text and metadata needed to pass.
3. REFACTOR: remove duplicate prose, long API manuals, and obsolete fallback
   language while keeping the audit green.

Required commands after implementation:

```bash
python3 -m unittest tests.test_audit_skill_sections -v
python3 -m unittest tests.test_literature_search_quality_audit -v
python3 scripts/audit_skill_sections.py --strict
```

Additional expected command after the semantic audit exists:

```bash
python3 -m unittest tests.test_b_literature_skill_precision -v
python3 scripts/audit_b_literature_skill_precision.py --strict
```

If registry or core summary tests are touched, run the specific affected tests
as well.

## Rollout

Implement in this order:

1. Add semantic audit tests for Stage B precision.
2. Add the audit script.
3. Rewrite `academic-searcher`, `paper-screener`, `fulltext-fetcher`, and
   `reference-manager-bridge` first because they are the live MCP/Zotero path.
4. Rewrite `concept-extractor`, `citation-snowballer`, `paper-extractor`,
   `literature-mapper`, and `citation-formatter`.
5. Sync `skills-core.md` and `registry.yaml`.
6. Run validation.
7. Review the diff for repository boundary problems before release work.

## Implementation Decision

Implement the semantic audit as `scripts/audit_b_literature_skill_precision.py`
with tests in `tests/test_b_literature_skill_precision.py`. Do not fold these
checks into `audit_skill_sections.py` unless a later refactor introduces a
shared audit plugin model.
