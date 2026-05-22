# Qiongli Core Skills Deepening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current verifiable-but-still-broad Qiongli skill system into a more precise academic workflow, with the deepest first upgrade applied to literature search quality, reproducibility, and corpus traceability.

**Architecture:** Keep `skills/` as the canonical source, keep `qiongli-workflow/` and `plugins/qiongli/skills/qiongli-workflow/` as synchronized package outputs, and keep literature internals in the provider/MCP layer rather than creating new top-level skills. The literature search upgrade should convert the current Semantic Scholar baseline into a contract-driven discovery pipeline: structured query planning, provider translation, execution logging, result normalization, deduplication, search diagnostics, snowball handoff, and regression evaluation.

**Tech Stack:** Python stdlib, `unittest`, YAML/Markdown contracts, existing provider bridge modules under `bridges/providers/`, existing scripts under `scripts/`, and existing package sync via `scripts/sync_skill_package.sh`.

---

## Scope

This plan covers the next optimization round for:

- `B_literature`: search, concept expansion, screening handoff, citation graph, metadata, full-text handoff, literature mapping.
- `C_design`: method diagnostics refinement and venue-aware method requirements.
- `F_writing`: claim-driven writing and evidence-ledger integration.
- `H_submission` / `J_proofread`: final risk review, citation safety, submission-readiness gates.
- Cross-cutting evals and validators that keep these improvements from regressing.

Primary emphasis: **literature search capability**. Other stage improvements should use the better literature corpus and evidence ledger rather than inventing separate workflows.

## Current State Observations

- `bridges/providers/literature_search.py` currently builds up to four simple query variants from topic, explicit query, research question, keywords, and title fields.
- `scripts/mcp_scholarly_search.py` currently calls `run_scholarly_search()` with the built-in Semantic Scholar `search_paper()` function.
- `academic-searcher` already owns `search_strategy.md`, `search_log.md`, `search_results.csv`, and `dedup_log.csv`.
- `citation-graph`, `metadata-registry`, and `fulltext-retrieval` already have clear ownership boundaries in `standards/mcp-agent-capability-map.yaml`.
- The previous optimization added evidence ledger, citation risk, stage handoff, method diagnostics, venue profiles, and offline academic quality evals. This plan should deepen behavior without undoing those boundaries.

## Design Constraints

- Do not add a new top-level skill for query building, keyword expansion, provider adapters, or database connectors. These remain embedded subflows under `academic-searcher` or provider-layer modules.
- Keep all tests offline. Network-backed providers can exist, but tests must use stubbed provider responses.
- Preserve current public CLI/task shape. New behavior should be controlled by task-packet fields, templates, and provider internals.
- Every search artifact must remain reproducible: exact query string, provider, filters, timestamp, count, and failure state.
- Unsupported literature claims must become evidence-ledger `gap_note` rows, not manuscript prose.

---

## File Structure

### Core Provider Code

- Modify `bridges/providers/literature_search.py`
  - Keep backward-compatible `run_scholarly_search(task_packet, search_fn, retrieved_at=None)`.
  - Add structured query-plan construction, provider execution summaries, coverage diagnostics, richer dedup reasons, and seed-recall checks.

- Create `bridges/providers/literature_query.py`
  - Own concept blocks, query modes, query variants, provider translation, and validation of machine-readable search strategy payloads.

- Create `bridges/providers/literature_schema.py`
  - Own stable constants for search-result fields, dedup fields, query-plan fields, provider names, and controlled vocabularies.

- Modify `bridges/providers/s2_client.py`
  - Add optional year range, publication type, venue, and field filter parameters without breaking `search_paper(query, limit)`.

- Modify `bridges/providers/citation_graph.py`
  - Add support for search-diagnostic-informed seed selection and saturation metadata, while preserving existing seed extraction from `search_results.csv`, `bibliography.bib`, and `notes/`.

### Scripts

- Create `scripts/audit_literature_search_contract.py`
  - Validate `search_strategy.md`, `search_log.md`, `search_results.csv`, `dedup_log.csv`, and optional `search_diagnostics.md`.

- Modify `scripts/validate_research_standard.py`
  - Call the literature search audit in strict mode for package-level bundled templates/contracts.

- Modify `scripts/mcp_scholarly_search.py`
  - Surface the richer query plan, diagnostics, and provider summaries in the JSON result.

### Templates and References

- Modify `templates/search-strategy.md`
  - Add machine-readable query-plan block, concept blocks, provider translations, sensitivity probes, exclusion terms, known-item recall targets, and stopping rules.

- Modify `templates/search-log.md`
  - Add provider execution table, zero-hit table, partial failure table, search-date/version metadata, and count reconciliation.

- Create `templates/search-diagnostics.md`
  - Store coverage diagnostics, seed recall, provider overlap, query saturation, missed-known-item notes, and recommended next search actions.

- Modify `templates/dedup-log.csv`
  - Add or document fields for `duplicate_cluster_id`, `confidence`, `kept_reason`, and `dropped_reason` if compatible with current tests.

- Modify `qiongli-workflow/references/stage-B-literature.md`
  - Add the new search-depth protocol and diagnostic gates.

- Modify `skills/B_literature/academic-searcher.md`
  - Replace broad search prose with a concrete staged search protocol.

- Modify these B-stage skills only where they consume search outputs:
  - `skills/B_literature/paper-screener.md`
  - `skills/B_literature/citation-snowballer.md`
  - `skills/B_literature/literature-mapper.md`
  - `skills/B_literature/fulltext-fetcher.md`
  - `skills/B_literature/paper-extractor.md`

### Tests and Evals

- Create `tests/test_literature_query_planning.py`
- Extend `tests/test_literature_search.py`
- Extend `tests/test_citation_graph.py`
- Extend `tests/test_literature_pipeline_integration.py`
- Create `tests/test_literature_search_contract_audit.py`
- Extend `tests/test_academic_quality_evals.py`
- Add eval fixtures under `evals/academic_quality/cases/` for:
  - `review-grade-literature-search.yaml`
  - `known-item-recall-failure.yaml`
  - `cross-provider-deduplication.yaml`
  - `snowball-saturation.yaml`

---

## Phase 0: Baseline and Branch Hygiene

### Task 0.1: Start from a clean branch

**Files:** none

- [ ] Run `git status --short`.
  - Expected: clean or only intentional plan docs.
- [ ] Create a new branch from the integration branch after the previous commit is merged or accepted.
  - Command: `git switch -c codex/qiongli-literature-search-depth`
- [ ] Record baseline:
  - Command: `python3 scripts/validate_research_standard.py --strict`
  - Command: `python3 scripts/audit_skill_sections.py --strict`
  - Command: `python3 -m unittest tests.test_literature_search tests.test_literature_pipeline_integration -v`
- [ ] Commit only if a plan doc is intentionally added.

---

## Phase 1: Literature Search Contract v2

### Task 1.1: Define search schema constants

**Files:**
- Create `bridges/providers/literature_schema.py`
- Test `tests/test_literature_query_planning.py`

- [ ] Write failing tests for stable field sets:
  - `SEARCH_RESULT_FIELDS` includes `record_id`, `source`, `query_id`, `retrieved_at`, `paper_id`, `title`, `authors`, `year`, `venue`, `doi`, `url`, `abstract`, `citation_count`, `open_access_pdf_url`, `provider_rank`, `relevance_reason`.
  - `SEARCH_LOG_FIELDS` includes `query_id`, `provider`, `translated_query`, `filters`, `retrieved_at`, `raw_count`, `normalized_count`, `status`, `error`.
  - `QUERY_PLAN_REQUIRED_KEYS` includes `search_mode`, `concept_blocks`, `provider_translations`, `filters`, `known_items`, `stopping_rules`.
- [ ] Run `python3 -m unittest tests.test_literature_query_planning -v`.
  - Expected: fail because `literature_schema.py` does not exist.
- [ ] Implement constants in `bridges/providers/literature_schema.py`.
- [ ] Run the test again.
  - Expected: pass.
- [ ] Commit: `git commit -m "Add literature search schema contract"`.

### Task 1.2: Add query-plan model and validation

**Files:**
- Create `bridges/providers/literature_query.py`
- Test `tests/test_literature_query_planning.py`

- [ ] Add tests for `build_structured_query_plan(task_packet)`:
  - Given `paper_type=systematic-review`, `research_question`, `keywords`, and `venue_profile=chi`, it returns `search_mode=systematic_review`.
  - It creates 2-5 concept blocks with stable ids like `c1_population`, `c2_construct`, `c3_context`.
  - Each concept block has `terms`, `phrases`, `required`, `exclusions`, and `controlled_vocab`.
  - It creates at least one broad query, one precise query, and one sensitivity-probe query.
- [ ] Add tests for `validate_query_plan(plan)`:
  - It fails if no concept block is marked required.
  - It fails if a provider translation references an unknown concept id.
  - It fails if known items are declared but lack a title, DOI, or paper id.
- [ ] Implement with simple Python dataclasses or dict builders. Prefer dicts if that fits existing provider style.
- [ ] Keep current `build_query_variants(task_packet)` as a backward-compatible wrapper that calls the new planner and returns the old list shape.
- [ ] Run:
  - `python3 -m unittest tests.test_literature_query_planning tests.test_literature_search -v`
- [ ] Commit: `git commit -m "Add structured literature query planning"`.

### Task 1.3: Upgrade templates for reproducible search strategy

**Files:**
- Modify `templates/search-strategy.md`
- Modify `qiongli-workflow/templates/search-strategy.md` after sync
- Modify plugin mirror after sync
- Test `tests/test_literature_search_contract_audit.py`

- [ ] Add a machine-readable fenced block to `templates/search-strategy.md`:

```yaml
search_mode: systematic_review
concept_blocks:
  - id: c1_population
    label: Population or corpus
    required: true
    terms: []
    phrases: []
    controlled_vocab: []
    exclusions: []
provider_translations:
  - provider: semantic_scholar
    query_id: q1
    translated_query: ""
filters:
  year_start: ""
  year_end: ""
  language: ""
known_items: []
stopping_rules:
  max_rounds: 2
  stop_when_new_included_below: 3
```

- [ ] Add tests that the template includes all required keys.
- [ ] Run `python3 -m unittest tests.test_literature_search_contract_audit -v`.
- [ ] Commit: `git commit -m "Define reproducible search strategy template"`.

---

## Phase 2: Search Execution and Provider Translation

### Task 2.1: Translate query plans into provider-specific queries

**Files:**
- Modify `bridges/providers/literature_query.py`
- Test `tests/test_literature_query_planning.py`

- [ ] Add tests for `translate_query_for_provider(plan, provider)`:
  - `semantic_scholar`: joins concept blocks into a readable keyword query without unsupported Boolean syntax overreach.
  - `openalex`: outputs a translation object with `search`, `filter`, and `sort`.
  - `crossref`: outputs `query.bibliographic`, date filters, and type filters.
  - `arxiv`: outputs `all:`, `ti:`, and `abs:` style clauses for CS/math/stat topics.
  - Unknown provider returns a validation error, not a silent fallback.
- [ ] Implement provider translations as pure functions with no network calls.
- [ ] Ensure all translations include `query_id`, `provider`, `translated_query`, `filters`, and `rationale`.
- [ ] Run `python3 -m unittest tests.test_literature_query_planning -v`.
- [ ] Commit: `git commit -m "Add provider-specific literature query translation"`.

### Task 2.2: Expand `run_scholarly_search` execution logging

**Files:**
- Modify `bridges/providers/literature_search.py`
- Modify `scripts/mcp_scholarly_search.py`
- Test `tests/test_literature_search.py`

- [ ] Add tests that `run_scholarly_search()` returns:
  - `data.query_plan`
  - `data.provider_summaries`
  - `data.search_diagnostics`
  - `data.search_log[*].translated_query`
  - `data.search_log[*].filters`
- [ ] Keep `search_fn(query, limit)` compatibility by treating it as the default `semantic_scholar` execution function.
- [ ] Add optional `provider_fns` keyword-only argument:
  - `provider_fns: dict[str, Callable[[dict[str, object], int], dict[str, object]]] | None = None`
  - Preserve existing callers by defaulting to the old `search_fn`.
- [ ] Implement partial failure behavior:
  - If one provider fails but another returns hits, status is `warning`.
  - If all providers fail, status is `error`.
  - If all providers succeed but zero hits, status is `warning`.
- [ ] Run:
  - `python3 -m unittest tests.test_literature_search -v`
  - `python3 -m unittest tests.test_literature_pipeline_integration -v`
- [ ] Commit: `git commit -m "Upgrade scholarly search execution diagnostics"`.

### Task 2.3: Add optional Semantic Scholar filters without breaking compatibility

**Files:**
- Modify `bridges/providers/s2_client.py`
- Test `tests/test_s2_client.py`

- [ ] Add tests for URL construction:
  - Existing `search_paper("query", 10)` still builds the current endpoint fields.
  - New `search_paper("query", 10, year_start=2020, year_end=2025)` includes `year=2020-2025`.
  - New `search_paper("query", 10, fields=[...])` preserves required baseline fields.
- [ ] Implement optional keyword parameters only. Do not change the first two positional parameters.
- [ ] Run `python3 -m unittest tests.test_s2_client tests.test_literature_search -v`.
- [ ] Commit: `git commit -m "Support filtered Semantic Scholar search requests"`.

---

## Phase 3: Search Quality Diagnostics

### Task 3.1: Add search diagnostics computation

**Files:**
- Modify `bridges/providers/literature_search.py`
- Test `tests/test_literature_search.py`

- [ ] Add tests for `compute_search_diagnostics(query_plan, unique_results, search_log, dedup_log)`:
  - Flags `known_item_missing` when declared seed DOI/title is absent from results.
  - Flags `provider_undercoverage` when only one provider returns hits in review-grade mode.
  - Flags `query_too_narrow` when a required concept query returns zero hits.
  - Computes `dedup_ratio` as duplicate decisions divided by raw hits.
  - Computes `query_overlap` from repeated canonical records across query ids.
- [ ] Implement diagnostics as a dict with:
  - `status`: `ok`, `warning`, or `error`
  - `known_item_recall`: count and missing items
  - `provider_coverage`: counts by provider
  - `query_coverage`: counts by query id
  - `dedup_ratio`
  - `recommended_actions`
- [ ] Run `python3 -m unittest tests.test_literature_search -v`.
- [ ] Commit: `git commit -m "Add literature search quality diagnostics"`.

### Task 3.2: Add `search_diagnostics.md` template and audit

**Files:**
- Create `templates/search-diagnostics.md`
- Create `scripts/audit_literature_search_contract.py`
- Create `tests/test_literature_search_contract_audit.py`

- [ ] Add template sections:
  - `Search Scope`
  - `Known-Item Recall`
  - `Provider Coverage`
  - `Query Coverage`
  - `Deduplication Summary`
  - `Coverage Gaps`
  - `Next Search Actions`
- [ ] Implement audit checks:
  - `search_strategy.md` has machine-readable query plan.
  - `search_log.md` has provider, query, filters, timestamp, status, and counts.
  - `search_results.csv` has all required fields.
  - `dedup_log.csv` has decision and match basis.
  - `search_diagnostics.md`, if present, has the required sections.
- [ ] Run `python3 -m unittest tests.test_literature_search_contract_audit -v`.
- [ ] Commit: `git commit -m "Add literature search contract audit"`.

### Task 3.3: Connect audit to strict validation

**Files:**
- Modify `scripts/validate_research_standard.py`
- Test `tests/test_validate_project_artifacts.py` or new audit-specific tests

- [ ] Add a strict-mode package audit for bundled search templates and B-stage references.
- [ ] Do not require a real project `RESEARCH/[topic]/search_diagnostics.md` unless the task output contract declares it.
- [ ] Run:
  - `python3 scripts/validate_research_standard.py --strict`
  - `python3 -m unittest tests.test_literature_search_contract_audit tests.test_validate_project_artifacts -v`
- [ ] Commit: `git commit -m "Gate literature search contracts in strict validation"`.

---

## Phase 4: Better Deduplication, Ranking, and Result Normalization

### Task 4.1: Improve dedup matching

**Files:**
- Modify `bridges/providers/literature_search.py`
- Test `tests/test_literature_search.py`

- [ ] Add tests for duplicate matching by:
  - DOI normalization.
  - Semantic Scholar paper id.
  - arXiv id when present.
  - normalized title plus first author plus year.
  - title-only fallback with low confidence and manual-review note.
- [ ] Add `duplicate_cluster_id` and `dedup_confidence` to dedup log entries when possible.
- [ ] Keep the existing `candidate_record_id`, `canonical_record_id`, `decision`, `match_basis`, `resolver`, `notes` fields.
- [ ] Run `python3 -m unittest tests.test_literature_search -v`.
- [ ] Commit: `git commit -m "Improve literature deduplication evidence"`.

### Task 4.2: Add relevance and provenance fields

**Files:**
- Modify `bridges/providers/literature_search.py`
- Test `tests/test_literature_search.py`

- [ ] Add tests that normalized records include:
  - `provider_rank`
  - `relevance_reason`
  - `provider_url`
  - `external_ids`
  - `source_query`
- [ ] Set fields conservatively:
  - If provider does not supply relevance, set `relevance_reason` to `matched query terms; provider rank unavailable`.
  - If provider does not supply external ids, use `{}`.
- [ ] Ensure CSV writers in tests and docs include the new fields only where contract version says they are required.
- [ ] Run `python3 -m unittest tests.test_literature_search tests.test_literature_pipeline_integration -v`.
- [ ] Commit: `git commit -m "Add result relevance and provenance fields"`.

---

## Phase 5: Snowballing and Screening Handoff

### Task 5.1: Add diagnostic-driven seed selection

**Files:**
- Modify `bridges/providers/citation_graph.py`
- Test `tests/test_citation_graph.py`

- [ ] Add tests that seed extraction prioritizes:
  - User-declared known items.
  - Included or high-relevance records.
  - High-citation records only after relevance filters.
  - Diverse concept clusters when `literature/literature_map.md` exists.
- [ ] Add `seed_selection_reason` to resolved seed rows.
- [ ] Preserve current extraction from `search_results.csv`, `bibliography.bib`, and `notes/`.
- [ ] Run `python3 -m unittest tests.test_citation_graph -v`.
- [ ] Commit: `git commit -m "Prioritize citation graph seeds from search diagnostics"`.

### Task 5.2: Add snowball stopping and saturation diagnostics

**Files:**
- Modify `bridges/providers/citation_graph.py`
- Modify `qiongli-workflow/references/stage-B-literature.md`
- Test `tests/test_citation_graph.py`

- [ ] Add tests for saturation:
  - If a round returns fewer than the configured threshold of unique candidates, `saturation_status=near_saturation`.
  - If zero new candidates after dedup, `saturation_status=saturated`.
  - If provider failures prevent saturation judgment, `saturation_status=unknown_provider_failure`.
- [ ] Add fields to `snowball_log` entries:
  - `round`
  - `seed_selection_reason`
  - `saturation_status`
- [ ] Update Stage B reference with stopping rules.
- [ ] Run `python3 -m unittest tests.test_citation_graph tests.test_literature_pipeline_integration -v`.
- [ ] Commit: `git commit -m "Add citation snowball saturation tracking"`.

### Task 5.3: Make screening consume search diagnostics

**Files:**
- Modify `skills/B_literature/paper-screener.md`
- Modify `templates/search-diagnostics.md`
- Test `tests/test_skill_contract_alignment.py`

- [ ] Update `paper-screener` so title/abstract screening must carry forward:
  - `record_id`
  - `query_id`
  - `source`
  - `relevance_reason`
  - `diagnostic_flags`
- [ ] Add rule: if `search_diagnostics.md` contains `known_item_missing`, screening cannot be treated as review-grade until a gap note or revised search is recorded.
- [ ] Run `python3 scripts/audit_skill_sections.py --strict`.
- [ ] Commit: `git commit -m "Pass literature search diagnostics into screening"`.

---

## Phase 6: Skill Prose Deepening for B Literature

### Task 6.1: Rewrite `academic-searcher` as an operational protocol

**Files:**
- Modify `skills/B_literature/academic-searcher.md`
- Modify synced package after sync
- Test `tests/test_skill_doc_generation.py`, `tests/test_skill_contract_alignment.py`

- [ ] Replace broad query construction prose with this exact staged protocol:
  - Stage 1: Clarify search mode and review type.
  - Stage 2: Build concept blocks.
  - Stage 3: Expand terms using synonyms, acronyms, controlled vocabulary, and near-miss terms.
  - Stage 4: Translate queries per provider.
  - Stage 5: Execute provider runs and log exact strings.
  - Stage 6: Normalize, deduplicate, and compute diagnostics.
  - Stage 7: Decide whether to widen, narrow, snowball, or move to screening.
- [ ] Add quality bar:
  - At least 2 concept blocks.
  - At least 2 query variants for non-targeted search.
  - Search log has exact query strings and counts.
  - Known-item recall is checked when known items exist.
  - Review-grade mode cannot pass with only one source unless a rationale is logged.
- [ ] Run `python3 scripts/audit_skill_sections.py --strict`.
- [ ] Commit: `git commit -m "Deepen academic searcher protocol"`.

### Task 6.2: Add literature search examples without bloating `SKILL.md`

**Files:**
- Create `qiongli-workflow/references/literature-search-examples.md`
- Create root or synced source reference if source package uses root references
- Modify `qiongli-workflow/SKILL.md`
- Test `tests/test_skill_resource_links.py`

- [ ] Add examples for:
  - Management qualitative theory search.
  - CHI/HCI empirical search.
  - Biomedical systematic review search.
  - NeurIPS/ACL method search.
- [ ] Each example includes:
  - research question
  - concept blocks
  - provider translations
  - exclusion terms
  - known-item recall check
  - action after diagnostics
- [ ] Link the reference from `qiongli-workflow/SKILL.md` and relevant B-stage docs.
- [ ] Run `python3 -m unittest tests.test_skill_resource_links -v`.
- [ ] Commit: `git commit -m "Add literature search examples reference"`.

---

## Phase 7: Core Stage Optimizations Beyond Literature

### Task 7.1: C Design consumes literature diagnostics

**Files:**
- Modify `skills/C_design/study-designer.md`
- Modify `skills/C_design/robustness-planner.md`
- Modify `skills/C_design/variable-operationalizer.md`
- Modify `templates/method-diagnostic-report.md`
- Test `tests/test_method_diagnostics.py`

- [ ] Add rule: method diagnostics must cite the literature corpus boundary from `search_diagnostics.md` or `literature/literature_map.md`.
- [ ] Add method diagnostic fields:
  - `prior_design_pattern`
  - `identified_validity_threat`
  - `literature_supporting_source_id`
  - `design_implication`
- [ ] Run `python3 -m unittest tests.test_method_diagnostics -v`.
- [ ] Commit: `git commit -m "Link method diagnostics to literature evidence"`.

### Task 7.2: F Writing becomes claim-ledger-first

**Files:**
- Modify `skills/F_writing/manuscript-architect.md`
- Modify `skills/F_writing/analysis-interpreter.md`
- Modify `skills/F_writing/discussion-writer.md`
- Modify `templates/evidence-ledger.md`
- Test `tests/test_evidence_ledger_contract.py`

- [ ] Add rule: before drafting central claims, the writing skill must read or request `evidence/claim-evidence-ledger.csv`.
- [ ] Add rule: literature-backed claims must cite `source_id` present in search or bibliography artifacts.
- [ ] Add rule: if a claim is central but not supported, write a gap note and downgrade the manuscript claim.
- [ ] Run `python3 -m unittest tests.test_evidence_ledger_contract -v`.
- [ ] Commit: `git commit -m "Make writing skills evidence-ledger-first"`.

### Task 7.3: H/J final risk reviews consume search diagnostics

**Files:**
- Modify `skills/H_submission/submission-packager.md`
- Modify `skills/H_submission/fatal-flaw-detector.md`
- Modify `skills/J_proofread/final-proofreader.md`
- Modify `skills/J_proofread/similarity-checker.md`
- Modify `templates/citation-risk-report.md`
- Test `tests/test_citation_risk_audit.py`

- [ ] Add citation risk checks for:
  - Central claim cites a paper outside `bibliography.bib`.
  - Manuscript cites a source not found in `search_results.csv`, `bibliography.bib`, or manually logged supplement.
  - Literature review claims comprehensiveness while search diagnostics show undercoverage.
  - Citation pile has more than 4 citations without differentiating source roles.
- [ ] Run `python3 -m unittest tests.test_citation_risk_audit -v`.
- [ ] Commit: `git commit -m "Connect submission risk checks to literature provenance"`.

---

## Phase 8: Offline Evaluation Corpus

### Task 8.1: Add literature-specific eval cases

**Files:**
- Add `evals/academic_quality/cases/review-grade-literature-search.yaml`
- Add `evals/academic_quality/cases/known-item-recall-failure.yaml`
- Add `evals/academic_quality/cases/cross-provider-deduplication.yaml`
- Add `evals/academic_quality/cases/snowball-saturation.yaml`
- Modify `scripts/run_academic_quality_evals.py`
- Test `tests/test_academic_quality_evals.py`

- [ ] Add scoring dimensions:
  - `search_reproducibility`
  - `query_coverage`
  - `known_item_recall`
  - `dedup_integrity`
  - `provider_provenance`
  - `screening_readiness`
- [ ] Keep previous dimensions intact:
  - `artifact_completeness`
  - `evidence_traceability`
  - `no_fabricated_sources`
  - `claim_calibration`
  - `venue_fit`
  - `method_validity`
  - `scholarly_voice`
- [ ] Run `python3 -m unittest tests.test_academic_quality_evals -v`.
- [ ] Commit: `git commit -m "Add literature search quality eval cases"`.

### Task 8.2: Add release warning gate for search quality

**Files:**
- Modify `scripts/run_academic_quality_evals.py`
- Modify `scripts/release_preflight.sh` only if release flow already supports warning mode cleanly
- Test `tests/test_release_automation.py`

- [ ] Keep search quality eval as warning at first.
- [ ] Emit machine-readable summary:
  - `status`
  - `failed_dimensions`
  - `warning_dimensions`
  - `case_count`
- [ ] Do not block release until at least one subsequent stabilization pass.
- [ ] Run `python3 -m unittest tests.test_release_automation tests.test_academic_quality_evals -v`.
- [ ] Commit: `git commit -m "Report literature search evals as release warnings"`.

---

## Phase 9: Sync, Docs, and Final Verification

### Task 9.1: Sync portable and plugin packages

**Files:**
- Generated/synced under `qiongli-workflow/`
- Generated/synced under `plugins/qiongli/skills/qiongli-workflow/`

- [ ] Run `bash scripts/sync_skill_package.sh --target all`.
- [ ] Run `bash scripts/sync_skill_package.sh --target all --dry-run`.
  - Expected: self-contained package.
- [ ] Run `python3 -m unittest tests.test_install_manifest tests.test_plugin_distribution_contract tests.test_skill_resource_links -v`.
- [ ] Commit: `git commit -m "Sync qiongli literature search package artifacts"`.

### Task 9.2: Update user-facing docs

**Files:**
- Modify `docs/advanced/rigorous-literature-search.md`
- Modify `docs/zh/advanced/rigorous-literature-search.md`
- Modify `docs/reference/skills.md` if generated docs require regeneration
- Modify `docs/zh/reference/skills.md` if generated docs require regeneration

- [ ] Document the new review-grade search workflow:
  - structured query plan
  - provider translation
  - diagnostics
  - known-item recall
  - snowball saturation
  - screening readiness
- [ ] Keep Chinese and English docs aligned.
- [ ] Run doc-related tests:
  - `python3 -m unittest tests.test_skill_doc_generation tests.test_literature_contract -v`
- [ ] Commit: `git commit -m "Document review-grade literature search workflow"`.

### Task 9.3: Full final verification

**Files:** none

- [ ] Run `python3 scripts/validate_research_standard.py --strict`.
  - Expected: 0 failed, 0 warnings.
- [ ] Run `python3 scripts/audit_skill_sections.py --strict`.
  - Expected: all skills complete.
- [ ] Run `bash scripts/sync_skill_package.sh --target all --dry-run`.
  - Expected: package self-contained.
- [ ] Run `python3 -m unittest discover -s tests -v`.
  - Expected: all tests pass.
- [ ] Run `git status --short`.
  - Expected: clean after final commit.

---

## Literature Search Acceptance Criteria

The literature search upgrade is accepted only when all of these are true:

- `run_scholarly_search()` still supports the old simple `search_fn(query, limit)` signature.
- A structured query plan is created for topic/question/keyword task packets.
- Provider translations are deterministic and logged.
- `search_log` records exact translated query strings, filters, counts, timestamps, status, and errors.
- `search_results` rows carry stable ids, source, query provenance, DOI normalization, author/year/venue fields, relevance metadata, and OA hints.
- `dedup_log` explains merge/drop decisions with match basis and confidence.
- `search_diagnostics` catches missing known items, provider undercoverage, zero-hit concept blocks, high duplicate rates, and weak screening readiness.
- Citation snowballing can use search diagnostics to select seeds and report saturation.
- Screening skills consume search diagnostics before claiming review-grade readiness.
- Docs and templates teach users how to run lightweight, review-grade, and local-library search stacks.
- Full unittest discovery passes offline.

## Risk Controls

- Backward compatibility risk: keep existing function signatures and add keyword-only optional parameters.
- Scope creep risk: do not add new top-level B skills; embed search internals under `academic-searcher` and provider modules.
- False rigor risk: diagnostics should produce warnings/gap notes when evidence is weak instead of letting skill prose overclaim.
- Provider instability risk: all tests use fake provider functions and static fixtures; real network providers are integration behavior, not unit-test requirements.
- Package drift risk: every source change must sync to `qiongli-workflow/` and plugin mirror before final verification.

## Recommended Execution Order

1. Implement Phases 1-4 first. This creates the improved search engine and diagnostic substrate.
2. Implement Phase 5 next. This makes snowballing and screening benefit from the better corpus.
3. Implement Phase 6. This updates skill behavior once the provider contract exists.
4. Implement Phase 7. This lets design, writing, proofread, and submission consume stronger literature evidence.
5. Implement Phase 8. This prevents regressions with offline eval cases.
6. Finish with Phase 9 sync, docs, and full verification.

## Commit Cadence

Use one commit per task unless two adjacent tasks are inseparable. Do not make one giant commit for the whole round. A reviewer should be able to inspect query planning, execution diagnostics, dedup, snowballing, skill prose, evals, and package sync independently.

