import { readConfig, providerStatus } from "./config.mjs";
import { buildEvidence } from "./evidence.mjs";
import { dedupeResults } from "./normalize.mjs";
import { searchOpenAlex } from "./providers/openalex.mjs";
import { searchSemanticScholar } from "./providers/semantic-scholar.mjs";

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

  if (name === "qiongli_literature_search") {
    return toolResult(await handleSearch(input, context));
  }

  return toolResult(await handleExportEvidence(input, context));
}

export async function startStdioServer() {
  const [{ Server }, { StdioServerTransport }, { CallToolRequestSchema, ListToolsRequestSchema }] =
    await Promise.all([
      import("@modelcontextprotocol/sdk/server/index.js"),
      import("@modelcontextprotocol/sdk/server/stdio.js"),
      import("@modelcontextprotocol/sdk/types.js")
    ]);

  const server = new Server(
    {
      name: "qiongli-literature-provider",
      version: "0.1.0"
    },
    {
      capabilities: {
        tools: {}
      }
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: listTools()
  }));

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    return handleToolCall(request.params.name, request.params.arguments ?? {});
  });

  await server.connect(new StdioServerTransport());
}

function isDirectRun() {
  return import.meta.url === `file://${process.argv[1]}`;
}

if (isDirectRun()) {
  await startStdioServer();
}
