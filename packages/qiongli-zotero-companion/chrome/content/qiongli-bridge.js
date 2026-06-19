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
  if (!incomingTitle || !incomingYear) {
    return null;
  }

  return existingItems.find((item) => {
    return comparableTitle(item.title) === incomingTitle && normalizeYear(item.date ?? item.year) === incomingYear;
  }) ?? null;
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
