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
import { buildEvidence } from "./evidence.mjs";
import { dedupeResults, rankResults } from "./normalize.mjs";
import { buildSearchIntent, providerLimitFor } from "./query.mjs";
import { searchOpenAlex } from "./providers/openalex.mjs";
import { searchSemanticScholar } from "./providers/semantic-scholar.mjs";
import { startJsonRpcStdioServer } from "./stdio.mjs";
import { startConfigWizard } from "./config-wizard.mjs";

const MIN_LIMIT = 1;
const STANDARD_MAX_LIMIT = 50;
const REVIEW_DEFAULT_LIMIT = 50;
const REVIEW_MAX_LIMIT = 100;

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
    description: "Search academic literature using configured OpenAlex and Semantic Scholar providers.",
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
    description: "Export an auditable provider capability and search evidence snapshot.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {}
    }
  }
];

export function listTools() {
  return TOOL_DECLARATIONS;
}

function resolveConfig(context = {}) {
  return context.config ?? readConfig(context.env);
}

function maxLimitForIntent(intent) {
  return intent?.mode === "review" ? REVIEW_MAX_LIMIT : STANDARD_MAX_LIMIT;
}

function defaultLimitForIntent(input = {}, config, intent) {
  if (input.limit === undefined && intent?.mode === "review") {
    return REVIEW_DEFAULT_LIMIT;
  }

  return config.defaultLimit;
}

function resolveLimit(input = {}, config, intent) {
  const rawLimit = input.limit ?? defaultLimitForIntent(input, config, intent);
  const numericLimit = typeof rawLimit === "number" ? rawLimit : Number(rawLimit);
  const fallbackLimit = defaultLimitForIntent(input, config, intent);
  const integerLimit = Number.isFinite(numericLimit) ? Math.trunc(numericLimit) : fallbackLimit;

  return Math.min(Math.max(integerLimit, MIN_LIMIT), maxLimitForIntent(intent));
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

  return providers;
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
      fetchImpl: params.fetchImpl
    });
  }

  return searchSemanticScholar({
    query: params.query,
    doi: params.intent.doi,
    exactTitle: params.intent.exactTitle,
    limit: params.limit,
    apiKey: params.config.semanticScholarApiKey,
    fromYear: params.fromYear,
    toYear: params.toYear,
    fetchImpl: params.fetchImpl
  });
}

export function handleStatus(context = {}) {
  return providerStatus(resolveConfig(context));
}

export function handleConfigStatus(context = {}) {
  const env = context.env ?? process.env;
  const config = resolveConfig(context);
  return {
    ...redactedProviderStatus(config),
    config_path: providerConfigPath(env),
    provider_env_aliases: providerFieldAliases()
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
  const limit = resolveLimit(input, config, intent);
  const providerLimit = providerLimitFor(limit, intent, maxLimitForIntent(intent));
  const fromYear = readOptionalYear(input.fromYear);
  const toYear = readOptionalYear(input.toYear);
  const attempted = providersFor(config);
  const responses = await Promise.all(
    attempted.map((provider) =>
      callProvider(provider, {
        query: intent.query,
        intent,
        limit: providerLimit,
        config,
        fromYear,
        toYear,
        fetchImpl: context.fetchImpl
      })
    )
  );

  const successful = [];
  const failed = [];
  const results = [];

  for (const response of responses) {
    if (response.error) {
      failed.push(response.provider);
      continue;
    }

    successful.push(response.provider);
    results.push(...response.results);
  }

  const evidence = buildEvidence({
    attemptedProviders: attempted,
    successfulProviders: successful,
    failedProviders: failed,
    resultCount: results.length
  });
  const dedupedResults = dedupeResults(results);
  const rankedResults = rankResults(dedupedResults, intent.query, { exactTitle: intent.exactTitle });
  const outputResults = intent.exactTitle || intent.doi ? rankedResults.slice(0, limit) : rankedResults;

  return {
    status: "ok",
    capability_mode: evidence.capability_mode,
    providers: evidence.providers,
    warnings: evidence.warnings,
    search_mode: intent.mode,
    results: outputResults
  };
}

export async function handleExportEvidence(input = {}, context = {}) {
  if (String(input.query ?? "").trim()) {
    const search = await handleSearch(input, context);
    return {
      status: search.status,
      capability_mode: search.capability_mode,
      providers: search.providers,
      result_count: search.results.length,
      warnings: search.warnings
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
