import { postCompanionJson, probeCompanion, probeConnector } from "./client.mjs";
import { resolveZoteroConfig } from "./config.mjs";
import { exportImportFiles } from "./exporters.mjs";
import { dedupeReferenceRecords, mapRecordToZoteroItem, normalizeReferenceInputs } from "./records.mjs";

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
  const deduped = dedupeReferenceRecords(records);
  const dryRun = resolveDryRun(input, config);
  const tags = normalizeStringList(input.tags);
  const collectionPath = input.collection_path ?? config.default_collection_path;
  const payload = {
    dry_run: dryRun,
    update_policy: input.update_policy ?? config.update_policy,
    collection_path: collectionPath,
    tags,
    records: deduped.records,
    items: deduped.records.map((record) => mapRecordToZoteroItem(record, { tags }))
  };

  const response = await postCompanionJson(config, "/qiongli/upsertItems", payload, context);
  if (response.status === "error") {
    const fallback = exportImportFiles({ records: deduped.records });
    return {
      ...response,
      dry_run: dryRun,
      dedup_log: deduped.dedup_log,
      fallback_import_files: fallback.fallback_import_files,
      import_files: fallback.files
    };
  }

  return {
    ...response,
    status: response.status ?? "ok",
    dry_run: response.dry_run ?? dryRun,
    dedup_log: deduped.dedup_log
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
