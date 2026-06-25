const DEFAULT_CONNECTOR_URL = "http://127.0.0.1:23119";
const WRITE_POLICIES = new Set(["dry_run", "explicit", "allow"]);
const UPDATE_POLICIES = new Set(["fill_blank", "prefer_zotero", "prefer_enriched"]);

export function resolveZoteroConfig({ env = process.env, input = {} } = {}) {
  const connectorUrl = String(input.connector_url ?? env.QIONGLI_ZOTERO_CONNECTOR_URL ?? DEFAULT_CONNECTOR_URL).trim();
  assertLoopbackUrl(connectorUrl);

  return {
    local_enabled: readBoolean(input.local_enabled ?? env.QIONGLI_ZOTERO_LOCAL_ENABLED, true),
    connector_url: stripTrailingSlash(connectorUrl),
    default_collection_path: cleanString(input.collection_path ?? env.QIONGLI_ZOTERO_DEFAULT_COLLECTION_PATH),
    default_review_tags: cleanString(input.review_tags ?? env.QIONGLI_ZOTERO_DEFAULT_REVIEW_TAGS),
    default_review_collection_path: cleanString(input.review_collection_path ?? env.QIONGLI_ZOTERO_DEFAULT_REVIEW_COLLECTION_PATH),
    crossref_verification_enabled: readBoolean(input.verify_crossref ?? env.QIONGLI_ZOTERO_CROSSREF_VERIFICATION_ENABLED, true),
    write_policy: normalizeEnum(input.write_policy ?? env.QIONGLI_ZOTERO_WRITE_POLICY, WRITE_POLICIES, "explicit"),
    update_policy: normalizeEnum(input.update_policy ?? env.QIONGLI_ZOTERO_UPDATE_POLICY, UPDATE_POLICIES, "fill_blank")
  };
}

export function assertLoopbackUrl(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("zotero.connector_url must be a valid URL");
  }

  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("zotero.connector_url must use http or https");
  }

  if (!["127.0.0.1", "localhost", "::1", "[::1]"].includes(url.hostname)) {
    throw new Error("zotero.connector_url must point to a loopback host");
  }
}

function normalizeEnum(value, allowed, fallback) {
  const normalized = cleanString(value);
  if (!normalized) {
    return fallback;
  }

  return allowed.has(normalized) ? normalized : fallback;
}

function readBoolean(value, fallback) {
  if (typeof value === "boolean") {
    return value;
  }

  const normalized = cleanString(value);
  if (!normalized) {
    return fallback;
  }

  if (["1", "true", "yes", "on"].includes(normalized.toLowerCase())) {
    return true;
  }

  if (["0", "false", "no", "off"].includes(normalized.toLowerCase())) {
    return false;
  }

  return fallback;
}

function cleanString(value) {
  if (typeof value !== "string") {
    return null;
  }

  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

function stripTrailingSlash(value) {
  return value.replace(/\/+$/, "");
}
