import { readConfig } from "../config.mjs";
import { postCompanionJson, probeCompanion, probeConnector } from "./client.mjs";
import { resolveZoteroConfig } from "./config.mjs";
import { verifyRecordsWithCrossref } from "./crossref-verifier.mjs";
import { exportImportFiles } from "./exporters.mjs";
import { dedupeReferenceRecords, mapRecordToZoteroItem, normalizeReferenceInputs } from "./records.mjs";
import { mergeReviewTags, resolveDefaultReviewTags, reviewStatusForVerification } from "./review-tags.mjs";

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
      endpoint_version: companion.body?.endpoint_version ?? null
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
  const collectionPath = input.collection_path
    ?? input.review_collection_path
    ?? config.default_review_collection_path
    ?? config.default_collection_path;
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
    update_policy: input.update_policy ?? config.update_policy,
    collection_path: collectionPath,
    tags: inputTags,
    records: itemPayloadRecords.map((entry) => entry.record),
    items: itemPayloadRecords.map((entry) => mapRecordToZoteroItem(entry.record, {
      tags: entry.tags,
      includeProviderTag: false
    }))
  };

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
  return payload;
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
