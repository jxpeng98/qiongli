import { readConfig } from "../config.mjs";
import { postCompanionJson, probeCompanion, probeConnector } from "./client.mjs";
import { resolveZoteroConfig } from "./config.mjs";
import { verifyRecordsWithCrossref } from "./crossref-verifier.mjs";
import { exportImportFiles } from "./exporters.mjs";
import { dedupeReferenceRecords, mapRecordToZoteroItem, normalizeReferenceInputs } from "./records.mjs";
import { mergeReviewTags, resolveDefaultReviewTags, reviewStatusForVerification } from "./review-tags.mjs";

const PROJECT_COLLECTION_ROOT = "Qiongli";
const SUPPORTED_COMPANION_ENDPOINT_VERSION = "2";
const DEFAULT_ZOTERO_SEARCH_LIMIT = 25;
const MAX_ZOTERO_SEARCH_LIMIT = 200;
const MAX_ZOTERO_UPSERT_ITEMS = 100;
const MAX_ZOTERO_REQUEST_CHARS = 1024 * 1024;
const PROJECT_TITLE_STOPWORDS = new Set([
  "a",
  "an",
  "and",
  "are",
  "as",
  "at",
  "by",
  "for",
  "from",
  "in",
  "into",
  "is",
  "of",
  "on",
  "or",
  "the",
  "to",
  "via",
  "with"
]);

export async function handleZoteroStatus(input = {}, context = {}) {
  const config = resolveZoteroConfig({
    env: context.env ?? process.env,
    input
  });

  if (!config.local_enabled) {
    return {
      status: "disabled",
      connector: { available: false },
      companion: { available: false },
      fallback_import_files: fallbackImportAvailability(),
      config: redactedConfig(config)
    };
  }

  const connector = await probeConnector(config, context);
  if (!connector.available) {
    return {
      status: "fallback_only",
      error_code: "zotero_not_running",
      connector: {
        available: false,
        status: connector.status
      },
      companion: {
        available: false
      },
      fallback_import_files: fallbackImportAvailability(),
      config: redactedConfig(config)
    };
  }

  const companion = await probeCompanion(config, context);
  if (!companion.available) {
    return {
      status: "companion_missing",
      error_code: "companion_missing",
      connector: {
        available: true,
        status: connector.status
      },
      companion: {
        available: false,
        status: companion.status
      },
      fallback_import_files: fallbackImportAvailability(),
      config: redactedConfig(config)
    };
  }

  const endpointVersion = companion.body?.endpoint_version ?? null;
  if (endpointVersion !== SUPPORTED_COMPANION_ENDPOINT_VERSION) {
    return {
      status: "companion_incompatible",
      error_code: "companion_incompatible",
      connector: {
        available: true,
        status: connector.status
      },
      companion: {
        available: true,
        status: companion.status,
        version: companion.body?.version ?? companion.body?.companion_version ?? null,
        endpoint_version: endpointVersion,
        supported_endpoint_version: SUPPORTED_COMPANION_ENDPOINT_VERSION
      },
      fallback_import_files: fallbackImportAvailability(),
      config: redactedConfig(config)
    };
  }

  return {
    status: "ok",
    connector: {
      available: true,
      status: connector.status
    },
    companion: {
      available: true,
      status: companion.status,
      version: companion.body?.version ?? companion.body?.companion_version ?? null,
      endpoint_version: endpointVersion,
      supported_endpoint_version: SUPPORTED_COMPANION_ENDPOINT_VERSION
    },
    fallback_import_files: fallbackImportAvailability(),
    config: redactedConfig(config)
  };
}

export async function handleZoteroSearch(input = {}, context = {}) {
  const config = resolveZoteroConfig({
    env: context.env ?? process.env,
    input
  });
  const qualification = await qualifyCompanion(config, context);
  if (!qualification.ready) {
    return {
      status: qualification.status,
      error_code: qualification.error_code,
      results: [],
      fallback_import_files: fallbackImportAvailability()
    };
  }
  const payload = searchPayload(input);
  const response = await postCompanionJson(config, "/qiongli/search", payload, context);
  return {
    status: response.status ?? "ok",
    results: Array.isArray(response.results) ? response.results : [],
    companion: {
      endpoint: "/qiongli/search"
    }
  };
}

export async function handleZoteroUpsertReferences(input = {}, context = {}) {
  const config = resolveZoteroConfig({
    env: context.env ?? process.env,
    input
  });
  const requestError = boundedUpsertInputError(input);
  if (requestError) {
    return {
      status: "invalid_request",
      error_code: requestError,
      dry_run: true,
      results: [],
      fallback_import_files: fallbackImportAvailability()
    };
  }
  const qualification = await qualifyCompanion(config, context);
  const writeRequested = input.dry_run === false && config.write_policy !== "dry_run";
  if (writeRequested && input.write_intent !== "apply") {
    return {
      status: "approval_required",
      error_code: "zotero_write_intent_required",
      dry_run: true,
      results: [],
      fallback_import_files: fallbackImportAvailability(),
      next_action: "run-dry-run"
    };
  }
  if (writeRequested && !/^zwr1_[0-9a-f]{64}$/.test(String(input.dry_run_receipt ?? ""))) {
    return {
      status: "approval_required",
      error_code: "zotero_dry_run_receipt_required",
      dry_run: true,
      results: [],
      fallback_import_files: fallbackImportAvailability(),
      next_action: "run-dry-run"
    };
  }
  const { records } = normalizeReferenceInputs(input);
  const providerConfig = context.config ?? readConfig(context.env ?? process.env);
  const verifiedRecords = await verifyRecordsWithCrossref({
    records,
    config: providerConfig,
    fetchImpl: context.fetchImpl,
    enabled: input.verify_crossref !== false && config.crossref_verification_enabled,
    enrichment: input.crossref_enrichment ?? "fill_blank"
  });
  const enrichedRecords = verifiedRecords.map((entry) => entry.record);
  const deduped = dedupeReferenceRecords(enrichedRecords);
  const dryRun = resolveDryRun(input, config);
  const inputTags = normalizeStringList(input.tags);
  const collectionPath = resolveCollectionPath(input, config);
  const defaultReviewTags = resolveDefaultReviewTags(config, input);
  const itemPayloadRecords = deduped.records.map((record) => {
    const crossrefStatus = record.verification?.crossref?.status ?? "skipped";
    const tags = mergeReviewTags({
      baseTags: [...inputTags, ...record.tags],
      provider: record.provider,
      crossrefStatus,
      defaultReviewTags
    });
    return {
      record: {
        ...record,
        tags
      },
      tags
    };
  });
  const payload = {
    dry_run: dryRun,
    write_intent: input.write_intent,
    dry_run_receipt: input.dry_run_receipt,
    update_policy: input.update_policy ?? config.update_policy,
    collection_path: collectionPath,
    tags: inputTags,
    records: itemPayloadRecords.map((entry) => entry.record),
    items: itemPayloadRecords.map((entry) => mapRecordToZoteroItem(entry.record, {
      tags: entry.tags,
      includeProviderTag: false
    }))
  };

  if (!qualification.ready) {
    const fallback = exportImportFiles({ records: itemPayloadRecords.map((entry) => entry.record) });
    return {
      status: qualification.status,
      error_code: qualification.error_code,
      dry_run: true,
      dedup_log: deduped.dedup_log,
      verification: itemPayloadRecords.map(
        (entry) => entry.record.verification ?? { crossref: { status: "skipped", filled_fields: [], conflicts: [] } }
      ),
      results: [],
      fallback_import_files: fallback.fallback_import_files,
      import_files: fallback.files
    };
  }

  const response = await postCompanionJson(config, "/qiongli/upsertItems", payload, context);
  const verification = itemPayloadRecords.map(
    (entry) => entry.record.verification ?? { crossref: { status: "skipped", filled_fields: [], conflicts: [] } }
  );
  const responseResults = Array.isArray(response.results) ? response.results : [];
  const results = responseResults.map((entry, index) => ({
    ...entry,
    review_status: reviewStatusForVerification({
      writeStatus: entry.status,
      crossrefStatus: verification[index]?.crossref?.status
    }),
    verification: verification[index]
  }));
  if (response.status === "error") {
    const fallback = exportImportFiles({ records: itemPayloadRecords.map((entry) => entry.record) });
    return {
      ...response,
      dry_run: dryRun,
      dedup_log: deduped.dedup_log,
      verification,
      results,
      fallback_import_files: fallback.fallback_import_files,
      import_files: fallback.files
    };
  }

  return {
    ...response,
    status: response.status ?? "ok",
    dry_run: response.dry_run ?? dryRun,
    dedup_log: deduped.dedup_log,
    verification,
    results
  };
}

export async function handleZoteroExportImportFiles(input = {}) {
  const { records } = normalizeReferenceInputs(input);
  return exportImportFiles({
    records,
    formats: input.formats
  });
}

function fallbackImportAvailability() {
  return {
    available: true,
    formats: ["references.json", "references.ris", "bibliography.bib", "zotero-import-report.md"]
  };
}

async function qualifyCompanion(config, context) {
  if (!config.local_enabled) {
    return {
      ready: false,
      status: "disabled",
      error_code: "zotero_local_disabled"
    };
  }
  const companion = await probeCompanion(config, context);
  if (!companion.available) {
    return {
      ready: false,
      status: "companion_missing",
      error_code: "companion_missing"
    };
  }
  if (companion.body?.endpoint_version !== SUPPORTED_COMPANION_ENDPOINT_VERSION) {
    return {
      ready: false,
      status: "companion_incompatible",
      error_code: "companion_incompatible"
    };
  }
  return { ready: true };
}

function redactedConfig(config) {
  return {
    local_enabled: config.local_enabled,
    connector_url: config.connector_url,
    default_collection_path: config.default_collection_path ? "configured" : "missing",
    write_policy: config.write_policy,
    update_policy: config.update_policy
  };
}

function searchPayload(input) {
  const payload = {};
  for (const key of ["doi", "title", "citekey", "creator", "year", "tag", "collection_path"]) {
    if (input[key] !== undefined && input[key] !== null && String(input[key]).trim() !== "") {
      payload[key] = input[key];
    }
  }
  const requestedLimit = Number.parseInt(String(input.limit ?? DEFAULT_ZOTERO_SEARCH_LIMIT), 10);
  payload.limit = Number.isFinite(requestedLimit)
    ? Math.min(Math.max(requestedLimit, 1), MAX_ZOTERO_SEARCH_LIMIT)
    : DEFAULT_ZOTERO_SEARCH_LIMIT;
  return payload;
}

function boundedUpsertInputError(input) {
  let serialized;
  try {
    serialized = JSON.stringify(input);
  } catch {
    return "zotero_request_invalid";
  }
  if (serialized.length > MAX_ZOTERO_REQUEST_CHARS) {
    return "zotero_request_too_large";
  }
  for (const field of ["records", "results"]) {
    if (input[field] !== undefined && !Array.isArray(input[field])) {
      return "zotero_request_invalid";
    }
    if (input[field]?.length > MAX_ZOTERO_UPSERT_ITEMS) {
      return "zotero_too_many_items";
    }
  }
  return null;
}

function resolveDryRun(input, config) {
  if (input.dry_run === false && config.write_policy !== "dry_run") {
    return false;
  }
  return true;
}

function normalizeStringList(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? [value] : [];
  return values.map((item) => String(item ?? "").trim()).filter(Boolean);
}

function resolveCollectionPath(input, config) {
  return cleanString(input.collection_path)
    ?? cleanString(input.review_collection_path)
    ?? config.default_review_collection_path
    ?? config.default_collection_path
    ?? deriveProjectCollectionPath(input);
}

function deriveProjectCollectionPath(input) {
  const title = cleanString(input.project_title)
    ?? cleanString(input.research_title)
    ?? cleanString(input.topic);
  const slug = slugFromProjectTitle(title);
  return slug ? `${PROJECT_COLLECTION_ROOT}/${slug}` : null;
}

function slugFromProjectTitle(value) {
  const rawTokens = String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .split(/[^a-z0-9]+/)
    .map((token) => token.trim())
    .filter(Boolean);
  const keywordTokens = rawTokens.filter((token) => !PROJECT_TITLE_STOPWORDS.has(token));
  return (keywordTokens.length > 0 ? keywordTokens : rawTokens).slice(0, 6).join("-");
}

function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}
