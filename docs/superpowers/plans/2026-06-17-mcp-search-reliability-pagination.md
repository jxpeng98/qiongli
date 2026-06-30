# MCP Search Reliability And Pagination Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development for each behavior change and superpowers:verification-before-completion before committing or updating the PR.

**Goal:** Improve the literature MCPB search path with retry/backoff, provider pagination, higher review/deep limits, and structured diagnostics.

**Architecture:** Add one zero-dependency provider HTTP helper used by provider adapters. Keep pagination inside each provider adapter because cursor/offset semantics differ by API. Keep MCP-level diagnostics in `server/index.mjs` because it owns provider fan-out, dedupe, filtering, and returned results.

**Tech Stack:** Node.js ESM, built-in `node:test`, existing Python artifact tests.

---

### Task 1: Provider HTTP retry helper

**Files:**
- Add: `packages/qiongli-literature-mcpb/server/providers/http.mjs`
- Add: `packages/qiongli-literature-mcpb/test/http.test.mjs`
- Modify: provider adapters under `packages/qiongli-literature-mcpb/server/providers/`

- [ ] Write failing tests that retryable statuses `429`, `500`, `502`, `503`, and `504` retry up to three attempts.
- [ ] Write failing tests that non-retryable statuses return after one attempt.
- [ ] Write failing tests that error messages remain sanitized and do not include response bodies.
- [ ] Implement `fetchJsonWithRetry()`.
- [ ] Update provider adapters to use the helper.

### Task 2: Provider pagination and higher review/deep limits

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/capabilities.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/crossref.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/pubmed.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [ ] Write failing tests that OpenAlex uses cursor pagination above one page.
- [ ] Write failing tests that Semantic Scholar uses offset pagination above one page.
- [ ] Write failing tests that Crossref uses cursor pagination above one page.
- [ ] Write failing tests that PubMed uses `retstart` pagination above one page.
- [ ] Raise review/deep per-provider limits to 200 while preserving standard-mode defaults.
- [ ] Implement provider pagination with per-page size capped at 100.

### Task 3: Structured search diagnostics

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/README.md`

- [ ] Write failing tests that `handleSearch()` returns provider-level diagnostics.
- [ ] Include raw, deduped, filtered, returned result counts.
- [ ] Include provider result counts, failed providers, and retry attempt counts when providers expose them.
- [ ] Document diagnostics payload in README.

### Task 4: Verify and update PR branch

**Files:**
- All modified MCPB runtime, tests, and docs.

- [ ] Run `npm --prefix packages/qiongli-literature-mcpb test`.
- [ ] Run relevant Python artifact tests.
- [ ] Run `git diff --check` and syntax checks for new files.
- [ ] Commit and push to the existing feature branch.
