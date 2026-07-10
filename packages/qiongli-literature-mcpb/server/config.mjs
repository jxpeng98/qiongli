import {
  closeSync,
  constants as fsConstants,
  fchmodSync,
  fstatSync,
  fsyncSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync
} from "node:fs";
import { randomBytes } from "node:crypto";
import os from "node:os";
import path from "node:path";

const DEFAULT_LIMIT = 25;
const MIN_LIMIT = 1;
const MAX_LIMIT = 50;
const SUPPORTED_CONFIG_VERSION = 1;
const TEMP_CREATE_ATTEMPTS = 128;
const CONFIG_HOME_PATH_ERROR =
  "QIONGLI_CONFIG_HOME must be a fully qualified absolute path or use '~' home notation";
const USER_HOME_RESOLUTION_ERROR =
  "platform user home directory must be a fully qualified absolute path";
const CONFIG_UNAVAILABLE_ERROR = "provider configuration is unavailable";
const CONFIG_SAVE_ERROR = "provider configuration could not be saved";
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
const PROVIDER_ACTIVATION_FIELDS = {
  openalex: ["api_key"],
  semantic_scholar: ["api_key"],
  crossref: ["email"],
  pubmed: ["api_key"],
  arxiv: []
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
  },
  arxiv: {
    title: "arXiv",
    config_field: "arxiv",
    apply_url: null,
    docs_url: "https://info.arxiv.org/help/api/index.html",
    summary: "arXiv does not require an API key for the public API used by Qiongli.",
    steps: [
      "No credential setup is needed.",
      "Qiongli can use arXiv search after the literature MCP runtime is installed."
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
  const resolved = resolveProviders(shared, env);
  return {
    openalexApiKey: resolved.openalex.values.api_key ?? "",
    openalexEmail: resolved.openalex.values.email ?? "",
    semanticScholarApiKey: resolved.semantic_scholar.values.api_key ?? "",
    crossrefEmail: resolved.crossref.values.email ?? "",
    pubmedApiKey: resolved.pubmed.values.api_key ?? "",
    providerStates: Object.fromEntries(
      Object.entries(resolved).map(([provider, state]) => [
        provider,
        { enabled: state.enabled, configured: state.configured }
      ])
    ),
    defaultLimit: readDefaultLimit(env)
  };
}

export function providerConfigPath(env = process.env) {
  const configured = readTrimmed(env, "QIONGLI_CONFIG_HOME");
  let root;
  if (!configured) {
    root = path.join(platformUserHome(env), ".config", "qiongli");
  } else if (isFullyQualifiedConfigHomePath(configured)) {
    root = configured;
  } else if (configured === "~") {
    root = platformUserHome(env);
  } else if (configured.startsWith("~/")) {
    const suffix = portableTildeSuffix(configured);
    root = path.join(platformUserHome(env), suffix);
  } else {
    throw new Error(CONFIG_HOME_PATH_ERROR);
  }
  return path.join(root, "providers.json");
}

export function isFullyQualifiedConfigHomePath(value, pathApi = path) {
  if (pathApi.sep === "\\") {
    const { root } = pathApi.parse(String(value ?? ""));
    return /^[A-Za-z]:[\\/]$/.test(root) || root.startsWith("\\\\");
  }
  return pathApi.isAbsolute(String(value ?? ""));
}

function portableTildeSuffix(value) {
  const suffix = value.slice(2);
  if (suffix.startsWith("/") || suffix.startsWith("\\") || /^[A-Za-z]:/.test(suffix)) {
    throw new Error(CONFIG_HOME_PATH_ERROR);
  }
  return suffix;
}

export function platformUserHome(env, pathApi = path, fallbackHome = os.homedir()) {
  let configuredHome = "";
  if (pathApi.sep === "\\") {
    configuredHome = readTrimmed(env, "USERPROFILE");
    if (!configuredHome) {
      const homeDrive = readTrimmed(env, "HOMEDRIVE");
      const homePath = readTrimmed(env, "HOMEPATH");
      if (homeDrive && homePath) {
        configuredHome = `${homeDrive}${homePath}`;
      }
    }
  } else {
    configuredHome = readTrimmed(env, "HOME");
  }

  const home = configuredHome || fallbackHome;
  if (!isFullyQualifiedConfigHomePath(home, pathApi)) {
    throw new Error(USER_HOME_RESOLUTION_ERROR);
  }
  return home;
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
  assertSecureProviderConfigWritesSupported();

  const configPath = providerConfigPath(env);
  try {
    const config = readSharedProviderConfig(env);
    canonicalizeKnownAliases(config);
    config.version = SUPPORTED_CONFIG_VERSION;
    if (!isObject(config.providers)) {
      config.providers = {};
    }
    const providerConfig = isObject(config.providers[providerId])
      ? config.providers[providerId]
      : {};
    providerConfig.enabled = true;
    removeNormalizedKey(providerConfig, fieldId);
    providerConfig[fieldId] = String(value ?? "");
    config.providers[providerId] = providerConfig;

    atomicWriteProviderConfig(configPath, `${JSON.stringify(config, null, 2)}\n`);
  } catch {
    throw new Error(CONFIG_SAVE_ERROR);
  }
  return { path: configPath, provider: providerId, field: fieldId };
}

export function providerStatus(config) {
  const states = providerRuntimeStates(config);
  const providers = {
    openalex: states.openalex.configured ? "configured" : "missing",
    semantic_scholar: states.semantic_scholar.configured ? "configured" : "missing",
    crossref: states.crossref.configured ? "configured" : "missing",
    pubmed: states.pubmed.configured ? "configured" : "missing",
    arxiv: states.arxiv.configured ? "configured" : "missing"
  };
  const activeProviders = Object.entries(states)
    .filter(([, state]) => state.enabled && state.configured)
    .map(([provider]) => provider);
  const missing = missingProviderFields(config);
  const nextAction = providerSetupNextAction(missing);

  const status = {
    status: "ok",
    capability_mode: activeProviders.length > 0 ? "provider_connected" : "strategy_only",
    providers,
    active_providers: activeProviders,
    missing
  };
  if (nextAction) {
    status.next_action = nextAction;
  }
  return status;
}

export function redactedProviderStatus(config) {
  const status = providerStatus(config);
  const states = providerRuntimeStates(config);
  const redacted = {
    status: status.status,
    capability_mode: status.capability_mode,
    active_providers: status.active_providers,
    missing: status.missing,
    providers: {
      openalex: {
        enabled: states.openalex.enabled,
        configured: states.openalex.configured,
        fields: {
          api_key: config.openalexApiKey ? "configured" : "missing",
          email: config.openalexEmail ? "configured" : "missing"
        }
      },
      semantic_scholar: {
        enabled: states.semantic_scholar.enabled,
        configured: states.semantic_scholar.configured,
        fields: {
          api_key: config.semanticScholarApiKey ? "configured" : "missing"
        }
      },
      crossref: {
        enabled: states.crossref.enabled,
        configured: states.crossref.configured,
        fields: {
          email: config.crossrefEmail ? "configured" : "missing"
        }
      },
      pubmed: {
        enabled: states.pubmed.enabled,
        configured: states.pubmed.configured,
        fields: {
          api_key: config.pubmedApiKey ? "configured" : "missing"
        }
      },
      arxiv: {
        enabled: states.arxiv.enabled,
        configured: states.arxiv.configured,
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

function readSharedProviderConfig(env) {
  const configPath = providerConfigPath(env);
  let metadata;
  try {
    metadata = lstatSync(configPath);
  } catch (error) {
    if (error?.code === "ENOENT") {
      return { version: SUPPORTED_CONFIG_VERSION, providers: {} };
    }
    throw new Error(CONFIG_UNAVAILABLE_ERROR);
  }

  if (metadata.isSymbolicLink() || !metadata.isFile()) {
    throw new Error(CONFIG_UNAVAILABLE_ERROR);
  }

  let descriptor;
  try {
    const noFollow = process.platform === "win32" ? 0 : (fsConstants.O_NOFOLLOW ?? 0);
    descriptor = openSync(configPath, fsConstants.O_RDONLY | noFollow);
    const opened = fstatSync(descriptor);
    if (!opened.isFile() || !sameFileIdentity(metadata, opened)) {
      throw new Error(CONFIG_UNAVAILABLE_ERROR);
    }
    tightenDescriptorPermissions(descriptor);
    const parsed = JSON.parse(readFileSync(descriptor, "utf8"));
    validateProviderConfig(parsed);
    return parsed;
  } catch {
    throw new Error(CONFIG_UNAVAILABLE_ERROR);
  } finally {
    if (descriptor !== undefined) {
      try {
        closeSync(descriptor);
      } catch {
        // The public error above is deliberately fixed and path-free.
      }
    }
  }
}

function resolveProviders(config, env) {
  return Object.fromEntries(
    Object.keys(PROVIDER_FIELDS).map((provider) => {
      const persisted = persistedProvider(config, provider);
      const values = {};
      let activationEnvironmentSupplied = false;
      for (const [field, aliases] of Object.entries(PROVIDER_FIELDS[provider])) {
        const environmentValue = aliases
          .map((alias) => readTrimmed(env, alias))
          .find(Boolean);
        if (environmentValue) {
          values[field] = environmentValue;
          activationEnvironmentSupplied ||= PROVIDER_ACTIVATION_FIELDS[provider].includes(field);
          continue;
        }
        const persistedValueForField = persistedValue(persisted, field);
        if (typeof persistedValueForField === "string" && persistedValueForField.trim()) {
          values[field] = persistedValueForField.trim();
        }
      }

      const configured = PROVIDER_ACTIVATION_FIELDS[provider]
        .every((field) => Boolean(values[field]));
      const explicitEnabled = persistedValue(persisted, "enabled");
      const enabled = activationEnvironmentSupplied
        ? true
        : typeof explicitEnabled === "boolean"
          ? explicitEnabled
          : configured;
      return [provider, { enabled, configured, values }];
    })
  );
}

function providerRuntimeStates(config) {
  const configured = {
    openalex: Boolean(config?.openalexApiKey),
    semantic_scholar: Boolean(config?.semanticScholarApiKey),
    crossref: Boolean(config?.crossrefEmail),
    pubmed: Boolean(config?.pubmedApiKey),
    arxiv: true
  };
  return Object.fromEntries(
    Object.entries(configured).map(([provider, fallbackConfigured]) => {
      const persisted = config?.providerStates?.[provider];
      const stateConfigured = typeof persisted?.configured === "boolean"
        ? persisted.configured
        : fallbackConfigured;
      const enabled = typeof persisted?.enabled === "boolean"
        ? persisted.enabled
        : stateConfigured;
      return [provider, { enabled, configured: stateConfigured }];
    })
  );
}

function persistedProvider(config, provider) {
  const providers = config.providers;
  if (!isObject(providers)) {
    return undefined;
  }
  for (const [rawProvider, entry] of Object.entries(providers)) {
    if (normalizeProvider(rawProvider) === provider) {
      return isObject(entry) ? entry : undefined;
    }
  }
  return undefined;
}

function persistedValue(entry, field) {
  if (!isObject(entry)) {
    return undefined;
  }
  for (const [rawField, value] of Object.entries(entry)) {
    if (normalizeField(rawField) === field) {
      return value;
    }
  }
  return undefined;
}

function validateProviderConfig(config) {
  if (!isObject(config)) {
    throw new Error(CONFIG_UNAVAILABLE_ERROR);
  }

  if (Object.hasOwn(config, "version")) {
    if (!Number.isInteger(config.version) || config.version < 1) {
      throw new Error(CONFIG_UNAVAILABLE_ERROR);
    }
    if (config.version !== SUPPORTED_CONFIG_VERSION) {
      throw new Error(CONFIG_UNAVAILABLE_ERROR);
    }
  }

  if (Object.hasOwn(config, "providers")) {
    if (!isObject(config.providers)) {
      throw new Error(CONFIG_UNAVAILABLE_ERROR);
    }
    const seenProviders = new Set();
    for (const [rawProvider, entry] of Object.entries(config.providers)) {
      const provider = normalizeProvider(rawProvider);
      if (!Object.hasOwn(PROVIDER_FIELDS, provider)) {
        continue;
      }
      if (seenProviders.has(provider) || !isObject(entry)) {
        throw new Error(CONFIG_UNAVAILABLE_ERROR);
      }
      seenProviders.add(provider);
      const seenFields = new Set();
      for (const [rawField, value] of Object.entries(entry)) {
        const field = normalizeField(rawField);
        const known = field === "enabled" || Object.hasOwn(PROVIDER_FIELDS[provider], field);
        if (known && seenFields.has(field)) {
          throw new Error(CONFIG_UNAVAILABLE_ERROR);
        }
        if (known) {
          seenFields.add(field);
        }
        if (field === "enabled" && typeof value !== "boolean") {
          throw new Error(CONFIG_UNAVAILABLE_ERROR);
        }
        if (Object.hasOwn(PROVIDER_FIELDS[provider], field) && typeof value !== "string") {
          throw new Error(CONFIG_UNAVAILABLE_ERROR);
        }
      }
    }
  }

  if (Object.hasOwn(config, "search")) {
    if (!isObject(config.search)) {
      throw new Error(CONFIG_UNAVAILABLE_ERROR);
    }
    const seenFields = new Set();
    for (const [rawField, value] of Object.entries(config.search)) {
      const field = normalizeField(rawField);
      const known = field === "minimum_productive_providers"
        || field === "allow_platform_search_supplement";
      if (known && seenFields.has(field)) {
        throw new Error(CONFIG_UNAVAILABLE_ERROR);
      }
      if (known) {
        seenFields.add(field);
      }
      if (field === "minimum_productive_providers"
        && (!Number.isInteger(value) || value < 1)) {
        throw new Error(CONFIG_UNAVAILABLE_ERROR);
      }
      if (field === "allow_platform_search_supplement" && typeof value !== "boolean") {
        throw new Error(CONFIG_UNAVAILABLE_ERROR);
      }
    }
  }
}

function canonicalizeKnownAliases(config) {
  const providerEntries = isObject(config.providers) ? Object.entries(config.providers) : [];
  config.providers = Object.fromEntries(
    providerEntries.map(([rawProvider, rawEntry]) => {
      const provider = normalizeProvider(rawProvider);
      if (!Object.hasOwn(PROVIDER_FIELDS, provider)) {
        return [rawProvider, rawEntry];
      }
      const entry = Object.fromEntries(
        Object.entries(rawEntry).map(([rawField, fieldValue]) => {
          const field = normalizeField(rawField);
          const canonical = field === "enabled" || Object.hasOwn(PROVIDER_FIELDS[provider], field);
          return [canonical ? field : rawField, fieldValue];
        })
      );
      return [provider, entry];
    })
  );

  if (isObject(config.search)) {
    config.search = Object.fromEntries(
      Object.entries(config.search).map(([rawField, fieldValue]) => {
        const field = normalizeField(rawField);
        const canonical = field === "minimum_productive_providers"
          || field === "allow_platform_search_supplement";
        return [canonical ? field : rawField, fieldValue];
      })
    );
  }
}

function removeNormalizedKey(entry, field) {
  for (const rawField of Object.keys(entry)) {
    if (normalizeField(rawField) === field) {
      delete entry[rawField];
    }
  }
}

export function atomicWriteProviderConfig(
  configPath,
  contents,
  replace = renameSync,
  platform = process.platform
) {
  assertSecureProviderConfigWritesSupported(platform);
  const directory = path.dirname(configPath);
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  assertSafeConfigTarget(configPath, true);

  let temporaryPath;
  let descriptor;
  try {
    for (let attempt = 0; attempt < TEMP_CREATE_ATTEMPTS; attempt += 1) {
      temporaryPath = path.join(
        directory,
        `.${path.basename(configPath)}.tmp-${process.pid}-${randomBytes(8).toString("hex")}`
      );
      try {
        descriptor = openSync(
          temporaryPath,
          fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL,
          0o600
        );
        break;
      } catch (error) {
        if (error?.code !== "EEXIST") {
          throw error;
        }
      }
    }
    if (descriptor === undefined) {
      throw new Error(CONFIG_SAVE_ERROR);
    }

    tightenDescriptorPermissions(descriptor);
    writeFileSync(descriptor, contents, "utf8");
    fsyncSync(descriptor);
    closeSync(descriptor);
    descriptor = undefined;

    assertSafeConfigTarget(configPath, true);
    replace(temporaryPath, configPath);
    temporaryPath = undefined;
    syncDirectory(directory);
  } finally {
    if (descriptor !== undefined) {
      try {
        closeSync(descriptor);
      } catch {
        // Cleanup is best effort; callers receive a fixed redacted error.
      }
    }
    if (temporaryPath !== undefined) {
      try {
        rmSync(temporaryPath, { force: true });
      } catch {
        // Cleanup is best effort; callers receive a fixed redacted error.
      }
    }
  }
}

export function secureProviderConfigWritesSupported(platform = process.platform) {
  return platform !== "win32";
}

export function assertSecureProviderConfigWritesSupported(platform = process.platform) {
  if (!secureProviderConfigWritesSupported(platform)) {
    throw new Error(CONFIG_SAVE_ERROR);
  }
}

export function sameFileIdentity(
  initial,
  opened,
  requireNonZeroIdentity = process.platform === "win32"
) {
  const sameIdentity = initial?.dev === opened?.dev && initial?.ino === opened?.ino;
  if (!sameIdentity) {
    return false;
  }
  return !requireNonZeroIdentity || initial.ino !== 0;
}

function assertSafeConfigTarget(configPath, allowMissing) {
  try {
    const metadata = lstatSync(configPath);
    if (metadata.isSymbolicLink() || !metadata.isFile()) {
      throw new Error(CONFIG_SAVE_ERROR);
    }
  } catch (error) {
    if (allowMissing && error?.code === "ENOENT") {
      return;
    }
    throw error;
  }
}

function tightenDescriptorPermissions(descriptor) {
  if (process.platform !== "win32") {
    const currentMode = fstatSync(descriptor).mode & 0o777;
    if (currentMode !== 0o600) {
      fchmodSync(descriptor, 0o600);
    }
  }
}

function syncDirectory(directory) {
  if (process.platform === "win32") {
    return;
  }
  let descriptor;
  try {
    descriptor = openSync(directory, fsConstants.O_RDONLY);
    fsyncSync(descriptor);
  } finally {
    if (descriptor !== undefined) {
      closeSync(descriptor);
    }
  }
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
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
  if (!Object.hasOwn(PROVIDER_FIELDS, provider)
    || !Object.hasOwn(PROVIDER_FIELDS[provider], field)) {
    throw new Error("unsupported provider field");
  }
}
