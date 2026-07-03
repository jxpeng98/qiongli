import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";
import { fetchJsonWithRetry } from "./http.mjs";

const PROVIDER = "openalex";
const ENDPOINT = "https://api.openalex.org/works";
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

function openAlexId(value) {
  if (typeof value !== "string") {
    return null;
  }

  const parts = value.split("/");
  return parts[parts.length - 1] || null;
}

function abstractFromInvertedIndex(index) {
  if (!index || typeof index !== "object" || Array.isArray(index)) {
    return null;
  }

  const words = [];
  for (const [word, positions] of Object.entries(index)) {
    if (!Array.isArray(positions)) {
      continue;
    }

    for (const position of positions) {
      if (Number.isInteger(position)) {
        words[position] = word;
      }
    }
  }

  return words.filter((word) => typeof word === "string").join(" ") || null;
}

function openAlexAuthors(work) {
  if (!Array.isArray(work?.authorships)) {
    return [];
  }

  return work.authorships
    .map((authorship) => authorship?.author?.display_name)
    .filter((name) => typeof name === "string");
}

function openAlexVenue(work) {
  return (
    work?.primary_location?.source?.display_name ??
    work?.host_venue?.display_name ??
    null
  );
}

function openAlexUrl(work) {
  return (
    work?.primary_location?.landing_page_url ??
    work?.primary_location?.pdf_url ??
    work?.doi ??
    work?.id ??
    null
  );
}

function openAlexPdfUrl(work) {
  return (
    work?.best_oa_location?.pdf_url ??
    work?.primary_location?.pdf_url ??
    null
  );
}

function openAlexAccessUrl(work, pdfUrl) {
  return (
    pdfUrl ??
    work?.open_access?.oa_url ??
    openAlexUrl(work)
  );
}

function normalizeDocumentTypes(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  return values
    .map((type) => String(type ?? "").trim())
    .filter((type) => type !== "");
}

function openAlexReferences(work) {
  if (!Array.isArray(work?.referenced_works)) {
    return [];
  }

  return work.referenced_works
    .filter((id) => typeof id === "string" && id.trim() !== "")
    .map((id) => ({
      title: null,
      authors: [],
      year: null,
      doi: null,
      url: id,
      provider: PROVIDER,
      source_id: openAlexId(id)
    }));
}

function mapWork(work) {
  const abstract = abstractFromInvertedIndex(work?.abstract_inverted_index);
  const pdfUrl = openAlexPdfUrl(work);
  const accessUrl = openAlexAccessUrl(work, pdfUrl);
  return normalizeResult({
    title: work?.title ?? work?.display_name,
    authors: openAlexAuthors(work),
    year: work?.publication_year,
    doi: work?.doi,
    url: openAlexUrl(work),
    abstract,
    open_access_pdf_url: pdfUrl,
    access_url: accessUrl,
    fulltext_status: pdfUrl || accessUrl ? "not_retrieved:oa_candidate" : "metadata_only",
    evidence_limit: abstract ? "abstract_only" : "metadata_only",
    license: work?.best_oa_location?.license ?? work?.primary_location?.license,
    venue: openAlexVenue(work),
    document_type: work?.type,
    citation_count: work?.cited_by_count,
    reference_count: work?.referenced_works_count ?? work?.referenced_works?.length,
    citations: [],
    references: openAlexReferences(work),
    provider: PROVIDER,
    source_id: openAlexId(work?.id)
  });
}

function applyAuthParams(params, { email, apiKey }) {
  const trimmedEmail = String(email ?? "").trim();
  if (trimmedEmail) {
    params.set("mailto", trimmedEmail);
  }

  const trimmedApiKey = String(apiKey ?? "").trim();
  if (trimmedApiKey) {
    params.set("api_key", trimmedApiKey);
  }
}

function buildSearchUrl({ query, limit, cursor, email, apiKey, fromYear, toYear, documentTypes }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("search", query);
  params.set("per-page", String(normalizePageLimit(limit)));
  params.set("sort", "relevance_score:desc");
  if (cursor) {
    params.set("cursor", cursor);
  }

  const filters = [];
  if (Number.isInteger(fromYear)) {
    filters.push(`from_publication_date:${fromYear}-01-01`);
  }
  if (Number.isInteger(toYear)) {
    filters.push(`to_publication_date:${toYear}-12-31`);
  }
  const typeFilters = normalizeDocumentTypes(documentTypes);
  if (typeFilters.length > 0) {
    filters.push(`type:${typeFilters.join("|")}`);
  }
  if (filters.length > 0) {
    params.set("filter", filters.join(","));
  }

  applyAuthParams(params, { email, apiKey });

  url.search = params.toString();
  return url;
}

function buildDoiUrl({ doi, email, apiKey }) {
  const url = new URL(`${ENDPOINT}/${encodeURIComponent(`doi:${doi}`)}`);
  const params = new URLSearchParams();
  applyAuthParams(params, { email, apiKey });
  url.search = params.toString();
  return url;
}

export async function searchOpenAlex({ query, doi, limit, email, apiKey, fromYear, toYear, documentTypes, fetchImpl } = {}) {
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);

  try {
    if (resolvedDoi) {
      const lookup = await fetchJsonWithRetry({
        provider: PROVIDER,
        url: buildDoiUrl({ doi: resolvedDoi, email, apiKey }),
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
        results: [lookup.body].filter(Boolean).map(mapWork),
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
        url: buildSearchUrl({ query, limit: pageLimit, cursor, email, apiKey, fromYear, toYear, documentTypes }),
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

      const works = Array.isArray(page.body?.results) ? page.body.results : [];
      results.push(...works.map(mapWork));
      remaining -= pageLimit;
      cursor = page.body?.meta?.next_cursor;
      if (!cursor || works.length === 0) {
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
