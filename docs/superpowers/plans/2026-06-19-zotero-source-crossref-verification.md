# Zotero Source And Crossref Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement opt-in Zotero source search for `qiongli_literature_search` and safer Zotero writes with Crossref DOI verification, conservative enrichment, and review-state tags.

**Architecture:** Keep the current local-first Zotero bridge. Add small MCPB-side modules for Zotero source search, Crossref verification, and review tag policy; wire them into `server/index.mjs` and `server/zotero/tools.mjs` without making Zotero a default discovery provider. Reuse the existing Crossref provider and Zotero companion endpoints.

**Tech Stack:** Node 18 ESM, `node:test`, Python `unittest`, zero runtime dependencies for the MCPB, existing Zotero companion JavaScript.

---

## File Structure

- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
  - Add tool schema fields for `include_zotero`, `zotero_limit`, `zotero_tag`, and `zotero_collection_path`.
  - Call Zotero source search only when `include_zotero: true`.
  - Add Zotero provider accounting and diagnostics without changing default provider behavior.
- Modify: `packages/qiongli-literature-mcpb/server/diagnostics.mjs`
  - Preserve `source_type` in provider and query diagnostics when a provider response supplies it.
- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
  - Preserve optional fields used by local Zotero results: `source_type`, `zotero`, `local_zotero_match`, `verification`, and `review_status`.
- Create: `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`
  - Convert literature search intent into companion search payloads.
  - Normalize compact Zotero companion results to Qiongli result records.
  - Annotate external results with `local_zotero_match`.
  - Convert companion failures into non-fatal warnings and provider diagnostics.
- Create: `packages/qiongli-literature-mcpb/server/zotero/crossref-verifier.mjs`
  - Use existing `searchCrossref` DOI singleton lookup for DOI-bearing records.
  - Fill blank metadata fields from Crossref.
  - Report `verified`, `conflict`, `not_found`, `unavailable`, or `skipped`.
- Create: `packages/qiongli-literature-mcpb/server/zotero/review-tags.mjs`
  - Generate default review tags and Crossref status tags.
  - Merge user-supplied tags without duplicates.
  - Resolve default review collection path from topic when supplied.
- Modify: `packages/qiongli-literature-mcpb/server/zotero/records.mjs`
  - Preserve `verification`, `review_status`, and enriched tags through normalization.
  - Map review tags into Zotero item tags.
- Modify: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
  - Run Crossref verification before mapping Zotero items.
  - Add review tags by default.
  - Return per-record verification traces and review status.
- Modify: `packages/qiongli-literature-mcpb/server/zotero/exporters.mjs`
  - Include review tags in CSL-JSON, RIS, and BibTeX exports.
  - Add verification counts to `zotero-import-report.md`.
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
  - Declare new tool schema fields and user config for default review tags, review collection, and Crossref verification.
- Modify: `packages/qiongli-literature-mcpb/README.md`
  - Document explicit Zotero source search and Crossref verification.
- Modify: `docs/advanced/mcp-zotero-integration.md`
  - Document default-off `include_zotero`, review tags, and Crossref DOI metadata limits.
- Modify: `docs/zh/advanced/mcp-zotero-integration.md`
  - Mirror the English docs.
- Modify: `content/skills/B_literature/reference-manager-bridge.md`
  - Update skill guidance for optional local Zotero source search and candidate-review imports.
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
  - Add schema, search integration, and routing tests.
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
  - Add Zotero source helper, Crossref verifier, review tag, and upsert tests.
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
  - Add normalizer preservation tests beside the existing normalize tests.
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
  - Add a narrow assertion that companion upsert preserves incoming review tags.
- Modify: `tests/test_literature_mcpb_artifact.py`
  - Assert new MCPB packaging expectations.

## Task 1: Baseline And Schema Tests

**Files:**
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `tests/test_literature_mcpb_artifact.py`

- [ ] **Step 1: Run baseline tests**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
npm --prefix packages/qiongli-zotero-companion test
python3 -m unittest tests.test_literature_mcpb_artifact tests.test_zotero_companion_artifact
```

Expected: all tests pass before the new feature work starts.

- [ ] **Step 2: Write failing tool schema tests**

Append these assertions to the existing `zotero tool schemas expose dry-run and import fallback controls` test in `packages/qiongli-literature-mcpb/test/tools.test.mjs`:

```js
  const searchTool = TOOL_DECLARATIONS.find((tool) => tool.name === "qiongli_literature_search");
  assert.equal(searchTool.inputSchema.properties.include_zotero.type, "boolean");
  assert.equal(searchTool.inputSchema.properties.zotero_limit.type, "number");
  assert.equal(searchTool.inputSchema.properties.zotero_tag.type, "string");
  assert.equal(searchTool.inputSchema.properties.zotero_collection_path.type, "string");
  assert.equal(upsertTool.inputSchema.properties.verify_crossref.type, "boolean");
  assert.deepEqual(upsertTool.inputSchema.properties.crossref_enrichment.enum, ["fill_blank", "off"]);
  assert.ok(upsertTool.inputSchema.properties.review_tags);
  assert.ok(upsertTool.inputSchema.properties.review_collection_path);
```

Add this assertion to `test_literature_mcpb_manifest_declares_sensitive_config` in `tests/test_literature_mcpb_artifact.py`:

```python
self.assertIn("zotero_default_review_tags", manifest["user_config"])
self.assertIn("zotero_default_review_collection_path", manifest["user_config"])
self.assertIn("zotero_crossref_verification_enabled", manifest["user_config"])
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: FAIL because schema fields and manifest config do not exist yet.

- [ ] **Step 4: Add minimal schema and manifest entries**

In `packages/qiongli-literature-mcpb/server/index.mjs`, add these properties to the `qiongli_literature_search` schema:

```js
        include_zotero: {
          type: "boolean",
          default: false
        },
        zotero_limit: {
          type: "number"
        },
        zotero_tag: {
          type: "string"
        },
        zotero_collection_path: {
          type: "string"
        },
```

Add these properties to the `qiongli_zotero_upsert_references` schema:

```js
        verify_crossref: {
          type: "boolean",
          default: true
        },
        crossref_enrichment: {
          type: "string",
          enum: ["fill_blank", "off"],
          default: "fill_blank"
        },
        review_tags: {
          type: "array",
          items: {
            type: "string"
          }
        },
        review_collection_path: {
          type: "string"
        },
```

In `packages/qiongli-literature-mcpb/manifest.json`, add user config fields:

```json
    "zotero_default_review_tags": {
      "type": "string",
      "title": "Default Zotero review tags",
      "description": "Comma-separated Qiongli tags added to Zotero imports that still need review.",
      "default": "qiongli:imported,qiongli:needs-review"
    },
    "zotero_default_review_collection_path": {
      "type": "string",
      "title": "Default Zotero review collection path",
      "description": "Optional collection path for newly imported references that need review.",
      "default": ""
    },
    "zotero_crossref_verification_enabled": {
      "type": "boolean",
      "title": "Verify Zotero imports with Crossref DOI metadata",
      "description": "When enabled, DOI-bearing Zotero imports use Crossref registry metadata to fill blank fields and report conflicts.",
      "default": true
    }
```

Add these env mappings under `server.mcp_config.env`:

```json
        "QIONGLI_ZOTERO_DEFAULT_REVIEW_TAGS": "${user_config.zotero_default_review_tags}",
        "QIONGLI_ZOTERO_DEFAULT_REVIEW_COLLECTION_PATH": "${user_config.zotero_default_review_collection_path}",
        "QIONGLI_ZOTERO_CROSSREF_VERIFICATION_ENABLED": "${user_config.zotero_crossref_verification_enabled}"
```

- [ ] **Step 5: Verify schema tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: PASS for the new schema assertions.

- [ ] **Step 6: Commit schema work**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/index.mjs packages/qiongli-literature-mcpb/manifest.json packages/qiongli-literature-mcpb/test/tools.test.mjs tests/test_literature_mcpb_artifact.py
git commit -m "feat(zotero): expose source and verification options"
```

## Task 2: Preserve Zotero Metadata In Normalized Results

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`

- [ ] **Step 1: Write failing normalizer preservation test**

Add this test near the existing `normalizeResult` tests in `packages/qiongli-literature-mcpb/test/providers.test.mjs`:

```js
test("normalizeResult preserves local Zotero metadata and verification fields", () => {
  const normalized = normalizeResult({
    title: "Local Paper",
    year: 2024,
    doi: "https://doi.org/10.1000/local",
    provider: "zotero",
    source_id: "ABC123",
    source_type: "local_reference_database",
    zotero: {
      item_key: "ABC123",
      select_uri: "zotero://select/library/items/ABC123",
      tags: ["qiongli:needs-review"],
      collections: ["Qiongli/topic"]
    },
    local_zotero_match: {
      item_key: "ABC123",
      match_basis: "doi",
      select_uri: "zotero://select/library/items/ABC123"
    },
    review_status: "needs_review",
    verification: {
      crossref: {
        status: "verified",
        doi: "10.1000/local",
        filled_fields: ["venue"],
        conflicts: []
      }
    }
  });

  assert.equal(normalized.source_type, "local_reference_database");
  assert.equal(normalized.zotero.item_key, "ABC123");
  assert.equal(normalized.local_zotero_match.match_basis, "doi");
  assert.equal(normalized.review_status, "needs_review");
  assert.equal(normalized.verification.crossref.status, "verified");
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/providers.test.mjs
```

Expected: FAIL because `normalizeResult` drops these optional fields.

- [ ] **Step 3: Preserve optional metadata in `normalize.mjs`**

Add helper functions after `normalizeLinkedRecords`:

```js
function clonePlainObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  return JSON.parse(JSON.stringify(value));
}

function normalizeSourceType(value) {
  return cleanString(value);
}
```

Add these properties at the end of the object returned by `normalizeResult`:

```js
    source_type: normalizeSourceType(record?.source_type),
    zotero: clonePlainObject(record?.zotero),
    local_zotero_match: clonePlainObject(record?.local_zotero_match),
    review_status: cleanString(record?.review_status),
    verification: clonePlainObject(record?.verification)
```

- [ ] **Step 4: Verify normalizer tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/providers.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit normalizer preservation**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/normalize.mjs packages/qiongli-literature-mcpb/test/providers.test.mjs
git commit -m "feat(mcpb): preserve zotero result metadata"
```

## Task 3: Zotero Source Search Helper

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [ ] **Step 1: Write failing helper tests**

Import the helpers in `packages/qiongli-literature-mcpb/test/zotero.test.mjs`:

```js
import {
  annotateLocalZoteroMatches,
  normalizeZoteroSourceResults,
  resolveZoteroSourceOptions,
  zoteroSourceSearchPayload
} from "../server/zotero/search-source.mjs";
```

Add these tests:

```js
test("resolveZoteroSourceOptions keeps Zotero disabled unless explicitly requested", () => {
  assert.equal(resolveZoteroSourceOptions({}).include, false);
  assert.equal(resolveZoteroSourceOptions({ include_zotero: false }).include, false);
  assert.equal(resolveZoteroSourceOptions({ include_zotero: true, zotero_limit: 7 }).include, true);
  assert.equal(resolveZoteroSourceOptions({ include_zotero: true, zotero_limit: 7 }).limit, 7);
});

test("zoteroSourceSearchPayload maps DOI and topic intents conservatively", () => {
  assert.deepEqual(
    zoteroSourceSearchPayload({
      intent: { doi: "10.1000/example", query: "10.1000/example", exactTitle: false },
      input: {}
    }),
    { doi: "10.1000/example" }
  );

  assert.deepEqual(
    zoteroSourceSearchPayload({
      intent: { doi: null, query: "platform governance", exactTitle: false },
      input: { zotero_tag: "project:platform", zotero_collection_path: "Qiongli/platform" }
    }),
    { title: "platform governance", tag: "project:platform", collection_path: "Qiongli/platform" }
  );
});

test("normalizeZoteroSourceResults maps compact local items to provider results", () => {
  const results = normalizeZoteroSourceResults([
    {
      item_key: "ABC123",
      title: "Local Paper",
      doi: "10.1000/local",
      year: 2024,
      item_type: "journalArticle",
      select_uri: "zotero://select/library/items/ABC123",
      tags: ["qiongli:verified"],
      collections: ["Qiongli/topic"]
    }
  ]);

  assert.equal(results[0].provider, "zotero");
  assert.equal(results[0].source_type, "local_reference_database");
  assert.equal(results[0].source_id, "ABC123");
  assert.equal(results[0].zotero.item_key, "ABC123");
});

test("annotateLocalZoteroMatches marks external DOI matches without dropping results", () => {
  const annotated = annotateLocalZoteroMatches({
    externalResults: [
      { title: "External Paper", doi: "10.1000/match", year: 2024, provider: "openalex" },
      { title: "Other Paper", doi: "10.1000/other", year: 2024, provider: "openalex" }
    ],
    zoteroResults: [
      {
        title: "Local Paper",
        doi: "10.1000/match",
        year: 2024,
        provider: "zotero",
        zotero: { item_key: "ABC123", select_uri: "zotero://select/library/items/ABC123" }
      }
    ]
  });

  assert.equal(annotated[0].local_zotero_match.item_key, "ABC123");
  assert.equal(annotated[0].local_zotero_match.match_basis, "doi");
  assert.equal(annotated[1].local_zotero_match, undefined);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because `search-source.mjs` does not exist.

- [ ] **Step 3: Implement `search-source.mjs`**

Create `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`:

```js
import { postCompanionJson } from "./client.mjs";

const DEFAULT_ZOTERO_LIMIT = 25;
const MAX_ZOTERO_LIMIT = 200;

export function resolveZoteroSourceOptions(input = {}, { perProviderLimit = DEFAULT_ZOTERO_LIMIT } = {}) {
  const include = input.include_zotero === true;
  const requestedLimit = integerOrNull(input.zotero_limit);
  const fallbackLimit = Math.max(1, Math.min(perProviderLimit, DEFAULT_ZOTERO_LIMIT));
  return {
    include,
    limit: include ? Math.min(Math.max(requestedLimit ?? fallbackLimit, 1), MAX_ZOTERO_LIMIT) : 0,
    tag: cleanString(input.zotero_tag),
    collection_path: cleanString(input.zotero_collection_path)
  };
}

export function zoteroSourceSearchPayload({ intent = {}, input = {}, sourceOptions = {} } = {}) {
  const payload = {};
  if (intent.doi) {
    payload.doi = intent.doi;
  } else {
    payload.title = cleanString(intent.query);
  }
  if (sourceOptions.tag ?? cleanString(input.zotero_tag)) {
    payload.tag = sourceOptions.tag ?? cleanString(input.zotero_tag);
  }
  if (sourceOptions.collection_path ?? cleanString(input.zotero_collection_path)) {
    payload.collection_path = sourceOptions.collection_path ?? cleanString(input.zotero_collection_path);
  }
  if (sourceOptions.limit) {
    payload.limit = sourceOptions.limit;
  }
  return payload;
}

export async function searchZoteroSource({ config, intent, input, sourceOptions, context = {} }) {
  const payload = zoteroSourceSearchPayload({ intent, input, sourceOptions });
  try {
    const response = await postCompanionJson(config, "/qiongli/search", payload, context);
    if (response.status === "error") {
      return {
        provider: "zotero",
        query_id: "zotero",
        query: payload.doi ?? payload.title ?? "",
        results: [],
        error: response.error_code ?? "zotero_companion_missing",
        request_count: 1,
        attempts: 1,
        source_type: "local_reference_database"
      };
    }
    return {
      provider: "zotero",
      query_id: "zotero",
      query: payload.doi ?? payload.title ?? "",
      results: normalizeZoteroSourceResults(response.results).slice(0, sourceOptions.limit),
      error: null,
      request_count: 1,
      attempts: 1,
      source_type: "local_reference_database"
    };
  } catch (error) {
    return {
      provider: "zotero",
      query_id: "zotero",
      query: payload.doi ?? payload.title ?? "",
      results: [],
      error: zoteroSourceErrorCode(error),
      request_count: 1,
      attempts: 1,
      source_type: "local_reference_database"
    };
  }
}

export function normalizeZoteroSourceResults(items = []) {
  return (Array.isArray(items) ? items : []).map((item) => ({
    title: cleanString(item.title),
    authors: Array.isArray(item.authors) ? item.authors : [],
    year: integerOrNull(item.year),
    doi: cleanString(item.doi),
    url: cleanString(item.url),
    abstract: cleanString(item.abstract),
    venue: cleanString(item.venue),
    document_type: cleanString(item.item_type ?? item.document_type),
    citation_count: null,
    reference_count: null,
    citations: [],
    references: [],
    provider: "zotero",
    source_id: cleanString(item.item_key),
    source_type: "local_reference_database",
    zotero: {
      item_key: cleanString(item.item_key),
      select_uri: cleanString(item.select_uri),
      tags: Array.isArray(item.tags) ? item.tags.filter(Boolean) : [],
      collections: Array.isArray(item.collections) ? item.collections.filter(Boolean) : []
    }
  })).filter((item) => item.title || item.doi || item.source_id);
}

export function annotateLocalZoteroMatches({ externalResults = [], zoteroResults = [] } = {}) {
  const byDoi = new Map();
  const byTitleYear = new Map();
  for (const result of zoteroResults) {
    if (result.doi) {
      byDoi.set(String(result.doi).toLowerCase(), result);
    }
    const titleKey = titleYearKey(result);
    if (titleKey) {
      byTitleYear.set(titleKey, result);
    }
  }

  return externalResults.map((result) => {
    const doiMatch = result.doi ? byDoi.get(String(result.doi).toLowerCase()) : null;
    const titleMatch = doiMatch ? null : byTitleYear.get(titleYearKey(result));
    const match = doiMatch ?? titleMatch;
    if (!match) {
      return result;
    }
    return {
      ...result,
      local_zotero_match: {
        item_key: match.zotero?.item_key ?? match.source_id ?? "",
        match_basis: doiMatch ? "doi" : "title_year",
        select_uri: match.zotero?.select_uri ?? ""
      }
    };
  });
}

export function zoteroSourceWarning(response) {
  if (!response?.error) {
    return null;
  }
  return response.error === "zotero_not_running" ? "zotero_not_running" : "zotero_companion_missing";
}

function zoteroSourceErrorCode(error) {
  const message = String(error?.message ?? "");
  return message.includes("ECONNREFUSED") || message.includes("fetch failed")
    ? "zotero_not_running"
    : "zotero_companion_missing";
}

function titleYearKey(record) {
  const title = comparableTitle(record?.title);
  if (!title) {
    return "";
  }
  return `${title}:${integerOrNull(record?.year) ?? ""}`;
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^\p{Letter}\p{Number}]+/gu, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function integerOrNull(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  const numeric = typeof value === "number" ? value : Number(String(value ?? "").trim());
  return Number.isFinite(numeric) ? Math.trunc(numeric) : null;
}

function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}
```

- [ ] **Step 4: Verify helper tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit source helper**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/zotero/search-source.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs
git commit -m "feat(zotero): normalize local source results"
```

## Task 4: Integrate Opt-In Zotero Source Into Main Search

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/diagnostics.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [ ] **Step 1: Write failing default-off search test**

Add this test to `packages/qiongli-literature-mcpb/test/tools.test.mjs` after the blank-query test:

```js
test("handleSearch does not call Zotero source by default", async () => {
  const urls = [];
  const response = await handleSearch(
    { query: "platform governance", per_provider_limit: 1 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
      },
      fetchImpl: async (url) => {
        urls.push(String(url));
        return {
          ok: true,
          status: 200,
          json: async () => ({
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Platform Governance",
                publication_year: 2024,
                doi: "https://doi.org/10.1000/platform"
              }
            ]
          })
        };
      }
    }
  );

  assert.equal(response.status, "ok");
  assert.deepEqual(response.providers.attempted, ["openalex"]);
  assert.equal(urls.some((url) => url.includes("/qiongli/search")), false);
});
```

- [ ] **Step 2: Write failing opt-in Zotero search test**

Add:

```js
test("handleSearch includes Zotero source only when requested", async () => {
  const calls = [];
  const response = await handleSearch(
    { query: "platform governance", include_zotero: true, zotero_limit: 5, per_provider_limit: 1 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
      },
      fetchImpl: async (url, options = {}) => {
        calls.push({ url: String(url), body: options.body ? JSON.parse(options.body) : null });
        if (String(url).endsWith("/qiongli/search")) {
          return {
            ok: true,
            status: 200,
            json: async () => ({
              status: "ok",
              results: [
                {
                  item_key: "ZOT123",
                  title: "Local Platform Governance",
                  doi: "10.1000/local-platform",
                  year: 2023,
                  item_type: "journalArticle",
                  select_uri: "zotero://select/library/items/ZOT123",
                  tags: ["qiongli:verified"],
                  collections: ["Qiongli/platform"]
                }
              ]
            })
          };
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Platform Governance",
                publication_year: 2024,
                doi: "https://doi.org/10.1000/platform"
              }
            ]
          })
        };
      }
    }
  );

  assert.deepEqual(response.providers.attempted, ["openalex", "zotero"]);
  assert.equal(response.providers.successful.includes("zotero"), true);
  assert.equal(response.results.some((result) => result.provider === "zotero"), true);
  assert.equal(response.results.find((result) => result.provider === "zotero").source_type, "local_reference_database");
  assert.equal(response.diagnostics.providers.find((provider) => provider.provider === "zotero").source_type, "local_reference_database");
  assert.equal(calls.find((call) => call.url.endsWith("/qiongli/search")).body.title, "platform governance");
});
```

- [ ] **Step 3: Write failing local match annotation test**

Add:

```js
test("handleSearch annotates external results that already exist in Zotero", async () => {
  const response = await handleSearch(
    { query: "platform governance", include_zotero: true, per_provider_limit: 1 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
      },
      fetchImpl: async (url, options = {}) => {
        if (String(url).endsWith("/qiongli/search")) {
          return {
            ok: true,
            status: 200,
            json: async () => ({
              status: "ok",
              results: [
                {
                  item_key: "ZOT123",
                  title: "Platform Governance",
                  doi: "10.1000/platform",
                  year: 2024,
                  select_uri: "zotero://select/library/items/ZOT123"
                }
              ]
            })
          };
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Platform Governance",
                publication_year: 2024,
                doi: "https://doi.org/10.1000/platform"
              }
            ]
          })
        };
      }
    }
  );

  const openAlexResult = response.results.find((result) => result.provider === "openalex");
  assert.equal(openAlexResult.local_zotero_match.item_key, "ZOT123");
  assert.equal(openAlexResult.local_zotero_match.match_basis, "doi");
});
```

- [ ] **Step 4: Write failing missing companion warning test**

Add:

```js
test("handleSearch keeps external results when requested Zotero source is unavailable", async () => {
  const response = await handleSearch(
    { query: "platform governance", include_zotero: true, per_provider_limit: 1 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
      },
      fetchImpl: async (url) => {
        if (String(url).endsWith("/qiongli/search")) {
          return {
            ok: false,
            status: 404,
            json: async () => ({})
          };
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Platform Governance",
                publication_year: 2024,
                doi: "https://doi.org/10.1000/platform"
              }
            ]
          })
        };
      }
    }
  );

  assert.equal(response.status, "ok");
  assert.equal(response.results.some((result) => result.provider === "openalex"), true);
  assert.equal(response.providers.failed.includes("zotero"), true);
  assert.equal(response.warnings.includes("zotero_companion_missing"), true);
});
```

- [ ] **Step 5: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
```

Expected: FAIL because `handleSearch` does not use Zotero source.

- [ ] **Step 6: Wire Zotero source into `handleSearch`**

In `packages/qiongli-literature-mcpb/server/diagnostics.mjs`, preserve source type in provider diagnostics:

```js
function providerDiagnostic(response) {
  const resultCount = Array.isArray(response?.results) ? response.results.length : 0;
  const requestCount = numericStat(response?.request_count, 1);
  const diagnostic = {
    provider: response?.provider ?? "unknown",
    status: response?.error ? "failed" : "success",
    result_count: resultCount,
    request_count: requestCount,
    attempts: numericStat(response?.attempts, requestCount),
    error: response?.error ?? null
  };
  if (response?.source_type) {
    diagnostic.source_type = response.source_type;
  }
  return diagnostic;
}
```

In `aggregateProviderDiagnostics`, add `source_type: null` to the summary object:

```js
      source_type: null
```

Set it while aggregating:

```js
    summary.source_type ??= response?.source_type ?? null;
```

Return it only when present:

```js
  return Array.from(summaries.values()).map((summary) => {
    const diagnostic = {
      provider: summary.provider,
      status: summary.success_count > 0 ? "success" : "failed",
      result_count: summary.result_count,
      request_count: summary.request_count,
      attempts: summary.attempts,
      error: summary.success_count > 0 ? null : summary.error
    };
    if (summary.source_type) {
      diagnostic.source_type = summary.source_type;
    }
    return diagnostic;
  });
```

At the top of `packages/qiongli-literature-mcpb/server/index.mjs`, extend the Zotero imports:

```js
import { resolveZoteroConfig } from "./zotero/config.mjs";
import {
  annotateLocalZoteroMatches,
  resolveZoteroSourceOptions,
  searchZoteroSource,
  zoteroSourceWarning
} from "./zotero/search-source.mjs";
```

Inside `handleSearch`, after `const options = resolveSearchOptions(...)`, add:

```js
  const zoteroSourceOptions = resolveZoteroSourceOptions(input, {
    perProviderLimit: options.perProviderLimit
  });
```

After external provider `responses` are created, add:

```js
  const zoteroResponses = [];
  if (zoteroSourceOptions.include) {
    zoteroResponses.push(await searchZoteroSource({
      config: resolveZoteroConfig({ env: context.env ?? process.env, input }),
      intent,
      input,
      sourceOptions: zoteroSourceOptions,
      context
    }));
  }
  const allResponses = [...responses, ...zoteroResponses];
```

Change provider outcome and result aggregation to use all responses and include `zotero` in attempted providers only when requested:

```js
  const attemptedWithZotero = zoteroSourceOptions.include ? [...attempted, "zotero"] : attempted;
  const { successful, failed } = providerOutcomes(attemptedWithZotero, allResponses);
  const externalResults = [];
  const zoteroResults = [];

  for (const response of allResponses) {
    if (response.error) {
      continue;
    }
    if (response.provider === "zotero") {
      zoteroResults.push(...response.results);
    } else {
      externalResults.push(...response.results);
    }
  }

  const results = [
    ...annotateLocalZoteroMatches({ externalResults, zoteroResults }),
    ...zoteroResults
  ];
```

Use `attemptedWithZotero` and `allResponses` when building evidence and diagnostics:

```js
  const evidence = buildEvidence({
    attemptedProviders: attemptedWithZotero,
    successfulProviders: successful,
    failedProviders: failed,
    resultCount: results.length
  });
```

```js
  const diagnostics = searchDiagnostics({
    responses: allResponses,
    rawResults: results,
    dedupedResults,
    filteredResults,
    outputResults,
    queryPlan
  });
```

Before returning, merge Zotero warnings:

```js
  const zoteroWarnings = zoteroResponses.map(zoteroSourceWarning).filter(Boolean);
  const warnings = appendSearchWarnings(evidence.warnings, outputResults, options, diagnostics);
```

Return:

```js
    warnings: [...new Set([...warnings, ...zoteroWarnings])],
```

- [ ] **Step 7: Verify search integration tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
```

Expected: PASS.

- [ ] **Step 8: Commit search integration**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/index.mjs packages/qiongli-literature-mcpb/server/diagnostics.mjs packages/qiongli-literature-mcpb/test/tools.test.mjs
git commit -m "feat(mcpb): include zotero as opt-in source"
```

## Task 5: Crossref Verifier Helper

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/crossref-verifier.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [ ] **Step 1: Write failing Crossref verifier tests**

Import the verifier:

```js
import { verifyRecordWithCrossref } from "../server/zotero/crossref-verifier.mjs";
```

Add tests:

```js
test("verifyRecordWithCrossref fills blank fields from DOI metadata", async () => {
  const result = await verifyRecordWithCrossref({
    record: { title: "Incoming Title", doi: "10.1000/verified" },
    config: { crossrefEmail: "person@example.com" },
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        message: {
          title: ["Incoming Title"],
          DOI: "10.1000/verified",
          author: [{ given: "Alex", family: "Smith" }],
          issued: { "date-parts": [[2024]] },
          "container-title": ["Journal of Tests"],
          type: "journal-article",
          URL: "https://doi.org/10.1000/verified"
        }
      })
    })
  });

  assert.equal(result.verification.crossref.status, "verified");
  assert.equal(result.record.title, "Incoming Title");
  assert.equal(result.record.year, 2024);
  assert.equal(result.record.venue, "Journal of Tests");
  assert.deepEqual(result.verification.crossref.filled_fields.sort(), ["authors", "document_type", "url", "venue", "year"].sort());
});

test("verifyRecordWithCrossref preserves non-empty incoming fields by default", async () => {
  const result = await verifyRecordWithCrossref({
    record: { title: "Incoming Title", authors: ["Original Author"], year: 2023, venue: "Original Venue", doi: "10.1000/preserve" },
    config: {},
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        message: {
          title: ["Incoming Title"],
          DOI: "10.1000/preserve",
          author: [{ given: "Alex", family: "Smith" }],
          issued: { "date-parts": [[2024]] },
          "container-title": ["Crossref Venue"]
        }
      })
    })
  });

  assert.equal(result.record.authors[0], "Original Author");
  assert.equal(result.record.year, 2023);
  assert.equal(result.record.venue, "Original Venue");
});

test("verifyRecordWithCrossref reports material title conflicts", async () => {
  const result = await verifyRecordWithCrossref({
    record: { title: "Working Paper Title", year: 2024, doi: "10.1000/conflict" },
    config: {},
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        message: {
          title: ["Published Article Title"],
          DOI: "10.1000/conflict",
          issued: { "date-parts": [[2024]] }
        }
      })
    })
  });

  assert.equal(result.verification.crossref.status, "conflict");
  assert.equal(result.verification.crossref.conflicts[0].field, "title");
  assert.equal(result.record.title, "Working Paper Title");
});

test("verifyRecordWithCrossref skips records without DOI", async () => {
  const result = await verifyRecordWithCrossref({
    record: { title: "No DOI Paper" },
    config: {},
    fetchImpl: async () => {
      throw new Error("should not fetch");
    }
  });

  assert.equal(result.verification.crossref.status, "skipped");
  assert.equal(result.record.title, "No DOI Paper");
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because `crossref-verifier.mjs` does not exist.

- [ ] **Step 3: Implement `crossref-verifier.mjs`**

Create `packages/qiongli-literature-mcpb/server/zotero/crossref-verifier.mjs`:

```js
import { searchCrossref } from "../providers/crossref.mjs";
import { normalizeReferenceRecord } from "./records.mjs";

export async function verifyRecordWithCrossref({
  record,
  config = {},
  fetchImpl,
  enabled = true,
  enrichment = "fill_blank"
} = {}) {
  const normalized = normalizeReferenceRecord(record);
  if (!enabled || !normalized.doi) {
    return withVerification(normalized, {
      status: "skipped",
      doi: normalized.doi || null,
      filled_fields: [],
      conflicts: []
    });
  }

  const response = await searchCrossref({
    query: normalized.doi,
    doi: normalized.doi,
    limit: 1,
    email: config.crossrefEmail,
    fetchImpl
  });

  if (response.error) {
    return withVerification(normalized, {
      status: "unavailable",
      doi: normalized.doi,
      filled_fields: [],
      conflicts: [],
      warning: response.error
    });
  }

  const candidate = response.results?.[0];
  if (!candidate) {
    return withVerification(normalized, {
      status: "not_found",
      doi: normalized.doi,
      filled_fields: [],
      conflicts: []
    });
  }

  const conflicts = crossrefConflicts(normalized, candidate);
  const { record: enriched, filledFields } = enrichment === "fill_blank"
    ? fillBlankFields(normalized, candidate)
    : { record: normalized, filledFields: [] };

  return withVerification(enriched, {
    status: conflicts.length > 0 ? "conflict" : "verified",
    doi: normalized.doi,
    filled_fields: filledFields,
    conflicts
  });
}

export async function verifyRecordsWithCrossref({ records = [], config = {}, fetchImpl, enabled = true, enrichment = "fill_blank" } = {}) {
  const verified = [];
  for (const record of records) {
    verified.push(await verifyRecordWithCrossref({ record, config, fetchImpl, enabled, enrichment }));
  }
  return verified;
}

export function crossrefStatusTag(status) {
  if (status === "verified") {
    return "qiongli:crossref-verified";
  }
  if (status === "conflict") {
    return "qiongli:metadata-conflict";
  }
  if (status === "unavailable") {
    return "qiongli:verification-unavailable";
  }
  return "qiongli:metadata-unverified";
}

function withVerification(record, crossref) {
  return {
    record: {
      ...record,
      verification: {
        ...(record.verification ?? {}),
        crossref
      }
    },
    verification: {
      crossref
    }
  };
}

function fillBlankFields(record, candidate) {
  const filledFields = [];
  const enriched = { ...record };
  for (const field of ["title", "authors", "year", "doi", "url", "abstract", "venue", "document_type", "reference_count", "references"]) {
    const value = candidate[field];
    if (isBlank(enriched[field]) && !isBlank(value)) {
      enriched[field] = value;
      filledFields.push(field);
    }
  }
  return { record: enriched, filledFields };
}

function crossrefConflicts(record, candidate) {
  const conflicts = [];
  if (record.title && candidate.title && titleConflict(record.title, candidate.title)) {
    conflicts.push({ field: "title", incoming: record.title, crossref: candidate.title });
  }
  if (record.year && candidate.year && record.year !== candidate.year) {
    conflicts.push({ field: "year", incoming: record.year, crossref: candidate.year });
  }
  return conflicts;
}

function titleConflict(left, right) {
  const leftTokens = new Set(comparableTitle(left).split(" ").filter(Boolean));
  const rightTokens = new Set(comparableTitle(right).split(" ").filter(Boolean));
  if (leftTokens.size === 0 || rightTokens.size === 0) {
    return false;
  }
  let overlap = 0;
  for (const token of leftTokens) {
    if (rightTokens.has(token)) {
      overlap += 1;
    }
  }
  return overlap / Math.max(leftTokens.size, rightTokens.size) < 0.5;
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^\p{Letter}\p{Number}]+/gu, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function isBlank(value) {
  if (Array.isArray(value)) {
    return value.length === 0;
  }
  return value === "" || value === null || value === undefined;
}
```

- [ ] **Step 4: Verify Crossref verifier tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit verifier**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/zotero/crossref-verifier.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs
git commit -m "feat(zotero): verify imports with crossref"
```

## Task 6: Review Tags, Upsert Verification, And Companion Tag Preservation

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/review-tags.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/zotero/config.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/zotero/records.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`

- [ ] **Step 1: Write failing review tag tests**

Import review tag helpers:

```js
import { mergeReviewTags, reviewStatusForVerification } from "../server/zotero/review-tags.mjs";
```

Add tests to `packages/qiongli-literature-mcpb/test/zotero.test.mjs`:

```js
test("mergeReviewTags adds imported needs-review source and crossref tags", () => {
  const tags = mergeReviewTags({
    baseTags: ["custom"],
    provider: "openalex",
    crossrefStatus: "verified",
    defaultReviewTags: ["qiongli:imported", "qiongli:needs-review"]
  });

  assert.deepEqual(tags, [
    "custom",
    "qiongli:imported",
    "qiongli:needs-review",
    "qiongli:source:openalex",
    "qiongli:crossref-verified"
  ]);
});

test("reviewStatusForVerification keeps newly imported verified records in needs_review", () => {
  assert.equal(reviewStatusForVerification({ writeStatus: "created", crossrefStatus: "verified" }), "needs_review");
  assert.equal(reviewStatusForVerification({ writeStatus: "unchanged", crossrefStatus: "verified" }), "unchanged");
  assert.equal(reviewStatusForVerification({ writeStatus: "skipped", crossrefStatus: "skipped" }), "skipped");
});
```

Add a new `handleZoteroUpsertReferences` test:

```js
test("handleZoteroUpsertReferences verifies DOI records with Crossref and sends review tags", async () => {
  const requests = [];
  const result = await handleZoteroUpsertReferences({
    dry_run: false,
    records: [{ title: "Crossref Verified", doi: "10.1000/verified", provider: "openalex" }]
  }, {
    env: {},
    fetchImpl: async (url, options = {}) => {
      requests.push({ url: String(url), body: options.body ? JSON.parse(options.body) : null });
      if (String(url).includes("api.crossref.org")) {
        return {
          ok: true,
          status: 200,
          json: async () => ({
            message: {
              title: ["Crossref Verified"],
              DOI: "10.1000/verified",
              issued: { "date-parts": [[2024]] },
              "container-title": ["Journal of Tests"]
            }
          })
        };
      }
      return {
        ok: true,
        status: 200,
        json: async () => ({
          status: "ok",
          dry_run: false,
          results: [{ status: "created", item_key: "NEW1" }]
        })
      };
    }
  });

  const upsertBody = requests.find((request) => request.url.endsWith("/qiongli/upsertItems")).body;
  assert.equal(upsertBody.items[0].publicationTitle, "Journal of Tests");
  assert.deepEqual(
    upsertBody.items[0].tags.map((tag) => tag.tag),
    ["qiongli:imported", "qiongli:needs-review", "qiongli:source:openalex", "qiongli:crossref-verified"]
  );
  assert.equal(result.results[0].review_status, "needs_review");
  assert.equal(result.verification[0].crossref.status, "verified");
});
```

- [ ] **Step 2: Write failing companion tag preservation test**

In `packages/qiongli-zotero-companion/test/bridge.test.mjs`, add:

```js
test("upsertItems preserves incoming Qiongli review tags on created items", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createItem: async (item) => {
      calls.push(item);
      return { key: "NEW1", ...item };
    }
  };

  const result = await upsertItems({
    dry_run: false,
    items: [
      {
        title: "Tagged Paper",
        tags: [
          { tag: "qiongli:imported" },
          { tag: "qiongli:needs-review" }
        ]
      }
    ]
  }, runtime);

  assert.deepEqual(calls[0].tags, [
    { tag: "qiongli:imported" },
    { tag: "qiongli:needs-review" }
  ]);
  assert.equal(result.results[0].item.tags[0], "qiongli:imported");
});
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
npm --prefix packages/qiongli-zotero-companion test
```

Expected: FAIL because review tag helper and upsert verification are not wired yet.

- [ ] **Step 4: Implement review tag helper**

Create `packages/qiongli-literature-mcpb/server/zotero/review-tags.mjs`:

```js
import { crossrefStatusTag } from "./crossref-verifier.mjs";

export const DEFAULT_REVIEW_TAGS = ["qiongli:imported", "qiongli:needs-review"];

export function resolveDefaultReviewTags(config = {}, input = {}) {
  const fromInput = normalizeStringList(input.review_tags);
  if (fromInput.length > 0) {
    return fromInput;
  }
  const fromConfig = normalizeStringList(config.default_review_tags);
  return fromConfig.length > 0 ? fromConfig : DEFAULT_REVIEW_TAGS;
}

export function mergeReviewTags({ baseTags = [], provider = "", crossrefStatus = "skipped", defaultReviewTags = DEFAULT_REVIEW_TAGS } = {}) {
  return normalizeStringList([
    ...normalizeStringList(baseTags),
    ...normalizeStringList(defaultReviewTags),
    provider ? `qiongli:source:${provider}` : "",
    crossrefStatusTag(crossrefStatus)
  ]);
}

export function reviewStatusForVerification({ writeStatus = "", crossrefStatus = "" } = {}) {
  if (writeStatus === "unchanged") {
    return "unchanged";
  }
  if (writeStatus === "skipped") {
    return "skipped";
  }
  return "needs_review";
}

export function normalizeStringList(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? value.split(",") : [];
  const output = [];
  const seen = new Set();
  for (const item of values) {
    const cleaned = String(item ?? "").trim();
    if (!cleaned) {
      continue;
    }
    const key = cleaned.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    output.push(cleaned);
  }
  return output;
}
```

- [ ] **Step 5: Extend Zotero config for review and verification settings**

In `packages/qiongli-literature-mcpb/server/zotero/config.mjs`, add to the returned object:

```js
    default_review_tags: cleanString(input.review_tags ?? env.QIONGLI_ZOTERO_DEFAULT_REVIEW_TAGS),
    default_review_collection_path: cleanString(input.review_collection_path ?? env.QIONGLI_ZOTERO_DEFAULT_REVIEW_COLLECTION_PATH),
    crossref_verification_enabled: readBoolean(input.verify_crossref ?? env.QIONGLI_ZOTERO_CROSSREF_VERIFICATION_ENABLED, true)
```

Keep `QIONGLI_ZOTERO_DEFAULT_COLLECTION_PATH` unchanged for backward compatibility.

- [ ] **Step 6: Preserve verification and review fields through record normalization**

In `packages/qiongli-literature-mcpb/server/zotero/records.mjs`, add to the normalized record object:

```js
    verification: clonePlainObject(record.verification),
    review_status: cleanString(record.review_status)
```

Add helper:

```js
function clonePlainObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return JSON.parse(JSON.stringify(value));
}
```

- [ ] **Step 7: Wire Crossref verification and review tags in upsert**

In `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`, import:

```js
import { readConfig } from "../config.mjs";
import { verifyRecordsWithCrossref } from "./crossref-verifier.mjs";
import { mergeReviewTags, resolveDefaultReviewTags, reviewStatusForVerification } from "./review-tags.mjs";
```

Replace the current `deduped` flow in `handleZoteroUpsertReferences` with:

```js
  const providerConfig = context.config ?? readConfig(context.env ?? process.env);
  const verifiedRecords = await verifyRecordsWithCrossref({
    records,
    config: providerConfig,
    fetchImpl: context.fetchImpl,
    enabled: input.verify_crossref !== false && config.crossref_verification_enabled,
    enrichment: input.crossref_enrichment ?? "fill_blank"
  });
  const enrichedRecords = verifiedRecords.map((entry) => entry.record);
  const deduped = dedupeReferenceRecords(enrichedRecords);
  const defaultReviewTags = resolveDefaultReviewTags(config, input);
  const itemPayloadRecords = deduped.records.map((record) => {
    const crossrefStatus = record.verification?.crossref?.status ?? "skipped";
    return {
      record,
      tags: mergeReviewTags({
        baseTags: [...normalizeStringList(input.tags), ...record.tags],
        provider: record.provider,
        crossrefStatus,
        defaultReviewTags
      })
    };
  });
```

Build payload items from `itemPayloadRecords`:

```js
    records: itemPayloadRecords.map((entry) => entry.record),
    items: itemPayloadRecords.map((entry) => mapRecordToZoteroItem(entry.record, { tags: entry.tags }))
```

After companion response, attach verification and review status:

```js
  const verification = itemPayloadRecords.map((entry) => entry.record.verification ?? { crossref: { status: "skipped", filled_fields: [], conflicts: [] } });
  const responseResults = Array.isArray(response.results) ? response.results : [];
  const results = responseResults.map((entry, index) => ({
    ...entry,
    review_status: reviewStatusForVerification({
      writeStatus: entry.status,
      crossrefStatus: verification[index]?.crossref?.status
    }),
    verification: verification[index]
  }));
```

Return `results` and top-level `verification` in both success and error branches.

- [ ] **Step 8: Verify review tag and upsert tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
npm --prefix packages/qiongli-zotero-companion test
```

Expected: PASS.

- [ ] **Step 9: Commit review tag and verification upsert work**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/zotero/config.mjs packages/qiongli-literature-mcpb/server/zotero/review-tags.mjs packages/qiongli-literature-mcpb/server/zotero/records.mjs packages/qiongli-literature-mcpb/server/zotero/tools.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs packages/qiongli-zotero-companion/test/bridge.test.mjs
git commit -m "feat(zotero): tag verified import candidates"
```

## Task 7: Import File Fallback And Documentation

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/zotero/exporters.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/README.md`
- Modify: `docs/advanced/mcp-zotero-integration.md`
- Modify: `docs/zh/advanced/mcp-zotero-integration.md`
- Modify: `content/skills/B_literature/reference-manager-bridge.md`

- [ ] **Step 1: Write failing import fallback report test**

Add to `packages/qiongli-literature-mcpb/test/zotero.test.mjs`:

```js
test("exportImportFiles includes review tags and verification counts", () => {
  const output = exportImportFiles({
    records: [
      {
        title: "Verified Export",
        year: 2024,
        doi: "10.1000/export",
        tags: ["qiongli:needs-review", "qiongli:crossref-verified"],
        verification: { crossref: { status: "verified", filled_fields: [], conflicts: [] } }
      },
      {
        title: "Conflict Export",
        year: 2024,
        doi: "10.1000/conflict",
        tags: ["qiongli:metadata-conflict"],
        verification: { crossref: { status: "conflict", filled_fields: [], conflicts: [{ field: "title" }] } }
      }
    ]
  });

  assert.ok(output.files["references.json"].includes("qiongli:needs-review"));
  assert.ok(output.files["references.ris"].includes("KW  - qiongli:metadata-conflict"));
  assert.ok(output.files["bibliography.bib"].includes("qiongli:crossref-verified"));
  assert.ok(output.files["zotero-import-report.md"].includes("- Crossref verified: 1"));
  assert.ok(output.files["zotero-import-report.md"].includes("- Metadata conflicts: 1"));
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because report counts do not exist yet.

- [ ] **Step 3: Update exporter report**

In `packages/qiongli-literature-mcpb/server/zotero/exporters.mjs`, add:

```js
function verificationCounts(records) {
  const counts = {
    verified: 0,
    conflict: 0,
    unverified: 0,
    unavailable: 0
  };
  for (const record of records) {
    const status = record.verification?.crossref?.status ?? "skipped";
    if (status === "verified") {
      counts.verified += 1;
    } else if (status === "conflict") {
      counts.conflict += 1;
    } else if (status === "unavailable") {
      counts.unavailable += 1;
    } else {
      counts.unverified += 1;
    }
  }
  return counts;
}
```

In `importReport`, compute counts:

```js
  const counts = verificationCounts(records);
```

Add these lines after the export format summary:

```js
    "",
    "## Verification Summary",
    "",
    `- Crossref verified: ${counts.verified}`,
    `- Metadata conflicts: ${counts.conflict}`,
    `- Unverified or no DOI: ${counts.unverified}`,
    `- Verification unavailable: ${counts.unavailable}`,
    "",
    "Crossref verification uses DOI registry metadata and is not human verification.",
```

- [ ] **Step 4: Verify exporter tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Update docs**

In `packages/qiongli-literature-mcpb/README.md`, add a subsection under `Local Zotero Reference Database`:

```md
### Opt-in Zotero source search

`qiongli_literature_search` does not search Zotero by default. Pass
`include_zotero: true` to include the local Zotero library as an additional
reference source. Local-only records return `provider: "zotero"` and external
records can include `local_zotero_match` when the DOI or title/year already
exists in Zotero.

### Crossref verification before Zotero writes

DOI-bearing imports use Crossref DOI registry metadata by default to fill blank
fields before writing to Zotero. Crossref metadata is not human verification, so
new or updated items still receive `qiongli:needs-review`.
```

In `docs/advanced/mcp-zotero-integration.md` and `docs/zh/advanced/mcp-zotero-integration.md`, add the same concepts in prose:

- Zotero is default-off in main search and only used with `include_zotero: true`.
- Writes add `qiongli:imported` and `qiongli:needs-review`.
- Crossref uses DOI registry metadata to fill blank fields.
- Conflicts produce `qiongli:metadata-conflict`.

In `content/skills/B_literature/reference-manager-bridge.md`, update `Zotero Integration Modes` so local Zotero has two explicit modes:

```md
| Local Zotero source search | Search inside the user's existing Zotero library only when explicitly requested | `qiongli_literature_search` with `include_zotero: true` |
| Local Zotero sync | Write selected candidate references to Zotero with review tags | `qiongli_zotero_upsert_references` |
```

- [ ] **Step 6: Verify docs mention required terms**

Run:

```bash
rg "include_zotero|local_zotero_match|qiongli:needs-review|Crossref verification|metadata-conflict|registry metadata" packages/qiongli-literature-mcpb/README.md docs/advanced/mcp-zotero-integration.md docs/zh/advanced/mcp-zotero-integration.md content/skills/B_literature/reference-manager-bridge.md
```

Expected: each required concept appears in at least one English doc and the Chinese advanced doc mirrors the user-facing behavior.

- [ ] **Step 7: Commit fallback docs**

Run:

```bash
git add packages/qiongli-literature-mcpb/server/zotero/exporters.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs packages/qiongli-literature-mcpb/README.md docs/advanced/mcp-zotero-integration.md docs/zh/advanced/mcp-zotero-integration.md content/skills/B_literature/reference-manager-bridge.md
git commit -m "docs(zotero): document verified import workflow"
```

## Task 8: Packaging And Final Verification

**Files:**
- Modify: `tests/test_literature_mcpb_artifact.py`
- Verify: all touched files.

- [ ] **Step 1: Write or update packaging expectations**

In `tests/test_literature_mcpb_artifact.py`, update `test_build_literature_mcpb_contains_required_files` to assert these files are included:

```python
self.assertIn("server/zotero/search-source.mjs", names)
self.assertIn("server/zotero/crossref-verifier.mjs", names)
self.assertIn("server/zotero/review-tags.mjs", names)
```

- [ ] **Step 2: Run packaging test**

Run:

```bash
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: PASS because the MCPB package script includes `server/` recursively.

- [ ] **Step 3: Commit packaging expectations**

Run:

```bash
git add tests/test_literature_mcpb_artifact.py
git commit -m "test(mcpb): package zotero source helpers"
```

- [ ] **Step 4: Run full MCPB tests**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: all Node tests pass.

- [ ] **Step 5: Run companion tests**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: all companion tests pass.

- [ ] **Step 6: Run Python contract and artifact tests**

Run:

```bash
python3 -m unittest tests.test_literature_mcpb_artifact tests.test_zotero_companion_artifact tests.test_literature_contract tests.test_mcp_provider_docs
```

Expected: all selected Python tests pass.

- [ ] **Step 7: Run boundary checks**

Run:

```bash
rg "secret-key|desktop-secret|api-key-value|/Users/|/private/tmp" packages/qiongli-literature-mcpb/server/zotero packages/qiongli-literature-mcpb/test/zotero.test.mjs packages/qiongli-zotero-companion/test/bridge.test.mjs docs/advanced/mcp-zotero-integration.md docs/zh/advanced/mcp-zotero-integration.md content/skills/B_literature/reference-manager-bridge.md
```

Expected: no matches except intentional fixture-secret tests outside the new Zotero files. If this command reports a new local path or fixture secret inside new Zotero code/docs, remove it before continuing.

- [ ] **Step 8: Verify clean worktree**

Run:

```bash
git status --short
```

Expected: no output.

## Completion Audit

Before marking the implementation complete, verify each acceptance criterion from `docs/superpowers/specs/2026-06-19-zotero-source-crossref-verification-design.md`:

- Main search behavior is unchanged unless `include_zotero: true`.
- `include_zotero: true` returns local Zotero results with `provider: "zotero"` and `source_type: "local_reference_database"`.
- External provider results can carry `local_zotero_match`.
- Zotero writes default to dry-run and include review-state tags in write payloads.
- DOI-bearing imports run Crossref verification by default unless disabled.
- Crossref fills blank fields only under default policy.
- Metadata conflicts appear in `verification.crossref.conflicts` and tags.
- Import fallback files include review tags and verification counts.
- Packaged artifacts contain the new helper modules.
- No local attachment paths or secrets are included.
