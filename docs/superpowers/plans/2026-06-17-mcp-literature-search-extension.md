# MCP Literature Search Extension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Qiongli literature MCPB search tool use more configured provider capacity by default, expose explicit search controls, and add Crossref/PubMed plus limited citation/reference metadata expansion.

**Architecture:** Keep the implementation in the zero-dependency Node MCPB runtime. Provider adapters remain small ESM modules under `packages/qiongli-literature-mcpb/server/providers/`; `index.mjs` owns MCP schema, search option normalization, provider fan-out, filtering, and warning policy. Shared config continues to live in `config.mjs`, and plugin artifacts copy the full MCPB `server/` directory.

**Tech Stack:** Node.js ESM, `node:test`, Python `unittest` artifact tests, no runtime npm dependencies.

---

### Task 1: Preserve existing advanced search controls

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Add: `packages/qiongli-literature-mcpb/server/capabilities.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`

- [x] Add schema aliases for `per_provider_limit`, `total_limit`, `search_depth`, expansion flags, `document_types`, and `venue_filter`.
- [x] Add `search_options` and provider capability payloads.
- [x] Add OpenAlex document type filtering and Semantic Scholar publication type metadata.
- [x] Verify with `node --test packages/qiongli-literature-mcpb/test/tools.test.mjs packages/qiongli-literature-mcpb/test/providers.test.mjs`.

### Task 2: Promote Crossref and PubMed from planned to configurable providers

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/config.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/capabilities.mjs`
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
- Modify: `packages/qiongli-literature-mcpb/test/config.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [x] Write failing tests that `readConfig()` reads `crossref.email` and `pubmed.api_key` from MCPB env/shared config.
- [x] Write failing tests that `providerStatus()` marks configured Crossref/PubMed as usable and redacts raw values.
- [x] Write failing tests that `handleStatus()` reports Crossref/PubMed capabilities as `implemented`.
- [x] Implement `crossrefEmail` and `pubmedApiKey` in `readConfig()`.
- [x] Add manifest user config fields and environment wiring for `crossref_email` and `pubmed_api_key`.
- [x] Run `node --test packages/qiongli-literature-mcpb/test/config.test.mjs packages/qiongli-literature-mcpb/test/tools.test.mjs`.

### Task 3: Add Crossref and PubMed provider adapters

**Files:**
- Add: `packages/qiongli-literature-mcpb/server/providers/crossref.mjs`
- Add: `packages/qiongli-literature-mcpb/server/providers/pubmed.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [x] Write failing provider tests for Crossref search URL construction, DOI lookup, normalized bibliographic fields, reference metadata, and sanitized errors.
- [x] Write failing provider tests for PubMed ESearch + ESummary fan-out, year filters, API-key query params, normalized bibliographic fields, and sanitized errors.
- [x] Write failing search handler tests that configured Crossref/PubMed are included in provider fan-out.
- [x] Implement Crossref adapter with `query.bibliographic`, `rows`, `mailto`, year filters, document type filters, DOI singleton lookup, and normalized results.
- [x] Implement PubMed adapter with `esearch.fcgi`, `esummary.fcgi`, `retmode=json`, `retmax`, optional `api_key`, publication year filters, DOI query mode, and normalized results.
- [x] Wire adapters into `providersFor()` and `callProvider()`.
- [x] Run `node --test packages/qiongli-literature-mcpb/test/providers.test.mjs packages/qiongli-literature-mcpb/test/tools.test.mjs`.

### Task 4: Add limited citation/reference metadata expansion

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/crossref.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [x] Write failing tests that normalized results include `citation_count`, `reference_count`, `citations`, and `references`.
- [x] Write failing tests that Semantic Scholar requests citation/reference fields only when expansion flags are set.
- [x] Write failing tests that expansion warnings become `citation_expansion_limited` and `reference_expansion_limited`, not unavailable.
- [x] Implement optional linked-record normalization with stable minimal fields.
- [x] Map OpenAlex counts and referenced work IDs.
- [x] Map Semantic Scholar citation/reference arrays.
- [x] Map Crossref reference arrays and reference counts.
- [x] Run targeted Node tests.

### Task 5: Update docs, artifact tests, and verify

**Files:**
- Modify: `packages/qiongli-literature-mcpb/README.md`
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
- Modify: `tests/test_literature_mcpb_artifact.py`

- [x] Document Crossref/PubMed setup fields and limited expansion behavior.
- [x] Ensure MCPB artifact includes all new provider files.
- [x] Run `npm --prefix packages/qiongli-literature-mcpb test`.
- [x] Run `python3 -m unittest tests.test_literature_mcpb_artifact tests.test_plugin_artifacts tests.test_plugin_distribution_contract tests.test_plugin_manifests tests.test_mcp_provider_docs -v`.
- [x] Run `git diff --check`.
