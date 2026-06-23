# Local Zotero Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the local-first Zotero bridge described in `docs/superpowers/specs/2026-06-19-local-zotero-bridge-design.md`.

**Architecture:** Add explicit Zotero storage/sync tools to the zero-dependency literature MCPB, backed by a loopback-only HTTP adapter for a Qiongli Zotero companion extension. Add the companion extension in its own package so Zotero Desktop owns local library reads/writes while Qiongli owns normalization, dedupe, import-file fallback, docs, and tool contracts.

**Tech Stack:** Node 18 ESM, `node:test`, zero runtime dependencies for the MCPB, Zotero Desktop bootstrapped extension JavaScript, Python `unittest` repository packaging checks.

---

## File Structure

- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
  - Register Zotero tools and route calls to the Zotero adapter.
- Modify: `packages/qiongli-literature-mcpb/server/config.mjs`
  - Read local Zotero config with loopback defaults and redacted status fields.
- Create: `packages/qiongli-literature-mcpb/server/zotero/config.mjs`
  - Resolve and validate local Zotero connector URL, write policy, update policy, and collection defaults.
- Create: `packages/qiongli-literature-mcpb/server/zotero/records.mjs`
  - Normalize tool inputs, dedupe records, generate tags, and map records to Zotero item payloads.
- Create: `packages/qiongli-literature-mcpb/server/zotero/exporters.mjs`
  - Generate CSL-JSON, RIS, BibTeX, and import reports from normalized records.
- Create: `packages/qiongli-literature-mcpb/server/zotero/client.mjs`
  - Probe default Zotero connector, probe Qiongli companion, search, and upsert through local HTTP.
- Create: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
  - Implement MCPB-facing handlers for status, search, upsert, and import-file export.
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
  - Declare Zotero tools and user config fields.
- Modify: `packages/qiongli-literature-mcpb/README.md`
  - Document local Zotero mode and import fallback.
- Modify: `tests/test_literature_mcpb_artifact.py`
  - Update manifest expectations and packaged file expectations.
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
  - Assert tool declaration and routing behavior.
- Create: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
  - Test local status, URL validation, export fallback, record mapping, dedupe, and upsert payloads.
- Create: `packages/qiongli-zotero-companion/package.json`
  - Define companion metadata and local test command.
- Create: `packages/qiongli-zotero-companion/README.md`
  - Explain development install, endpoint contract, and dry-run behavior.
- Create: `packages/qiongli-zotero-companion/manifest.json`
  - Zotero extension manifest metadata.
- Create: `packages/qiongli-zotero-companion/bootstrap.js`
  - Register and unregister `/qiongli/*` endpoints.
- Create: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
  - Runtime-independent bridge helpers for search, dry-run, and upsert decisions.
- Create: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
  - Test companion bridge helpers against a mocked Zotero runtime.
- Modify: `docs/advanced/mcp-zotero-integration.md`
  - Replace "external Zotero MCP as search override" with local reference database workflow.
- Modify: `docs/zh/advanced/mcp-zotero-integration.md`
  - Mirror the English documentation.
- Modify: `content/skills/B_literature/reference-manager-bridge.md`
  - Document three modes: local Zotero sync, import files, optional future Web API sync.

## Task 1: Baseline And Tool Declaration Tests

**Files:**
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `tests/test_literature_mcpb_artifact.py`

- [x] **Step 1: Run baseline package tests before changing behavior**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: existing tests pass before Zotero changes. If they fail, record the failure and fix only unrelated environment issues before continuing.

- [x] **Step 2: Write failing MCPB tool declaration test**

In `packages/qiongli-literature-mcpb/test/tools.test.mjs`, update the `tool declarations match manifest tool names` expected list to include:

```js
"qiongli_zotero_status",
"qiongli_zotero_search",
"qiongli_zotero_upsert_references",
"qiongli_zotero_export_import_files"
```

Also add:

```js
test("zotero tool schemas expose dry-run and import fallback controls", () => {
  const upsertTool = TOOL_DECLARATIONS.find((tool) => tool.name === "qiongli_zotero_upsert_references");
  const exportTool = TOOL_DECLARATIONS.find((tool) => tool.name === "qiongli_zotero_export_import_files");

  assert.ok(upsertTool.inputSchema.properties.records);
  assert.ok(upsertTool.inputSchema.properties.dry_run);
  assert.ok(upsertTool.inputSchema.properties.collection_path);
  assert.ok(upsertTool.inputSchema.properties.update_policy);
  assert.deepEqual(upsertTool.inputSchema.properties.update_policy.enum, ["fill_blank", "prefer_zotero", "prefer_enriched"]);
  assert.ok(exportTool.inputSchema.properties.formats);
  assert.ok(exportTool.inputSchema.properties.project_root);
});
```

- [x] **Step 3: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
```

Expected: FAIL because Zotero tools are not declared yet.

- [x] **Step 4: Add minimal tool declarations and manifest entries**

Modify `packages/qiongli-literature-mcpb/server/index.mjs` and `packages/qiongli-literature-mcpb/manifest.json` to declare the four Zotero tools. The minimal declarations should not implement behavior yet; routing can return an unknown handler until later tasks.

Use descriptions that identify Zotero as local storage/sync, not a literature search provider.

- [x] **Step 5: Verify declaration tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/tools.test.mjs
```

Expected: PASS for tool schema tests.

## Task 2: Zotero Config And Loopback Validation

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/config.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/config.mjs`
- Create or modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [x] **Step 1: Write failing config tests**

Add to `packages/qiongli-literature-mcpb/test/zotero.test.mjs`:

```js
import test from "node:test";
import assert from "node:assert/strict";
import { resolveZoteroConfig } from "../server/zotero/config.mjs";

test("resolveZoteroConfig defaults to local connector with explicit write policy", () => {
  const config = resolveZoteroConfig({ env: {} });

  assert.equal(config.local_enabled, true);
  assert.equal(config.connector_url, "http://127.0.0.1:23119");
  assert.equal(config.write_policy, "explicit");
  assert.equal(config.update_policy, "fill_blank");
});

test("resolveZoteroConfig rejects non-loopback connector URLs", () => {
  assert.throws(
    () => resolveZoteroConfig({ env: { QIONGLI_ZOTERO_CONNECTOR_URL: "http://192.168.1.50:23119" } }),
    /loopback/
  );
});
```

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because `server/zotero/config.mjs` does not exist.

- [x] **Step 3: Implement config helper**

Create `server/zotero/config.mjs` with:

```js
const DEFAULT_CONNECTOR_URL = "http://127.0.0.1:23119";
const WRITE_POLICIES = new Set(["dry_run", "explicit", "allow"]);
const UPDATE_POLICIES = new Set(["fill_blank", "prefer_zotero", "prefer_enriched"]);

export function resolveZoteroConfig({ env = process.env, input = {} } = {}) {
  const connectorUrl = String(input.connector_url ?? env.QIONGLI_ZOTERO_CONNECTOR_URL ?? DEFAULT_CONNECTOR_URL).trim();
  assertLoopbackUrl(connectorUrl);

  const writePolicy = normalizeEnum(input.write_policy ?? env.QIONGLI_ZOTERO_WRITE_POLICY, WRITE_POLICIES, "explicit");
  const updatePolicy = normalizeEnum(input.update_policy ?? env.QIONGLI_ZOTERO_UPDATE_POLICY, UPDATE_POLICIES, "fill_blank");

  return {
    local_enabled: readBoolean(input.local_enabled ?? env.QIONGLI_ZOTERO_LOCAL_ENABLED, true),
    connector_url: stripTrailingSlash(connectorUrl),
    default_collection_path: cleanString(input.collection_path ?? env.QIONGLI_ZOTERO_DEFAULT_COLLECTION_PATH),
    write_policy: writePolicy,
    update_policy: updatePolicy
  };
}

function assertLoopbackUrl(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("zotero.connector_url must be a valid URL");
  }
  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("zotero.connector_url must use http or https");
  }
  if (!["127.0.0.1", "localhost", "::1", "[::1]"].includes(url.hostname)) {
    throw new Error("zotero.connector_url must point to a loopback host");
  }
}
```

Add the remaining small helpers in the same file: `normalizeEnum`, `readBoolean`, `cleanString`, and `stripTrailingSlash`.

- [x] **Step 4: Verify config tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS for config tests.

## Task 3: Record Mapping, Deduplication, And Import Exporters

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/records.mjs`
- Create: `packages/qiongli-literature-mcpb/server/zotero/exporters.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [x] **Step 1: Write failing mapping and export tests**

Add tests:

```js
import {
  dedupeReferenceRecords,
  mapRecordToZoteroItem,
  normalizeReferenceInputs
} from "../server/zotero/records.mjs";
import { exportImportFiles } from "../server/zotero/exporters.mjs";

test("mapRecordToZoteroItem maps journal metadata conservatively", () => {
  const item = mapRecordToZoteroItem({
    title: "Platform Governance in Practice",
    authors: ["Smith, Alex", "Jordan Lee"],
    year: 2024,
    doi: "10.1000/platform-governance",
    url: "https://example.test/paper",
    abstract: "Useful abstract",
    venue: "Organization Science",
    document_type: "journal-article",
    provider: "openalex",
    source_id: "W123"
  });

  assert.equal(item.itemType, "journalArticle");
  assert.equal(item.title, "Platform Governance in Practice");
  assert.equal(item.DOI, "10.1000/platform-governance");
  assert.equal(item.publicationTitle, "Organization Science");
  assert.deepEqual(item.creators[0], { creatorType: "author", firstName: "Alex", lastName: "Smith" });
  assert.deepEqual(item.creators[1], { creatorType: "author", name: "Jordan Lee" });
  assert.match(item.extra, /Qiongli Provider: openalex/);
});

test("dedupeReferenceRecords prefers DOI before title-year fallback", () => {
  const records = dedupeReferenceRecords([
    { title: "Same Paper", year: 2024, doi: "https://doi.org/10.1000/example", provider: "openalex" },
    { title: "Same Paper", year: 2024, doi: "10.1000/example", provider: "crossref" },
    { title: "Title Only", year: 2023, provider: "semantic_scholar" },
    { title: "Title Only", year: 2023, provider: "openalex" }
  ]);

  assert.equal(records.records.length, 2);
  assert.equal(records.dedup_log.length, 2);
});

test("exportImportFiles returns CSL JSON RIS BibTeX and report", () => {
  const output = exportImportFiles({
    records: normalizeReferenceInputs({
      records: [{ title: "Exported Paper", authors: ["Smith, Alex"], year: 2024, doi: "10.1000/export" }]
    }).records
  });

  assert.ok(output.files["references.json"].includes("Exported Paper"));
  assert.ok(output.files["references.ris"].includes("TY  - JOUR"));
  assert.ok(output.files["bibliography.bib"].includes("@article{smith2024exported"));
  assert.ok(output.files["zotero-import-report.md"].includes("Export Summary"));
});
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because record and exporter helpers do not exist.

- [x] **Step 3: Implement records helper**

Implement `normalizeReferenceInputs`, `dedupeReferenceRecords`, and `mapRecordToZoteroItem`.

Rules:

- Accept either `{ records: [...] }` or a search result payload with `{ results: [...] }`.
- Normalize DOI by stripping `https://doi.org/`.
- Generate tags from provider and explicit tags.
- Generate citekeys as `firstauthorYearTitleword`.
- Keep author parsing conservative: split only `Family, Given`; keep other strings as `name`.
- Do not include empty strings in Zotero item fields.

- [x] **Step 4: Implement import exporters**

Implement `exportImportFiles({ records, formats })` returning:

```js
{
  status: "ok",
  record_count: records.length,
  files: {
    "references.json": "...",
    "references.ris": "...",
    "bibliography.bib": "...",
    "zotero-import-report.md": "..."
  }
}
```

- [x] **Step 5: Verify mapping and export tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

## Task 4: Local Zotero HTTP Client And Status Tool

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/zotero/client.mjs`
- Create: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [x] **Step 1: Write failing status tests**

Add tests:

```js
import { handleZoteroStatus } from "../server/zotero/tools.mjs";

test("handleZoteroStatus reports fallback-only when Zotero connector is absent", async () => {
  const status = await handleZoteroStatus({}, {
    fetchImpl: async () => {
      throw new Error("ECONNREFUSED");
    },
    env: {}
  });

  assert.equal(status.status, "fallback_only");
  assert.equal(status.connector.available, false);
  assert.equal(status.companion.available, false);
  assert.equal(status.fallback_import_files.available, true);
  assert.equal(status.error_code, "zotero_not_running");
});

test("handleZoteroStatus detects connector without Qiongli companion", async () => {
  const calls = [];
  const status = await handleZoteroStatus({}, {
    fetchImpl: async (url) => {
      calls.push(String(url));
      if (String(url).endsWith("/connector/ping")) {
        return { ok: true, status: 200, text: async () => "Zotero Connector Server is Available" };
      }
      return { ok: false, status: 404, text: async () => "not found" };
    },
    env: {}
  });

  assert.equal(status.status, "companion_missing");
  assert.equal(status.connector.available, true);
  assert.equal(status.companion.available, false);
  assert.equal(status.error_code, "companion_missing");
  assert.deepEqual(calls, [
    "http://127.0.0.1:23119/connector/ping",
    "http://127.0.0.1:23119/qiongli/ping"
  ]);
});
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because status tool is missing.

- [x] **Step 3: Implement client probes and status handler**

Implement `probeConnector`, `probeCompanion`, and `handleZoteroStatus`.

Rules:

- Use `fetchImpl` from context when provided.
- Default timeout should use `AbortSignal.timeout(5000)` when using global fetch.
- Return sanitized error codes; do not include stack traces.
- Always include fallback import availability.

- [x] **Step 4: Wire status routing in `index.mjs`**

Import Zotero handlers and route:

```js
if (name === "qiongli_zotero_status") {
  return toolResult(await handleZoteroStatus(input, context));
}
```

- [x] **Step 5: Verify status tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

## Task 5: Search, Upsert, And Export Tool Handlers

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/zotero/client.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/zotero/tools.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [x] **Step 1: Write failing tool handler tests**

Add tests:

```js
import {
  handleZoteroExportImportFiles,
  handleZoteroSearch,
  handleZoteroUpsertReferences
} from "../server/zotero/tools.mjs";

test("handleZoteroSearch forwards structured query to companion", async () => {
  const requests = [];
  const result = await handleZoteroSearch({ doi: "10.1000/example" }, {
    fetchImpl: async (url, options = {}) => {
      requests.push({ url: String(url), body: JSON.parse(options.body) });
      return {
        ok: true,
        status: 200,
        json: async () => ({ status: "ok", results: [{ title: "Local Paper", zotero: { item_key: "ABC123" } }] })
      };
    },
    env: {}
  });

  assert.equal(result.status, "ok");
  assert.equal(result.results[0].zotero.item_key, "ABC123");
  assert.equal(requests[0].url, "http://127.0.0.1:23119/qiongli/search");
  assert.equal(requests[0].body.doi, "10.1000/example");
});

test("handleZoteroUpsertReferences defaults to dry run and sends mapped items", async () => {
  const requests = [];
  const result = await handleZoteroUpsertReferences({
    records: [{ title: "Dry Run Paper", authors: ["Smith, Alex"], year: 2024, doi: "10.1000/dry" }]
  }, {
    fetchImpl: async (url, options = {}) => {
      requests.push({ url: String(url), body: JSON.parse(options.body) });
      return {
        ok: true,
        status: 200,
        json: async () => ({
          status: "ok",
          dry_run: true,
          results: [{ status: "created", planned: true, item: { title: "Dry Run Paper" } }]
        })
      };
    },
    env: {}
  });

  assert.equal(result.status, "ok");
  assert.equal(result.dry_run, true);
  assert.equal(requests[0].body.dry_run, true);
  assert.equal(requests[0].body.items[0].DOI, "10.1000/dry");
});

test("handleZoteroExportImportFiles works without local Zotero", async () => {
  const result = await handleZoteroExportImportFiles({
    records: [{ title: "Fallback Paper", year: 2024, doi: "10.1000/fallback" }]
  });

  assert.equal(result.status, "ok");
  assert.equal(result.fallback_import_files.available, true);
  assert.ok(result.files["references.json"]);
});
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: FAIL because handlers are not implemented.

- [x] **Step 3: Implement search, upsert, and export handlers**

Rules:

- `handleZoteroSearch` requires companion; if unavailable, return `companion_missing` with fallback guidance.
- `handleZoteroUpsertReferences` defaults `dry_run` to `true` unless `dry_run === false` and write policy permits explicit writes.
- Upsert payload includes `items`, `records`, `collection_path`, `tags`, `update_policy`, and `dry_run`.
- Failed companion writes include `fallback_import_files` generated from the same records.
- `handleZoteroExportImportFiles` never contacts Zotero.

- [x] **Step 4: Wire routing in `index.mjs`**

Route the remaining three tools to their handlers.

- [x] **Step 5: Verify handler tests pass**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test -- test/zotero.test.mjs
```

Expected: PASS.

## Task 6: Zotero Companion Package

**Files:**
- Create: `packages/qiongli-zotero-companion/package.json`
- Create: `packages/qiongli-zotero-companion/README.md`
- Create: `packages/qiongli-zotero-companion/manifest.json`
- Create: `packages/qiongli-zotero-companion/bootstrap.js`
- Create: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
- Create: `packages/qiongli-zotero-companion/test/bridge.test.mjs`

- [x] **Step 1: Write failing companion bridge tests**

Create `test/bridge.test.mjs`:

```js
import test from "node:test";
import assert from "node:assert/strict";
import {
  findDuplicateItem,
  planUpsert,
  qiongliPingResponse,
  toCompactItem
} from "../chrome/content/qiongli-bridge.js";

test("qiongliPingResponse exposes endpoint contract version", () => {
  const response = qiongliPingResponse({ zoteroVersion: "7.0.0" });

  assert.equal(response.status, "ok");
  assert.equal(response.companion, "qiongli-zotero-companion");
  assert.equal(response.endpoint_version, 1);
  assert.deepEqual(response.endpoints, ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"]);
});

test("findDuplicateItem matches DOI before title-year", () => {
  const existing = [
    { key: "A", DOI: "10.1000/example", title: "Wrong Title", date: "2020" },
    { key: "B", DOI: "", title: "Same Title", date: "2024" }
  ];

  assert.equal(findDuplicateItem({ DOI: "https://doi.org/10.1000/example", title: "Same Title", date: "2024" }, existing).key, "A");
});

test("planUpsert preserves non-empty Zotero fields by default", () => {
  const plan = planUpsert({
    incoming: { title: "Enriched Title", DOI: "10.1000/example", abstractNote: "New abstract" },
    existing: { key: "ABC123", title: "User Title", DOI: "", abstractNote: "" },
    updatePolicy: "fill_blank"
  });

  assert.equal(plan.status, "updated");
  assert.equal(plan.patch.title, undefined);
  assert.equal(plan.patch.DOI, "10.1000/example");
  assert.equal(plan.patch.abstractNote, "New abstract");
});

test("toCompactItem returns no local file paths", () => {
  const compact = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    DOI: "10.1000/local",
    path: "/Users/person/Zotero/storage/file.pdf"
  });

  assert.equal(compact.item_key, "ABC123");
  assert.equal(Object.hasOwn(compact, "path"), false);
});
```

- [x] **Step 2: Run companion test to verify it fails**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: FAIL because package files do not exist.

- [x] **Step 3: Implement companion helper module**

Create `chrome/content/qiongli-bridge.js` with exported pure helpers first. Keep Zotero runtime calls behind function parameters so tests can run in Node.

- [x] **Step 4: Implement bootstrap endpoint registration**

Create `bootstrap.js` that registers:

```js
Zotero.Server.Endpoints["/qiongli/ping"]
Zotero.Server.Endpoints["/qiongli/search"]
Zotero.Server.Endpoints["/qiongli/upsertItems"]
Zotero.Server.Endpoints["/qiongli/collections"]
```

Each endpoint should parse JSON for POST requests, call bridge helpers, and return JSON. The first implementation can use conservative runtime adapters and clear `not_implemented` responses for APIs that need a live Zotero runtime, but `/qiongli/ping` must be fully functional and helper tests must pass.

- [x] **Step 5: Add package metadata and README**

`package.json` should include:

```json
{
  "name": "qiongli-zotero-companion",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "node --test test/*.test.mjs"
  }
}
```

- [x] **Step 6: Verify companion tests pass**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: PASS.

## Task 7: Documentation And Reference Manager Skill

**Files:**
- Modify: `docs/advanced/mcp-zotero-integration.md`
- Modify: `docs/zh/advanced/mcp-zotero-integration.md`
- Modify: `content/skills/B_literature/reference-manager-bridge.md`
- Modify: `packages/qiongli-literature-mcpb/README.md`

- [x] **Step 1: Write failing doc checks**

Add or update relevant tests if an existing doc test covers these files. If no existing doc test covers them, use `rg` verification in Step 4.

Expected content phrases:

```text
local reference database
Qiongli Zotero companion
qiongli_zotero_status
qiongli_zotero_upsert_references
references.json
references.ris
bibliography.bib
```

- [x] **Step 2: Update English docs**

Rewrite `docs/advanced/mcp-zotero-integration.md` to describe:

- local-first companion workflow
- why the companion is needed for reliable local read/write
- status, dry-run, explicit write, and import fallback examples
- Web API as optional future/cloud mode, not default

- [x] **Step 3: Update Chinese docs and skill text**

Mirror the English docs in `docs/zh/advanced/mcp-zotero-integration.md`.

Update `content/skills/B_literature/reference-manager-bridge.md` to define:

1. Local Zotero sync through Qiongli companion.
2. Import-file generation.
3. Optional future Zotero Web API sync.

- [x] **Step 4: Verify docs include the required workflow terms**

Run:

```bash
rg "local reference database|Qiongli Zotero companion|qiongli_zotero_status|qiongli_zotero_upsert_references|references\\.json|references\\.ris|bibliography\\.bib" docs/advanced/mcp-zotero-integration.md docs/zh/advanced/mcp-zotero-integration.md content/skills/B_literature/reference-manager-bridge.md packages/qiongli-literature-mcpb/README.md
```

Expected: each required workflow term appears in the relevant docs.

## Task 8: Packaging And Repository Boundary Checks

**Files:**
- Modify: `tests/test_literature_mcpb_artifact.py`
- No generated artifacts committed.

- [x] **Step 1: Write failing packaging expectations**

Update `tests/test_literature_mcpb_artifact.py`:

- Expected manifest tool set includes the four Zotero tools.
- MCPB artifact contains:
  - `server/zotero/config.mjs`
  - `server/zotero/records.mjs`
  - `server/zotero/exporters.mjs`
  - `server/zotero/client.mjs`
  - `server/zotero/tools.mjs`
- Package dependencies remain `{}`.

- [x] **Step 2: Run packaging test to verify it fails**

Run:

```bash
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: FAIL until manifest and files are fully wired.

- [x] **Step 3: Fix packaging behavior**

The existing build script already includes all files under `server/`, so this should require only manifest and test expectation updates. Do not add runtime dependencies.

- [x] **Step 4: Run boundary review checks**

Run:

```bash
git status --short
rg "secret-key|desktop-secret|api-key-value|/Users/pengjiaxin|/private/tmp" packages/qiongli-literature-mcpb packages/qiongli-zotero-companion docs/advanced docs/zh/advanced content/skills/B_literature/reference-manager-bridge.md
```

Expected: no secrets, no local absolute paths, no built `.xpi` or `.mcpb` artifacts.

## Task 9: Full Verification

**Files:**
- All touched implementation, docs, and tests.

- [x] **Step 1: Run MCPB tests**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS.

- [x] **Step 2: Run companion tests**

Run:

```bash
npm --prefix packages/qiongli-zotero-companion test
```

Expected: PASS.

- [x] **Step 3: Run packaging tests**

Run:

```bash
python3 -m unittest tests.test_literature_mcpb_artifact
```

Expected: PASS.

- [x] **Step 4: Run targeted docs/skill checks**

Run:

```bash
python3 -m unittest tests.test_literature_contract tests.test_mcp_provider_docs
```

Expected: PASS, or document unrelated pre-existing failures with exact output.

- [x] **Step 5: Final repository audit**

Run:

```bash
git status --short
git diff --stat
```

Expected: only source, test, and documentation files are modified; no generated artifacts or local config are staged.
