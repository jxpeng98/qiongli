# LR CLI Desktop Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create two supported Literature Review usage modes, full CLI and Claude Desktop ZIP, that share the same evidence, provenance, no-hallucination, and review-grade gating rules.

**Architecture:** Move LR search truth into one shared contract, then make CLI and Desktop consume that contract through different execution adapters. CLI can execute MCP/provider commands and run Python audits; Desktop ZIP stays under the 180-file upload budget and enforces the same contract through bundled workflow text, templates, diagnostics YAML, and a capability gate that blocks unsupported search execution.

**Tech Stack:** Python 3.12, existing Qiongli skill package materializer, MCP subprocess providers, Markdown/YAML contracts, CSV artifacts, pytest.

---

## File Structure

- Modify `qiongli-workflow/references/literature-search-quality-contract.md`: canonical shared LR search contract for CLI and Desktop.
- Create `qiongli-workflow/references/desktop-literature-search-boundary.md`: Desktop-specific capability and blocked-mode rules.
- Modify `qiongli-workflow/references/stage-B-literature.md`: Stage B must reference the shared contract and Desktop boundary.
- Modify `qiongli-workflow/workflows/lit-review.md`: add capability gate before search execution.
- Modify `skills/B_literature/academic-searcher.md`: make search execution mode explicit and forbid ungrounded result generation.
- Modify `templates/search-diagnostics.md`: add execution surface, capability mode, provider status, and blocked reasons.
- Modify `bridges/providers/literature_diagnostics.py`: parse and enforce the new diagnostics fields.
- Modify `bridges/providers/literature_artifacts.py`: materialize the expanded diagnostics fields and CSV schema.
- Modify `bridges/providers/literature_search.py`: include execution-surface metadata in provider output.
- Modify `scripts/audit_literature_search_quality.py`: enforce the same blocking gates from project artifacts or a diagnostics file.
- Modify `scripts/materialize_literature_search_bundle.py`: write the new schema.
- Modify `qiongli/subject_materializer.py`: ensure Desktop focused packages include the new contract and boundary files without exceeding 180 files.
- Modify `scripts/build_plugin_artifacts.py`: keep Desktop ZIP artifact generation aligned with the new required files.
- Test `tests/test_literature_search_quality_audit.py`: shared gates.
- Test `tests/test_literature_artifact_materialization.py`: materialized bundle fields.
- Test `tests/test_literature_search.py`: CLI provider output metadata.
- Test `tests/test_subject_materializer.py`: Desktop package includes required boundary files and remains under budget.
- Test `tests/test_claude_desktop_skill_artifact.py`: Desktop ZIP contains the new Desktop boundary and diagnostics template.
- Test `tests/test_plugin_artifacts.py`: release ZIPs still build.

## Shared Contract Rules

Both modes must produce or consume the same artifact contract:

- `search_strategy.md`: exact query strings, limits, date range, databases/providers.
- `search_log.md`: execution timestamp, provider, query ID, hit count, error state.
- `search_results.csv`: one row per candidate record.
- `dedup_log.csv`: merge/drop/keep decisions.
- `search_diagnostics.md`: machine-readable YAML gate state plus human-readable coverage notes.
- `bibliography.bib`: only for verified included or intentionally retained references.

Required `search_diagnostics.md` YAML fields:

```yaml
execution_surface: cli_full | desktop_zip
search_mode: systematic_review | targeted_search
capability_mode: provider_connected | user_supplied_corpus | strategy_only
review_grade: true | false
gate_status: pass | warn | fail | blocked
productive_providers: []
failed_providers: []
blocking_reasons: []
provider_coverage:
  minimum_required_productive_providers: 2
query_coverage:
  zero_hit_required_queries: []
known_item_recall:
  missing: []
source_policy:
  academic_only: true
  allow_unverified_candidates: false
```

Required `search_results.csv` columns:

```csv
record_id,source_provider,source_type,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract,academic_source_verified,evidence_limit
```

Allowed `capability_mode` behavior:

- `provider_connected`: may execute search and produce `search_results.csv`.
- `user_supplied_corpus`: may normalize, screen, extract, and summarize supplied records; must not claim a completed systematic search unless diagnostics show external search provenance.
- `strategy_only`: may produce `search_strategy.md` and a blocked diagnostics file; must not produce invented paper lists, PRISMA counts, or synthesis claims.

## Task 1: Contract And Desktop Boundary

**Files:**
- Modify: `qiongli-workflow/references/literature-search-quality-contract.md`
- Create: `qiongli-workflow/references/desktop-literature-search-boundary.md`
- Modify: `qiongli-workflow/references/stage-B-literature.md`
- Test: `tests/test_literature_search_contract_audit.py`

- [ ] **Step 1: Write failing contract tests**

Add assertions that the shared contract contains `execution_surface`, `capability_mode`, `source_policy`, `desktop_zip`, `strategy_only`, and the required CSV columns.

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_contract_audit.py -q
```

Expected: FAIL because the Desktop boundary file and new required fields do not exist yet.

- [ ] **Step 2: Add the Desktop boundary reference**

Create `qiongli-workflow/references/desktop-literature-search-boundary.md` with these concrete rules:

```markdown
# Desktop Literature Search Boundary

Desktop ZIP installs are skill-only packages. They can guide, validate, and summarize literature work, but they must not assume a local Python runtime or hidden provider execution.

## Capability Modes

- `provider_connected`: an explicit scholarly search provider or Desktop-accessible connector is available.
- `user_supplied_corpus`: the user supplied DOI, BibTeX, RIS, CSL JSON, CSV, PDF, or copied metadata.
- `strategy_only`: no executable provider and no user corpus are available.

## Hard Stops

When `capability_mode: strategy_only`, write `search_strategy.md` and `search_diagnostics.md` with `gate_status: blocked`. Do not create paper lists, citations, PRISMA counts, included-study claims, evidence synthesis, or bibliography entries.

When `capability_mode: user_supplied_corpus`, summarize only supplied records. Mark `review_grade: false` unless the supplied artifacts include reproducible provider logs satisfying the shared quality contract.

When `capability_mode: provider_connected`, apply the same provider coverage, known-item recall, query coverage, deduplication, and source-policy gates as CLI.

## Source Boundaries

Every record must include a source provider, retrieval timestamp, URL or DOI/provider ID, source type, academic source verification state, and evidence limit. Unsupported claims become gap notes.
```

- [ ] **Step 3: Update the shared quality contract**

Extend `literature-search-quality-contract.md` to name both execution surfaces and define the required YAML fields and CSV columns listed above.

- [ ] **Step 4: Update Stage B**

In `stage-B-literature.md`, add Desktop ZIP to the provider contract:

```markdown
Desktop ZIP execution must first classify `capability_mode`. If the mode is `strategy_only`, B1 is blocked after strategy and diagnostics. If the mode is `user_supplied_corpus`, B1 can process supplied records but cannot claim review-grade search coverage without reproducible provider logs.
```

- [ ] **Step 5: Run contract tests**

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_contract_audit.py -q
```

Expected: PASS.

## Task 2: Workflow And Skill Instructions

**Files:**
- Modify: `qiongli-workflow/workflows/lit-review.md`
- Modify: `skills/B_literature/academic-searcher.md`
- Test: `tests/test_literature_search_contract_audit.py`

- [ ] **Step 1: Write failing workflow tests**

Add tests that `lit-review.md` and `academic-searcher.md` mention `Search Capability Gate`, `provider_connected`, `user_supplied_corpus`, `strategy_only`, and the hard stop against invented citations.

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_contract_audit.py -q
```

Expected: FAIL until workflow text is updated.

- [ ] **Step 2: Add Phase 2.5 to `lit-review.md`**

Insert before Phase 3:

```markdown
### Phase 2.5: Search Capability Gate

Classify the execution surface and capability mode before any search result generation.

Set `execution_surface`:
- `cli_full` when running through Qiongli CLI/npm/source runtime with MCP/provider execution available.
- `desktop_zip` when running from an uploaded Desktop/Web skill ZIP.

Set `capability_mode`:
- `provider_connected` when an explicit scholarly search provider or connector is available.
- `user_supplied_corpus` when the user has supplied DOI/BibTeX/RIS/CSL JSON/CSV/PDF/metadata.
- `strategy_only` when neither provider nor corpus is available.

Hard stop: if `capability_mode: strategy_only`, write `search_strategy.md` and a blocked `search_diagnostics.md`, then ask the user to connect a provider or upload a corpus. Do not create `search_results.csv`, citations, PRISMA counts, screening decisions, or synthesis claims.
```

- [ ] **Step 3: Update `academic-searcher.md`**

Add an execution-mode section that states:

```markdown
The academic-searcher may only output records that come from an executed provider response or user-supplied corpus. It must not use model memory to invent papers, titles, authors, venues, DOIs, abstracts, hit counts, or PRISMA numbers.
```

- [ ] **Step 4: Run workflow tests**

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_contract_audit.py -q
```

Expected: PASS.

## Task 3: Diagnostics Template And Runtime Parsing

**Files:**
- Modify: `templates/search-diagnostics.md`
- Modify: `bridges/providers/literature_diagnostics.py`
- Modify: `scripts/audit_literature_search_quality.py`
- Test: `tests/test_literature_search_quality_audit.py`

- [ ] **Step 1: Write failing audit tests**

Add cases for:

- `desktop_zip + strategy_only` returns blocking failure.
- `desktop_zip + user_supplied_corpus + review_grade: true` fails unless provider logs satisfy two productive providers.
- `cli_full + systematic_review + one productive provider` fails.
- `targeted_search + one productive provider` warns but does not hard-block.

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_quality_audit.py -q
```

Expected: FAIL until parser and audit rules understand the new fields.

- [ ] **Step 2: Expand the diagnostics template**

Add the YAML block from the Shared Contract Rules section to `templates/search-diagnostics.md`.

- [ ] **Step 3: Update diagnostics parser**

Update `literature_diagnostics.py` so parsed diagnostics expose:

- `execution_surface`
- `capability_mode`
- `productive_providers`
- `failed_providers`
- `source_policy`
- `blocking_reasons`

Unknown `execution_surface` or `capability_mode` should produce an audit failure.

- [ ] **Step 4: Update audit logic**

In `audit_literature_search_quality.py`, apply these shared rules:

- `strategy_only` always blocks result generation and review-grade claims.
- `systematic_review` or `review_grade: true` requires at least two productive providers.
- `all_providers_failed`, `zero_hits`, `missing_known_items`, and `weak_screening_readiness` remain blocking.
- `targeted_search` can warn on single provider but must not allow exhaustive-coverage wording.

- [ ] **Step 5: Run audit tests**

Run:

```bash
uv run --with pytest pytest tests/test_literature_search_quality_audit.py -q
```

Expected: PASS.

## Task 4: CLI Provider Output And Materialization

**Files:**
- Modify: `bridges/providers/literature_search.py`
- Modify: `bridges/providers/literature_artifacts.py`
- Modify: `scripts/materialize_literature_search_bundle.py`
- Test: `tests/test_literature_search.py`
- Test: `tests/test_literature_artifact_materialization.py`

- [ ] **Step 1: Write failing runtime tests**

Add tests that provider JSON includes:

- `execution_surface: cli_full`
- `capability_mode: provider_connected` when a provider returns records
- `failed_providers` when a provider errors
- `source_policy.academic_only: true`

Add materialization tests that `search_results.csv` writes the expanded schema.

Run:

```bash
uv run --with pytest pytest tests/test_literature_search.py tests/test_literature_artifact_materialization.py -q
```

Expected: FAIL until runtime output and materializer are updated.

- [ ] **Step 2: Update provider metadata**

In `literature_search.py`, set default metadata:

```python
execution_surface = "cli_full"
capability_mode = "provider_connected"
source_policy = {"academic_only": True, "allow_unverified_candidates": False}
```

When every provider fails, return `gate_status: fail` with `blocking_reasons` including `all_providers_failed`.

- [ ] **Step 3: Update result rows**

In `literature_artifacts.py`, map provider records into the expanded CSV columns. Use:

- `source_provider` from provider name.
- `source_type` from provider metadata when present, otherwise `uncertain`.
- `academic_source_verified` as `yes` for known academic providers and `uncertain` for imported candidates.
- `evidence_limit` as `metadata_only` unless abstract or full text is actually available.

- [ ] **Step 4: Update materialization CLI**

Make `materialize_literature_search_bundle.py` write the new diagnostics block and expanded CSV schema from provider JSON.

- [ ] **Step 5: Run runtime tests**

Run:

```bash
uv run --with pytest pytest tests/test_literature_search.py tests/test_literature_artifact_materialization.py -q
```

Expected: PASS.

## Task 5: Desktop ZIP Packaging Under 180 Files

**Files:**
- Modify: `qiongli/subject_materializer.py`
- Modify: `scripts/build_plugin_artifacts.py`
- Test: `tests/test_subject_materializer.py`
- Test: `tests/test_claude_desktop_skill_artifact.py`
- Test: `tests/test_plugin_artifacts.py`

- [ ] **Step 1: Write failing Desktop package tests**

Add assertions that every Desktop ZIP contains:

- `qiongli/references/literature-search-quality-contract.md`
- `qiongli/references/desktop-literature-search-boundary.md`
- `qiongli/templates/search-diagnostics.md`
- `qiongli/workflows/lit-review.md`
- `qiongli/skills/registry.yaml`

Also assert file count remains `<= 180`.

Run:

```bash
uv run --with pytest pytest tests/test_subject_materializer.py tests/test_claude_desktop_skill_artifact.py tests/test_plugin_artifacts.py -q
```

Expected: FAIL until the Desktop package includes the new boundary file.

- [ ] **Step 2: Keep Desktop package focused**

Do not add CLI runtime directories to Desktop ZIP:

- Do not include `bridges/`.
- Do not include `scripts/`.
- Do not include `qiongli/` Python package internals.
- Do not include complete subject payloads.

Only include the shared contract, Desktop boundary, workflow, templates, standards, selected profiles, registry, and selected skill markdown already allowed by focused packaging.

- [ ] **Step 3: Update materializer required assets**

Ensure `subject_materializer.py` copies `references/desktop-literature-search-boundary.md` through existing reference-directory copy behavior. If focused packages later prune references, add an allowlist entry rather than copying the full runtime.

- [ ] **Step 4: Run Desktop package tests**

Run:

```bash
uv run --with pytest pytest tests/test_subject_materializer.py tests/test_claude_desktop_skill_artifact.py tests/test_plugin_artifacts.py -q
```

Expected: PASS and every Desktop package stays under 180 files.

## Task 6: Documentation For Two Usage Modes

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `docs/advanced/rigorous-literature-search.md`
- Modify: `docs/zh/advanced/rigorous-literature-search.md`
- Modify: `docs/advanced/mcp-providers-setup.md`
- Modify: `docs/zh/advanced/mcp-providers-setup.md`
- Test: `tests/test_mcp_provider_docs.py`
- Test: `tests/test_npm_package_contract.py`

- [ ] **Step 1: Write failing docs tests**

Add tests that docs mention:

- CLI full mode
- Desktop ZIP mode
- Desktop `strategy_only` hard stop
- shared `search_diagnostics.md` contract
- two productive providers for review-grade search

Run:

```bash
uv run --with pytest pytest tests/test_mcp_provider_docs.py tests/test_npm_package_contract.py -q
```

Expected: FAIL until docs are updated.

- [ ] **Step 2: Update install docs**

In README Desktop section, state:

```markdown
Desktop ZIP installs are skill-only packages. They enforce the same literature-review contract as CLI, but search execution requires either a Desktop-accessible scholarly provider/connector or a user-supplied corpus. Without either, Desktop mode stops at search strategy plus blocked diagnostics.
```

- [ ] **Step 3: Update rigorous search docs**

Add a two-mode table:

```markdown
| Mode | Execution | Allowed outputs | Review-grade condition |
|---|---|---|---|
| CLI full | MCP/provider subprocess + audits | full search bundle | at least two productive academic providers and passing diagnostics |
| Desktop ZIP | uploaded skill + available Desktop tools/corpus | strategy, diagnostics, corpus processing | same diagnostics; otherwise blocked or non-review-grade |
```

- [ ] **Step 4: Run docs tests**

Run:

```bash
uv run --with pytest pytest tests/test_mcp_provider_docs.py tests/test_npm_package_contract.py -q
```

Expected: PASS.

## Task 7: End-To-End Verification

**Files:**
- Existing tests only unless gaps are found.

- [ ] **Step 1: Run literature test group**

Run:

```bash
uv run --with pytest pytest tests/test_literature_query_planning.py tests/test_literature_search.py tests/test_literature_diagnostics.py tests/test_literature_artifact_materialization.py tests/test_literature_search_quality_audit.py tests/test_literature_pipeline_integration.py tests/test_mcp_connectors.py tests/test_fulltext_retrieval.py tests/test_metadata_registry.py tests/test_citation_graph.py -q
```

Expected: PASS.

- [ ] **Step 2: Run Desktop/package test group**

Run:

```bash
uv run --with pytest pytest tests/test_subject_materializer.py tests/test_claude_desktop_skill_artifact.py tests/test_plugin_artifacts.py tests/test_plugin_distribution_contract.py tests/test_distribution_payloads.py -q
```

Expected: PASS.

- [ ] **Step 3: Run standard validation**

Run:

```bash
.venv/bin/python scripts/validate_research_standard.py --strict
```

Expected: `0 failed`.

- [ ] **Step 4: Run orchestrator doctor**

Run:

```bash
.venv/bin/python -m bridges.orchestrator doctor --cwd .
```

Expected: no errors. Warnings for optional external-only MCP slots are acceptable if documented.

- [ ] **Step 5: Build release artifacts in a temp dir**

Run:

```bash
.venv/bin/python scripts/build_plugin_artifacts.py --dist-dir /private/tmp/qiongli-lr-parity-dist --tag "$(cat qiongli-workflow/VERSION)"
```

Expected: Desktop ZIP artifacts build, include the new boundary file, and stay under 180 files.

## Completion Criteria

- CLI and Desktop both reference one shared LR search contract.
- CLI full mode can execute providers and audit results.
- Desktop ZIP mode can enforce the same rules without shipping runtime code.
- Desktop `strategy_only` mode blocks paper-list and synthesis generation.
- Desktop `user_supplied_corpus` mode can process supplied records but cannot claim review-grade search unless reproducible provider logs satisfy the same gates.
- Review-grade claims require at least two productive academic providers in both modes.
- `search_results.csv` and `search_diagnostics.md` use the same schema in both modes.
- Desktop ZIP artifacts remain at or below 180 files.
- Tests listed above pass.

## Self-Review

- Spec coverage: The plan covers separate CLI and Desktop usage modes, shared constraints, Desktop 180-file budget, provider/MCP execution, no-hallucination boundaries, academic-only source policy, and consistency of outputs.
- Placeholder scan: No `TBD`, `TODO`, or deferred unspecified tasks remain.
- Type consistency: The same field names are used across diagnostics YAML, audit tests, runtime output, docs, and packaging checks.
