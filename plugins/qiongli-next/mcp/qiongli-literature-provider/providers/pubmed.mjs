import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";
import { fetchJsonWithRetry } from "./http.mjs";

const PROVIDER = "pubmed";
const BASE_URL = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const DEFAULT_LIMIT = 25;
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

function buildSearchUrl({ query, doi, limit, retstart, apiKey, fromYear, toYear }) {
  const url = new URL(`${BASE_URL}/esearch.fcgi`);
  const params = new URLSearchParams();
  params.set("db", "pubmed");
  params.set("retmode", "json");
  params.set("sort", "relevance");
  params.set("term", buildSearchTerm({ query, doi }));
  params.set("retmax", String(normalizePageLimit(limit)));
  params.set("retstart", String(Number.isInteger(retstart) ? retstart : 0));
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

export async function searchPubMed({ query, doi, limit, apiKey, fromYear, toYear, fetchImpl } = {}) {
  try {
    const targetLimit = normalizeTotalLimit(limit);
    const results = [];
    let remaining = targetLimit;
    let retstart = 0;
    let requestCount = 0;
    let attempts = 0;

    while (remaining > 0) {
      const pageLimit = Math.min(remaining, PAGE_LIMIT);
      const search = await fetchJsonWithRetry({
        provider: PROVIDER,
        url: buildSearchUrl({ query, doi, limit: pageLimit, retstart, apiKey, fromYear, toYear }),
        fetchImpl
      });
      requestCount += 1;
      attempts += search.attempts;
      if (search.error) {
        return {
          provider: PROVIDER,
          results: [],
          error: search.error,
          request_count: requestCount,
          attempts
        };
      }

      const ids = idsFromSearch(search.body);
      if (ids.length === 0) {
        break;
      }

      const summary = await fetchJsonWithRetry({
        provider: PROVIDER,
        url: buildSummaryUrl({ ids, apiKey }),
        fetchImpl
      });
      requestCount += 1;
      attempts += summary.attempts;
      if (summary.error) {
        return {
          provider: PROVIDER,
          results: [],
          error: summary.error,
          request_count: requestCount,
          attempts
        };
      }

      results.push(...summariesFromBody(summary.body).map(mapSummary));
      remaining -= pageLimit;
      retstart += pageLimit;

      const totalAvailable = Number(search.body?.esearchresult?.count);
      if (
        (Number.isFinite(totalAvailable) && retstart >= Math.min(totalAvailable, targetLimit)) ||
        (!Number.isFinite(totalAvailable) && ids.length < pageLimit)
      ) {
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
