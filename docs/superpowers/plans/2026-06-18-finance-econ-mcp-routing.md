# Finance/Econ MCP Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the literature MCPB better for finance and economics while preserving a generic path for future community disciplines.

**Architecture:** Keep literature search improvements in the zero-dependency MCPB runtime. Add domain routing and diagnostics to `server/query.mjs` and `server/index.mjs`, then document FRED/SEC as a separate future data MCP surface instead of mixing data APIs into literature search.

**Tech Stack:** Node.js ESM, built-in `node:test`, Markdown docs, existing Python artifact/doc tests.

---

### Task 1: Finance/Econ Query Routing

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/query.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [x] Write a failing test that finance/econ deep search creates field-aware variants such as working-paper, JEL, and review variants.
- [x] Write a failing test that `search_plan.domain` is `finance_economics` when the query uses finance/econ terms.
- [x] Implement deterministic domain detection from finance/econ keyword families.
- [x] Implement finance/econ variants without affecting DOI or exact-title lookup.
- [x] Return `search_plan.domain` and domain-specific variant rationale.

### Task 2: Finance/Econ Diagnostics

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [x] Write a failing test that diagnostics include `field_term_coverage`, `working_paper_coverage`, and `published_version_coverage`.
- [x] Implement coverage using query plan terms and normalized result metadata.
- [x] Keep diagnostics generic for non-finance/econ searches by returning `domain: "general"` and neutral coverage values.
- [x] Ensure diagnostics never include provider secrets or raw credential fields.

### Task 3: Data MCP Design Boundary

**Files:**
- Add: `docs/advanced/finance-econ-data-mcp.md`
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `README.md`

- [x] Document why FRED and SEC EDGAR belong in a separate data MCP surface.
- [x] Define proposed tools `qiongli_finance_data_status` and `qiongli_finance_data_search`.
- [x] Document first-source scope: FRED/ALFRED for macro time series, SEC EDGAR JSON APIs for filings and company facts.
- [x] Document why RePEc/NBER should start as metadata enrichment and published-version linking, not as primary search providers.

### Task 4: Verify And Ship

**Files:**
- All modified MCPB runtime, tests, and docs.

- [x] Run `npm --prefix packages/qiongli-literature-mcpb test`.
- [x] Run `python3 -m unittest tests.test_literature_mcpb_artifact tests.test_mcp_provider_docs -v`.
- [x] Run `git diff --check`.
- [ ] Commit and push to `feat/mcp-literature-search-extension`.
