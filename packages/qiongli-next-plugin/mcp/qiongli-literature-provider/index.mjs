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
import { dedupeResults } from "./normalize.mjs";
import { searchOpenAlex } from "./providers/openalex.mjs";
import { searchSemanticScholar } from "./providers/semantic-scholar.mjs";
import { startJsonRpcStdioServer } from "./stdio.mjs";
import { startConfigWizard } from "./config-wizard.mjs";

const MIN_LIMIT = 1;
const MAX_LIMIT = 50;

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

function resolveLimit(input = {}, config) {
  const rawLimit = input.limit ?? config.defaultLimit;
  const numericLimit = typeof rawLimit === "number" ? rawLimit : Number(rawLimit);
  const integerLimit = Number.isFinite(numericLimit) ? Math.trunc(numericLimit) : config.defaultLimit;

  return Math.min(Math.max(integerLimit, MIN_LIMIT), MAX_LIMIT);
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

function searchQuery(input = {}) {
  const query = String(input.query ?? "").trim();
  if (!query) {
    throw new Error("query is required");
  }

  return query;
}

function providersFor(config) {
  const status = providerStatus(config);
  const providers = [];

  if (status.providers.openalex === "configured" || status.providers.openalex === "configured_without_email") {
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
      limit: params.limit,
      email: params.config.openalexEmail,
      fromYear: params.fromYear,
      toYear: params.toYear,
      fetchImpl: params.fetchImpl
    });
  }

  return searchSemanticScholar({
    query: params.query,
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
  const query = searchQuery(input);
  const limit = resolveLimit(input, config);
  const fromYear = readOptionalYear(input.fromYear);
  const toYear = readOptionalYear(input.toYear);
  const attempted = providersFor(config);
  const responses = await Promise.all(
    attempted.map((provider) =>
      callProvider(provider, {
        query,
        limit,
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

  return {
    status: "ok",
    capability_mode: evidence.capability_mode,
    providers: evidence.providers,
    warnings: evidence.warnings,
    results: dedupeResults(results)
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
