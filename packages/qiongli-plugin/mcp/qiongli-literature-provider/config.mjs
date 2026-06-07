import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const DEFAULT_LIMIT = 10;
const MIN_LIMIT = 1;
const MAX_LIMIT = 50;
const PROVIDER_FIELDS = {
  openalex: {
    email: ["QIONGLI_OPENALEX_EMAIL", "OPENALEX_EMAIL", "QIONGLI_MCPB_OPENALEX_EMAIL"]
  },
  semantic_scholar: {
    api_key: [
      "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
      "SEMANTIC_SCHOLAR_API_KEY",
      "S2_API_KEY",
      "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"
    ]
  },
  crossref: {
    email: ["QIONGLI_CROSSREF_EMAIL", "CROSSREF_EMAIL"]
  },
  pubmed: {
    api_key: ["QIONGLI_NCBI_API_KEY", "NCBI_API_KEY", "PUBMED_API_KEY"]
  }
};

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
  const shared = readSharedProviderConfig(env);
  return {
    openalexEmail: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_OPENALEX_EMAIL"),
      readSharedField(shared, "openalex", "email")
    ]),
    semanticScholarApiKey: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"),
      readSharedField(shared, "semantic_scholar", "api_key")
    ]),
    defaultLimit: readDefaultLimit(env)
  };
}

export function providerConfigPath(env = process.env) {
  const configured = readTrimmed(env, "QIONGLI_CONFIG_HOME");
  const root = configured || path.join(os.homedir(), ".config", "qiongli");
  return path.join(root, "providers.json");
}

export function providerFieldAliases() {
  return PROVIDER_FIELDS;
}

export function saveProviderValue({ provider, field, value, env = process.env } = {}) {
  const providerId = normalizeProvider(provider);
  const fieldId = normalizeField(field);
  assertKnownField(providerId, fieldId);

  const configPath = providerConfigPath(env);
  const config = readSharedProviderConfig(env);
  if (!config.providers || typeof config.providers !== "object" || Array.isArray(config.providers)) {
    config.providers = {};
  }
  const providerConfig = config.providers[providerId] && typeof config.providers[providerId] === "object"
    ? config.providers[providerId]
    : {};
  providerConfig.enabled = true;
  providerConfig[fieldId] = String(value ?? "");
  config.providers[providerId] = providerConfig;

  mkdirSync(path.dirname(configPath), { recursive: true });
  writeFileSync(configPath, `${JSON.stringify(config, null, 2)}\n`, { encoding: "utf8", mode: 0o600 });
  return { path: configPath, provider: providerId, field: fieldId };
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

export function redactedProviderStatus(config) {
  const status = providerStatus(config);
  return {
    status: status.status,
    capability_mode: status.capability_mode,
    providers: {
      openalex: {
        configured: status.providers.openalex === "configured" || status.providers.openalex === "configured_without_email",
        fields: {
          email: config.openalexEmail ? "configured" : "missing"
        }
      },
      semantic_scholar: {
        configured: status.providers.semantic_scholar === "configured",
        fields: {
          api_key: config.semanticScholarApiKey ? "configured" : "missing"
        }
      },
      crossref: {
        configured: false,
        fields: {
          email: "missing"
        }
      },
      pubmed: {
        configured: false,
        fields: {
          api_key: "missing"
        }
      }
    }
  };
}

function firstConfigured(values) {
  return values.map((value) => String(value ?? "").trim()).find((value) => value) ?? "";
}

function readSharedProviderConfig(env) {
  try {
    const parsed = JSON.parse(readFileSync(providerConfigPath(env), "utf8"));
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

function readSharedField(config, provider, field) {
  const providers = config.providers;
  if (!providers || typeof providers !== "object" || Array.isArray(providers)) {
    return "";
  }
  const providerConfig = providers[provider];
  if (!providerConfig || typeof providerConfig !== "object" || Array.isArray(providerConfig)) {
    return "";
  }
  return String(providerConfig[field] ?? "").trim();
}

function normalizeLabel(value) {
  return String(value ?? "").trim().toLowerCase().replaceAll("-", "_");
}

function normalizeProvider(value) {
  const normalized = normalizeLabel(value);
  const aliases = {
    s2: "semantic_scholar",
    semanticscholar: "semantic_scholar",
    semantic_scholar: "semantic_scholar",
    ncbi: "pubmed"
  };
  return aliases[normalized] ?? normalized;
}

function normalizeField(value) {
  return normalizeLabel(value);
}

function assertKnownField(provider, field) {
  if (!PROVIDER_FIELDS[provider]?.[field]) {
    throw new Error(`unsupported provider field: ${provider}.${field}`);
  }
}
