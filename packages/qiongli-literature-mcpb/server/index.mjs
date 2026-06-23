import {
  providerConfigPath,
  providerFieldAliases,
  readConfig,
  providerStatus,
  redactedProviderStatus,
  saveProviderValue
} from "./config.mjs";
import { fileURLToPath } from "node:url";
import { realpathSync } from "node:fs";
import path from "node:path";
import { providerCapabilities } from "./capabilities.mjs";
import { appendSearchWarnings, searchDiagnostics } from "./diagnostics.mjs";
import { buildEvidence } from "./evidence.mjs";
import { dedupeResults, rankResults } from "./normalize.mjs";
import { buildQueryPlan, buildSearchIntent, providerLimitFor } from "./query.mjs";
import { searchCrossref } from "./providers/crossref.mjs";
import { searchOpenAlex } from "./providers/openalex.mjs";
import { searchPubMed } from "./providers/pubmed.mjs";
import { searchSemanticScholar } from "./providers/semantic-scholar.mjs";
import { startJsonRpcStdioServer } from "./stdio.mjs";
import { startConfigWizard } from "./config-wizard.mjs";
import { resolveZoteroConfig } from "./zotero/config.mjs";
import {
  annotateLocalZoteroMatches,
  resolveZoteroSourceOptions,
  searchZoteroSource,
  zoteroSourceWarning
} from "./zotero/search-source.mjs";
import {
  handleZoteroExportImportFiles,
  handleZoteroSearch,
  handleZoteroStatus,
  handleZoteroUpsertReferences
} from "./zotero/tools.mjs";

const MIN_LIMIT = 1;
const STANDARD_MAX_LIMIT = 50;
const REVIEW_DEFAULT_LIMIT = 50;
const REVIEW_MAX_LIMIT = 200;
const TOTAL_MAX_LIMIT = 500;
const REVIEW_MINIMUM_RESULTS = 25;
const DEEP_MINIMUM_RESULTS = 50;
const SEARCH_DEPTHS = ["quick", "standard", "review", "deep"];

export const TOOL_DECLARATIONS = [
  {
    name: "qiongli_literature_status",
    description: "Report configured literature providers and capability mode without exposing secrets.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {}
    }
  },
  {
    name: "qiongli_config_status",
    description: "Report shared Qiongli provider configuration status without exposing secrets.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {}
    }
  },
  {
    name: "qiongli_configure_provider",
    description: "Open a local browser-based setup page for Qiongli provider credentials. Prefer this for API keys so secrets do not enter chat history.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {
        provider: {
          type: "string",
          enum: ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"]
        },
        host: {
          type: "string",
          default: "127.0.0.1"
        },
        port: {
          type: "integer",
          default: 0
        }
      }
    }
  },
  {
    name: "qiongli_save_provider_config",
    description: "Save explicit Qiongli provider config values from chat or scripts. Prefer qiongli_configure_provider for API keys.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      required: ["provider", "field", "value"],
      properties: {
        provider: {
          type: "string"
        },
        field: {
          type: "string"
        },
        value: {
          type: "string"
        }
      }
    }
  },
  {
    name: "qiongli_open_config_wizard",
    description: "Compatibility alias for qiongli_configure_provider. Starts a local browser-based provider configuration wizard.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {
        provider: {
          type: "string",
          enum: ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"]
        },
        host: {
          type: "string",
          default: "127.0.0.1"
        },
        port: {
          type: "integer",
          default: 0
        }
      }
    }
  },
  {
    name: "qiongli_literature_search",
    description: "Search academic literature using configured OpenAlex, Semantic Scholar, Crossref, and PubMed providers with query variants, finance/economics deep-search routing, diagnostics, and metadata filters.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {
        query: {
          type: "string"
        },
        limit: {
          type: "number"
        },
        per_provider_limit: {
          type: "number"
        },
        perProviderLimit: {
          type: "number"
        },
        total_limit: {
          type: "number"
        },
        totalLimit: {
          type: "number"
        },
        search_depth: {
          type: "string",
          enum: SEARCH_DEPTHS
        },
        searchDepth: {
          type: "string",
          enum: SEARCH_DEPTHS
        },
        include_citations: {
          type: "boolean"
        },
        includeCitations: {
          type: "boolean"
        },
        include_references: {
          type: "boolean"
        },
        includeReferences: {
          type: "boolean"
        },
        document_types: {
          type: "array",
          items: {
            type: "string"
          }
        },
        documentTypes: {
          type: "array",
          items: {
            type: "string"
          }
        },
        venue_filter: {
          type: "string"
        },
        venueFilter: {
          type: "string"
        },
        query_variants: {
          type: "array",
          items: {
            type: "string"
          }
        },
        queryVariants: {
          type: "array",
          items: {
            type: "string"
          }
        },
        include_zotero: {
          type: "boolean",
          default: false
        },
        zotero_limit: {
          type: "number"
        },
        zotero_tag: {
          type: "string"
        },
        zotero_collection_path: {
          type: "string"
        },
        search_mode: {
          type: "string",
          enum: ["auto", "topic", "title", "doi", "review", "systematic_review"]
        },
        searchMode: {
          type: "string",
          enum: ["auto", "topic", "title", "doi", "review", "systematic_review"]
        },
        exact_title: {
          type: "boolean"
        },
        exactTitle: {
          type: "boolean"
        },
        fromYear: {
          type: ["integer", "string"]
        },
        toYear: {
          type: ["integer", "string"]
        }
      }
    }
  },
  {
    name: "qiongli_literature_export_evidence",
    description: "Export an auditable provider capability, search plan, diagnostics, and result snapshot.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {}
    }
  },
  {
    name: "qiongli_zotero_status",
    description: "Report local Zotero Desktop connector, Qiongli companion, and import-file fallback availability.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {
        connector_url: {
          type: "string"
        }
      }
    }
  },
  {
    name: "qiongli_zotero_search",
    description: "Search the local Zotero Desktop library through the Qiongli Zotero companion extension.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {
        doi: {
          type: "string"
        },
        title: {
          type: "string"
        },
        citekey: {
          type: "string"
        },
        creator: {
          type: "string"
        },
        year: {
          type: ["integer", "string"]
        },
        tag: {
          type: "string"
        },
        collection_path: {
          type: "string"
        },
        connector_url: {
          type: "string"
        }
      }
    }
  },
  {
    name: "qiongli_zotero_upsert_references",
    description: "Dry-run or explicitly write normalized Qiongli references to the local Zotero Desktop library.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {
        records: {
          type: "array",
          items: {
            type: "object"
          }
        },
        results: {
          type: "array",
          items: {
            type: "object"
          }
        },
        dry_run: {
          type: "boolean",
          default: true
        },
        collection_path: {
          type: "string"
        },
        tags: {
          type: "array",
          items: {
            type: "string"
          }
        },
        update_policy: {
          type: "string",
          enum: ["fill_blank", "prefer_zotero", "prefer_enriched"]
        },
        verify_crossref: {
          type: "boolean",
          default: true
        },
        crossref_enrichment: {
          type: "string",
          enum: ["fill_blank", "off"],
          default: "fill_blank"
        },
        review_tags: {
          type: "array",
          items: {
            type: "string"
          }
        },
        review_collection_path: {
          type: "string"
        },
        write_policy: {
          type: "string",
          enum: ["dry_run", "explicit", "allow"]
        },
        connector_url: {
          type: "string"
        }
      }
    }
  },
  {
    name: "qiongli_zotero_export_import_files",
    description: "Generate Zotero-compatible CSL-JSON, RIS, BibTeX, and import-report files from Qiongli references.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {
        records: {
          type: "array",
          items: {
            type: "object"
          }
        },
        results: {
          type: "array",
          items: {
            type: "object"
          }
        },
        formats: {
          type: "array",
          items: {
            type: "string",
            enum: ["csl-json", "ris", "bibtex", "report"]
          }
        },
        project_root: {
          type: "string"
        }
      }
    }
  }
];

export function listTools() {
  return TOOL_DECLARATIONS;
}

function resolveConfig(context = {}) {
  return context.config ?? readConfig(context.env);
}

function defaultLimitForIntent(input = {}, config, intent) {
  if (input.limit === undefined && intent?.mode === "review") {
    return REVIEW_DEFAULT_LIMIT;
  }

  return config.defaultLimit;
}

function readAlias(input, names) {
  for (const name of names) {
    if (input[name] !== undefined) {
      return input[name];
    }
  }

  return undefined;
}

function readOptionalInteger(input, names) {
  const rawValue = readAlias(input, names);
  if (rawValue === undefined) {
    return undefined;
  }

  const numericValue = typeof rawValue === "number" ? rawValue : Number(rawValue);
  if (!Number.isFinite(numericValue)) {
    return undefined;
  }

  return Math.trunc(numericValue);
}

function clampInteger(value, { min = MIN_LIMIT, max }) {
  return Math.min(Math.max(value, min), max);
}

function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }

  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
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

function normalizeSearchDepth(input = {}, intent) {
  const rawDepth = cleanString(readAlias(input, ["search_depth", "searchDepth"]));
  const depth = rawDepth?.toLowerCase().replace(/[\s-]+/g, "_");
  if (SEARCH_DEPTHS.includes(depth)) {
    return depth;
  }

  return intent?.mode === "review" ? "review" : "standard";
}

function maxLimitForSearchDepth(intent, searchDepth) {
  if (intent?.mode === "review" || searchDepth === "review" || searchDepth === "deep") {
    return REVIEW_MAX_LIMIT;
  }

  return STANDARD_MAX_LIMIT;
}

function defaultLimitForSearchDepth(input = {}, config, intent, searchDepth) {
  if (input.limit !== undefined) {
    return defaultLimitForIntent(input, config, intent);
  }

  if (searchDepth === "quick") {
    return Math.min(config.defaultLimit, 10);
  }

  if (searchDepth === "deep") {
    return REVIEW_MAX_LIMIT;
  }

  if (searchDepth === "review" || intent?.mode === "review") {
    return REVIEW_DEFAULT_LIMIT;
  }

  return config.defaultLimit;
}

function resolveLegacyLimit(input = {}, config, intent, searchDepth) {
  const rawLimit = input.limit ?? defaultLimitForSearchDepth(input, config, intent, searchDepth);
  const numericLimit = typeof rawLimit === "number" ? rawLimit : Number(rawLimit);
  const fallbackLimit = defaultLimitForSearchDepth(input, config, intent, searchDepth);
  const integerLimit = Number.isFinite(numericLimit) ? Math.trunc(numericLimit) : fallbackLimit;

  return clampInteger(integerLimit, {
    max: maxLimitForSearchDepth(intent, searchDepth)
  });
}

function minimumResultThreshold(searchDepth) {
  if (searchDepth === "deep") {
    return DEEP_MINIMUM_RESULTS;
  }

  if (searchDepth === "review") {
    return REVIEW_MINIMUM_RESULTS;
  }

  return 0;
}

function resolveSearchOptions(input = {}, config, intent) {
  const searchDepth = normalizeSearchDepth(input, intent);
  const maxLimit = maxLimitForSearchDepth(intent, searchDepth);
  const legacyLimit = resolveLegacyLimit(input, config, intent, searchDepth);
  const requestedProviderLimit = readOptionalInteger(input, ["per_provider_limit", "perProviderLimit"]);
  const baseProviderLimit = requestedProviderLimit === undefined
    ? legacyLimit
    : clampInteger(requestedProviderLimit, { max: maxLimit });
  const perProviderLimit = providerLimitFor(baseProviderLimit, intent, maxLimit);
  const requestedTotalLimit = readOptionalInteger(input, ["total_limit", "totalLimit"]);
  const totalLimit = requestedTotalLimit === undefined
    ? null
    : clampInteger(requestedTotalLimit, { max: TOTAL_MAX_LIMIT });
  const documentTypes = normalizeStringList(readAlias(input, ["document_types", "documentTypes"]));
  const venueFilter = cleanString(readAlias(input, ["venue_filter", "venueFilter"]));

  return {
    legacyLimit,
    perProviderLimit,
    totalLimit,
    searchDepth,
    includeCitations: readAlias(input, ["include_citations", "includeCitations"]) === true,
    includeReferences: readAlias(input, ["include_references", "includeReferences"]) === true,
    minimumResultThreshold: minimumResultThreshold(searchDepth),
    filters: {
      documentTypes,
      venueFilter
    }
  };
}

function readOptionalYear(value) {
  if (Number.isInteger(value)) {
    return value;
  }

  if (typeof value === "string" && /^\d{4}$/.test(value.trim())) {
    return Number(value.trim());
  }

  return undefined;
}

function providersFor(config) {
  const status = providerStatus(config);
  const providers = [];

  if (status.providers.openalex === "configured") {
    providers.push("openalex");
  }

  if (status.providers.semantic_scholar === "configured") {
    providers.push("semantic_scholar");
  }

  if (status.providers.crossref === "configured") {
    providers.push("crossref");
  }

  if (status.providers.pubmed === "configured") {
    providers.push("pubmed");
  }

  return providers;
}

function comparableFilterValue(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "");
}

function venueMatches(result, venueFilter) {
  if (!venueFilter) {
    return true;
  }

  return String(result?.venue ?? "")
    .toLowerCase()
    .includes(venueFilter.toLowerCase());
}

function documentTypeMatches(result, documentTypes) {
  if (documentTypes.length === 0) {
    return true;
  }

  const candidate = comparableFilterValue(result?.document_type);
  if (!candidate) {
    return false;
  }

  return documentTypes.some((type) => comparableFilterValue(type) === candidate);
}

function filterResults(results, options) {
  return results.filter(
    (result) =>
      venueMatches(result, options.filters.venueFilter) &&
      documentTypeMatches(result, options.filters.documentTypes)
  );
}

function searchOptionsPayload(options, queryPlan, perQueryProviderLimit) {
  return {
    per_provider_limit: options.perProviderLimit,
    per_query_provider_limit: perQueryProviderLimit,
    total_limit: options.totalLimit,
    search_depth: options.searchDepth,
    query_count: queryPlan.query_count,
    include_citations: options.includeCitations,
    include_references: options.includeReferences,
    minimum_result_threshold: options.minimumResultThreshold,
    filters: {
      document_types: options.filters.documentTypes,
      venue_filter: options.filters.venueFilter
    }
  };
}

async function callProvider(provider, params) {
  if (provider === "openalex") {
    return searchOpenAlex({
      query: params.query,
      doi: params.intent.doi,
      limit: params.limit,
      email: params.config.openalexEmail,
      apiKey: params.config.openalexApiKey,
      fromYear: params.fromYear,
      toYear: params.toYear,
      documentTypes: params.documentTypes,
      fetchImpl: params.fetchImpl
    });
  }

  if (provider === "semantic_scholar") {
    return searchSemanticScholar({
      query: params.query,
      doi: params.intent.doi,
      exactTitle: params.intent.exactTitle,
      limit: params.limit,
      apiKey: params.config.semanticScholarApiKey,
      fromYear: params.fromYear,
      toYear: params.toYear,
      includeCitations: params.includeCitations,
      includeReferences: params.includeReferences,
      fetchImpl: params.fetchImpl
    });
  }

  if (provider === "crossref") {
    return searchCrossref({
      query: params.query,
      doi: params.intent.doi,
      limit: params.limit,
      email: params.config.crossrefEmail,
      fromYear: params.fromYear,
      toYear: params.toYear,
      documentTypes: params.documentTypes,
      fetchImpl: params.fetchImpl
    });
  }

  return searchPubMed({
    query: params.query,
    doi: params.intent.doi,
    limit: params.limit,
    apiKey: params.config.pubmedApiKey,
    fromYear: params.fromYear,
    toYear: params.toYear,
    fetchImpl: params.fetchImpl
  });
}

function providerOutcomes(attempted, responses) {
  const successful = [];
  const failed = [];

  for (const provider of attempted) {
    const providerResponses = responses.filter((response) => response.provider === provider);
    if (providerResponses.some((response) => !response.error)) {
      successful.push(provider);
      continue;
    }

    failed.push(provider);
  }

  return { successful, failed };
}

function perQueryProviderLimit(options, queryPlan) {
  return Math.max(1, Math.ceil(options.perProviderLimit / queryPlan.query_count));
}

export function handleStatus(context = {}) {
  return {
    ...providerStatus(resolveConfig(context)),
    provider_capabilities: providerCapabilities()
  };
}

export function handleConfigStatus(context = {}) {
  const env = context.env ?? process.env;
  const config = resolveConfig(context);
  return {
    ...redactedProviderStatus(config),
    config_path: providerConfigPath(env),
    provider_env_aliases: providerFieldAliases(),
    provider_capabilities: providerCapabilities()
  };
}

export async function handleSaveProviderConfig(input = {}, context = {}) {
  const env = context.env ?? process.env;
  const result = saveProviderValue({
    provider: input.provider,
    field: input.field,
    value: input.value,
    env
  });
  const response = {
    status: "saved",
    provider: result.provider,
    field: result.field,
    config_path: result.path
  };
  if (result.field === "api_key") {
    response.warning = "api_key was saved from chat input. Prefer qiongli_configure_provider so provider secrets do not enter chat history.";
  }
  return response;
}

export async function handleOpenConfigWizard(input = {}, context = {}) {
  const env = context.env ?? process.env;
  return startConfigWizard({
    provider: input.provider,
    host: input.host,
    port: input.port,
    env
  });
}

export async function handleSearch(input = {}, context = {}) {
  const config = resolveConfig(context);
  const intent = buildSearchIntent(input);
  const options = resolveSearchOptions(input, config, intent);
  const zoteroSourceOptions = resolveZoteroSourceOptions(input, {
    perProviderLimit: options.perProviderLimit
  });
  const queryPlan = buildQueryPlan(input, intent, options);
  const perQueryLimit = perQueryProviderLimit(options, queryPlan);
  const fromYear = readOptionalYear(input.fromYear);
  const toYear = readOptionalYear(input.toYear);
  const attempted = providersFor(config);
  const responses = await Promise.all(
    queryPlan.queries.flatMap((plannedQuery) =>
      attempted.map(async (provider) => {
        const response = await callProvider(provider, {
          query: plannedQuery.query,
          intent,
          limit: perQueryLimit,
          config,
          fromYear,
          toYear,
          documentTypes: options.filters.documentTypes,
          includeCitations: options.includeCitations,
          includeReferences: options.includeReferences,
          fetchImpl: context.fetchImpl
        });
        return {
          ...response,
          query_id: plannedQuery.query_id,
          query: plannedQuery.query
        };
      })
    )
  );
  const zoteroResponses = [];
  if (zoteroSourceOptions.include) {
    zoteroResponses.push(await searchZoteroSource({
      config: resolveZoteroConfig({ env: context.env ?? process.env, input }),
      intent,
      input,
      sourceOptions: zoteroSourceOptions,
      context
    }));
  }
  const allResponses = [...responses, ...zoteroResponses];
  const attemptedWithZotero = zoteroSourceOptions.include ? [...attempted, "zotero"] : attempted;

  const { successful, failed } = providerOutcomes(attemptedWithZotero, allResponses);
  const externalResults = [];
  const zoteroResults = [];

  for (const response of allResponses) {
    if (response.error) {
      continue;
    }

    if (response.provider === "zotero") {
      zoteroResults.push(...response.results);
    } else {
      externalResults.push(...response.results);
    }
  }
  const results = [
    ...annotateLocalZoteroMatches({ externalResults, zoteroResults }),
    ...zoteroResults
  ];

  const evidence = buildEvidence({
    attemptedProviders: attemptedWithZotero,
    successfulProviders: successful,
    failedProviders: failed,
    resultCount: results.length
  });
  const dedupedResults = dedupeResults(results);
  const filteredResults = filterResults(dedupedResults, options);
  const rankedResults = rankResults(filteredResults, intent.query, { exactTitle: intent.exactTitle });
  const finalLimit = intent.exactTitle || intent.doi ? options.totalLimit ?? options.legacyLimit : options.totalLimit;
  const outputResults = finalLimit === null ? rankedResults : rankedResults.slice(0, finalLimit);
  const diagnostics = searchDiagnostics({
    responses: allResponses,
    rawResults: results,
    dedupedResults,
    filteredResults,
    outputResults,
    queryPlan
  });

  return {
    status: "ok",
    capability_mode: evidence.capability_mode,
    providers: evidence.providers,
    provider_capabilities: providerCapabilities(),
    warnings: [
      ...new Set([
        ...appendSearchWarnings(evidence.warnings, outputResults, options, diagnostics),
        ...zoteroResponses.map(zoteroSourceWarning).filter(Boolean)
      ])
    ],
    search_mode: intent.mode,
    search_options: searchOptionsPayload(options, queryPlan, perQueryLimit),
    search_plan: queryPlan,
    diagnostics,
    results: outputResults
  };
}

export async function handleExportEvidence(input = {}, context = {}) {
  if (String(input.query ?? "").trim()) {
    const search = await handleSearch(input, context);
    return {
      ...search,
      result_count: search.results.length
    };
  }

  return {
    ...handleStatus(context),
    result_count: 0,
    warnings: []
  };
}

function toolResult(payload) {
  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(payload)
      }
    ]
  };
}

export async function handleToolCall(name, input = {}, context = {}) {
  const tool = TOOL_DECLARATIONS.find((candidate) => candidate.name === name);
  if (!tool) {
    throw new Error(`Unknown tool: ${name}`);
  }

  if (name === "qiongli_literature_status") {
    return toolResult(handleStatus(context));
  }

  if (name === "qiongli_config_status") {
    return toolResult(handleConfigStatus(context));
  }

  if (name === "qiongli_configure_provider") {
    return toolResult(await handleOpenConfigWizard(input, context));
  }

  if (name === "qiongli_save_provider_config") {
    return toolResult(await handleSaveProviderConfig(input, context));
  }

  if (name === "qiongli_open_config_wizard") {
    return toolResult(await handleOpenConfigWizard(input, context));
  }

  if (name === "qiongli_literature_search") {
    return toolResult(await handleSearch(input, context));
  }

  if (name === "qiongli_zotero_status") {
    return toolResult(await handleZoteroStatus(input, context));
  }

  if (name === "qiongli_zotero_search") {
    return toolResult(await handleZoteroSearch(input, context));
  }

  if (name === "qiongli_zotero_upsert_references") {
    return toolResult(await handleZoteroUpsertReferences(input, context));
  }

  if (name === "qiongli_zotero_export_import_files") {
    return toolResult(await handleZoteroExportImportFiles(input, context));
  }

  return toolResult(await handleExportEvidence(input, context));
}

export async function startStdioServer() {
  await startJsonRpcStdioServer({
    serverInfo: {
      name: "qiongli-literature-provider",
      version: "0.1.4"
    },
    listTools,
    handleToolCall
  });
}

function isDirectRun() {
  if (!process.argv[1]) {
    return false;
  }
  return realpathSync(fileURLToPath(import.meta.url)) === realpathSync(path.resolve(process.argv[1]));
}

if (isDirectRun()) {
  await startStdioServer();
}
