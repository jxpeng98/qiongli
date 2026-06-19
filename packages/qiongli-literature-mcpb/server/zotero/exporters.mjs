import { generateCitekey, normalizeReferenceRecord } from "./records.mjs";

export function exportImportFiles({ records = [], formats = ["csl-json", "ris", "bibtex", "report"] } = {}) {
  const normalizedRecords = records.map((record) => normalizeReferenceRecord(record));
  const requested = new Set(Array.isArray(formats) && formats.length > 0 ? formats : ["csl-json", "ris", "bibtex", "report"]);
  const files = {};

  if (requested.has("csl-json")) {
    files["references.json"] = `${JSON.stringify(normalizedRecords.map(toCslJson), null, 2)}\n`;
  }
  if (requested.has("ris")) {
    files["references.ris"] = normalizedRecords.map(toRis).join("\n");
  }
  if (requested.has("bibtex")) {
    files["bibliography.bib"] = normalizedRecords.map(toBibtex).join("\n\n");
  }
  if (requested.has("report")) {
    files["zotero-import-report.md"] = importReport(normalizedRecords, files);
  }

  return {
    status: "ok",
    fallback_import_files: {
      available: true
    },
    record_count: normalizedRecords.length,
    files
  };
}

function toCslJson(record) {
  const item = {
    id: record.citekey || generateCitekey(record),
    type: cslType(record),
    title: record.title,
    author: record.authors.map(toCslAuthor)
  };
  if (record.year) {
    item.issued = { "date-parts": [[record.year]] };
  }
  if (record.venue) {
    item["container-title"] = record.venue;
  }
  if (record.doi) {
    item.DOI = record.doi;
  }
  if (record.url) {
    item.URL = record.url;
  }
  if (record.abstract) {
    item.abstract = record.abstract;
  }
  if (record.tags.length > 0) {
    item.keyword = record.tags.join(", ");
  }
  return item;
}

function toRis(record) {
  const lines = [`TY  - ${risType(record)}`];
  for (const author of record.authors) {
    lines.push(`AU  - ${author}`);
  }
  addRis(lines, "TI", record.title);
  addRis(lines, "JO", record.venue);
  addRis(lines, "PY", record.year ? String(record.year) : "");
  addRis(lines, "DO", record.doi);
  addRis(lines, "UR", record.url);
  addRis(lines, "AB", record.abstract);
  for (const tag of record.tags) {
    addRis(lines, "KW", tag);
  }
  lines.push("ER  - ");
  return `${lines.join("\n")}\n`;
}

function toBibtex(record) {
  const citekey = record.citekey || generateCitekey(record);
  const type = bibtexType(record);
  const fields = [
    ["author", record.authors.join(" and ")],
    ["title", record.title],
    [type === "inproceedings" ? "booktitle" : "journal", record.venue],
    ["year", record.year ? String(record.year) : ""],
    ["doi", record.doi],
    ["url", record.url],
    ["abstract", record.abstract],
    ["keywords", record.tags.join(", ")]
  ].filter(([, value]) => value);

  const body = fields
    .map(([key, value]) => `  ${key} = {${escapeBibtex(value)}},`)
    .join("\n");
  return `@${type}{${citekey},\n${body}\n}`;
}

function importReport(records, files) {
  const lines = [
    "# Zotero Import Report",
    "",
    "## Export Summary",
    "",
    `- Records: ${records.length}`,
    `- CSL-JSON: ${Object.hasOwn(files, "references.json") ? "generated" : "not requested"}`,
    `- RIS: ${Object.hasOwn(files, "references.ris") ? "generated" : "not requested"}`,
    `- BibTeX: ${Object.hasOwn(files, "bibliography.bib") ? "generated" : "not requested"}`,
    "",
    "Import these files into Zotero when the Qiongli Zotero companion is not available."
  ];
  return `${lines.join("\n")}\n`;
}

function toCslAuthor(author) {
  if (author.includes(",")) {
    const [family, ...givenParts] = author.split(",");
    const given = givenParts.join(",").trim();
    return given ? { family: family.trim(), given } : { literal: author };
  }
  return { literal: author };
}

function cslType(record) {
  return isConference(record) ? "paper-conference" : "article-journal";
}

function risType(record) {
  return isConference(record) ? "CPAPER" : "JOUR";
}

function bibtexType(record) {
  return isConference(record) ? "inproceedings" : "article";
}

function isConference(record) {
  return String(record.document_type ?? "").toLowerCase().includes("conference");
}

function addRis(lines, tag, value) {
  if (value) {
    lines.push(`${tag}  - ${value}`);
  }
}

function escapeBibtex(value) {
  return String(value ?? "").replace(/[{}]/g, "");
}
