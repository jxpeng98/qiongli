import {
  detectSearchDomain,
  domainAutomaticVariants,
  domainProfilePayload,
  domainVariantRationale,
  domainVariantSource
} from "./domain-profiles.mjs";

const DOI_PATTERN = /(?:doi:\s*|https?:\/\/(?:dx\.)?doi\.org\/)?(10\.\d{4,9}\/[^\s"'<>]+)/i;
const MAX_QUERY_COUNT = 4;

function cleanText(value) {
  return String(value ?? "").trim();
}

function stripOuterQuotes(value) {
  const text = cleanText(value);
  if (text.length < 2) {
    return text;
  }

  const first = text[0];
  const last = text[text.length - 1];
  if ((first === '"' && last === '"') || (first === "'" && last === "'")) {
    return text.slice(1, -1).trim();
  }

  return text;
}

function isQuoted(value) {
  return stripOuterQuotes(value) !== cleanText(value);
}

export function normalizeDoi(value) {
  const text = cleanText(value);
  const match = text.match(DOI_PATTERN);
  if (!match) {
    return null;
  }

  return match[1].replace(/[),.;]+$/g, "");
}

function normalizeSearchMode(input = {}) {
  if (input.exact_title === true || input.exactTitle === true) {
    return "title";
  }

  const rawMode = cleanText(input.search_mode ?? input.searchMode ?? input.mode).toLowerCase();
  const mode = rawMode.replace(/[\s-]+/g, "_");
  if (["title", "exact_title", "known_item"].includes(mode)) {
    return "title";
  }
  if (mode === "doi") {
    return "doi";
  }
  if (["review", "systematic_review", "literature_review", "lit_review"].includes(mode)) {
    return "review";
  }
  if (["topic", "keyword", "broad"].includes(mode)) {
    return "topic";
  }

  return "auto";
}

export function buildSearchIntent(input = {}) {
  const rawQuery = cleanText(input.query);
  if (!rawQuery) {
    throw new Error("query is required");
  }

  const query = stripOuterQuotes(rawQuery);
  const doi = normalizeDoi(query);
  const requestedMode = normalizeSearchMode(input);
  const exactTitle = requestedMode === "title" || (requestedMode === "auto" && isQuoted(rawQuery));
  const mode = doi && requestedMode !== "topic" ? "doi" : exactTitle ? "title" : requestedMode;

  return {
    rawQuery,
    query: doi && mode === "doi" ? doi : query,
    doi: mode === "doi" ? doi : null,
    exactTitle: mode === "title",
    mode
  };
}

export function providerLimitFor(limit, intent, maxLimit) {
  if (intent?.exactTitle) {
    return Math.min(Math.max(limit, 10), maxLimit);
  }

  return Math.min(limit, maxLimit);
}

function readAlias(input, names) {
  for (const name of names) {
    if (input[name] !== undefined) {
      return input[name];
    }
  }

  return undefined;
}

function comparableQuery(value) {
  return cleanText(value)
    .toLowerCase()
    .replace(/\s+/g, " ");
}

function normalizeQueryVariants(value, primaryQuery) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  const variants = [];
  const seen = new Set([comparableQuery(primaryQuery)]);

  for (const item of values) {
    const variant = stripOuterQuotes(item);
    const key = comparableQuery(variant);
    if (!key || seen.has(key)) {
      continue;
    }

    seen.add(key);
    variants.push(variant);
  }

  return variants;
}

function automaticDeepVariants(query) {
  return normalizeQueryVariants([
    `${query} review`,
    `${query} systematic review`
  ], query);
}

function automaticVariants(query, domain) {
  const domainVariants = domainAutomaticVariants(query, domain);
  if (domainVariants.length > 0) {
    return normalizeQueryVariants(domainVariants, query);
  }

  return automaticDeepVariants(query);
}

function variantSource(explicitVariantsProvided) {
  return explicitVariantsProvided ? "explicit_variant" : "auto_variant";
}

function variantRationale(explicitVariantsProvided) {
  return explicitVariantsProvided ? "user supplied query variant" : "automatic deep-search query variant";
}

export function buildQueryPlan(input = {}, intent, options = {}) {
  const primaryQuery = intent?.query ?? cleanText(input.query);
  const domain = detectSearchDomain(primaryQuery);
  const explicitValue = readAlias(input, ["query_variants", "queryVariants"]);
  const explicitVariantsProvided = explicitValue !== undefined;
  const variants = explicitVariantsProvided
    ? normalizeQueryVariants(explicitValue, primaryQuery)
    : options.searchDepth === "deep" && intent?.doi === null && intent?.exactTitle !== true
      ? automaticVariants(primaryQuery, domain)
      : [];

  const queries = [
    {
      query_id: "q1",
      query: primaryQuery,
      source: "primary",
      rationale: "primary query"
    }
  ];

  if (intent?.doi || intent?.exactTitle) {
    return {
      mode: "single",
      domain: domain.id,
      domain_terms: domain.matched_terms,
      domain_profile: domainProfilePayload(domain),
      query_count: queries.length,
      queries
    };
  }

  for (const variant of variants.slice(0, MAX_QUERY_COUNT - 1)) {
    queries.push({
      query_id: `q${queries.length + 1}`,
      query: variant,
      source: explicitVariantsProvided ? variantSource(true) : domainVariantSource(domain),
      rationale: explicitVariantsProvided ? variantRationale(true) : domainVariantRationale(domain)
    });
  }

  return {
    mode: explicitVariantsProvided ? "explicit" : queries.length > 1 ? "auto_deep" : "single",
    domain: domain.id,
    domain_terms: domain.matched_terms,
    domain_profile: domainProfilePayload(domain),
    query_count: queries.length,
    queries
  };
}
