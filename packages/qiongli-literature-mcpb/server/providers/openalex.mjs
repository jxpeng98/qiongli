import { normalizeResult } from "../normalize.mjs";

const PROVIDER = "openalex";
const ENDPOINT = "https://api.openalex.org/works";
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 50;

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

function mapWork(work) {
  return normalizeResult({
    title: work?.title ?? work?.display_name,
    authors: openAlexAuthors(work),
    year: work?.publication_year,
    doi: work?.doi,
    url: openAlexUrl(work),
    abstract: abstractFromInvertedIndex(work?.abstract_inverted_index),
    venue: openAlexVenue(work),
    provider: PROVIDER,
    source_id: openAlexId(work?.id)
  });
}

function buildUrl({ query, limit, email, apiKey, fromYear, toYear }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  params.set("search", query);
  params.set("per-page", String(normalizeLimit(limit)));

  const filters = [];
  if (Number.isInteger(fromYear)) {
    filters.push(`from_publication_date:${fromYear}-01-01`);
  }
  if (Number.isInteger(toYear)) {
    filters.push(`to_publication_date:${toYear}-12-31`);
  }
  if (filters.length > 0) {
    params.set("filter", filters.join(","));
  }

  const trimmedEmail = String(email ?? "").trim();
  if (trimmedEmail) {
    params.set("mailto", trimmedEmail);
  }

  const trimmedApiKey = String(apiKey ?? "").trim();
  if (trimmedApiKey) {
    params.set("api_key", trimmedApiKey);
  }

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

export async function searchOpenAlex({ query, limit, email, apiKey, fromYear, toYear, fetchImpl } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const url = buildUrl({ query, limit, email, apiKey, fromYear, toYear });

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
    const results = Array.isArray(body?.results) ? body.results.map(mapWork) : [];
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
