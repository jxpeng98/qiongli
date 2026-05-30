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
