import { postCompanionJson } from "./client.mjs";

const DEFAULT_ZOTERO_LIMIT = 25;
const MAX_ZOTERO_LIMIT = 200;
const DOI_PREFIX_RE = /^https?:\/\/(?:dx\.)?doi\.org\//i;

export function resolveZoteroSourceOptions(input = {}, { perProviderLimit = DEFAULT_ZOTERO_LIMIT } = {}) {
  const include = input.include_zotero === true;
  const requestedLimit = integerOrNull(input.zotero_limit);
  const fallbackLimit = Math.max(1, Math.min(integerOrNull(perProviderLimit) ?? DEFAULT_ZOTERO_LIMIT, DEFAULT_ZOTERO_LIMIT));

  return {
    include,
    limit: include ? Math.min(Math.max(requestedLimit ?? fallbackLimit, 1), MAX_ZOTERO_LIMIT) : 0,
    tag: cleanString(input.zotero_tag),
    collection_path: cleanString(input.zotero_collection_path)
  };
}

export function zoteroSourceSearchPayload({ intent = {}, input = {}, sourceOptions = {} } = {}) {
  const payload = {};
  const doi = normalizeDoi(intent.doi);
  const title = cleanString(intent.query);
  const tag = sourceOptions.tag ?? cleanString(input.zotero_tag);
  const collectionPath = sourceOptions.collection_path ?? cleanString(input.zotero_collection_path);

  if (doi) {
    payload.doi = doi;
  } else if (title) {
    payload.title = title;
  }
  if (tag) {
    payload.tag = tag;
  }
  if (collectionPath) {
    payload.collection_path = collectionPath;
  }
  if (sourceOptions.limit) {
    payload.limit = sourceOptions.limit;
  }
  return payload;
}

export async function searchZoteroSource({ config, intent, input, sourceOptions, context = {} }) {
  const payload = zoteroSourceSearchPayload({ intent, input, sourceOptions });
  const query = payload.doi ?? payload.title ?? "";

  try {
    const response = await postCompanionJson(config, "/qiongli/search", payload, context);
    if (response.status === "error") {
      return zoteroSourceResponse({
        query,
        results: [],
        error: response.error_code ?? "zotero_companion_missing"
      });
    }

    return zoteroSourceResponse({
      query,
      results: normalizeZoteroSourceResults(response.results).slice(0, sourceOptions.limit),
      error: null
    });
  } catch (error) {
    return zoteroSourceResponse({
      query,
      results: [],
      error: zoteroSourceErrorCode(error)
    });
  }
}

export function normalizeZoteroSourceResults(items = []) {
  return (Array.isArray(items) ? items : [])
    .map((item) => {
      const itemKey = cleanString(item.item_key ?? item.key);
      return {
        title: cleanString(item.title),
        authors: normalizeAuthors(item.authors ?? item.creators),
        year: integerOrNull(item.year),
        doi: normalizeDoi(item.doi ?? item.DOI),
        url: cleanString(item.url ?? item.URL),
        abstract: cleanString(item.abstract ?? item.abstractNote),
        venue: cleanString(item.venue ?? item.publicationTitle),
        document_type: cleanString(item.item_type ?? item.document_type ?? item.itemType),
        citation_count: null,
        reference_count: null,
        citations: [],
        references: [],
        provider: "zotero",
        source_id: itemKey,
        source_type: "local_reference_database",
        zotero: {
          item_key: itemKey,
          select_uri: cleanString(item.select_uri),
          tags: normalizeStringList(item.tags),
          collections: normalizeStringList(item.collections)
        }
      };
    })
    .filter((item) => item.title || item.doi || item.source_id);
}

export function annotateLocalZoteroMatches({ externalResults = [], zoteroResults = [] } = {}) {
  const byDoi = new Map();
  const byTitleYear = new Map();

  for (const result of zoteroResults) {
    const doi = normalizeDoi(result.doi);
    if (doi) {
      byDoi.set(doi.toLowerCase(), result);
    }

    const titleKey = titleYearKey(result);
    if (titleKey) {
      byTitleYear.set(titleKey, result);
    }
  }

  return externalResults.map((result) => {
    const doi = normalizeDoi(result.doi);
    const doiMatch = doi ? byDoi.get(doi.toLowerCase()) : null;
    const titleMatch = doiMatch ? null : byTitleYear.get(titleYearKey(result));
    const match = doiMatch ?? titleMatch;
    if (!match) {
      return result;
    }

    return {
      ...result,
      local_zotero_match: {
        item_key: match.zotero?.item_key ?? match.source_id ?? "",
        match_basis: doiMatch ? "doi" : "title_year",
        select_uri: match.zotero?.select_uri ?? ""
      }
    };
  });
}

export function zoteroSourceWarning(response) {
  if (!response?.error) {
    return null;
  }
  return response.error === "zotero_not_running" ? "zotero_not_running" : "zotero_companion_missing";
}

function zoteroSourceResponse({ query, results, error }) {
  return {
    provider: "zotero",
    query_id: "zotero",
    query,
    results,
    error,
    request_count: 1,
    attempts: 1,
    source_type: "local_reference_database"
  };
}

function zoteroSourceErrorCode(error) {
  const message = String(error?.message ?? "");
  return message.includes("ECONNREFUSED") || message.includes("fetch failed")
    ? "zotero_not_running"
    : "zotero_companion_missing";
}

function titleYearKey(record) {
  const title = comparableTitle(record?.title);
  if (!title) {
    return "";
  }
  return `${title}:${integerOrNull(record?.year) ?? ""}`;
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^\p{Letter}\p{Number}]+/gu, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function normalizeDoi(value) {
  const cleaned = cleanString(value);
  return cleaned ? cleaned.replace(DOI_PREFIX_RE, "").trim() : null;
}

function normalizeAuthors(value) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .map((author) => {
      if (typeof author === "string") {
        return cleanString(author);
      }
      if (author && typeof author === "object") {
        const name = cleanString(author.name);
        if (name) {
          return name;
        }
        const firstName = cleanString(author.firstName);
        const lastName = cleanString(author.lastName);
        return [lastName, firstName].filter(Boolean).join(", ") || null;
      }
      return null;
    })
    .filter(Boolean);
}

function normalizeStringList(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  return values.map(cleanString).filter(Boolean);
}

function integerOrNull(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return Math.trunc(value);
  }
  const cleaned = typeof value === "string" ? value.trim() : "";
  if (!cleaned || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) {
    return null;
  }
  return Math.trunc(Number(cleaned));
}

function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}
