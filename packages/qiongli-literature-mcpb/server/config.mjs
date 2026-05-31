const DEFAULT_LIMIT = 10;
const MIN_LIMIT = 1;
const MAX_LIMIT = 50;

function readTrimmed(env, name) {
  return String(env[name] ?? "").trim();
}

function readDefaultLimit(env) {
  const rawLimit = readTrimmed(env, "QIONGLI_MCPB_DEFAULT_LIMIT");
  if (rawLimit === "") {
    return DEFAULT_LIMIT;
  }

  const parsedLimit = Number(rawLimit);
  if (!Number.isInteger(parsedLimit)) {
    return DEFAULT_LIMIT;
  }

  return Math.min(Math.max(parsedLimit, MIN_LIMIT), MAX_LIMIT);
}

export function readConfig(env = process.env) {
  return {
    openalexEmail: readTrimmed(env, "QIONGLI_MCPB_OPENALEX_EMAIL"),
    semanticScholarApiKey: readTrimmed(env, "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"),
    defaultLimit: readDefaultLimit(env)
  };
}

export function providerStatus(config) {
  const providers = {
    openalex: config.openalexEmail ? "configured" : "configured_without_email",
    semantic_scholar: config.semanticScholarApiKey ? "configured" : "missing",
    crossref: "not_implemented",
    pubmed: "not_implemented"
  };
  const openalexUsable = providers.openalex === "configured" || providers.openalex === "configured_without_email";
  const semanticScholarUsable = providers.semantic_scholar === "configured";

  return {
    status: "ok",
    capability_mode: openalexUsable || semanticScholarUsable ? "provider_connected" : "strategy_only",
    providers
  };
}
