import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";
import { fetchJsonWithRetry } from "./http.mjs";

const PROVIDER = "semantic_scholar";
const ENDPOINT = "https://api.semanticscholar.org/graph/v1/paper/search";
const MATCH_ENDPOINT = "https://api.semanticscholar.org/graph/v1/paper/search/match";
const PAPER_ENDPOINT = "https://api.semanticscholar.org/graph/v1/paper";
const DEFAULT_LIMIT = 25;
const PAGE_LIMIT = 100;
const MAX_TOTAL_LIMIT = 200;
const BASE_FIELDS = [
  "paperId",
  "title",
  "year",
  "authors",
  "abstract",
  "url",
  "venue",
  "publicationTypes",
  "externalIds"
];
const CITATION_FIELDS = [
  "citationCount",
  "citations.paperId",
  "citations.title",
  "citations.year",
  "citations.authors",
  "citations.externalIds",
  "citations.url"
];
const REFERENCE_FIELDS = [
  "referenceCount",
  "references.paperId",
  "references.title",
  "references.year",
  "references.authors",
  "references.externalIds",
  "references.url"
];

function normalizeLimit(limit) {
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

function buildYearFilter(fromYear, toYear) {
  if (Number.isInteger(fromYear) && Number.isInteger(toYear)) {
    return `${fromYear}-${toYear}`;
  }

  if (Number.isInteger(fromYear)) {
    return `${fromYear}-`;
  }

  if (Number.isInteger(toYear)) {
    return `-${toYear}`;
  }

  return null;
}

function fieldsFor({ includeCitations, includeReferences } = {}) {
  const fields = [...BASE_FIELDS];
  if (includeCitations) {
    fields.push(...CITATION_FIELDS);
  }
  if (includeReferences) {
    fields.push(...REFERENCE_FIELDS);
  }
  return fields.join(",");
}

function buildUrl({ query, limit, offset, fromYear, toYear, includeCitations, includeReferences }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("query", query);
  params.set("limit", String(normalizePageLimit(limit)));
  params.set("offset", String(Number.isInteger(offset) ? offset : 0));
  params.set("fields", fieldsFor({ includeCitations, includeReferences }));

  const year = buildYearFilter(fromYear, toYear);
  if (year) {
    params.set("year", year);
  }

  url.search = params.toString();
  return url;
}

function buildMatchUrl({ query }) {
  const url = new URL(MATCH_ENDPOINT);
  const params = new URLSearchParams();
  params.set("query", query);
  params.set("fields", fieldsFor());
  url.search = params.toString();
  return url;
}

function buildDoiUrl({ doi, includeCitations, includeReferences }) {
  const url = new URL(`${PAPER_ENDPOINT}/${encodeURIComponent(`DOI:${doi}`)}`);
  const params = new URLSearchParams();
  params.set("fields", fieldsFor({ includeCitations, includeReferences }));
  url.search = params.toString();
  return url;
}

function headersFor(apiKey) {
  const trimmedApiKey = String(apiKey ?? "").trim();
  if (!trimmedApiKey) {
    return {};
  }

  return {
    "x-api-key": trimmedApiKey
  };
}

function fetchOptions(fetchImpl, apiKey) {
  const options = {
    headers: headersFor(apiKey)
  };

  if (!fetchImpl) {
    options.signal = AbortSignal.timeout(15000);
  }

  return options;
}

function authorsFor(paper) {
  if (!Array.isArray(paper?.authors)) {
    return [];
  }

  return paper.authors
    .map((author) => author?.name)
    .filter((name) => typeof name === "string");
}

function doiFor(paper) {
  return paper?.externalIds?.DOI ?? paper?.externalIds?.doi ?? null;
}

function linkedPapers(papers) {
  if (!Array.isArray(papers)) {
    return [];
  }

  return papers.map((paper) => ({
    title: paper?.title,
    authors: authorsFor(paper),
    year: paper?.year,
    doi: doiFor(paper),
    url: paper?.url,
    provider: PROVIDER,
    source_id: paper?.paperId
  }));
}

function publicationTypeFor(paper) {
  if (!Array.isArray(paper?.publicationTypes)) {
    return null;
  }

  return paper.publicationTypes.find((type) => typeof type === "string") ?? null;
}

function mapPaper(paper) {
  return normalizeResult({
    title: paper?.title,
    authors: authorsFor(paper),
    year: paper?.year,
    doi: doiFor(paper),
    url: paper?.url,
    abstract: paper?.abstract,
    venue: paper?.venue,
    document_type: publicationTypeFor(paper),
    citation_count: paper?.citationCount,
    reference_count: paper?.referenceCount,
    citations: linkedPapers(paper?.citations),
    references: linkedPapers(paper?.references),
    provider: PROVIDER,
    source_id: paper?.paperId
  });
}

function papersFromBody(body) {
  if (Array.isArray(body?.data)) {
    return body.data;
  }

  if (body?.data && typeof body.data === "object") {
    return [body.data];
  }

  if (body?.paperId || body?.title) {
    return [body];
  }

  return [];
}

function errorMessage(response) {
  return `${PROVIDER} HTTP ${response.status}`;
}

async function fetchPapers(fetcher, url, options) {
  const fetched = await fetchJsonWithRetry({
    provider: PROVIDER,
    url,
    fetchImpl: fetcher === fetch ? undefined : fetcher,
    options
  });
  if (fetched.error) {
    return {
      results: [],
      error: fetched.error,
      request_count: 1,
      attempts: fetched.attempts
    };
  }

  return {
    results: papersFromBody(fetched.body).map(mapPaper),
    error: null,
    request_count: 1,
    attempts: fetched.attempts
  };
}

export async function searchSemanticScholar({ query, doi, exactTitle, searchMode, limit, apiKey, fromYear, toYear, includeCitations, includeReferences, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);
  const shouldMatchTitle = exactTitle === true || searchMode === "title";
  const options = fetchOptions(fetchImpl, apiKey);

  try {
    if (resolvedDoi) {
      const lookup = await fetchPapers(fetcher, buildDoiUrl({ doi: resolvedDoi, includeCitations, includeReferences }), options);
      return {
        provider: PROVIDER,
        results: lookup.results,
        error: lookup.error,
        request_count: lookup.request_count,
        attempts: lookup.attempts
      };
    }

    const lookups = [];
    if (shouldMatchTitle) {
      lookups.push(await fetchPapers(fetcher, buildMatchUrl({ query }), options));
    }
    const targetLimit = normalizeLimit(limit);
    let remaining = targetLimit;
    let offset = 0;
    while (remaining > 0) {
      const pageLimit = Math.min(remaining, PAGE_LIMIT);
      const lookup = await fetchPapers(
        fetcher,
        buildUrl({ query, limit: pageLimit, offset, fromYear, toYear, includeCitations, includeReferences }),
        options
      );
      lookups.push(lookup);
      if (lookup.error || lookup.results.length === 0) {
        break;
      }
      remaining -= pageLimit;
      offset += pageLimit;
    }

    const successful = lookups.filter((lookup) => !lookup.error);
    const requestCount = lookups.reduce((sum, lookup) => sum + (lookup.request_count ?? 0), 0);
    const attempts = lookups.reduce((sum, lookup) => sum + (lookup.attempts ?? 0), 0);
    if (successful.length === 0) {
      return {
        provider: PROVIDER,
        results: [],
        error: lookups[0]?.error ?? `${PROVIDER} request failed: Error`,
        request_count: requestCount,
        attempts
      };
    }

    return {
      provider: PROVIDER,
      results: successful.flatMap((lookup) => lookup.results),
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
