import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";

const PROVIDER = "pubmed";
const BASE_URL = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 100;

function normalizeLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_LIMIT);
}

function applyApiKey(params, apiKey) {
  const trimmedApiKey = String(apiKey ?? "").trim();
  if (trimmedApiKey) {
    params.set("api_key", trimmedApiKey);
  }
}

function buildSearchTerm({ query, doi }) {
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);
  if (resolvedDoi) {
    return `${resolvedDoi}[doi]`;
  }

  return String(query ?? "").trim();
}

function buildSearchUrl({ query, doi, limit, apiKey, fromYear, toYear }) {
  const url = new URL(`${BASE_URL}/esearch.fcgi`);
  const params = new URLSearchParams();
  params.set("db", "pubmed");
  params.set("retmode", "json");
  params.set("sort", "relevance");
  params.set("term", buildSearchTerm({ query, doi }));
  params.set("retmax", String(normalizeLimit(limit)));
  if (Number.isInteger(fromYear)) {
    params.set("mindate", String(fromYear));
  }
  if (Number.isInteger(toYear)) {
    params.set("maxdate", String(toYear));
  }
  if (Number.isInteger(fromYear) || Number.isInteger(toYear)) {
    params.set("datetype", "pdat");
  }
  applyApiKey(params, apiKey);
  url.search = params.toString();
  return url;
}

function buildSummaryUrl({ ids, apiKey }) {
  const url = new URL(`${BASE_URL}/esummary.fcgi`);
  const params = new URLSearchParams();
  params.set("db", "pubmed");
  params.set("retmode", "json");
  params.set("id", ids.join(","));
  applyApiKey(params, apiKey);
  url.search = params.toString();
  return url;
}

function fetchOptions(fetchImpl) {
  if (fetchImpl) {
    return {};
  }

  return { signal: AbortSignal.timeout(15000) };
}

function authorsFor(record) {
  if (!Array.isArray(record?.authors)) {
    return [];
  }

  return record.authors
    .map((author) => author?.name)
    .filter((name) => typeof name === "string");
}

function yearFor(record) {
  const match = String(record?.pubdate ?? "").match(/\b(\d{4})\b/);
  return match ? Number(match[1]) : null;
}

function doiFor(record) {
  if (!Array.isArray(record?.articleids)) {
    return record?.elocationid ?? null;
  }

  const doi = record.articleids.find((item) => String(item?.idtype ?? "").toLowerCase() === "doi");
  return doi?.value ?? record?.elocationid ?? null;
}

function documentTypeFor(record) {
  if (!Array.isArray(record?.pubtype)) {
    return null;
  }

  return record.pubtype.find((type) => typeof type === "string") ?? null;
}

function mapSummary(record) {
  const sourceId = String(record?.uid ?? "").trim() || null;
  return normalizeResult({
    title: record?.title,
    authors: authorsFor(record),
    year: yearFor(record),
    doi: doiFor(record),
    url: sourceId ? `https://pubmed.ncbi.nlm.nih.gov/${sourceId}/` : null,
    abstract: null,
    venue: record?.fulljournalname ?? record?.source,
    document_type: documentTypeFor(record),
    citation_count: null,
    reference_count: null,
    citations: [],
    references: [],
    provider: PROVIDER,
    source_id: sourceId
  });
}

function idsFromSearch(body) {
  return Array.isArray(body?.esearchresult?.idlist)
    ? body.esearchresult.idlist.filter((id) => typeof id === "string" && id.trim() !== "")
    : [];
}

function summariesFromBody(body) {
  const uids = Array.isArray(body?.result?.uids) ? body.result.uids : [];
  return uids
    .map((uid) => body?.result?.[uid])
    .filter((record) => record && typeof record === "object");
}

function errorMessage(response) {
  return `${PROVIDER} HTTP ${response.status}`;
}

async function fetchJson(fetcher, url, options) {
  const response = await fetcher(url, options);
  if (!response.ok) {
    return {
      body: null,
      error: errorMessage(response)
    };
  }

  return {
    body: await response.json(),
    error: null
  };
}

export async function searchPubMed({ query, doi, limit, apiKey, fromYear, toYear, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const options = fetchOptions(fetchImpl);

  try {
    const search = await fetchJson(
      fetcher,
      buildSearchUrl({ query, doi, limit, apiKey, fromYear, toYear }),
      options
    );
    if (search.error) {
      return {
        provider: PROVIDER,
        results: [],
        error: search.error
      };
    }

    const ids = idsFromSearch(search.body);
    if (ids.length === 0) {
      return {
        provider: PROVIDER,
        results: [],
        error: null
      };
    }

    const summary = await fetchJson(fetcher, buildSummaryUrl({ ids, apiKey }), options);
    if (summary.error) {
      return {
        provider: PROVIDER,
        results: [],
        error: summary.error
      };
    }

    return {
      provider: PROVIDER,
      results: summariesFromBody(summary.body).map(mapSummary),
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
