import test from "node:test";
import assert from "node:assert/strict";
import { webcrypto } from "node:crypto";
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

async function applyApproved(payload, runtime) {
  const preview = await upsertItems({ ...payload, dry_run: true }, runtime);
  return upsertItems({
    ...payload,
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: preview.write_approval.receipt
  }, runtime);
}

test("qiongliPingResponse exposes endpoint contract version", () => {
  const response = qiongliPingResponse({ zoteroVersion: "7.0.0" });

  assert.equal(response.status, "ok");
  assert.equal(response.companion, "qiongli-zotero-companion");
  assert.equal(response.endpoint_version, "2");
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

test("planUpsert preserves Zotero-authored metadata and appends only missing tags", () => {
  const plan = planUpsert({
    incoming: {
      title: "Provider Title",
      tags: [{ tag: "user-curated" }, { tag: "qiongli:needs-review" }]
    },
    existing: {
      key: "ABC123",
      title: "User Title",
      tags: [{ tag: "user-curated" }]
    },
    updatePolicy: "fill_blank"
  });

  assert.equal(plan.patch.title, undefined);
  assert.deepEqual(plan.patch.tags, [{ tag: "qiongli:needs-review" }]);
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

test("normalizeAttachments omits local URL values from default summaries", () => {
  const defaultSummary = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    attachments: [
      {
        attachment_key: "FILEURL",
        filename: "file-url.pdf",
        url: "file:///Users/person/Zotero/storage/FILEURL/file-url.pdf",
        path: "/Users/person/Zotero/storage/FILEURL/file-url.pdf"
      },
      {
        attachment_key: "UNIXPATH",
        filename: "unix-path.pdf",
        url: "/private/tmp/zotero/unix-path.pdf",
        path: "/private/tmp/zotero/unix-path.pdf"
      },
      {
        attachment_key: "WINPATH",
        filename: "windows-path.pdf",
        url: "C:\\Users\\person\\Zotero\\windows-path.pdf",
        path: "C:\\Users\\person\\Zotero\\windows-path.pdf"
      },
      {
        attachment_key: "REMOTE",
        filename: "remote.pdf",
        url: "https://example.test/remote.pdf"
      }
    ]
  });

  assert.deepEqual(defaultSummary.attachments.map((attachment) => attachment.url), [
    "",
    "",
    "",
    "https://example.test/remote.pdf"
  ]);
  assert.equal(defaultSummary.attachments.some((attachment) => Object.hasOwn(attachment, "path")), false);

  const explicitPathSummary = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    attachments: [
      {
        attachment_key: "FILEURL",
        filename: "file-url.pdf",
        url: "file:///Users/person/Zotero/storage/FILEURL/file-url.pdf",
        path: "/Users/person/Zotero/storage/FILEURL/file-url.pdf"
      }
    ]
  }, { include_attachment_paths: true });

  assert.equal(explicitPathSummary.attachments[0].url, "");
  assert.equal(explicitPathSummary.attachments[0].path, "/Users/person/Zotero/storage/FILEURL/file-url.pdf");
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

test("toCompactItem bounds repeated metadata and attachment summaries", () => {
  const compact = toCompactItem({
    key: "A",
    title: "t".repeat(3000),
    creators: Array.from({ length: 60 }, (_, index) => ({ name: `Creator ${index}` })),
    tags: Array.from({ length: 110 }, (_, index) => ({ tag: `tag-${index}` })),
    collections: Array.from({ length: 110 }, (_, index) => `Qiongli/project-${index}`),
    notes: Array.from({ length: 25 }, (_, index) => ({
      key: `NOTE${index}`,
      title: `Note ${index}`,
      summary: "s".repeat(600)
    })),
    attachments: Array.from({ length: 25 }, (_, index) => ({
      key: `ATT${index}`,
      filename: `${index}.pdf`,
      mime_type: "application/pdf"
    }))
  });

  assert.equal(compact.title.length, 2048);
  assert.equal(compact.creators.length, 50);
  assert.equal(compact.tags.length, 100);
  assert.equal(compact.collections.length, 100);
  assert.equal(compact.notes.length, 20);
  assert.equal(compact.notes[0].summary.length, 500);
  assert.equal(compact.attachments.length, 20);
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

test("searchLocalItems qualifies creator tag collection note and bounded attachment summaries", async () => {
  const runtime = {
    listItems: async () => [
      {
        key: "A",
        title: "Platform Governance",
        creators: [{ firstName: "Alex", lastName: "Smith" }],
        tags: [{ tag: "screened" }],
        collections: ["Qiongli/platform-governance"],
        notes: [{ key: "NOTE1", title: "Reading note", summary: "Mechanism evidence" }],
        attachments: [{
          key: "ATT1",
          filename: "paper.pdf",
          mime_type: "application/pdf",
          path: "/private/library/ATT1/paper.pdf"
        }]
      },
      {
        key: "B",
        title: "Other Paper",
        creators: [{ firstName: "Taylor", lastName: "Jones" }],
        tags: [{ tag: "other" }],
        collections: ["Qiongli/other"]
      }
    ]
  };

  const result = await searchLocalItems({
    creator: "smith",
    tag: "screened",
    collection_path: "Qiongli/platform-governance",
    limit: 1
  }, runtime);

  assert.equal(result.limit, 1);
  assert.equal(result.results.length, 1);
  assert.deepEqual(result.results[0].creators, ["Alex Smith"]);
  assert.deepEqual(result.results[0].notes, [{
    note_key: "NOTE1",
    title: "Reading note",
    summary: "Mechanism evidence"
  }]);
  assert.equal(result.results[0].attachments[0].local_file_available, true);
  assert.equal(Object.hasOwn(result.results[0].attachments[0], "path"), false);
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

test("upsertItems rejects an oversized batch before reading or mutating Zotero", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => {
      calls.push("list");
      return [];
    },
    createItem: async () => {
      calls.push("create");
      return { key: "UNEXPECTED" };
    }
  };

  const result = await upsertItems({
    dry_run: true,
    items: Array.from({ length: 101 }, (_, index) => ({ title: `Paper ${index}` }))
  }, runtime);

  assert.equal(result.status, "error");
  assert.equal(result.error_code, "companion_too_many_items");
  assert.deepEqual(calls, []);
});

test("upsertItems refuses direct writes and consumes an approved dry-run receipt once", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createItem: async (item) => {
      calls.push(["create", item.title]);
      return { key: "NEW1", ...item };
    }
  };
  const payload = {
    items: [{ title: "Approval Paper", DOI: "10.1000/approval" }]
  };

  const blocked = await upsertItems({ ...payload, dry_run: false }, runtime);
  assert.equal(blocked.status, "approval_required");
  assert.equal(blocked.error_code, "zotero_write_intent_required");
  assert.deepEqual(calls, []);

  const applied = await upsertItems({
    ...payload,
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: blocked.write_approval.receipt
  }, runtime);
  assert.equal(applied.status, "ok");
  assert.equal(applied.write_approval.consumed, true);
  assert.deepEqual(calls, [["create", "Approval Paper"]]);

  const replayed = await upsertItems({
    ...payload,
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: blocked.write_approval.receipt
  }, runtime);
  assert.equal(replayed.status, "approval_required");
  assert.equal(replayed.error_code, "zotero_dry_run_receipt_invalid");
  assert.deepEqual(calls, [["create", "Approval Paper"]]);
});

test("upsertItems rejects a changed plan after dry run", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createItem: async (item) => {
      calls.push(item.title);
      return { key: "NEW1", ...item };
    }
  };
  const preview = await upsertItems({
    dry_run: true,
    items: [{ title: "Original Plan" }]
  }, runtime);
  const changed = await upsertItems({
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: preview.write_approval.receipt,
    items: [{ title: "Changed Plan" }]
  }, runtime);

  assert.equal(changed.status, "approval_required");
  assert.equal(changed.error_code, "zotero_dry_run_plan_changed");
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

  const result = await applyApproved({
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

test("upsertItems dry run reports target collection without mutating runtime", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    ensureCollectionPath: async (collectionPath) => {
      calls.push(["ensureCollectionPath", collectionPath]);
      return { key: "COLL1", path: collectionPath };
    },
    addItemToCollection: async (itemKey, collectionKey) => {
      calls.push(["addItemToCollection", itemKey, collectionKey]);
    }
  };

  const result = await upsertItems({
    dry_run: true,
    collection_path: "Qiongli/platform-governance",
    items: [{ title: "Project Paper", DOI: "10.1000/project" }]
  }, runtime);

  assert.equal(result.results[0].status, "created");
  assert.equal(result.results[0].collection_path, "Qiongli/platform-governance");
  assert.deepEqual(calls, []);
});

test("upsertItems creates missing collection and adds created items to it", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createItem: async (item) => {
      calls.push(["create", item.title]);
      return { key: "NEW1", collections: [], ...item };
    },
    ensureCollectionPath: async (collectionPath) => {
      calls.push(["ensureCollectionPath", collectionPath]);
      return { key: "COLL1", path: collectionPath };
    },
    addItemToCollection: async (itemKey, collectionKey) => {
      calls.push(["addItemToCollection", itemKey, collectionKey]);
      return { key: itemKey, collections: [collectionKey] };
    }
  };

  const result = await applyApproved({
    collection_path: "Qiongli/platform-governance",
    items: [{ title: "Created Paper", DOI: "10.1000/created" }]
  }, runtime);

  assert.deepEqual(calls, [
    ["ensureCollectionPath", "Qiongli/platform-governance"],
    ["create", "Created Paper"],
    ["addItemToCollection", "NEW1", "COLL1"]
  ]);
  assert.equal(result.results[0].item_key, "NEW1");
  assert.deepEqual(result.results[0].collection, {
    key: "COLL1",
    path: "Qiongli/platform-governance",
    status: "added"
  });
  assert.deepEqual(result.results[0].item.collections, ["COLL1"]);
});

test("upsertItems adds unchanged duplicate items to target collection", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [{ key: "A", title: "Existing Paper", DOI: "10.1000/existing", collections: [] }],
    ensureCollectionPath: async (collectionPath) => {
      calls.push(["ensureCollectionPath", collectionPath]);
      return { key: "COLL1", path: collectionPath };
    },
    addItemToCollection: async (itemKey, collectionKey) => {
      calls.push(["addItemToCollection", itemKey, collectionKey]);
      return { key: itemKey, collections: [collectionKey] };
    }
  };

  const result = await applyApproved({
    collection_path: "Qiongli/platform-governance",
    items: [{ title: "Existing Paper", DOI: "10.1000/existing" }]
  }, runtime);

  assert.deepEqual(calls, [
    ["ensureCollectionPath", "Qiongli/platform-governance"],
    ["addItemToCollection", "A", "COLL1"]
  ]);
  assert.equal(result.results[0].status, "unchanged");
  assert.deepEqual(result.results[0].collection, {
    key: "COLL1",
    path: "Qiongli/platform-governance",
    status: "added"
  });
});

test("upsertItems dry run reports planned child notes without mutating runtime", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createChildNote: async (parentItemKey, note) => {
      calls.push(["createChildNote", parentItemKey, note.title]);
      return { key: "NOTE1", parent_item_key: parentItemKey };
    }
  };

  const result = await upsertItems({
    dry_run: true,
    items: [
      {
        title: "Noted Paper",
        DOI: "10.1000/noted",
        qiongli_notes: [{ title: "Qiongli Reading Note", html: "<p>Important finding.</p>" }]
      }
    ]
  }, runtime);

  assert.equal(result.results[0].status, "created");
  assert.deepEqual(result.results[0].notes, [
    { status: "planned", title: "Qiongli Reading Note" }
  ]);
  assert.deepEqual(calls, []);
});

test("upsertItems does not duplicate an existing matching child note", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [{
      key: "A",
      title: "Noted Paper",
      notes: [{
        key: "NOTE1",
        title: "Qiongli Reading Note",
        summary: "Important finding."
      }]
    }],
    createChildNote: async (...args) => {
      calls.push(args);
      return { key: "NOTE2" };
    }
  };
  const payload = {
    items: [{
      title: "Noted Paper",
      qiongli_notes: [{
        title: "Qiongli Reading Note",
        html: "<p>Important finding.</p>"
      }]
    }]
  };

  const preview = await upsertItems({ ...payload, dry_run: true }, runtime);
  assert.deepEqual(preview.results[0].notes, [{
    status: "already_present",
    title: "Qiongli Reading Note"
  }]);
  const applied = await upsertItems({
    ...payload,
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: preview.write_approval.receipt
  }, runtime);
  assert.equal(applied.status, "ok");
  assert.deepEqual(applied.results[0].notes ?? [], []);
  assert.deepEqual(calls, []);
});

test("upsertItems creates child notes on written items", async () => {
  const calls = [];
  const runtime = {
    listItems: async () => [],
    createItem: async (item) => {
      calls.push(["create", item.title]);
      return { key: "NEW1", ...item };
    },
    createChildNote: async (parentItemKey, note) => {
      calls.push(["createChildNote", parentItemKey, note.title, note.html]);
      return { key: "NOTE1", parent_item_key: parentItemKey, title: note.title };
    }
  };

  const result = await applyApproved({
    items: [
      {
        title: "Noted Paper",
        DOI: "10.1000/noted",
        qiongli_notes: [{ title: "Qiongli Reading Note", html: "<p>Important finding.</p>" }]
      }
    ]
  }, runtime);

  assert.deepEqual(calls, [
    ["create", "Noted Paper"],
    ["createChildNote", "NEW1", "Qiongli Reading Note", "<p>Important finding.</p>"]
  ]);
  assert.deepEqual(result.results[0].notes, [
    {
      status: "created",
      note_key: "NOTE1",
      title: "Qiongli Reading Note"
    }
  ]);
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

  const result = await applyApproved({
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
  assert.equal(manifest.version, "0.3.0");
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
  const itemCollections = [];
  const noteItems = [];
  const libraryItem = {
    id: 10,
    key: "ABC123",
    itemType: "journalArticle",
    isRegularItem: () => true,
    getField: (field) => ({
      title: "Platform Governance",
      DOI: "10.1000/platform",
      date: "2024"
    })[field] ?? "",
    getTags: () => [{ tag: "qiongli:imported" }],
    getCollections: () => itemCollections,
    getAttachments: () => [123],
    addToCollection: (collectionKey) => {
      itemCollections.push(collectionKey);
    },
    saveTx: async () => {}
  };
  const attachmentItem = {
    key: "ATT123",
    itemType: "attachment",
    attachmentLinkMode: "imported_file",
    attachmentContentType: "application/pdf",
    attachmentFilename: "platform-governance.pdf",
    getField: (field) => ({
      title: "Platform Governance PDF",
      url: "file:///Users/person/Zotero/storage/ATT123/platform-governance.pdf",
      filename: "platform-governance.pdf",
      contentType: "application/pdf"
    })[field] ?? "",
    getFilePath: () => "/zotero-fixture/storage/ATT123/platform-governance.pdf"
  };
  const collections = [{ id: 1, key: "COLL1", name: "Qiongli", parentID: null }];
  const Zotero = {
    version: "9.0.4",
    Server: { Endpoints: {} },
    Libraries: { userLibraryID: 1 },
    Item: class {
      constructor(itemType) {
        this.itemType = itemType;
        this.key = itemType === "note" ? "NOTE1" : "NEWITEM";
        this.fields = {};
        this.tags = [];
        this.collections = [];
        this.parentItemID = null;
        this.note = "";
      }

      setField(field, value) {
        this.fields[field] = value;
      }

      getField(field) {
        return this.fields[field] ?? "";
      }

      setCreators(creators) {
        this.creators = creators;
      }

      addTag(tag) {
        this.tags.push({ tag });
      }

      getTags() {
        return this.tags;
      }

      getCollections() {
        return this.collections;
      }

      addToCollection(collectionKey) {
        this.collections.push(collectionKey);
      }

      getAttachments() {
        return [];
      }

      setNote(noteHtml) {
        this.note = noteHtml;
      }

      getNote() {
        return this.note;
      }

      async saveTx() {
        if (this.itemType === "note") {
          noteItems.push(this);
        }
      }
    },
    Collection: class {
      async saveTx() {
        this.id = 2;
        this.key = "COLL2";
        collections.push(this);
      }
    },
    Items: {
      getAll: async () => [libraryItem],
      getAsync: async (id) => id === 123 ? attachmentItem : null
    },
    Collections: { getByLibrary: async () => collections }
  };
  const context = vm.createContext({
    Zotero,
    globalThis: { Zotero, crypto: webcrypto }
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
  assert.equal(response.body.version, "0.3.0");
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
  assert.equal(response.body.results[0].attachments[0].url, "");
  assert.equal(Object.hasOwn(response.body.results[0].attachments[0], "path"), false);

  await Zotero.Server.Endpoints["/qiongli/collections"].prototype.init("", (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });

  assert.equal(response.status, 200);
  assert.equal(response.body.collections[0].key, "COLL1");

  const upsertPayload = {
    collection_path: "Qiongli/platform-governance",
    items: [
      {
        title: "Platform Governance",
        DOI: "10.1000/platform",
        qiongli_notes: [{ title: "Qiongli Reading Note", html: "<p>Key finding.</p>" }]
      }
    ]
  };
  await Zotero.Server.Endpoints["/qiongli/upsertItems"].prototype.init({
    ...upsertPayload,
    dry_run: true
  }, (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });
  const dryRunReceipt = response.body.write_approval.receipt;
  assert.equal(response.body.dry_run, true);
  assert.deepEqual(itemCollections, []);
  assert.deepEqual(noteItems, []);

  await Zotero.Server.Endpoints["/qiongli/upsertItems"].prototype.init({
    ...upsertPayload,
    dry_run: false,
    write_intent: "apply",
    dry_run_receipt: dryRunReceipt
  }, (status, contentType, body) => {
    response = { status, contentType, body: JSON.parse(body) };
  });
  assert.equal(response.status, 200);
  assert.equal(response.body.results[0].status, "unchanged");
  assert.deepEqual(response.body.results[0].collection, {
    key: "COLL2",
    path: "Qiongli/platform-governance",
    status: "added"
  });
  assert.equal(collections[1].name, "platform-governance");
  assert.equal(collections[1].parentID, 1);
  assert.deepEqual(itemCollections, [2]);
  assert.deepEqual(response.body.results[0].notes, [
    {
      status: "created",
      note_key: "NOTE1",
      title: "Qiongli Reading Note"
    }
  ]);
  assert.equal(noteItems[0].parentItemID, 10);
  assert.equal(noteItems[0].note, "<p>Key finding.</p>");

  context.shutdown({}, 4);
  assert.deepEqual(Object.keys(Zotero.Server.Endpoints), []);
});
