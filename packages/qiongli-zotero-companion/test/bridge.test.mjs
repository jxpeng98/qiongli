import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import {
  findDuplicateItem,
  planUpsert,
  qiongliPingResponse,
  toCompactItem
} from "../chrome/content/qiongli-bridge.js";

const PACKAGE_ROOT = path.resolve(import.meta.dirname, "..");

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

test("companion package declares Zotero install metadata and qiongli endpoints", async () => {
  const manifest = JSON.parse(await readFile(path.join(PACKAGE_ROOT, "manifest.json"), "utf8"));
  const bootstrap = await readFile(path.join(PACKAGE_ROOT, "bootstrap.js"), "utf8");
  const readme = await readFile(path.join(PACKAGE_ROOT, "README.md"), "utf8");

  assert.equal(manifest.name, "qiongli-zotero-companion");
  assert.equal(manifest.version, "0.1.0");
  assert.equal(manifest.applications.zotero.strict_min_version, "7.0");
  for (const endpoint of ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"]) {
    assert.ok(bootstrap.includes(endpoint), `${endpoint} missing from bootstrap.js`);
  }
  assert.match(readme, /Qiongli Zotero companion/);
  assert.match(readme, /local reference database/);
});
