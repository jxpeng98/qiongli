function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }

  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

function normalizeYear(value) {
  if (Number.isInteger(value)) {
    return value;
  }

  if (typeof value === "string" && /^\d{4}$/.test(value.trim())) {
    return Number(value.trim());
  }

  return null;
}

function normalizeAuthors(value) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.map(cleanString).filter((author) => author !== null);
}

export function normalizeDoi(value) {
  const cleaned = cleanString(value);
  if (!cleaned) {
    return null;
  }

  return cleaned.replace(/^https?:\/\/(?:dx\.)?doi\.org\//i, "");
}

export function normalizeResult(record) {
  return {
    title: cleanString(record?.title),
    authors: normalizeAuthors(record?.authors),
    year: normalizeYear(record?.year),
    doi: normalizeDoi(record?.doi),
    url: cleanString(record?.url),
    abstract: cleanString(record?.abstract),
    venue: cleanString(record?.venue),
    provider: cleanString(record?.provider),
    source_id: cleanString(record?.source_id)
  };
}

function compactTitle(value) {
  return String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/\s+/g, " ");
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^\p{Letter}\p{Number}]+/gu, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function titleTokens(value) {
  return comparableTitle(value)
    .split(" ")
    .filter((token) => token.length > 0);
}

export function titleMatchScore(query, title) {
  const queryTitle = comparableTitle(query);
  const candidateTitle = comparableTitle(title);
  if (!queryTitle || !candidateTitle) {
    return 0;
  }

  if (queryTitle === candidateTitle) {
    return 1;
  }

  if (candidateTitle.includes(queryTitle) || queryTitle.includes(candidateTitle)) {
    return 0.9;
  }

  const queryTokens = new Set(titleTokens(queryTitle));
  const candidateTokens = new Set(titleTokens(candidateTitle));
  if (queryTokens.size === 0 || candidateTokens.size === 0) {
    return 0;
  }

  let overlap = 0;
  for (const token of queryTokens) {
    if (candidateTokens.has(token)) {
      overlap += 1;
    }
  }

  return overlap / Math.max(queryTokens.size, candidateTokens.size);
}

export function rankResults(results, query, options = {}) {
  const ranked = results.map((record, index) => ({
    index,
    result: normalizeResult(record),
    score: titleMatchScore(query, record?.title)
  }));

  if (!options.exactTitle && !ranked.some((entry) => entry.score >= 0.9)) {
    return ranked.map((entry) => entry.result);
  }

  return ranked
    .sort((left, right) => right.score - left.score || left.index - right.index)
    .map((entry) => entry.result);
}

function dedupeKey(record) {
  if (record.doi) {
    return `doi:${record.doi.toLowerCase()}`;
  }

  return [
    "fallback",
    record.provider ?? "",
    record.source_id ?? "",
    compactTitle(record.title),
    record.year ?? ""
  ].join(":");
}

export function dedupeResults(results) {
  const seen = new Set();
  const deduped = [];

  for (const result of results) {
    const normalized = normalizeResult(result);
    const key = dedupeKey(normalized);
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(normalized);
  }

  return deduped;
}
