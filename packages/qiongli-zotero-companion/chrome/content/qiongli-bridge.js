const ENDPOINTS = ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"];
const DOI_PREFIX_RE = /^https?:\/\/(?:dx\.)?doi\.org\//i;
const NON_ALNUM_RE = /[^a-z0-9]+/g;

export function qiongliPingResponse({ zoteroVersion = "" } = {}) {
  return {
    status: "ok",
    companion: "qiongli-zotero-companion",
    version: "0.2.2",
    endpoint_version: 1,
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
  const items = await listRuntimeItems(runtime);
  const results = items
    .filter((item) => itemMatchesQuery(item, query))
    .map((item) => toCompactItem(item, query));
  return {
    status: "ok",
    results
  };
}

export async function upsertItems(payload = {}, runtime = {}) {
  const dryRun = payload.dry_run !== false;
  const updatePolicy = payload.update_policy ?? "fill_blank";
  const collectionPath = normalizeCollectionPath(payload.collection_path);
  const incomingItems = Array.isArray(payload.items) ? payload.items : [];
  const existingItems = await listRuntimeItems(runtime);
  const targetCollection = !dryRun && collectionPath ? await ensureRuntimeCollection(runtime, collectionPath) : null;
  const results = [];

  for (const incoming of incomingItems) {
    const existing = findDuplicateItem(incoming, existingItems);
    const plan = planUpsert({ incoming, existing, updatePolicy });

    if (dryRun) {
      const plannedNotes = plannedNoteResults(incoming);
      results.push({
        ...plan,
        planned: true,
        item_key: plan.item_key ?? existing?.key ?? null,
        ...(collectionPath ? { collection_path: collectionPath } : {}),
        ...(plannedNotes.length > 0 ? { notes: plannedNotes } : {})
      });
      continue;
    }

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
      const notes = await createChildNotes({ item: existing, incoming, runtime });
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
    const notes = await createChildNotes({ item: existing, incoming, runtime });
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
    dry_run: dryRun,
    results
  };
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

function plannedNoteResults(incoming) {
  return normalizeIncomingNotes(incoming).map((note) => ({
    status: "planned",
    title: note.title
  }));
}

async function createChildNotes({ item, incoming, runtime }) {
  const notes = normalizeIncomingNotes(incoming);
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
    if (field === "qiongli_notes") {
      continue;
    }
    if (value === "" || value === null || value === undefined) {
      continue;
    }
    const existingValue = existing[field];
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

export function toCompactItem(item = {}, options = {}) {
  return {
    item_key: item.key ?? "",
    title: item.title ?? "",
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: normalizeYear(item.date ?? item.year),
    item_type: item.itemType ?? "",
    select_uri: item.key ? `zotero://select/library/items/${item.key}` : "",
    tags: Array.isArray(item.tags) ? item.tags.map((tag) => tag.tag ?? tag).filter(Boolean) : [],
    collections: Array.isArray(item.collections) ? item.collections : [],
    attachments: normalizeAttachments(item.attachments, options)
  };
}

export function normalizeAttachments(value = [], options = {}) {
  const includePaths = options.include_attachment_paths === true || options.includeAttachmentPaths === true;
  const attachments = Array.isArray(value) ? value : [];

  return attachments
    .map((attachment) => {
      if (!attachment || typeof attachment !== "object" || Array.isArray(attachment)) {
        return null;
      }

      const attachmentKey = attachmentString(attachment.attachment_key ?? attachment.key ?? attachment.item_key);
      if (!attachmentKey) {
        return null;
      }

      const path = attachmentString(attachment.path);
      const normalized = {
        attachment_key: attachmentKey,
        title: attachmentString(attachment.title),
        filename: attachmentString(attachment.filename ?? attachment.attachmentFilename),
        mime_type: attachmentString(attachment.mime_type ?? attachment.contentType ?? attachment.mimeType ?? attachment.attachmentContentType),
        link_mode: attachmentString(attachment.link_mode ?? attachment.linkMode ?? attachment.attachmentLinkMode),
        url: sanitizeAttachmentUrl(attachment.url ?? attachment.URL),
        select_uri: attachmentString(attachment.select_uri) || `zotero://select/library/items/${attachmentKey}`,
        local_file_available: Boolean(attachment.local_file_available ?? attachment.localFileAvailable ?? path)
      };

      if (includePaths && path) {
        normalized.path = path;
      }

      return normalized;
    })
    .filter(Boolean);
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

  return true;
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
