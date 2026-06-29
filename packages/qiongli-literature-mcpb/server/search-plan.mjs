const PROVIDER_PROVENANCE_LABELS = Object.freeze([
  "mcp:semantic_scholar",
  "mcp:openalex",
  "mcp:crossref",
  "mcp:pubmed",
  "mcp:arxiv"
]);
const PROVIDER_NAMES = Object.freeze(PROVIDER_PROVENANCE_LABELS.map((label) => label.replace(/^mcp:/, "")));
const AGENT_INSTRUCTIONS = Object.freeze([
  "MCP servers must not call Codex or Claude native search directly.",
  "The active agent executes native_search_queries only when the platform exposes native search.",
  "Do not treat native-search results as provider-reproducible records.",
  "Write provider, native, and user-corpus records with distinct provenance labels."
]);

const DEFAULT_NATIVE_TOOLS = Object.freeze({
  codex: "codex_web_search",
  claude: "claude_web_search",
  claude_code: "claude_web_search",
  claudecode: "claude_web_search",
  antigravity: "antigravity_search",
  platform: "platform_native_search"
});

const LIMITATIONS = Object.freeze({
  emptyQuery: "Search query is empty.",
  noNative: "Platform-native search was not declared available.",
  noProvider: "Provider MCP search is unavailable; native results require explicit provenance labels.",
  noSearch: "No provider MCP search or platform-native search is available."
});

function cleanString(value, fallback = "") {
  if (value === undefined || value === null) {
    return fallback;
  }

  if (typeof value !== "string" && typeof value !== "number" && typeof value !== "boolean") {
    return "";
  }

  return String(value).trim();
}

function readAlias(input, names) {
  for (const name of names) {
    if (input[name] !== undefined) {
      return input[name];
    }
  }

  return undefined;
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

function readBoolean(value) {
  if (typeof value === "boolean") {
    return value;
  }

  if (typeof value === "string") {
    return ["1", "true", "yes", "y", "available"].includes(value.trim().toLowerCase());
  }

  return Boolean(value);
}

function normalizeProviderCapabilityMode(value) {
  return cleanString(value) === "provider_connected" ? "provider_connected" : "strategy_only";
}

function normalizePlatform(value) {
  const normalized = cleanString(value, "unknown").replace(/[\s-]+/g, "_");
  return normalized || "unknown";
}

function defaultNativeTool(platform) {
  return DEFAULT_NATIVE_TOOLS[platform.toLowerCase()] ?? DEFAULT_NATIVE_TOOLS.platform;
}

function normalizeTool(value) {
  return cleanString(value).replace(/\s+/g, "_");
}

function nativeSearchTools(input, platform, nativeAvailable) {
  if (!nativeAvailable) {
    return [];
  }

  const rawTools = readAlias(input, ["native_search_tools", "nativeSearchTools"]);
  const configuredTools = normalizeStringList(rawTools).map(normalizeTool).filter(Boolean);
  if (configuredTools.length > 0) {
    return configuredTools;
  }

  return [defaultNativeTool(platform)];
}

function nativeSearchAvailable(input) {
  const raw = readAlias(input, ["native_search_available", "nativeSearchAvailable"]);
  return readBoolean(raw ?? false);
}

function hasOwn(input, name) {
  return Object.prototype.hasOwnProperty.call(input, name);
}

function firstPresent(input, names) {
  for (const name of names) {
    if (hasOwn(input, name) && input[name] !== null && input[name] !== undefined) {
      return input[name];
    }
  }

  return undefined;
}

function searchFilters(input) {
  const filters = {};
  for (const [outputKey, names] of [
    ["include_working_papers", ["include_working_papers", "includeWorkingPapers"]],
    ["fromYear", ["fromYear"]],
    ["toYear", ["toYear"]],
    ["search_mode", ["search_mode", "searchMode"]],
    ["venue_filter", ["venue_filter", "venueFilter"]]
  ]) {
    const value = firstPresent(input, names);
    if (value !== undefined) {
      filters[outputKey] = value;
    }
  }

  const documentTypes = normalizeStringList(firstPresent(input, ["document_types", "documentTypes"]));
  if (documentTypes.length > 0) {
    filters.document_types = documentTypes;
  }

  return filters;
}

function queryEntries(input, query) {
  if (!query) {
    return [];
  }

  const queries = [query, ...normalizeStringList(readAlias(input, ["query_variants", "queryVariants"]))];
  const entries = [];
  const seen = new Set();
  for (const candidate of queries) {
    const key = candidate.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    entries.push({
      query_id: `Q${entries.length + 1}`,
      query: candidate,
      source: entries.length === 0 ? "primary" : "variant"
    });
  }
  return entries;
}

function providerNames(providerStatus) {
  if (providerStatus === null || providerStatus === undefined) {
    return [...PROVIDER_NAMES];
  }

  return PROVIDER_NAMES.filter((provider) => {
    const value = providerStatus[provider];
    if (value === "configured" || value === true) {
      return true;
    }
    return Boolean(value && typeof value === "object" && value.configured);
  });
}

function buildProviderQueries(entries, filters, names, providerConnected) {
  if (!providerConnected || entries.length === 0) {
    return [];
  }

  return names.flatMap((provider) =>
    entries.map((entry) => ({
      provider,
      query_id: entry.query_id,
      query: entry.query,
      source: entry.source,
      filters: { ...filters },
      provenance_label: `mcp:${provider}`
    }))
  );
}

function buildNativeQueries(entries, platform, nativeTools, filters, nativeEnabled) {
  if (!nativeEnabled || entries.length === 0) {
    return [];
  }

  return nativeTools.flatMap((tool) =>
    entries.map((entry) => ({
      tool,
      platform,
      query_id: entry.query_id,
      query: entry.query,
      source: entry.source,
      filters: { ...filters },
      provenance_label: `native:${tool}`
    }))
  );
}

function executionSequence(providerQueries, nativeQueries) {
  const sequence = [
    {
      actor: "agent",
      action: "call qiongli_literature_status",
      tool: "qiongli_literature_status"
    },
    {
      actor: "agent",
      action: "call qiongli_search_plan",
      tool: "qiongli_search_plan"
    }
  ];

  if (providerQueries.length > 0) {
    sequence.push({
      actor: "agent",
      action: "call qiongli_literature_search",
      tool: "qiongli_literature_search",
      queries: "provider_queries"
    });
  }

  if (nativeQueries.length > 0) {
    sequence.push({
      actor: "agent",
      action: "execute platform-native search",
      queries: "native_search_queries"
    });
  }

  sequence.push({
    actor: "agent",
    action: "merge/dedupe/search_log",
    inputs: ["provider_queries", "native_search_queries", "user_corpus"]
  });
  return sequence;
}

function mergePolicy() {
  return {
    dedupe_keys: ["doi", "title", "year", "provider_record_id", "native_url"],
    provider_records: "Prefer provider MCP metadata for reproducible bibliographic fields.",
    native_records: "Keep native-search records only with native provenance labels and source URLs.",
    user_corpus_records: "Keep user-corpus records separate from provider and native search records.",
    search_log: "Record provider and native query execution separately before merge and dedupe."
  };
}

function executionMode({ query, providerConnected, nativeAvailable }) {
  if (!query) {
    return "strategy_only";
  }
  if (providerConnected && nativeAvailable) {
    return "hybrid_search";
  }
  if (providerConnected) {
    return "provider_connected";
  }
  if (nativeAvailable) {
    return "native_only";
  }

  return "strategy_only";
}

function planLimitations({ query, providerConnected, nativeAvailable, mode }) {
  if (!query) {
    return [LIMITATIONS.emptyQuery];
  }
  if (providerConnected && !nativeAvailable) {
    return [LIMITATIONS.noNative];
  }
  if (!providerConnected && nativeAvailable) {
    return [LIMITATIONS.noProvider];
  }
  if (mode === "strategy_only") {
    return [LIMITATIONS.noSearch];
  }

  return [];
}

export function buildHybridSearchPlan(input = {}, providerCapabilityMode = "strategy_only", providerStatus = undefined) {
  const query = cleanString(input.query);
  const platform = normalizePlatform(input.platform);
  const nativeAvailable = nativeSearchAvailable(input);
  const tools = nativeSearchTools(input, platform, nativeAvailable);
  const names = providerNames(providerStatus);
  let normalizedProviderMode = normalizeProviderCapabilityMode(providerCapabilityMode);
  if (providerStatus !== undefined && names.length === 0) {
    normalizedProviderMode = "strategy_only";
  }
  const providerConnected = normalizedProviderMode === "provider_connected" && names.length > 0;
  const mode = executionMode({ query, providerConnected, nativeAvailable });
  const filters = searchFilters(input);
  const entries = queryEntries(input, query);
  const providerQueries = buildProviderQueries(entries, filters, names, ["hybrid_search", "provider_connected"].includes(mode));
  const nativeQueries = buildNativeQueries(entries, platform, tools, filters, ["hybrid_search", "native_only"].includes(mode));

  return {
    artifact_type: "qiongli_hybrid_search_plan",
    query,
    platform,
    search_execution_mode: mode,
    provider_capability_mode: normalizedProviderMode,
    native_search_available: nativeAvailable,
    native_search_tools: tools,
    provider_queries: providerQueries,
    native_search_queries: nativeQueries,
    provenance_labels: {
      provider: providerConnected ? names.map((provider) => `mcp:${provider}`) : [],
      native: tools.map((tool) => `native:${tool}`),
      user_corpus: ["user_corpus"]
    },
    execution_sequence: executionSequence(providerQueries, nativeQueries),
    agent_instructions: [...AGENT_INSTRUCTIONS],
    merge_policy: mergePolicy(),
    limitations: planLimitations({
      query,
      providerConnected,
      nativeAvailable,
      mode
    })
  };
}
