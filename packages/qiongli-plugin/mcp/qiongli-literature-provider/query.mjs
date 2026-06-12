const DOI_PATTERN = /(?:doi:\s*|https?:\/\/(?:dx\.)?doi\.org\/)?(10\.\d{4,9}\/[^\s"'<>]+)/i;

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
