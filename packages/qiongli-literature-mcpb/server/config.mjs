import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const DEFAULT_LIMIT = 25;
const MIN_LIMIT = 1;
const MAX_LIMIT = 50;
const PROVIDER_FIELDS = {
  openalex: {
    api_key: [
      "QIONGLI_OPENALEX_API_KEY",
      "OPENALEX_API_KEY",
      "QIONGLI_MCPB_OPENALEX_API_KEY"
    ],
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
    email: ["QIONGLI_CROSSREF_EMAIL", "CROSSREF_EMAIL", "QIONGLI_MCPB_CROSSREF_EMAIL"]
  },
  pubmed: {
    api_key: ["QIONGLI_NCBI_API_KEY", "NCBI_API_KEY", "PUBMED_API_KEY", "QIONGLI_MCPB_PUBMED_API_KEY"]
  },
  arxiv: {}
};
const PROVIDER_ACCESS_GUIDANCE = {
  openalex: {
    title: "OpenAlex API key",
    config_field: "openalex.api_key",
    apply_url: "https://openalex.org/settings/api",
    docs_url: "https://developers.openalex.org/api-reference/authentication",
    summary: "OpenAlex requires a free API key for API calls at scale. Store the key locally here; email is optional contact metadata.",
    steps: [
      "Sign in to OpenAlex and open the API settings page.",
      "Copy the free API key from the OpenAlex settings page.",
      "Paste the key into openalex.api_key below; optionally add openalex.email for contact attribution."
    ]
  },
  semantic_scholar: {
    title: "Semantic Scholar API key",
    config_field: "semantic_scholar.api_key",
    apply_url: "https://www.semanticscholar.org/product/api",
    docs_url: "https://api.semanticscholar.org/api-docs/",
    summary: "Semantic Scholar sends private API keys by email and recommends using a key for supported requests.",
    steps: [
      "Open the Semantic Scholar API page.",
      "Use the Request an API key section.",
      "Paste only the private key you receive by email into this local setup page."
    ]
  },
  crossref: {
    title: "Crossref polite access",
    config_field: "crossref.email",
    apply_url: null,
    docs_url: "https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication/",
    summary: "Crossref public REST API access does not require signup; provide an email for polite access so Crossref can contact you about problematic traffic.",
    steps: [
      "No public API key application is required for polite access.",
      "Use an email address you monitor.",
      "Use Metadata Plus only if you separately subscribe to Crossref's premium API-key service."
    ]
  },
  pubmed: {
    title: "NCBI API key",
    config_field: "pubmed.api_key",
    apply_url: "https://support.nlm.nih.gov/kbArticle/?pn=KA-05317",
    docs_url: "https://www.ncbi.nlm.nih.gov/books/NBK25501/",
    summary: "NCBI API keys are generated from an NCBI account and can increase E-Utilities request limits.",
    steps: [
      "Sign in to NCBI.",
      "Open Account settings from your username menu.",
      "Create a key in the API Key Management section and paste it here."
    ]
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
    openalexApiKey: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_OPENALEX_API_KEY"),
      readSharedField(shared, "openalex", "api_key")
    ]),
    openalexEmail: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_OPENALEX_EMAIL"),
      readSharedField(shared, "openalex", "email")
    ]),
    semanticScholarApiKey: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"),
      readSharedField(shared, "semantic_scholar", "api_key")
    ]),
    crossrefEmail: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_CROSSREF_EMAIL"),
      readSharedField(shared, "crossref", "email")
    ]),
    pubmedApiKey: firstConfigured([
      readTrimmed(env, "QIONGLI_MCPB_PUBMED_API_KEY"),
      readSharedField(shared, "pubmed", "api_key")
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

export function providerAccessGuidance() {
  return JSON.parse(JSON.stringify(PROVIDER_ACCESS_GUIDANCE));
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
    openalex: config.openalexApiKey ? "configured" : "missing",
    semantic_scholar: config.semanticScholarApiKey ? "configured" : "missing",
    crossref: config.crossrefEmail ? "configured" : "missing",
    pubmed: config.pubmedApiKey ? "configured" : "missing",
    arxiv: "configured"
  };
  const openalexUsable = providers.openalex === "configured";
  const semanticScholarUsable = providers.semantic_scholar === "configured";
  const crossrefUsable = providers.crossref === "configured";
  const pubmedUsable = providers.pubmed === "configured";
  const arxivUsable = providers.arxiv === "configured";
  const missing = missingProviderFields(config);
  const nextAction = providerSetupNextAction(missing);

  const status = {
    status: "ok",
    capability_mode: openalexUsable || semanticScholarUsable || crossrefUsable || pubmedUsable || arxivUsable
      ? "provider_connected"
      : "strategy_only",
    providers,
    missing
  };
  if (nextAction) {
    status.next_action = nextAction;
  }
  return status;
}

export function redactedProviderStatus(config) {
  const status = providerStatus(config);
  const redacted = {
    status: status.status,
    capability_mode: status.capability_mode,
    missing: status.missing,
    providers: {
      openalex: {
        configured: status.providers.openalex === "configured",
        fields: {
          api_key: config.openalexApiKey ? "configured" : "missing",
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
        configured: status.providers.crossref === "configured",
        fields: {
          email: config.crossrefEmail ? "configured" : "missing"
        }
      },
      pubmed: {
        configured: status.providers.pubmed === "configured",
        fields: {
          api_key: config.pubmedApiKey ? "configured" : "missing"
        }
      },
      arxiv: {
        configured: true,
        fields: {}
      }
    },
    provider_access_guidance: providerAccessGuidance()
  };
  if (status.next_action) {
    redacted.next_action = status.next_action;
  }
  return redacted;
}

function missingProviderFields(config) {
  const missing = [];
  if (!config.openalexApiKey) {
    missing.push("openalex.api_key");
  }
  if (!config.semanticScholarApiKey) {
    missing.push("semantic_scholar.api_key");
  }
  if (!config.crossrefEmail) {
    missing.push("crossref.email");
  }
  if (!config.pubmedApiKey) {
    missing.push("pubmed.api_key");
  }
  return missing;
}

function providerSetupNextAction(missing) {
  if (missing.includes("openalex.api_key")) {
    return {
      tool: "qiongli_configure_provider",
      args: {
        provider: "openalex"
      },
      message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
    };
  }

  if (!missing.includes("semantic_scholar.api_key")) {
    if (missing.includes("crossref.email")) {
      return {
        tool: "qiongli_configure_provider",
        args: {
          provider: "crossref"
        },
        message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
      };
    }

    if (missing.includes("pubmed.api_key")) {
      return {
        tool: "qiongli_configure_provider",
        args: {
          provider: "pubmed"
        },
        message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
      };
    }

    return undefined;
  }

  return {
    tool: "qiongli_configure_provider",
    args: {
      provider: "semantic_scholar"
    },
    message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
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
