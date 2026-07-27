const ENDPOINTS = ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"];
const DOI_PREFIX_RE = /^https?:\/\/(?:dx\.)?doi\.org\//i;
const NON_ALNUM_RE = /[^a-z0-9]+/g;
const WRITE_RECEIPT_PREFIX = "zwr1_";
const WRITE_RECEIPT_TTL_MS = 5 * 60 * 1000;
const MAX_WRITE_RECEIPTS = 64;
const DEFAULT_SEARCH_LIMIT = 25;
const MAX_SEARCH_LIMIT = 200;
const MAX_UPSERT_ITEMS = 100;
const MAX_REQUEST_JSON_CHARS = 1024 * 1024;
const MAX_ITEM_CREATORS = 50;
const MAX_ITEM_TAGS = 100;
const MAX_ITEM_COLLECTIONS = 100;
const MAX_ITEM_NOTES = 20;
const MAX_ITEM_ATTACHMENTS = 20;
const writeApprovals = new Map();

export function qiongliPingResponse({ zoteroVersion = "" } = {}) {
  return {
    status: "ok",
    companion: "qiongli-zotero-companion",
    version: "0.3.0",
    endpoint_version: "2",
    zotero_version: zoteroVersion,
    endpoints: ENDPOINTS
  };
}

export function findDuplicateItem(incoming = {}, existingItems = []) {
  const incomingDoi = normalizeDoi(incoming.DOI ?? incoming.doi);
  if (incomingDoi) {
    const doiMatch = existingItems.find((item) => normalizeDoi(item.DOI ?? item.doi) === incomingDoi);
    if (doiMatch) {
      return doiMatch;
    }
  }

  const incomingTitle = comparableTitle(incoming.title);
  const incomingYear = normalizeYear(incoming.date ?? incoming.year);
  if (!incomingTitle) {
    return null;
  }

  return existingItems.find((item) => {
    const titleMatches = comparableTitle(item.title) === incomingTitle;
    if (!titleMatches) {
      return false;
    }
    if (!incomingYear) {
      return true;
    }
    return normalizeYear(item.date ?? item.year) === incomingYear;
  }) ?? null;
}

export async function searchLocalItems(query = {}, runtime = {}) {
  const requestError = validateRequestPayload(query);
  if (requestError) {
    return { status: "error", error_code: requestError, results: [] };
  }
  const items = await listRuntimeItems(runtime);
  const limit = boundedSearchLimit(query.limit);
  const results = items
    .filter((item) => itemMatchesQuery(item, query))
    .slice(0, limit)
    .map((item) => toCompactItem(item, query));
  return {
    status: "ok",
    limit,
    results
  };
}

export async function upsertItems(payload = {}, runtime = {}) {
  const requestError = validateRequestPayload(payload, { requireItems: true });
  if (requestError) {
    return { status: "error", error_code: requestError, dry_run: true, results: [] };
  }
  const dryRun = payload.dry_run !== false;
  const updatePolicy = payload.update_policy ?? "fill_blank";
  const collectionPath = normalizeCollectionPath(payload.collection_path);
  const incomingItems = Array.isArray(payload.items) ? payload.items : [];
  const existingItems = await listRuntimeItems(runtime);
  const plans = incomingItems.map((incoming) => {
    const existing = findDuplicateItem(incoming, existingItems);
    return {
      existing,
      notePlans: plannedNoteResults(incoming, existing),
      plan: planUpsert({ incoming, existing, updatePolicy })
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
      write_approval: issueWriteApproval(approvalPlan, runtime)
    };
  }

  const approvalError = consumeWriteApproval({
    approvalPlan,
    receipt: payload.dry_run_receipt,
    runtime,
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
      write_approval: issueWriteApproval(approvalPlan, runtime)
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
    write_approval: {
      consumed: true
    },
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

function issueWriteApproval(approvalPlan, runtime) {
  pruneWriteApprovals(runtime);
  while (writeApprovals.size >= MAX_WRITE_RECEIPTS) {
    writeApprovals.delete(writeApprovals.keys().next().value);
  }
  const receipt = `${WRITE_RECEIPT_PREFIX}${randomHex(32, runtime)}`;
  writeApprovals.set(receipt, {
    approvalPlan,
    expiresAt: nowMilliseconds(runtime) + WRITE_RECEIPT_TTL_MS
  });
  return {
    receipt,
    expires_in_seconds: WRITE_RECEIPT_TTL_MS / 1000,
    required_write_intent: "apply"
  };
}

function consumeWriteApproval({ approvalPlan, receipt, runtime, writeIntent }) {
  pruneWriteApprovals(runtime);
  if (writeIntent !== "apply") {
    return "zotero_write_intent_required";
  }
  if (typeof receipt !== "string" || !receipt.startsWith(WRITE_RECEIPT_PREFIX)) {
    return "zotero_dry_run_receipt_required";
  }
  const stored = writeApprovals.get(receipt);
  writeApprovals.delete(receipt);
  if (!stored) {
    return "zotero_dry_run_receipt_invalid";
  }
  return stored.approvalPlan === approvalPlan
    ? null
    : "zotero_dry_run_plan_changed";
}

function pruneWriteApprovals(runtime) {
  const now = nowMilliseconds(runtime);
  for (const [receipt, approval] of writeApprovals) {
    if (approval.expiresAt <= now) {
      writeApprovals.delete(receipt);
    }
  }
}

function nowMilliseconds(runtime) {
  return typeof runtime.nowMilliseconds === "function"
    ? runtime.nowMilliseconds()
    : Date.now();
}

function randomHex(byteLength, runtime) {
  if (typeof runtime.randomBytes === "function") {
    const value = runtime.randomBytes(byteLength);
    if (value instanceof Uint8Array && value.length === byteLength) {
      return [...value].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    }
  }
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
  if (serialized.length > MAX_REQUEST_JSON_CHARS) {
    return "companion_request_too_large";
  }
  if (requireItems) {
    if (!Array.isArray(value.items)
      || value.items.length > MAX_UPSERT_ITEMS
      || value.items.some((item) => !item || typeof item !== "object" || Array.isArray(item))) {
      return value.items?.length > MAX_UPSERT_ITEMS
        ? "companion_too_many_items"
        : "companion_request_invalid";
    }
  }
  return null;
}

function normalizeCollectionPath(value) {
  const parts = String(value ?? "")
    .split("/")
    .map((part) => part.trim())
    .filter(Boolean);
  return parts.join("/");
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
  const incomingSummary = htmlTextSummary(incomingNote.html);
  return (Array.isArray(existingNotes) ? existingNotes : []).some((note) => (
    attachmentString(note?.title) === incomingNote.title
    && attachmentString(note?.summary) === incomingSummary
  ));
}

function htmlTextSummary(value) {
  return String(value ?? "")
    .replace(/<[^>]*>/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 500);
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

export function planUpsert({ incoming = {}, existing = null, updatePolicy = "fill_blank" } = {}) {
  if (!existing) {
    return {
      status: "created",
      item: incoming,
      patch: incoming
    };
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
    if (updatePolicy === "prefer_enriched" || isBlank(existingValue)) {
      if (!valuesEqual(existingValue, value)) {
        patch[field] = value;
      }
    }
  }

  return {
    status: Object.keys(patch).length > 0 ? "updated" : "unchanged",
    item_key: existing.key,
    patch
  };
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

export function toCompactItem(item = {}, options = {}) {
  const itemKey = boundedOutputString(item.key, 128);
  return {
    item_key: itemKey,
    title: boundedOutputString(item.title, 2048),
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: normalizeYear(item.date ?? item.year),
    item_type: boundedOutputString(item.itemType, 128),
    citekey: boundedOutputString(item.citekey ?? item.citationKey, 512),
    creators: normalizeCreators(item.creators),
    url: boundedOutputString(sanitizeAttachmentUrl(item.url ?? item.URL), 2048),
    abstract: boundedOutputString(item.abstract ?? item.abstractNote, 16384),
    venue: boundedOutputString(item.venue ?? item.publicationTitle, 1024),
    select_uri: itemKey ? `zotero://select/library/items/${itemKey}` : "",
    tags: normalizeStringSummaries(item.tags, MAX_ITEM_TAGS, 256, (tag) => tag?.tag ?? tag),
    collections: normalizeStringSummaries(
      item.collections,
      MAX_ITEM_COLLECTIONS,
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
    .slice(0, MAX_ITEM_CREATORS)
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
  return notes.slice(0, MAX_ITEM_NOTES).map((note) => ({
    note_key: boundedOutputString(note?.note_key ?? note?.key, 128),
    title: boundedOutputString(note?.title, 1024),
    summary: boundedOutputString(note?.summary, 500)
  })).filter((note) => note.note_key);
}

export function normalizeAttachments(value = [], options = {}) {
  const includePaths = options.include_attachment_paths === true || options.includeAttachmentPaths === true;
  const attachments = Array.isArray(value) ? value : [];

  return attachments
    .slice(0, MAX_ITEM_ATTACHMENTS)
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

export function normalizeDoi(value) {
  return String(value ?? "").trim().replace(DOI_PREFIX_RE, "").toLowerCase();
}

async function listRuntimeItems(runtime) {
  if (typeof runtime.listItems !== "function") {
    return [];
  }
  const items = await runtime.listItems();
  return Array.isArray(items) ? items : [];
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

  const queryYear = normalizeYear(query.year);
  if (queryYear && normalizeYear(item.date ?? item.year) !== queryYear) {
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

function boundedSearchLimit(value) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed)) {
    return DEFAULT_SEARCH_LIMIT;
  }
  return Math.min(Math.max(parsed, 1), MAX_SEARCH_LIMIT);
}

function comparableText(value) {
  return String(value ?? "").trim().toLowerCase();
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(NON_ALNUM_RE, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function normalizeYear(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  const match = String(value ?? "").match(/\b(\d{4})\b/);
  return match ? Number(match[1]) : null;
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

function isBlank(value) {
  if (Array.isArray(value)) {
    return value.length === 0;
  }
  return value === "" || value === null || value === undefined;
}

function valuesEqual(left, right) {
  return JSON.stringify(left ?? null) === JSON.stringify(right ?? null);
}
