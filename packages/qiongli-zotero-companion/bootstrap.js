const QIONGLI_WRITE_RECEIPT_PREFIX = "zwr1_";
const QIONGLI_WRITE_RECEIPT_TTL_MS = 5 * 60 * 1000;
const QIONGLI_MAX_WRITE_RECEIPTS = 64;
const QIONGLI_DEFAULT_SEARCH_LIMIT = 25;
const QIONGLI_MAX_SEARCH_LIMIT = 200;
const QIONGLI_MAX_UPSERT_ITEMS = 100;
const QIONGLI_MAX_REQUEST_JSON_CHARS = 1024 * 1024;
const QIONGLI_MAX_ITEM_CREATORS = 50;
const QIONGLI_MAX_ITEM_TAGS = 100;
const QIONGLI_MAX_ITEM_COLLECTIONS = 100;
const QIONGLI_MAX_ITEM_NOTES = 20;
const QIONGLI_MAX_ITEM_ATTACHMENTS = 20;
const QiongliZoteroWriteApprovals = new Map();

var QiongliZoteroCompanion = {
  endpoints: [
    "/qiongli/ping",
    "/qiongli/search",
    "/qiongli/upsertItems",
    "/qiongli/collections"
  ],

  startup() {
    const Zotero = getZotero();
    if (!Zotero?.Server?.Endpoints) {
      return;
    }

    registerEndpoint(Zotero, "/qiongli/ping", ["GET"], (_postData, sendResponse) => {
      sendJson(sendResponse, 200, {
        status: "ok",
        companion: "qiongli-zotero-companion",
        version: "0.3.0",
        endpoint_version: "2",
        zotero_version: Zotero.version ?? "",
        endpoints: this.endpoints
      });
    });

    registerEndpoint(Zotero, "/qiongli/search", ["POST"], async (postData, sendResponse) => {
      const query = parseJson(postData);
      const requestError = validateRequestPayload(query);
      if (requestError) {
        sendJson(sendResponse, 400, { status: "error", error_code: requestError, results: [] });
        return;
      }
      const runtime = createRuntime(Zotero);
      const items = await runtime.listItems();
      const limit = boundedSearchLimit(query.limit);
      sendJson(sendResponse, 200, {
        status: "ok",
        limit,
        results: items
          .filter((item) => itemMatchesQuery(item, query))
          .slice(0, limit)
          .map((item) => toCompactItem(item, query))
      });
    });

    registerEndpoint(Zotero, "/qiongli/upsertItems", ["POST"], async (postData, sendResponse) => {
      const payload = parseJson(postData);
      const runtime = createRuntime(Zotero);
      const result = await upsertRuntimeItems(payload, runtime);
      sendJson(sendResponse, 200, result);
    });

    registerEndpoint(Zotero, "/qiongli/collections", ["GET"], async (_postData, sendResponse) => {
      const runtime = createRuntime(Zotero);
      const collections = await runtime.listCollections();
      sendJson(sendResponse, 200, {
        status: "ok",
        collections
      });
    });
  },

  shutdown() {
    const Zotero = getZotero();
    if (!Zotero?.Server?.Endpoints) {
      return;
    }
    for (const endpoint of this.endpoints) {
      delete Zotero.Server.Endpoints[endpoint];
    }
  }
};

function startup(data, reason) {
  QiongliZoteroCompanion.startup(data, reason);
}

function shutdown(data, reason) {
  QiongliZoteroCompanion.shutdown(data, reason);
}

function install() {}

function uninstall() {}

function registerEndpoint(Zotero, path, supportedMethods, handler) {
  const endpoint = Zotero.Server.Endpoints[path] = function() {};
  endpoint.prototype = {
    supportedMethods,
    async init(postData, sendResponseCallback) {
      try {
        await handler(postData, sendResponseCallback);
      } catch (error) {
        sendJson(sendResponseCallback, 500, {
          status: "error",
          error_code: "companion_runtime_error",
          message: String(error?.message ?? error ?? "runtime error")
        });
      }
    }
  };
}

function sendJson(sendResponseCallback, status, payload) {
  sendResponseCallback(status, "application/json", JSON.stringify(payload));
}

function getZotero() {
  if (typeof Zotero !== "undefined" && Zotero?.Server?.Endpoints) {
    return Zotero;
  }
  if (typeof globalThis !== "undefined" && globalThis.Zotero?.Server?.Endpoints) {
    return globalThis.Zotero;
  }
  try {
    if (typeof Components === "undefined") {
      return null;
    }
    return Components.classes["@zotero.org/Zotero;1"]
      .getService(Components.interfaces.nsISupports)
      .wrappedJSObject;
  } catch (_error) {
    return null;
  }
}

function createRuntime(Zotero) {
  return {
    async listItems() {
      const libraryID = Zotero.Libraries?.userLibraryID;
      const rawItems = typeof Zotero.Items?.getAll === "function" ? await Zotero.Items.getAll(libraryID) : [];
      const collections = await listPlainCollections(Zotero);
      const collectionPaths = new Map(
        collections
          .filter((collection) => collection.id !== null)
          .map((collection) => [collection.id, collection.path])
      );
      const plainItems = [];
      for (const item of asArray(rawItems).filter((candidate) => typeof candidate.isRegularItem !== "function" || candidate.isRegularItem())) {
        plainItems.push(await itemToPlainObject(Zotero, item, collectionPaths));
      }
      return plainItems;
    },

    async createItem(itemData) {
      const item = new Zotero.Item(itemData.itemType || "journalArticle");
      applyItemData(item, itemData);
      await item.saveTx();
      return itemToPlainObject(Zotero, item);
    },

    async updateItem(key, patch) {
      const item = await getItemByKey(Zotero, key);
      applyItemData(item, patch);
      await item.saveTx();
      return itemToPlainObject(Zotero, item);
    },

    async ensureCollectionPath(collectionPath) {
      return ensureCollectionPath(Zotero, collectionPath);
    },

    async addItemToCollection(itemKey, collectionKey, collectionID) {
      const item = await getItemByKey(Zotero, itemKey);
      if (typeof item.addToCollection !== "function") {
        throw new Error("Zotero item collection API is unavailable");
      }
      item.addToCollection(collectionID ?? collectionKey);
      await item.saveTx();
      return itemToPlainObject(Zotero, item);
    },

    async createChildNote(parentItemKey, note) {
      return createChildNote(Zotero, parentItemKey, note);
    },

    async listCollections() {
      return listPlainCollections(Zotero);
    }
  };
}

async function getItemByKey(Zotero, key) {
  if (typeof Zotero.Items?.getByLibraryAndKeyAsync === "function") {
    return Zotero.Items.getByLibraryAndKeyAsync(Zotero.Libraries.userLibraryID, key);
  }
  const items = typeof Zotero.Items?.getAll === "function" ? await Zotero.Items.getAll(Zotero.Libraries.userLibraryID) : [];
  const item = asArray(items).find((candidate) => candidate.key === key);
  if (!item) {
    throw new Error(`Zotero item not found: ${key}`);
  }
  return item;
}

async function ensureCollectionPath(Zotero, collectionPath) {
  const normalizedPath = normalizeCollectionPath(collectionPath);
  const parts = normalizedPath.split("/").filter(Boolean);
  if (parts.length === 0) {
    throw new Error("Zotero collection path is empty");
  }

  const libraryID = Zotero.Libraries?.userLibraryID;
  const collections = await listPlainCollections(Zotero);
  let parentID = null;
  let current = null;
  let currentPath = "";

  for (const part of parts) {
    currentPath = currentPath ? `${currentPath}/${part}` : part;
    current = collections.find((collection) => (
      collection.name === part && sameCollectionParent(collection.parentID, parentID)
    ));

    if (!current) {
      current = await createCollection(Zotero, {
        libraryID,
        name: part,
        parentID,
        path: currentPath
      });
      collections.push(current);
    } else {
      current.path = currentPath;
    }

    parentID = current.id;
  }

  if (!current?.key) {
    throw new Error(`Zotero collection key missing for ${normalizedPath}`);
  }

  return {
    id: current.id,
    key: current.key,
    path: currentPath
  };
}

async function listPlainCollections(Zotero) {
  const libraryID = Zotero.Libraries?.userLibraryID;
  const rawCollections = typeof Zotero.Collections?.getByLibrary === "function"
    ? await Zotero.Collections.getByLibrary(libraryID)
    : [];
  const plain = asArray(rawCollections).map((collection) => plainCollection(collection));
  const byID = new Map(plain.map((collection) => [collection.id, collection]).filter(([id]) => id !== null));
  return plain.map((collection) => ({
    ...collection,
    path: collectionPathFor(collection, byID)
  }));
}

async function createCollection(Zotero, { libraryID, name, parentID, path }) {
  if (typeof Zotero.Collection !== "function") {
    throw new Error("Zotero collection creation API is unavailable");
  }

  const collection = new Zotero.Collection();
  collection.libraryID = libraryID;
  collection.name = name;
  if (parentID !== null && parentID !== undefined) {
    collection.parentID = parentID;
  }
  await collection.saveTx();

  return {
    ...plainCollection(collection),
    path
  };
}

async function createChildNote(Zotero, parentItemKey, note) {
  if (typeof Zotero.Item !== "function") {
    throw new Error("Zotero child note API is unavailable");
  }
  const parentItem = await getItemByKey(Zotero, parentItemKey);
  const noteItem = new Zotero.Item("note");
  noteItem.libraryID = Zotero.Libraries?.userLibraryID;
  noteItem.parentItemID = parentItem.id ?? parentItem.itemID;
  if (!noteItem.parentItemID) {
    throw new Error(`Zotero parent item id missing for ${parentItemKey}`);
  }

  if (typeof noteItem.setNote === "function") {
    noteItem.setNote(note.html);
  } else if (typeof noteItem.setField === "function") {
    noteItem.setField("note", note.html);
  } else {
    throw new Error("Zotero note content API is unavailable");
  }

  await noteItem.saveTx();
  return {
    key: noteItem.key,
    parent_item_key: parentItemKey,
    title: attachmentString(note.title) || "Qiongli Reading Note"
  };
}

function plainCollection(collection) {
  return {
    id: normalizeCollectionID(collection.id ?? collection.collectionID),
    key: attachmentString(collection.key),
    name: attachmentString(collection.name),
    parentID: normalizeCollectionID(collection.parentID)
  };
}

function collectionPathFor(collection, byID, seen = new Set()) {
  if (collection.parentID === null || seen.has(collection.parentID)) {
    return collection.name;
  }

  seen.add(collection.parentID);
  const parent = byID.get(collection.parentID);
  if (!parent) {
    return collection.name;
  }
  const parentPath = collectionPathFor(parent, byID, seen);
  return parentPath ? `${parentPath}/${collection.name}` : collection.name;
}

function normalizeCollectionID(value) {
  if (value === "" || value === null || value === undefined || value === false) {
    return null;
  }
  return value;
}

function sameCollectionParent(left, right) {
  return normalizeCollectionID(left) === normalizeCollectionID(right);
}

async function itemToPlainObject(Zotero, item, collectionPaths = new Map()) {
  const collectionIDs = typeof item.getCollections === "function" ? item.getCollections() : [];
  return {
    key: item.key,
    itemType: item.itemType,
    title: getField(item, "title"),
    DOI: getField(item, "DOI"),
    citekey: getField(item, "citationKey"),
    url: getField(item, "url"),
    abstractNote: getField(item, "abstractNote"),
    publicationTitle: getField(item, "publicationTitle"),
    date: getField(item, "date"),
    creators: typeof item.getCreators === "function" ? item.getCreators() : [],
    tags: typeof item.getTags === "function" ? item.getTags() : [],
    collections: asArray(collectionIDs).map((id) => collectionPaths.get(id) ?? id),
    notes: await notePlainObjects(Zotero, item),
    attachments: await attachmentPlainObjects(Zotero, item)
  };
}

async function notePlainObjects(Zotero, item) {
  const noteIds = typeof item.getNotes === "function" ? asArray(item.getNotes()).slice(0, 20) : [];
  const notes = [];
  for (const id of noteIds) {
    const note = await getAttachmentItem(Zotero, id);
    const key = attachmentString(note?.key);
    if (!key) {
      continue;
    }
    const html = attachmentString(
      typeof note.getNote === "function" ? note.getNote() : getField(note, "note")
    );
    notes.push({
      key,
      title: getField(note, "title") || "Zotero Note",
      summary: noteTextSummary(html)
    });
  }
  return notes;
}

function noteTextSummary(value) {
  return String(value ?? "")
    .replace(/<[^>]*>/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 500);
}

async function attachmentPlainObjects(Zotero, item) {
  const attachmentIds = typeof item.getAttachments === "function" ? asArray(item.getAttachments()) : [];
  const attachments = [];

  for (const id of attachmentIds) {
    const attachment = await getAttachmentItem(Zotero, id);
    if (!attachment) {
      continue;
    }

    const path = attachmentString(typeof attachment.getFilePath === "function"
      ? await attachment.getFilePath()
      : attachment.path);
    const key = attachmentString(attachment.key);
    if (!key) {
      continue;
    }

    attachments.push({
      key,
      title: getField(attachment, "title"),
      filename: getField(attachment, "filename") || attachmentString(attachment.attachmentFilename) || filenameFromPath(path),
      mime_type: getField(attachment, "contentType")
        || getField(attachment, "mimeType")
        || getField(attachment, "mime_type")
        || attachmentString(attachment.attachmentContentType),
      link_mode: attachmentString(attachment.attachmentLinkMode ?? attachment.linkMode),
      url: getField(attachment, "url"),
      select_uri: `zotero://select/library/items/${key}`,
      local_file_available: Boolean(path),
      path
    });
  }

  return attachments;
}

async function getAttachmentItem(Zotero, id) {
  if (id && typeof id === "object") {
    return id;
  }
  if (typeof Zotero.Items?.getAsync === "function") {
    return Zotero.Items.getAsync(id);
  }
  if (typeof Zotero.Items?.get === "function") {
    return Zotero.Items.get(id);
  }
  return null;
}

function applyItemData(item, data) {
  for (const [field, value] of Object.entries(data)) {
    if (["itemType", "creators", "tags", "collections"].includes(field)) {
      continue;
    }
    if (value !== undefined && value !== null && typeof item.setField === "function") {
      item.setField(field, value);
    }
  }
  if (Array.isArray(data.creators)) {
    item.setCreators(data.creators);
  }
  if (Array.isArray(data.tags)) {
    for (const tag of data.tags) {
      item.addTag(tag.tag ?? tag);
    }
  }
}

async function upsertRuntimeItems(payload, runtime) {
  const requestError = validateRequestPayload(payload, { requireItems: true });
  if (requestError) {
    return { status: "error", error_code: requestError, dry_run: true, results: [] };
  }
  const dryRun = payload.dry_run !== false;
  const updatePolicy = payload.update_policy ?? "fill_blank";
  const collectionPath = normalizeCollectionPath(payload.collection_path);
  const incomingItems = Array.isArray(payload.items) ? payload.items : [];
  const existingItems = await runtime.listItems();
  const plans = incomingItems.map((incoming) => {
    const existing = findDuplicateItem(incoming, existingItems);
    return {
      existing,
      notePlans: plannedNoteResults(incoming, existing),
      plan: planUpsert(incoming, existing, updatePolicy)
    };
  });
  const approvalPlan = canonicalWritePlan({
    collectionPath,
    incomingItems,
    plans: plans.map(({ notePlans, plan }) => ({ notePlans, plan })),
    updatePolicy
  });

  if (dryRun) {
    return {
      status: "ok",
      dry_run: true,
      results: plans.map(({ existing, notePlans, plan }, index) => plannedUpsertResult({
        collectionPath,
        existing,
        incoming: incomingItems[index],
        notePlans,
        plan
      })),
      write_approval: issueWriteApproval(approvalPlan)
    };
  }

  const approvalError = consumeWriteApproval({
    approvalPlan,
    receipt: payload.dry_run_receipt,
    writeIntent: payload.write_intent
  });
  if (approvalError) {
    return {
      status: "approval_required",
      error_code: approvalError,
      dry_run: true,
      results: plans.map(({ existing, notePlans, plan }, index) => plannedUpsertResult({
        collectionPath,
        existing,
        incoming: incomingItems[index],
        notePlans,
        plan
      })),
      write_approval: issueWriteApproval(approvalPlan)
    };
  }

  const targetCollection = collectionPath ? await ensureRuntimeCollection(runtime, collectionPath) : null;
  const results = [];

  for (const [index, incoming] of incomingItems.entries()) {
    const { existing, plan } = plans[index];
    if (!existing) {
      const created = await runtime.createItem(incoming);
      const collection = await addItemToTargetCollection({ item: created, targetCollection, runtime });
      const notes = await createChildNotes({ item: created, incoming, runtime });
      existingItems.push(created);
      results.push({
        status: "created",
        item_key: created.key,
        ...(collection ? { collection } : {}),
        ...(notes.length > 0 ? { notes } : {}),
        item: toCompactItem(created)
      });
      continue;
    }
    if (plan.status === "unchanged") {
      const collection = await addItemToTargetCollection({ item: existing, targetCollection, runtime });
      const notes = await createChildNotes({ existing, item: existing, incoming, runtime });
      results.push({
        status: "unchanged",
        item_key: existing.key,
        ...(collection ? { collection } : {}),
        ...(notes.length > 0 ? { notes } : {}),
        item: toCompactItem(existing)
      });
      continue;
    }
    const updated = await runtime.updateItem(existing.key, plan.patch);
    Object.assign(existing, updated);
    const collection = await addItemToTargetCollection({ item: existing, targetCollection, runtime });
    const notes = await createChildNotes({ existing, item: existing, incoming, runtime });
    results.push({
      status: "updated",
      item_key: existing.key,
      patch: plan.patch,
      ...(collection ? { collection } : {}),
      ...(notes.length > 0 ? { notes } : {}),
      item: toCompactItem(existing)
    });
  }

  return {
    status: "ok",
    dry_run: false,
    write_approval: { consumed: true },
    results
  };
}

function plannedUpsertResult({ collectionPath, existing, notePlans, plan }) {
  return {
    ...plan,
    planned: true,
    item_key: plan.item_key ?? existing?.key ?? null,
    ...(collectionPath ? { collection_path: collectionPath } : {}),
    ...(notePlans.length > 0 ? { notes: notePlans } : {})
  };
}

function canonicalWritePlan({ collectionPath, incomingItems, plans, updatePolicy }) {
  return canonicalJson({
    endpoint_version: "2",
    update_policy: updatePolicy,
    collection_path: collectionPath || null,
    items: incomingItems,
    plans
  });
}

function issueWriteApproval(approvalPlan) {
  pruneWriteApprovals();
  while (QiongliZoteroWriteApprovals.size >= QIONGLI_MAX_WRITE_RECEIPTS) {
    QiongliZoteroWriteApprovals.delete(QiongliZoteroWriteApprovals.keys().next().value);
  }
  const receipt = `${QIONGLI_WRITE_RECEIPT_PREFIX}${randomHex(32)}`;
  QiongliZoteroWriteApprovals.set(receipt, {
    approvalPlan,
    expiresAt: Date.now() + QIONGLI_WRITE_RECEIPT_TTL_MS
  });
  return {
    receipt,
    expires_in_seconds: QIONGLI_WRITE_RECEIPT_TTL_MS / 1000,
    required_write_intent: "apply"
  };
}

function consumeWriteApproval({ approvalPlan, receipt, writeIntent }) {
  pruneWriteApprovals();
  if (writeIntent !== "apply") {
    return "zotero_write_intent_required";
  }
  if (typeof receipt !== "string" || !receipt.startsWith(QIONGLI_WRITE_RECEIPT_PREFIX)) {
    return "zotero_dry_run_receipt_required";
  }
  const stored = QiongliZoteroWriteApprovals.get(receipt);
  QiongliZoteroWriteApprovals.delete(receipt);
  if (!stored) {
    return "zotero_dry_run_receipt_invalid";
  }
  return stored.approvalPlan === approvalPlan
    ? null
    : "zotero_dry_run_plan_changed";
}

function pruneWriteApprovals() {
  const now = Date.now();
  for (const [receipt, approval] of QiongliZoteroWriteApprovals) {
    if (approval.expiresAt <= now) {
      QiongliZoteroWriteApprovals.delete(receipt);
    }
  }
}

function randomHex(byteLength) {
  const bytes = new Uint8Array(byteLength);
  if (typeof globalThis.crypto?.getRandomValues !== "function") {
    throw new Error("secure write-approval receipts are unavailable");
  }
  globalThis.crypto.getRandomValues(bytes);
  return [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function canonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map((entry) => canonicalJson(entry)).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value ?? null);
}

function normalizeCollectionPath(value) {
  return String(value ?? "")
    .split("/")
    .map((part) => part.trim())
    .filter(Boolean)
    .join("/");
}

async function ensureRuntimeCollection(runtime, collectionPath) {
  if (typeof runtime.ensureCollectionPath !== "function") {
    throw new Error("Zotero collection writes are unavailable in this companion runtime");
  }
  const collection = await runtime.ensureCollectionPath(collectionPath);
  const key = attachmentString(collection?.key ?? collection?.collection_key);
  if (!key) {
    throw new Error(`Zotero collection key missing for ${collectionPath}`);
  }
  return {
    ...(collection?.id !== undefined && collection?.id !== null ? { id: collection.id } : {}),
    key,
    path: attachmentString(collection?.path ?? collection?.collection_path) || collectionPath
  };
}

async function addItemToTargetCollection({ item, targetCollection, runtime }) {
  if (!targetCollection) {
    return null;
  }

  if (itemBelongsToCollection(item, targetCollection)) {
    return {
      key: targetCollection.key,
      path: targetCollection.path,
      status: "already_member"
    };
  }

  if (typeof runtime.addItemToCollection !== "function") {
    throw new Error("Zotero collection membership writes are unavailable in this companion runtime");
  }

  const updated = await runtime.addItemToCollection(item.key, targetCollection.key, targetCollection.id);
  if (updated && typeof updated === "object") {
    Object.assign(item, updated);
  } else {
    item.collections = [...asCollectionList(item.collections), targetCollection.key];
  }

  return {
    key: targetCollection.key,
    path: targetCollection.path,
    status: "added"
  };
}

function itemBelongsToCollection(item, targetCollection) {
  return asCollectionList(item.collections).some((collection) => {
    if (typeof collection === "string") {
      return collection === targetCollection.key || collection === targetCollection.path;
    }
    if (Number.isInteger(collection) || typeof collection === "number") {
      return collection === targetCollection.id;
    }
    if (!collection || typeof collection !== "object") {
      return false;
    }
    return collection.key === targetCollection.key
      || collection.path === targetCollection.path
      || collection.collection_key === targetCollection.key
      || collection.collection_path === targetCollection.path;
  });
}

function asCollectionList(value) {
  return Array.isArray(value) ? value : [];
}

function plannedNoteResults(incoming, existing = null) {
  return normalizeIncomingNotes(incoming).map((note) => ({
    status: existingNoteMatches(existing?.notes, note) ? "already_present" : "planned",
    title: note.title
  }));
}

async function createChildNotes({ existing = null, item, incoming, runtime }) {
  const notes = normalizeIncomingNotes(incoming).filter(
    (note) => !existingNoteMatches(existing?.notes, note)
  );
  if (notes.length === 0) {
    return [];
  }
  if (typeof runtime.createChildNote !== "function") {
    throw new Error("Zotero child note writes are unavailable in this companion runtime");
  }

  const results = [];
  for (const note of notes) {
    const created = await runtime.createChildNote(item.key, note);
    results.push({
      status: "created",
      note_key: attachmentString(created?.key ?? created?.note_key),
      title: attachmentString(created?.title) || note.title
    });
  }
  return results;
}

function existingNoteMatches(existingNotes, incomingNote) {
  const incomingSummary = noteTextSummary(incomingNote.html);
  return (Array.isArray(existingNotes) ? existingNotes : []).some((note) => (
    attachmentString(note?.title) === incomingNote.title
    && attachmentString(note?.summary) === incomingSummary
  ));
}

function normalizeIncomingNotes(incoming) {
  const notes = Array.isArray(incoming?.qiongli_notes) ? incoming.qiongli_notes : [];
  return notes
    .map((note) => {
      if (!note || typeof note !== "object" || Array.isArray(note)) {
        return null;
      }
      const html = attachmentString(note.html);
      const text = attachmentString(note.text);
      const title = attachmentString(note.title) || "Qiongli Reading Note";
      if (!html && !text) {
        return null;
      }
      return {
        title,
        html: html || `<p>${escapeHtml(text)}</p>`
      };
    })
    .filter(Boolean);
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function findDuplicateItem(incoming, existingItems) {
  const incomingDoi = normalizeDoi(incoming.DOI ?? incoming.doi);
  if (incomingDoi) {
    const doiMatch = existingItems.find((item) => normalizeDoi(item.DOI ?? item.doi) === incomingDoi);
    if (doiMatch) {
      return doiMatch;
    }
  }
  const incomingTitle = comparableTitle(incoming.title);
  if (!incomingTitle) {
    return null;
  }
  return existingItems.find((item) => comparableTitle(item.title) === incomingTitle) ?? null;
}

function planUpsert(incoming, existing, updatePolicy) {
  if (!existing) {
    return { status: "created", item: incoming, patch: incoming };
  }
  const patch = {};
  for (const [field, value] of Object.entries(incoming)) {
    if (["collections", "itemType", "qiongli_notes"].includes(field)) {
      continue;
    }
    if (value === "" || value === null || value === undefined) {
      continue;
    }
    const existingValue = existing[field];
    if (field === "tags" && Array.isArray(value)) {
      const missingTags = missingIncomingTags(existingValue, value);
      if (missingTags.length > 0) {
        patch.tags = missingTags;
      }
      continue;
    }
    if (updatePolicy === "prefer_enriched" || existingValue === "" || existingValue === null || existingValue === undefined) {
      patch[field] = value;
    }
  }
  return { status: Object.keys(patch).length > 0 ? "updated" : "unchanged", item_key: existing.key, patch };
}

function missingIncomingTags(existingValue, incomingValue) {
  const existingTags = new Set(
    (Array.isArray(existingValue) ? existingValue : [])
      .map((tag) => attachmentString(tag?.tag ?? tag).toLowerCase())
      .filter(Boolean)
  );
  return incomingValue.filter((tag) => {
    const normalized = attachmentString(tag?.tag ?? tag).toLowerCase();
    return normalized && !existingTags.has(normalized);
  });
}

function itemMatchesQuery(item, query) {
  const queryDoi = normalizeDoi(query.doi ?? query.DOI);
  if (queryDoi) {
    return normalizeDoi(item.DOI ?? item.doi) === queryDoi;
  }
  const queryTitle = comparableTitle(query.title);
  if (queryTitle && !comparableTitle(item.title).includes(queryTitle)) {
    return false;
  }
  const queryYear = parseYear(query.year);
  if (queryYear && parseYear(item.date ?? item.year) !== queryYear) {
    return false;
  }

  const queryCitekey = comparableText(query.citekey);
  if (queryCitekey && comparableText(item.citekey ?? item.citationKey) !== queryCitekey) {
    return false;
  }

  const queryCreator = comparableText(query.creator);
  if (queryCreator && !normalizeCreators(item.creators).some(
    (creator) => comparableText(creator).includes(queryCreator)
  )) {
    return false;
  }

  const queryTag = comparableText(query.tag);
  if (queryTag && !(Array.isArray(item.tags) ? item.tags : []).some(
    (tag) => comparableText(tag?.tag ?? tag) === queryTag
  )) {
    return false;
  }

  const queryCollection = normalizeCollectionPath(query.collection_path).toLowerCase();
  if (queryCollection && !(Array.isArray(item.collections) ? item.collections : []).some(
    (collection) => normalizeCollectionPath(
      collection?.path ?? collection?.collection_path ?? collection
    ).toLowerCase() === queryCollection
  )) {
    return false;
  }
  return true;
}

function toCompactItem(item, options = {}) {
  const itemKey = boundedOutputString(item.key, 128);
  return {
    item_key: itemKey,
    title: boundedOutputString(item.title, 2048),
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: parseYear(item.date ?? item.year),
    item_type: boundedOutputString(item.itemType, 128),
    citekey: boundedOutputString(item.citekey ?? item.citationKey, 512),
    creators: normalizeCreators(item.creators),
    url: boundedOutputString(sanitizeAttachmentUrl(item.url ?? item.URL), 2048),
    abstract: boundedOutputString(item.abstract ?? item.abstractNote, 16384),
    venue: boundedOutputString(item.venue ?? item.publicationTitle, 1024),
    select_uri: itemKey ? `zotero://select/library/items/${itemKey}` : "",
    tags: normalizeStringSummaries(item.tags, QIONGLI_MAX_ITEM_TAGS, 256, (tag) => tag?.tag ?? tag),
    collections: normalizeStringSummaries(
      item.collections,
      QIONGLI_MAX_ITEM_COLLECTIONS,
      1024,
      (collection) => collection?.path ?? collection?.collection_path ?? collection
    ),
    notes: normalizeNoteSummaries(item.notes),
    attachments: normalizeAttachments(item.attachments, options)
  };
}

function normalizeCreators(value) {
  const creators = Array.isArray(value) ? value : [];
  return creators
    .slice(0, QIONGLI_MAX_ITEM_CREATORS)
    .map((creator) => {
      if (typeof creator === "string") {
        return boundedOutputString(creator, 512);
      }
      if (!creator || typeof creator !== "object") {
        return "";
      }
      return boundedOutputString(
        creator.name
          ?? [creator.firstName, creator.lastName].filter(Boolean).join(" "),
        512
      );
    })
    .filter(Boolean);
}

function normalizeNoteSummaries(value) {
  const notes = Array.isArray(value) ? value : [];
  return notes.slice(0, QIONGLI_MAX_ITEM_NOTES).map((note) => ({
    note_key: boundedOutputString(note?.note_key ?? note?.key, 128),
    title: boundedOutputString(note?.title, 1024),
    summary: boundedOutputString(note?.summary, 500)
  })).filter((note) => note.note_key);
}

function boundedSearchLimit(value) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed)) {
    return QIONGLI_DEFAULT_SEARCH_LIMIT;
  }
  return Math.min(Math.max(parsed, 1), QIONGLI_MAX_SEARCH_LIMIT);
}

function comparableText(value) {
  return String(value ?? "").trim().toLowerCase();
}

function normalizeAttachments(value = [], options = {}) {
  const includePaths = options.include_attachment_paths === true || options.includeAttachmentPaths === true;
  const attachments = Array.isArray(value) ? value : [];

  return attachments
    .slice(0, QIONGLI_MAX_ITEM_ATTACHMENTS)
    .map((attachment) => {
      if (!attachment || typeof attachment !== "object" || Array.isArray(attachment)) {
        return null;
      }

      const attachmentKey = boundedOutputString(
        attachment.attachment_key ?? attachment.key ?? attachment.item_key,
        128
      );
      if (!attachmentKey) {
        return null;
      }

      const path = attachmentString(attachment.path);
      const normalized = {
        attachment_key: attachmentKey,
        title: boundedOutputString(attachment.title, 1024),
        filename: boundedOutputString(attachment.filename ?? attachment.attachmentFilename, 512),
        mime_type: boundedOutputString(
          attachment.mime_type
            ?? attachment.contentType
            ?? attachment.mimeType
            ?? attachment.attachmentContentType,
          128
        ),
        link_mode: boundedOutputString(
          attachment.link_mode ?? attachment.linkMode ?? attachment.attachmentLinkMode,
          128
        ),
        url: boundedOutputString(sanitizeAttachmentUrl(attachment.url ?? attachment.URL), 2048),
        select_uri: boundedOutputString(attachment.select_uri, 1024)
          || `zotero://select/library/items/${attachmentKey}`,
        local_file_available: Boolean(attachment.local_file_available ?? attachment.localFileAvailable ?? path)
      };

      if (includePaths && path) {
        normalized.path = boundedOutputString(path, 4096);
      }

      return normalized;
    })
    .filter(Boolean);
}

function normalizeStringSummaries(value, maximumItems, maximumChars, selector) {
  return (Array.isArray(value) ? value : [])
    .slice(0, maximumItems)
    .map((entry) => boundedOutputString(selector(entry), maximumChars))
    .filter(Boolean);
}

function boundedOutputString(value, maximumChars) {
  return attachmentString(value).slice(0, maximumChars);
}

function parseJson(postData) {
  if (!postData) {
    return {};
  }
  if (typeof postData === "object") {
    return postData;
  }
  return JSON.parse(postData);
}

function validateRequestPayload(value, { requireItems = false } = {}) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return "companion_request_invalid";
  }
  let serialized;
  try {
    serialized = canonicalJson(value);
  } catch {
    return "companion_request_invalid";
  }
  if (serialized.length > QIONGLI_MAX_REQUEST_JSON_CHARS) {
    return "companion_request_too_large";
  }
  if (requireItems) {
    if (!Array.isArray(value.items)
      || value.items.length > QIONGLI_MAX_UPSERT_ITEMS
      || value.items.some((item) => !item || typeof item !== "object" || Array.isArray(item))) {
      return value.items?.length > QIONGLI_MAX_UPSERT_ITEMS
        ? "companion_too_many_items"
        : "companion_request_invalid";
    }
  }
  return null;
}

function asArray(value) {
  if (Array.isArray(value)) {
    return value;
  }
  if (!value) {
    return [];
  }
  if (typeof value[Symbol.iterator] === "function") {
    return Array.from(value);
  }
  return [];
}

function getField(item, field) {
  return typeof item.getField === "function" ? item.getField(field) || "" : item[field] || "";
}

function attachmentString(value) {
  return String(value ?? "").trim();
}

function sanitizeAttachmentUrl(value) {
  const url = attachmentString(value);
  return isLocalAttachmentUrl(url) ? "" : url;
}

function isLocalAttachmentUrl(value) {
  return /^file:/i.test(value)
    || /^\//.test(value)
    || /^[A-Za-z]:[\\/]/.test(value);
}

function filenameFromPath(path) {
  return attachmentString(path).split(/[\\/]/).filter(Boolean).pop() ?? "";
}

function normalizeDoi(value) {
  return String(value ?? "").trim().replace(/^https?:\/\/(?:dx\.)?doi\.org\//i, "").toLowerCase();
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function parseYear(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  const match = String(value ?? "").match(/\b(\d{4})\b/);
  return match ? Number(match[1]) : null;
}
