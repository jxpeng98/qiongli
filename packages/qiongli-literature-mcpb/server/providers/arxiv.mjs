import { normalizeResult } from "../normalize.mjs";
import { fetchTextWithRetry } from "./http.mjs";

const PROVIDER = "arxiv";
const ENDPOINT = "http://export.arxiv.org/api/query";
const DEFAULT_LIMIT = 25;
const MAX_TOTAL_LIMIT = 200;
const ARXIV_ID_PATTERN = /(?:arxiv:\s*|arxiv\.org\/abs\/)?([a-z.-]+\/\d{7}(?:v\d+)?|\d{4}\.\d{4,5}(?:v\d+)?)/i;

function normalizeTotalLimit(limit) {
  if (!Number.isInteger(limit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(limit, 1), MAX_TOTAL_LIMIT);
}

function cleanString(value) {
  return String(value ?? "").trim();
}

function collapseWhitespace(value) {
  return cleanString(value).replace(/\s+/g, " ");
}

function fieldValue(value) {
  const cleaned = collapseWhitespace(value).replaceAll('"', " ");
  return /\s/.test(cleaned) ? `"${collapseWhitespace(cleaned)}"` : cleaned;
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

function submittedDateClause(fromYear, toYear) {
  const startYear = normalizeYear(fromYear);
  const endYear = normalizeYear(toYear);
  if (!startYear && !endYear) {
    return null;
  }

  const start = startYear ? `${startYear}01010000` : "*";
  const end = endYear ? `${endYear}12312359` : "*";
  return `submittedDate:[${start} TO ${end}]`;
}

function arxivId(value) {
  const match = cleanString(value).match(ARXIV_ID_PATTERN);
  return match?.[1] ?? null;
}

function buildSearchQuery({ query, doi, fromYear, toYear }) {
  const clauses = [];
  const normalizedQuery = cleanString(doi) || cleanString(query);
  if (normalizedQuery) {
    clauses.push(`all:${fieldValue(normalizedQuery)}`);
  }

  const dateClause = submittedDateClause(fromYear, toYear);
  if (dateClause) {
    clauses.push(dateClause);
  }

  return clauses.join(" AND ");
}

function buildUrl({ query, arxivIdentifier, doi, limit, fromYear, toYear }) {
  const url = new URL(ENDPOINT);
  const params = new URLSearchParams();
  const resolvedId = cleanString(arxivIdentifier) || arxivId(query);
  if (resolvedId) {
    params.set("id_list", resolvedId);
  } else {
    params.set("search_query", buildSearchQuery({ query, doi, fromYear, toYear }));
  }
  params.set("start", "0");
  params.set("max_results", String(normalizeTotalLimit(limit)));
  params.set("sortBy", "relevance");
  params.set("sortOrder", "descending");
  url.search = params.toString();
  return url;
}

function decodeXml(value) {
  return cleanString(value)
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&apos;/g, "'")
    .replace(/&amp;/g, "&")
    .replace(/&#(\d+);/g, (_match, codepoint) => String.fromCodePoint(Number(codepoint)))
    .replace(/&#x([0-9a-f]+);/gi, (_match, codepoint) => String.fromCodePoint(Number.parseInt(codepoint, 16)));
}

function stripTags(value) {
  return value.replace(/<[^>]+>/g, "");
}

function tagText(entry, tag) {
  const pattern = new RegExp(`<${tag}\\b[^>]*>([\\s\\S]*?)<\\/${tag}>`, "i");
  const match = entry.match(pattern);
  return match ? collapseWhitespace(decodeXml(stripTags(match[1]))) : null;
}

function prefixedTagText(entry, prefix, tag) {
  const pattern = new RegExp(`<${prefix}:${tag}\\b[^>]*>([\\s\\S]*?)<\\/${prefix}:${tag}>`, "i");
  const match = entry.match(pattern);
  return match ? collapseWhitespace(decodeXml(stripTags(match[1]))) : null;
}

function attributes(value) {
  const output = {};
  for (const match of value.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)) {
    output[match[1]] = decodeXml(match[2] ?? match[3] ?? "");
  }
  return output;
}

function linkHref(entry, { rel, type } = {}) {
  for (const match of entry.matchAll(/<link\b([^>]*)\/?>/gi)) {
    const attrs = attributes(match[1]);
    if (rel && attrs.rel !== rel) {
      continue;
    }
    if (type && attrs.type !== type) {
      continue;
    }
    if (attrs.href) {
      return attrs.href;
    }
  }
  return null;
}

function authors(entry) {
  const names = [];
  for (const match of entry.matchAll(/<author\b[^>]*>([\s\S]*?)<\/author>/gi)) {
    const name = tagText(match[1], "name");
    if (name) {
      names.push(name);
    }
  }
  return names;
}

function category(entry) {
  const match = entry.match(/<category\b([^>]*)\/?>/i);
  if (!match) {
    return null;
  }
  return attributes(match[1]).term ?? null;
}

function publishedYear(value) {
  const match = cleanString(value).match(/\b(19|20)\d{2}\b/);
  return match ? Number(match[0]) : null;
}

function sourceId(value) {
  const cleaned = cleanString(value);
  if (!cleaned) {
    return null;
  }
  return cleaned.split("/").pop()?.replace(/^arXiv:/i, "") ?? null;
}

function normalizeEntry(entry) {
  const id = tagText(entry, "id");
  const abstract = tagText(entry, "summary");
  const absUrl = linkHref(entry, { rel: "alternate" }) ?? id;
  const pdfUrl = linkHref(entry, { type: "application/pdf" });
  return normalizeResult({
    title: tagText(entry, "title"),
    authors: authors(entry),
    year: publishedYear(tagText(entry, "published")),
    doi: prefixedTagText(entry, "arxiv", "doi"),
    url: absUrl,
    abstract,
    open_access_pdf_url: pdfUrl,
    access_url: pdfUrl ?? absUrl,
    fulltext_status: pdfUrl ? "not_retrieved:oa_candidate" : "metadata_only",
    evidence_limit: abstract ? "abstract_only" : "metadata_only",
    venue: "arXiv",
    document_type: category(entry),
    provider: PROVIDER,
    source_id: sourceId(id),
    source_type: "preprint"
  });
}

function normalizeFeed(xml) {
  return [...String(xml ?? "").matchAll(/<entry\b[^>]*>([\s\S]*?)<\/entry>/gi)]
    .map((match) => normalizeEntry(match[1]));
}

export async function searchArxiv({ query, arxivIdentifier, doi, limit, fromYear, toYear, fetchImpl } = {}) {
  const lookup = await fetchTextWithRetry({
    provider: PROVIDER,
    url: buildUrl({ query, arxivIdentifier, doi, limit, fromYear, toYear }),
    fetchImpl,
    options: {
      headers: {
        Accept: "application/atom+xml"
      }
    }
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
    results: normalizeFeed(lookup.body),
    error: null,
    request_count: 1,
    attempts: lookup.attempts
  };
}
