import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import {
  findDuplicateItem,
  planUpsert,
  qiongliPingResponse,
  searchLocalItems,
  toCompactItem,
  upsertItems
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
    path: "Zotero/storage/file.pdf"
  });

  assert.equal(compact.item_key, "ABC123");
  assert.equal(Object.hasOwn(compact, "path"), false);
});

test("searchLocalItems filters runtime items by DOI and title", async () => {
  const runtime = {
    listItems: async () => [
      { key: "A", title: "Platform Governance", DOI: "10.1000/platform", date: "2024" },
      { key: "B", title: "Unrelated Paper", DOI: "10.1000/other", date: "2020" }
    ]
  };

  const doiResults = await searchLocalItems({ doi: "https://doi.org/10.1000/platform" }, runtime);
  const titleResults = await searchLocalItems({ title: "platform governance", year: 2024 }, runtime);

  assert.equal(doiResults.results.length, 1);
  assert.equal(doiResults.results[0].item_key, "A");
  assert.equal(titleResults.results.length, 1);
  assert.equal(titleResults.results[0].item_key, "A");
});

test("upsertItems dry run returns planned operations without mutating runtime", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [{ key: "A", title: "User Title", DOI: "", abstractNote: "" }],
    createItem: async (item) => {
      calls.push(["create", item]);
      return { key: "NEW", ...item };
    },
    updateItem: async (key, patch) => {
      calls.push(["update", key, patch]);
      return { key, ...patch };
    }
  };

  const result = await upsertItems({
    dry_run: true,
    update_policy: "fill_blank",
    items: [{ title: "User Title", DOI: "10.1000/user", abstractNote: "Abstract" }]
  }, runtime);

  assert.equal(result.status, "ok");
  assert.equal(result.dry_run, true);
  assert.equal(result.results[0].status, "updated");
  assert.deepEqual(calls, []);
});

test("upsertItems writes creates and updates through runtime when dry_run is false", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [{ key: "A", title: "Existing Paper", DOI: "", abstractNote: "" }],
    createItem: async (item) => {
      calls.push(["create", item.title]);
      return { key: "NEW1", ...item };
    },
    updateItem: async (key, patch) => {
      calls.push(["update", key, patch.DOI]);
      return { key, ...patch };
    }
  };

  const result = await upsertItems({
    dry_run: false,
    update_policy: "fill_blank",
    items: [
      { title: "Existing Paper", DOI: "10.1000/existing" },
      { title: "Created Paper", DOI: "10.1000/created" }
    ]
  }, runtime);

  assert.equal(result.status, "ok");
  assert.equal(result.dry_run, false);
  assert.deepEqual(calls, [
    ["update", "A", "10.1000/existing"],
    ["create", "Created Paper"]
  ]);
  assert.equal(result.results[0].item_key, "A");
  assert.equal(result.results[1].item_key, "NEW1");
});

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

test("companion package declares Zotero install metadata and qiongli endpoints", async () => {
  const manifest = JSON.parse(await readFile(path.join(PACKAGE_ROOT, "manifest.json"), "utf8"));
  const bootstrap = await readFile(path.join(PACKAGE_ROOT, "bootstrap.js"), "utf8");
  const readme = await readFile(path.join(PACKAGE_ROOT, "README.md"), "utf8");

  assert.equal(manifest.name, "qiongli-zotero-companion");
  assert.equal(manifest.version, "0.1.1");
  assert.equal(manifest.applications.zotero.strict_min_version, "7.0");
  assert.equal(manifest.applications.zotero.strict_max_version, "9.*");
  for (const endpoint of ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"]) {
    assert.ok(bootstrap.includes(endpoint), `${endpoint} missing from bootstrap.js`);
  }
  assert.equal(bootstrap.includes("not_implemented"), false);
  assert.match(readme, /Qiongli Zotero companion/);
  assert.match(readme, /local reference database/);
});
