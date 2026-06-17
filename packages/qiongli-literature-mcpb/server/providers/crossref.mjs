import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";

const PROVIDER = "crossref";
const ENDPOINT = "https://api.crossref.org/works";
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 100;

function normalizeLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_LIMIT);
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

function buildSearchUrl({ query, limit, email, fromYear, toYear, documentTypes }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("query.bibliographic", query);
  params.set("rows", String(normalizeLimit(limit)));
  params.set("sort", "relevance");
  params.set("order", "desc");

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

function fetchOptions(fetchImpl) {
  if (fetchImpl) {
    return {};
  }

  return { signal: AbortSignal.timeout(15000) };
}

function errorMessage(response) {
  return `${PROVIDER} HTTP ${response.status}`;
}

export async function searchCrossref({ query, doi, limit, email, fromYear, toYear, documentTypes, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);
  const url = resolvedDoi
    ? buildDoiUrl({ doi: resolvedDoi, email })
    : buildSearchUrl({ query, limit, email, fromYear, toYear, documentTypes });

  try {
    const response = await fetcher(url, fetchOptions(fetchImpl));
    if (!response.ok) {
      return {
        provider: PROVIDER,
        results: [],
        error: errorMessage(response)
      };
    }

    const body = await response.json();
    const items = resolvedDoi
      ? [body?.message]
      : Array.isArray(body?.message?.items) ? body.message.items : [];
    return {
      provider: PROVIDER,
      results: items.filter(Boolean).map(mapWork),
      error: null
    };
  } catch (error) {
    return {
      provider: PROVIDER,
      results: [],
      error: `${PROVIDER} request failed: ${error?.name ?? "Error"}`
    };
  }
}
