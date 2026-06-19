import test from "node:test";
import assert from "node:assert/strict";
import { resolveZoteroConfig } from "../server/zotero/config.mjs";
import {
  dedupeReferenceRecords,
  mapRecordToZoteroItem,
  normalizeReferenceInputs
} from "../server/zotero/records.mjs";
import { exportImportFiles } from "../server/zotero/exporters.mjs";
import {
  handleZoteroExportImportFiles,
  handleZoteroSearch,
  handleZoteroStatus,
  handleZoteroUpsertReferences
} from "../server/zotero/tools.mjs";
import {
  annotateLocalZoteroMatches,
  normalizeZoteroSourceResults,
  resolveZoteroSourceOptions,
  zoteroSourceSearchPayload
} from "../server/zotero/search-source.mjs";

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
