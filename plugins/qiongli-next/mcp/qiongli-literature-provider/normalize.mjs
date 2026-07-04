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

function normalizeInteger(value) {
  if (Number.isInteger(value)) {
    return value;
  }

  if (typeof value === "string" && /^\d+$/.test(value.trim())) {
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

function normalizeLinkedRecord(record) {
  return {
    title: cleanString(record?.title),
    authors: normalizeAuthors(record?.authors),
    year: normalizeYear(record?.year),
    doi: normalizeDoi(record?.doi),
    url: cleanString(record?.url),
    provider: cleanString(record?.provider),
    source_id: cleanString(record?.source_id)
  };
}

function normalizeLinkedRecords(value) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.map(normalizeLinkedRecord);
}

function clonePlainObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  return JSON.parse(JSON.stringify(value));
}

export function normalizeResult(record) {
  return {
    title: cleanString(record?.title),
    authors: normalizeAuthors(record?.authors),
    year: normalizeYear(record?.year),
    doi: normalizeDoi(record?.doi),
    url: cleanString(record?.url),
    abstract: cleanString(record?.abstract),
    open_access_pdf_url: cleanString(record?.open_access_pdf_url ?? record?.openAccessPdf?.url),
    access_url: cleanString(record?.access_url),
    fulltext_status: cleanString(record?.fulltext_status),
    evidence_limit: cleanString(record?.evidence_limit),
    license: cleanString(record?.license),
    venue: cleanString(record?.venue),
    document_type: cleanString(record?.document_type),
    citation_count: normalizeInteger(record?.citation_count),
    reference_count: normalizeInteger(record?.reference_count),
    citations: normalizeLinkedRecords(record?.citations),
    references: normalizeLinkedRecords(record?.references),
    provider: cleanString(record?.provider),
    source_id: cleanString(record?.source_id),
    source_type: cleanString(record?.source_type),
    zotero: clonePlainObject(record?.zotero),
    local_zotero_match: clonePlainObject(record?.local_zotero_match),
    review_status: cleanString(record?.review_status),
    verification: clonePlainObject(record?.verification)
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
