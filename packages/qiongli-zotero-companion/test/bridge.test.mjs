import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import {
  findDuplicateItem,
  normalizeAttachments,
  planUpsert,
  qiongliPingResponse,
  searchLocalItems,
  toCompactItem,
  upsertItems
} from "../chrome/content/qiongli-bridge.js";

const PACKAGE_ROOT = path.resolve(import.meta.dirname, "..");
const ZOTERO_UPDATE_URL = "https://github.com/jxpeng98/qiongli/releases/latest/download/qiongli-zotero-companion-updates.json";

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

test("toCompactItem exposes attachment summaries without local paths by default", () => {
  const compact = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    attachments: [
      {
        attachment_key: "ATT123",
        title: "Local Paper PDF",
        filename: "local-paper.pdf",
        mime_type: "application/pdf",
        link_mode: "imported_file",
        url: "https://example.test/local-paper.pdf",
        select_uri: "zotero://select/library/items/ATT123",
        local_file_available: true,
        path: "/zotero-fixture/storage/ATT123/local-paper.pdf"
      }
    ]
  });

  assert.equal(compact.attachments.length, 1);
  assert.equal(compact.attachments[0].attachment_key, "ATT123");
  assert.equal(compact.attachments[0].filename, "local-paper.pdf");
  assert.equal(compact.attachments[0].mime_type, "application/pdf");
  assert.equal(compact.attachments[0].local_file_available, true);
  assert.equal(compact.attachments[0].select_uri, "zotero://select/library/items/ATT123");
  assert.equal(Object.hasOwn(compact.attachments[0], "path"), false);
});

test("toCompactItem exposes attachment paths only when explicitly requested", () => {
  const compact = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    attachments: [
      {
        attachment_key: "ATT123",
        filename: "local-paper.pdf",
        path: "/zotero-fixture/storage/ATT123/local-paper.pdf"
      }
    ]
  }, { include_attachment_paths: true });

  assert.equal(compact.attachments[0].path, "/zotero-fixture/storage/ATT123/local-paper.pdf");
});

test("normalizeAttachments keeps only structured attachment metadata", () => {
  const normalized = normalizeAttachments([
    null,
    {
      attachment_key: "A",
      title: " Attachment A ",
      filename: "a.pdf",
      mime_type: "application/pdf",
      link_mode: "imported_file",
      url: "https://example.test/a.pdf",
      select_uri: "zotero://select/library/items/A",
      local_file_available: true,
      path: "/zotero-fixture/storage/A/a.pdf",
      note: "drop me"
    },
    { title: "No key", filename: "missing-key.pdf" }
  ], { includeAttachmentPaths: true });

  assert.deepEqual(normalized, [
    {
      attachment_key: "A",
      title: "Attachment A",
      filename: "a.pdf",
      mime_type: "application/pdf",
      link_mode: "imported_file",
      url: "https://example.test/a.pdf",
      select_uri: "zotero://select/library/items/A",
      local_file_available: true,
      path: "/zotero-fixture/storage/A/a.pdf"
    }
  ]);
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

  assert.equal(manifest.name, "Qiongli Zotero Companion");
  assert.match(manifest.description, /Zotero 9\.0\.4/);
  assert.equal(manifest.version, "0.2.2");
  assert.equal(manifest.applications.zotero.update_url, ZOTERO_UPDATE_URL);
  assert.equal(manifest.applications.zotero.strict_min_version, "8.0");
  assert.equal(manifest.applications.zotero.strict_max_version, "9.0.*");
  assert.equal(Object.hasOwn(manifest, "browser_specific_settings"), false);
  for (const endpoint of ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"]) {
    assert.ok(bootstrap.includes(endpoint), `${endpoint} missing from bootstrap.js`);
  }
  assert.equal(bootstrap.includes("not_implemented"), false);
  assert.match(readme, /Qiongli Zotero Companion/);
  assert.match(readme, /Zotero 9\.0\.4/);
  assert.match(readme, /local reference database/);
});

test("bootstrap startup registers endpoints from Zotero 8 and 9 global object", async () => {
  const bootstrap = await readFile(path.join(PACKAGE_ROOT, "bootstrap.js"), "utf8");
  const libraryItem = {
    key: "ABC123",
    itemType: "journalArticle",
    isRegularItem: () => true,
    getField: (field) => ({
      title: "Platform Governance",
      DOI: "10.1000/platform",
      date: "2024"
    })[field] ?? "",
    getTags: () => [{ tag: "qiongli:imported" }],
    getCollections: () => ["COLL1"],
    getAttachments: () => [123]
  };
  const attachmentItem = {
    key: "ATT123",
    itemType: "attachment",
    attachmentLinkMode: "imported_file",
    attachmentContentType: "application/pdf",
    attachmentFilename: "platform-governance.pdf",
    getField: (field) => ({
      title: "Platform Governance PDF",
      url: "https://example.test/platform-governance.pdf",
      filename: "platform-governance.pdf",
      contentType: "application/pdf"
    })[field] ?? "",
    getFilePath: () => "/zotero-fixture/storage/ATT123/platform-governance.pdf"
  };
  const Zotero = {
    version: "9.0.4",
    Server: { Endpoints: {} },
    Libraries: { userLibraryID: 1 },
    Items: {
      getAll: async () => [libraryItem],
      getAsync: async (id) => id === 123 ? attachmentItem : null
    },
    Collections: { getByLibrary: async () => [{ key: "COLL1", name: "Qiongli" }] }
  };
  const context = vm.createContext({
    Zotero,
    globalThis: { Zotero }
  });

  vm.runInContext(bootstrap, context);
  context.startup({}, 3);

  assert.deepEqual(Object.keys(Zotero.Server.Endpoints).sort(), [
    "/qiongli/collections",
    "/qiongli/ping",
    "/qiongli/search",
    "/qiongli/upsertItems"
  ]);

  let response;
  await Zotero.Server.Endpoints["/qiongli/ping"].prototype.init("", (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });

  assert.equal(response.status, 200);
  assert.equal(response.contentType, "application/json");
  assert.equal(response.body.status, "ok");
  assert.equal(response.body.version, "0.2.2");
  assert.equal(response.body.zotero_version, "9.0.4");

  await Zotero.Server.Endpoints["/qiongli/search"].prototype.init({ title: "platform" }, (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });

  assert.equal(response.status, 200);
  assert.equal(response.contentType, "application/json");
  assert.equal(response.body.status, "ok");
  assert.equal(response.body.results.length, 1);
  assert.equal(response.body.results[0].item_key, "ABC123");
  assert.equal(response.body.results[0].attachments.length, 1);
  assert.equal(response.body.results[0].attachments[0].attachment_key, "ATT123");
  assert.equal(response.body.results[0].attachments[0].mime_type, "application/pdf");
  assert.equal(response.body.results[0].attachments[0].local_file_available, true);
  assert.equal(Object.hasOwn(response.body.results[0].attachments[0], "path"), false);

  await Zotero.Server.Endpoints["/qiongli/collections"].prototype.init("", (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });

  assert.equal(response.status, 200);
  assert.equal(response.body.collections[0].key, "COLL1");
});
