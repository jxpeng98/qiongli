import test from "node:test";
import assert from "node:assert/strict";
import { resolveZoteroConfig } from "../server/zotero/config.mjs";
import {
  dedupeReferenceRecords,
  mapRecordToZoteroItem,
  normalizeReferenceInputs
} from "../server/zotero/records.mjs";
import { exportImportFiles } from "../server/zotero/exporters.mjs";

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
