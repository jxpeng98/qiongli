const ENDPOINTS = ["/qiongli/ping", "/qiongli/search", "/qiongli/upsertItems", "/qiongli/collections"];
const DOI_PREFIX_RE = /^https?:\/\/(?:dx\.)?doi\.org\//i;
const NON_ALNUM_RE = /[^a-z0-9]+/g;

export function qiongliPingResponse({ zoteroVersion = "" } = {}) {
  return {
    status: "ok",
    companion: "qiongli-zotero-companion",
    version: "0.1.0",
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
    .map((item) => toCompactItem(item));
  return {
    status: "ok",
    results
  };
}

export async function upsertItems(payload = {}, runtime = {}) {
  const dryRun = payload.dry_run !== false;
  const updatePolicy = payload.update_policy ?? "fill_blank";
  const incomingItems = Array.isArray(payload.items) ? payload.items : [];
  const existingItems = await listRuntimeItems(runtime);
  const results = [];

  for (const incoming of incomingItems) {
    const existing = findDuplicateItem(incoming, existingItems);
    const plan = planUpsert({ incoming, existing, updatePolicy });

    if (dryRun) {
      results.push({
        ...plan,
        planned: true,
        item_key: plan.item_key ?? existing?.key ?? null
      });
      continue;
    }

    if (!existing) {
      const created = await runtime.createItem(incoming);
      existingItems.push(created);
      results.push({
        status: "created",
        item_key: created.key,
        item: toCompactItem(created)
      });
      continue;
    }

    if (plan.status === "unchanged") {
      results.push({
        status: "unchanged",
        item_key: existing.key,
        item: toCompactItem(existing)
      });
      continue;
    }

    const updated = await runtime.updateItem(existing.key, plan.patch);
    Object.assign(existing, updated);
    results.push({
      status: "updated",
      item_key: existing.key,
      patch: plan.patch,
      item: toCompactItem(existing)
    });
  }

  return {
    status: "ok",
    dry_run: dryRun,
    results
  };
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

export function toCompactItem(item = {}) {
  return {
    item_key: item.key ?? "",
    title: item.title ?? "",
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: normalizeYear(item.date ?? item.year),
    item_type: item.itemType ?? "",
    select_uri: item.key ? `zotero://select/library/items/${item.key}` : "",
    tags: Array.isArray(item.tags) ? item.tags.map((tag) => tag.tag ?? tag).filter(Boolean) : [],
    collections: Array.isArray(item.collections) ? item.collections : []
  };
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

function isBlank(value) {
  if (Array.isArray(value)) {
    return value.length === 0;
  }
  return value === "" || value === null || value === undefined;
}

function valuesEqual(left, right) {
  return JSON.stringify(left ?? null) === JSON.stringify(right ?? null);
}
