const DOI_PREFIX_RE = /^https?:\/\/(?:dx\.)?doi\.org\//i;
const NON_ALNUM_RE = /[^a-z0-9]+/g;
const STOPWORDS = new Set(["a", "an", "and", "for", "from", "in", "of", "on", "the", "to", "with"]);

export function normalizeReferenceInputs(input = {}) {
  const rawRecords = Array.isArray(input.records)
    ? input.records
    : Array.isArray(input.results)
      ? input.results
      : [];
  const records = rawRecords
    .filter((record) => record && typeof record === "object" && !Array.isArray(record))
    .map(normalizeReferenceRecord)
    .filter((record) => record.title || record.doi);

  return { records };
}

export function normalizeReferenceRecord(record = {}) {
  const normalized = {
    title: cleanString(record.title),
    authors: normalizeAuthors(record.authors),
    year: normalizeYear(record.year),
    doi: normalizeDoi(record.doi ?? record.DOI),
    url: cleanString(record.url ?? record.URL),
    abstract: cleanString(record.abstract ?? record.abstractNote),
    venue: cleanString(record.venue ?? record.publicationTitle ?? record.container_title),
    document_type: cleanString(record.document_type ?? record.documentType ?? record.itemType),
    citation_count: normalizeInteger(record.citation_count ?? record.citationCount),
    reference_count: normalizeInteger(record.reference_count ?? record.referenceCount),
    citations: Array.isArray(record.citations) ? record.citations : [],
    references: Array.isArray(record.references) ? record.references : [],
    provider: cleanString(record.provider),
    source_id: cleanString(record.source_id ?? record.sourceId),
    tags: normalizeStringList(record.tags),
    citekey: cleanString(record.citekey ?? record.id),
    verification: clonePlainObject(record.verification),
    review_status: cleanString(record.review_status),
    qiongli_notes: normalizeRecordNotes(record)
  };
  normalized.citekey = normalized.citekey || generateCitekey(normalized);
  return normalized;
}

export function dedupeReferenceRecords(records = []) {
  const deduped = [];
  const seen = new Map();
  const dedupLog = [];

  for (const rawRecord of records) {
    const record = normalizeReferenceRecord(rawRecord);
    const key = dedupeKey(record);
    const existingIndex = seen.get(key);
    if (existingIndex === undefined) {
      seen.set(key, deduped.length);
      deduped.push(record);
      continue;
    }

    dedupLog.push({
      candidate_index: dedupLog.length + deduped.length,
      canonical_index: existingIndex,
      decision: "dedupe_duplicate",
      match_basis: key.startsWith("doi:") ? "doi" : "title_year"
    });
  }

  return {
    records: deduped,
    dedup_log: dedupLog
  };
}

export function mapRecordToZoteroItem(record = {}, options = {}) {
  const normalized = normalizeReferenceRecord(record);
  const itemType = zoteroItemType(normalized);
  const item = compactObject({
    itemType,
    title: normalized.title,
    creators: normalized.authors.map((author) => mapAuthor(author)),
    date: normalized.year ? String(normalized.year) : "",
    DOI: normalized.doi,
    url: normalized.url,
    abstractNote: normalized.abstract,
    publicationTitle: itemType === "journalArticle" ? normalized.venue : "",
    conferenceName: itemType === "conferencePaper" ? normalized.venue : "",
    extra: provenanceExtra(normalized),
    tags: mapTags(normalized, options.tags, options),
    qiongli_notes: normalized.qiongli_notes
  });

  return item;
}

export function generateCitekey(record = {}) {
  const authors = normalizeAuthors(record.authors);
  const firstAuthor = authors[0] ?? "reference";
  const family = authorFamily(firstAuthor) || "reference";
  const year = normalizeYear(record.year) ?? "n.d.";
  const titleWord = firstTitleWord(record.title) || "item";
  return `${slugToken(family)}${year}${slugToken(titleWord)}`;
}

export function normalizeDoi(value) {
  const cleaned = cleanString(value);
  if (!cleaned) {
    return "";
  }
  return cleaned.replace(DOI_PREFIX_RE, "").trim();
}

function dedupeKey(record) {
  if (record.doi) {
    return `doi:${record.doi.toLowerCase()}`;
  }
  return `title-year:${comparableTitle(record.title)}:${record.year ?? ""}`;
}

function zoteroItemType(record) {
  const documentType = String(record.document_type ?? "").toLowerCase();
  if (["journal-article", "journalarticle", "article"].includes(documentType) || (record.venue && record.doi)) {
    return "journalArticle";
  }
  if (["proceedings-article", "conference", "conferencepaper", "conference-paper"].includes(documentType)) {
    return "conferencePaper";
  }
  if (["book"].includes(documentType)) {
    return "book";
  }
  if (["book-chapter", "chapter"].includes(documentType)) {
    return "bookSection";
  }
  if (["preprint", "report"].includes(documentType)) {
    return "report";
  }
  if (["webpage", "web-page"].includes(documentType)) {
    return "webpage";
  }
  return "document";
}

function mapAuthor(author) {
  const cleaned = cleanString(author);
  if (!cleaned) {
    return { creatorType: "author", name: "" };
  }
  if (cleaned.includes(",")) {
    const [family, ...givenParts] = cleaned.split(",");
    const lastName = cleanString(family);
    const firstName = cleanString(givenParts.join(","));
    if (lastName && firstName) {
      return { creatorType: "author", firstName, lastName };
    }
  }
  return { creatorType: "author", name: cleaned };
}

function mapTags(record, extraTags = [], options = {}) {
  const tags = normalizeStringList([
    ...record.tags,
    options.includeProviderTag === false || !record.provider ? "" : `provider:${record.provider}`,
    ...normalizeStringList(extraTags)
  ]);
  return tags.map((tag) => ({ tag }));
}

function provenanceExtra(record) {
  const lines = [];
  if (record.provider) {
    lines.push(`Qiongli Provider: ${record.provider}`);
  }
  if (record.source_id) {
    lines.push(`Qiongli Source ID: ${record.source_id}`);
  }
  if (record.citekey) {
    lines.push(`Qiongli Citekey: ${record.citekey}`);
  }
  return lines.join("\n");
}

function normalizeRecordNotes(record = {}) {
  const values = [];
  collectNoteValues(values, record.qiongli_notes);
  collectNoteValues(values, record.reading_note);
  collectNoteValues(values, record.reading_notes);
  collectNoteValues(values, record.notes);
  collectNoteValues(values, record.note);
  return values.map(normalizeRecordNote).filter(Boolean);
}

function collectNoteValues(values, value) {
  if (Array.isArray(value)) {
    for (const item of value) {
      collectNoteValues(values, item);
    }
    return;
  }
  if (value !== undefined && value !== null && value !== "") {
    values.push(value);
  }
}

function normalizeRecordNote(value) {
  if (typeof value === "string") {
    const text = cleanString(value);
    return text ? {
      title: "Qiongli Reading Note",
      html: `<h2>Qiongli Reading Note</h2><p>${escapeHtml(text)}</p>`
    } : null;
  }

  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  const explicitHtml = cleanString(value.html);
  const title = cleanString(value.title) || "Qiongli Reading Note";
  if (explicitHtml) {
    return { title, html: explicitHtml };
  }

  const sections = [
    noteSection("Summary", value.summary),
    noteSection("Key findings", value.key_findings ?? value.keyFindings),
    noteSection("Limitations", value.limitations),
    noteSection("Evidence limit", value.evidence_limit ?? value.evidenceLimit),
    noteSection("Review status", value.review_status ?? value.reviewStatus),
    noteSection("Screening decision", value.screening_decision ?? value.screeningDecision),
    noteSection("Source anchor", value.source_anchor ?? value.sourceAnchor)
  ].filter(Boolean);

  const freeText = cleanString(value.text ?? value.note);
  if (freeText) {
    sections.push(`<p>${escapeHtml(freeText)}</p>`);
  }

  return sections.length > 0 ? {
    title,
    html: `<h2>${escapeHtml(title)}</h2>${sections.join("")}`
  } : null;
}

function noteSection(label, value) {
  if (Array.isArray(value)) {
    const items = value.map(cleanString).filter(Boolean);
    if (items.length === 0) {
      return "";
    }
    return `<p><strong>${escapeHtml(label)}:</strong></p><ul>${items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul>`;
  }

  const cleaned = cleanString(value);
  return cleaned ? `<p><strong>${escapeHtml(label)}:</strong> ${escapeHtml(cleaned)}</p>` : "";
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function normalizeAuthors(value) {
  if (Array.isArray(value)) {
    return value.map(cleanString).filter(Boolean);
  }
  const cleaned = cleanString(value);
  if (!cleaned) {
    return [];
  }
  return cleaned
    .split(/\s+and\s+|;/i)
    .map(cleanString)
    .filter(Boolean);
}

function normalizeStringList(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  const normalized = [];
  const seen = new Set();
  for (const item of values) {
    const cleaned = cleanString(item);
    if (!cleaned) {
      continue;
    }
    const key = cleaned.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    normalized.push(cleaned);
  }
  return normalized;
}

function normalizeYear(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  const cleaned = cleanString(value);
  if (cleaned && /^\d{4}$/.test(cleaned)) {
    return Number(cleaned);
  }
  return null;
}

function normalizeInteger(value) {
  if (Number.isInteger(value)) {
    return value;
  }
  const cleaned = cleanString(value);
  if (cleaned && /^\d+$/.test(cleaned)) {
    return Number(cleaned);
  }
  return null;
}

function cleanString(value) {
  if (typeof value !== "string") {
    return "";
  }
  return value.trim();
}

function compactObject(value) {
  const compacted = {};
  for (const [key, item] of Object.entries(value)) {
    if (item === "" || item === null || item === undefined) {
      continue;
    }
    if (Array.isArray(item) && item.length === 0) {
      continue;
    }
    compacted[key] = item;
  }
  return compacted;
}

function clonePlainObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return JSON.parse(JSON.stringify(value));
}

function comparableTitle(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(NON_ALNUM_RE, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function authorFamily(value) {
  const cleaned = cleanString(value);
  if (!cleaned) {
    return "";
  }
  if (cleaned.includes(",")) {
    return cleanString(cleaned.split(",", 1)[0]);
  }
  const parts = cleaned.split(/\s+/);
  return parts[parts.length - 1] ?? "";
}

function firstTitleWord(value) {
  return comparableTitle(value)
    .split(" ")
    .find((token) => token && !STOPWORDS.has(token)) ?? "";
}

function slugToken(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(NON_ALNUM_RE, "");
}
