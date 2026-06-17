import { normalizeResult } from "../normalize.mjs";
import { normalizeDoi } from "../query.mjs";

const PROVIDER = "openalex";
const ENDPOINT = "https://api.openalex.org/works";
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 100;

function normalizeLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_LIMIT);
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
  return normalizeResult({
    title: work?.title ?? work?.display_name,
    authors: openAlexAuthors(work),
    year: work?.publication_year,
    doi: work?.doi,
    url: openAlexUrl(work),
    abstract: abstractFromInvertedIndex(work?.abstract_inverted_index),
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

function buildSearchUrl({ query, limit, email, apiKey, fromYear, toYear, documentTypes }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("search", query);
  params.set("per-page", String(normalizeLimit(limit)));
  params.set("sort", "relevance_score:desc");

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

function fetchOptions(fetchImpl) {
  if (fetchImpl) {
    return {};
  }

  return { signal: AbortSignal.timeout(15000) };
}

function errorMessage(response) {
  return `${PROVIDER} HTTP ${response.status}`;
}

export async function searchOpenAlex({ query, doi, limit, email, apiKey, fromYear, toYear, documentTypes, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const resolvedDoi = typeof doi === "string" ? doi : normalizeDoi(query);
  const url = resolvedDoi
    ? buildDoiUrl({ doi: resolvedDoi, email, apiKey })
    : buildSearchUrl({ query, limit, email, apiKey, fromYear, toYear, documentTypes });

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
    const works = resolvedDoi ? [body] : Array.isArray(body?.results) ? body.results : [];
    const results = works.map(mapWork);
    return {
      provider: PROVIDER,
      results,
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
