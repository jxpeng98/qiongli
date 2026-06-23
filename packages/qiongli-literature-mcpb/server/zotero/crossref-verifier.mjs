import { searchCrossref } from "../providers/crossref.mjs";
import { normalizeReferenceRecord } from "./records.mjs";

export async function verifyRecordWithCrossref({
  record,
  config = {},
  fetchImpl,
  enabled = true,
  enrichment = "fill_blank"
} = {}) {
  const normalized = normalizeReferenceRecord(record);
  if (!enabled || !normalized.doi) {
    return withVerification(normalized, {
      status: "skipped",
      doi: normalized.doi || null,
      filled_fields: [],
      conflicts: []
    });
  }

  const response = await searchCrossref({
    query: normalized.doi,
    doi: normalized.doi,
    limit: 1,
    email: config.crossrefEmail,
    fetchImpl
  });

  if (response.error) {
    return withVerification(normalized, {
      status: "unavailable",
      doi: normalized.doi,
      filled_fields: [],
      conflicts: [],
      warning: response.error
    });
  }

  const candidate = response.results?.[0];
  if (!candidate) {
    return withVerification(normalized, {
      status: "not_found",
      doi: normalized.doi,
      filled_fields: [],
      conflicts: []
    });
  }

  const conflicts = crossrefConflicts(normalized, candidate);
  const { record: enriched, filledFields } = enrichment === "fill_blank"
    ? fillBlankFields(normalized, candidate)
    : { record: normalized, filledFields: [] };

  return withVerification(enriched, {
    status: conflicts.length > 0 ? "conflict" : "verified",
    doi: normalized.doi,
    filled_fields: filledFields,
    conflicts
  });
}

export async function verifyRecordsWithCrossref({
  records = [],
  config = {},
  fetchImpl,
  enabled = true,
  enrichment = "fill_blank"
} = {}) {
  const verified = [];
  for (const record of records) {
    verified.push(await verifyRecordWithCrossref({ record, config, fetchImpl, enabled, enrichment }));
  }
  return verified;
}

export function crossrefStatusTag(status) {
  if (status === "verified") {
    return "qiongli:crossref-verified";
  }
  if (status === "conflict") {
    return "qiongli:metadata-conflict";
  }
  if (status === "unavailable") {
    return "qiongli:verification-unavailable";
  }
  return "qiongli:metadata-unverified";
}

function withVerification(record, crossref) {
  return {
    record: {
      ...record,
      verification: {
        ...(record.verification ?? {}),
        crossref
      }
    },
    verification: {
      crossref
    }
  };
}

function fillBlankFields(record, candidate) {
  const filledFields = [];
  const enriched = { ...record };
  for (const field of [
    "title",
    "authors",
    "year",
    "doi",
    "url",
    "abstract",
    "venue",
    "document_type",
    "reference_count",
    "references"
  ]) {
    const value = candidate[field];
    if (isBlank(enriched[field]) && !isBlank(value)) {
      enriched[field] = value;
      filledFields.push(field);
    }
  }
  return { record: enriched, filledFields };
}

function crossrefConflicts(record, candidate) {
  const conflicts = [];
  if (record.title && candidate.title && titleConflict(record.title, candidate.title)) {
    conflicts.push({ field: "title", incoming: record.title, crossref: candidate.title });
  }
  if (record.year && candidate.year && record.year !== candidate.year) {
    conflicts.push({ field: "year", incoming: record.year, crossref: candidate.year });
  }
  return conflicts;
}

function titleConflict(left, right) {
  const leftTokens = new Set(comparableTitle(left).split(" ").filter(Boolean));
  const rightTokens = new Set(comparableTitle(right).split(" ").filter(Boolean));
  if (leftTokens.size === 0 || rightTokens.size === 0) {
    return false;
  }

  let overlap = 0;
  for (const token of leftTokens) {
    if (rightTokens.has(token)) {
      overlap += 1;
    }
  }

  return overlap / Math.max(leftTokens.size, rightTokens.size) < 0.5;
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

function isBlank(value) {
  if (Array.isArray(value)) {
    return value.length === 0;
  }
  return value === "" || value === null || value === undefined;
}
