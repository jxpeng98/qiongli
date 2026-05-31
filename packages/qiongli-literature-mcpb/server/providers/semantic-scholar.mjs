import { normalizeResult } from "../normalize.mjs";

const PROVIDER = "semantic_scholar";
const ENDPOINT = "https://api.semanticscholar.org/graph/v1/paper/search";
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 100;
const FIELDS = "paperId,title,year,authors,abstract,url,venue,externalIds";

function normalizeLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_LIMIT);
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

function buildUrl({ query, limit, fromYear, toYear }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("query", query);
  params.set("limit", String(normalizeLimit(limit)));
  params.set("fields", FIELDS);

  const year = buildYearFilter(fromYear, toYear);
  if (year) {
    params.set("year", year);
  }

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

function mapPaper(paper) {
  return normalizeResult({
    title: paper?.title,
    authors: authorsFor(paper),
    year: paper?.year,
    doi: doiFor(paper),
    url: paper?.url,
    abstract: paper?.abstract,
    venue: paper?.venue,
    provider: PROVIDER,
    source_id: paper?.paperId
  });
}

function errorMessage(response) {
  return `${PROVIDER} HTTP ${response.status}`;
}

export async function searchSemanticScholar({ query, limit, apiKey, fromYear, toYear, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const url = buildUrl({ query, limit, fromYear, toYear });

  try {
    const response = await fetcher(url, fetchOptions(fetchImpl, apiKey));
    if (!response.ok) {
      return {
        provider: PROVIDER,
        results: [],
        error: errorMessage(response)
      };
    }

    const body = await response.json();
    const papers = Array.isArray(body?.data) ? body.data : [];
    return {
      provider: PROVIDER,
      results: papers.map(mapPaper),
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
