import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";
import { fetchJsonWithRetry } from "./http.mjs";

const PROVIDER = "crossref";
const ENDPOINT = "https://api.crossref.org/works";
const DEFAULT_LIMIT = 10;
const PAGE_LIMIT = 100;
const MAX_TOTAL_LIMIT = 200;

function normalizeTotalLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_TOTAL_LIMIT);
}

function normalizePageLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), PAGE_LIMIT);
}

function firstString(value) {
  if (Array.isArray(value)) {
    return value.find((item) => typeof item === "string") ?? null;
  }

  return typeof value === "string" ? value : null;
}

function crossrefAuthors(item) {
  if (!Array.isArray(item?.author)) {
    return [];
  }

  return item.author
    .map((author) => {
      const given = typeof author?.given === "string" ? author.given.trim() : "";
      const family = typeof author?.family === "string" ? author.family.trim() : "";
      return [given, family].filter(Boolean).join(" ");
    })
    .filter((name) => name !== "");
}

function yearFromDateParts(value) {
  const parts = value?.["date-parts"];
  const year = Array.isArray(parts) && Array.isArray(parts[0]) ? parts[0][0] : null;
  return Number.isInteger(year) ? year : null;
}

function crossrefYear(item) {
  return (
    yearFromDateParts(item?.["published-print"]) ??
    yearFromDateParts(item?.["published-online"]) ??
    yearFromDateParts(item?.issued) ??
    null
  );
}

function sourceId(item) {
  return typeof item?.DOI === "string" ? item.DOI : typeof item?.URL === "string" ? item.URL : null;
}

function crossrefReferences(item) {
  if (!Array.isArray(item?.reference)) {
    return [];
  }

  return item.reference.map((reference) => normalizeReference(reference));
}

function normalizeReference(reference) {
  const author = typeof reference?.author === "string" ? reference.author : null;
  return {
    title: reference?.["article-title"] ?? reference?.["series-title"] ?? reference?.unstructured,
    authors: author ? [author] : [],
    year: reference?.year,
    doi: reference?.DOI,
    url: null,
    provider: PROVIDER,
    source_id: null
  };
}

function mapWork(item) {
  return normalizeResult({
    title: firstString(item?.title),
    authors: crossrefAuthors(item),
    year: crossrefYear(item),
    doi: item?.DOI,
    url: item?.URL,
    abstract: item?.abstract,
    venue: firstString(item?.["container-title"]),
    document_type: item?.type,
    citation_count: item?.["is-referenced-by-count"],
    reference_count: item?.["reference-count"],
    citations: [],
    references: crossrefReferences(item),
    provider: PROVIDER,
    source_id: sourceId(item)
  });
}

function applyEmail(params, email) {
  const trimmedEmail = String(email ?? "").trim();
  if (trimmedEmail) {
    params.set("mailto", trimmedEmail);
  }
}

function normalizeDocumentTypes(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  return values
    .map((type) => String(type ?? "").trim())
    .filter((type) => type !== "");
}

function buildSearchUrl({ query, limit, cursor, email, fromYear, toYear, documentTypes }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("query.bibliographic", query);
  params.set("rows", String(normalizePageLimit(limit)));
  params.set("sort", "relevance");
  params.set("order", "desc");
  if (cursor) {
    params.set("cursor", cursor);
  }

  const filters = [];
  if (Number.isInteger(fromYear)) {
    filters.push(`from-pub-date:${fromYear}-01-01`);
  }
  if (Number.isInteger(toYear)) {
    filters.push(`until-pub-date:${toYear}-12-31`);
  }
  const typeFilters = normalizeDocumentTypes(documentTypes);
  if (typeFilters.length > 0) {
    filters.push(`type:${typeFilters.join("|")}`);
  }
  if (filters.length > 0) {
    params.set("filter", filters.join(","));
  }

  applyEmail(params, email);
  url.search = params.toString();
  return url;
}

function buildDoiUrl({ doi, email }) {
  const url = new URL(`${ENDPOINT}/${encodeURIComponent(doi)}`);
  const params = new URLSearchParams();
  applyEmail(params, email);
  url.search = params.toString();
  return url;
}

export async function searchCrossref({ query, doi, limit, email, fromYear, toYear, documentTypes, fetchImpl } = {}) {
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);

  try {
    if (resolvedDoi) {
      const lookup = await fetchJsonWithRetry({
        provider: PROVIDER,
        url: buildDoiUrl({ doi: resolvedDoi, email }),
        fetchImpl
      });
      if (lookup.error) {
        return {
          provider: PROVIDER,
          results: [],
          error: lookup.error,
          request_count: 1,
          attempts: lookup.attempts
        };
      }

      return {
        provider: PROVIDER,
        results: [lookup.body?.message].filter(Boolean).map(mapWork),
        error: null,
        request_count: 1,
        attempts: lookup.attempts
      };
    }

    const targetLimit = normalizeTotalLimit(limit);
    const results = [];
    let remaining = targetLimit;
    let cursor = "*";
    let requestCount = 0;
    let attempts = 0;

    while (remaining > 0) {
      const pageLimit = Math.min(remaining, PAGE_LIMIT);
      const page = await fetchJsonWithRetry({
        provider: PROVIDER,
        url: buildSearchUrl({ query, limit: pageLimit, cursor, email, fromYear, toYear, documentTypes }),
        fetchImpl
      });
      requestCount += 1;
      attempts += page.attempts;
      if (page.error) {
        return {
          provider: PROVIDER,
          results: [],
          error: page.error,
          request_count: requestCount,
          attempts
        };
      }

      const items = Array.isArray(page.body?.message?.items) ? page.body.message.items : [];
      results.push(...items.map(mapWork));
      remaining -= pageLimit;
      cursor = page.body?.message?.["next-cursor"];
      if (!cursor || items.length === 0) {
        break;
      }
    }

    return {
      provider: PROVIDER,
      results,
      error: null,
      request_count: requestCount,
      attempts
    };
  } catch (error) {
    return {
      provider: PROVIDER,
      results: [],
      error: `${PROVIDER} request failed: ${error?.name ?? "Error"}`,
      request_count: 0,
      attempts: 0
    };
  }
}
