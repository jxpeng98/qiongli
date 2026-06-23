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
        version: "0.2.2",
        endpoint_version: 1,
        zotero_version: Zotero.version ?? "",
        endpoints: this.endpoints
      });
    });

    registerEndpoint(Zotero, "/qiongli/search", ["POST"], async (postData, sendResponse) => {
      const query = parseJson(postData);
      const runtime = createRuntime(Zotero);
      const items = await runtime.listItems();
      sendJson(sendResponse, 200, {
        status: "ok",
        results: items.filter((item) => itemMatchesQuery(item, query)).map(toCompactItem)
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
      return asArray(rawItems)
        .filter((item) => typeof item.isRegularItem !== "function" || item.isRegularItem())
        .map((item) => itemToPlainObject(item));
    },

    async createItem(itemData) {
      const item = new Zotero.Item(itemData.itemType || "journalArticle");
      applyItemData(item, itemData);
      await item.saveTx();
      return itemToPlainObject(item);
    },

    async updateItem(key, patch) {
      const item = await getItemByKey(Zotero, key);
      applyItemData(item, patch);
      await item.saveTx();
      return itemToPlainObject(item);
    },

    async listCollections() {
      const libraryID = Zotero.Libraries?.userLibraryID;
      const collections = typeof Zotero.Collections?.getByLibrary === "function"
        ? await Zotero.Collections.getByLibrary(libraryID)
        : [];
      return asArray(collections).map((collection) => ({
        key: collection.key,
        name: collection.name,
        path: collection.name
      }));
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

function itemToPlainObject(item) {
  return {
    key: item.key,
    itemType: item.itemType,
    title: getField(item, "title"),
    DOI: getField(item, "DOI"),
    url: getField(item, "url"),
    abstractNote: getField(item, "abstractNote"),
    publicationTitle: getField(item, "publicationTitle"),
    date: getField(item, "date"),
    tags: typeof item.getTags === "function" ? item.getTags() : [],
    collections: typeof item.getCollections === "function" ? item.getCollections() : []
  };
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
  const dryRun = payload.dry_run !== false;
  const updatePolicy = payload.update_policy ?? "fill_blank";
  const incomingItems = Array.isArray(payload.items) ? payload.items : [];
  const existingItems = await runtime.listItems();
  const results = [];

  for (const incoming of incomingItems) {
    const existing = findDuplicateItem(incoming, existingItems);
    const plan = planUpsert(incoming, existing, updatePolicy);
    if (dryRun) {
      results.push({ ...plan, planned: true });
      continue;
    }
    if (!existing) {
      const created = await runtime.createItem(incoming);
      existingItems.push(created);
      results.push({ status: "created", item_key: created.key, item: toCompactItem(created) });
      continue;
    }
    if (plan.status === "unchanged") {
      results.push({ status: "unchanged", item_key: existing.key, item: toCompactItem(existing) });
      continue;
    }
    const updated = await runtime.updateItem(existing.key, plan.patch);
    Object.assign(existing, updated);
    results.push({ status: "updated", item_key: existing.key, patch: plan.patch, item: toCompactItem(existing) });
  }

  return { status: "ok", dry_run: dryRun, results };
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
    if (value === "" || value === null || value === undefined) {
      continue;
    }
    const existingValue = existing[field];
    if (updatePolicy === "prefer_enriched" || existingValue === "" || existingValue === null || existingValue === undefined) {
      patch[field] = value;
    }
  }
  return { status: Object.keys(patch).length > 0 ? "updated" : "unchanged", item_key: existing.key, patch };
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
  return true;
}

function toCompactItem(item) {
  return {
    item_key: item.key ?? "",
    title: item.title ?? "",
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: parseYear(item.date ?? item.year),
    item_type: item.itemType ?? "",
    select_uri: item.key ? `zotero://select/library/items/${item.key}` : "",
    tags: Array.isArray(item.tags) ? item.tags.map((tag) => tag.tag ?? tag).filter(Boolean) : [],
    collections: Array.isArray(item.collections) ? item.collections : []
  };
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
