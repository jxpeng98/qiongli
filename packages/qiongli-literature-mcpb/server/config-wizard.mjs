import { createServer } from "node:http";
import { randomBytes } from "node:crypto";
import { providerConfigPath, providerFieldAliases, saveProviderValue } from "./config.mjs";

const DEFAULT_HOST = "127.0.0.1";
const ALLOWED_HOSTS = new Set(["127.0.0.1", "localhost"]);
const TOKEN_BYTES = 18;

export async function startConfigWizard({ host = DEFAULT_HOST, port = 0, provider, env = process.env } = {}) {
  const normalizedHost = normalizeHost(host);
  const selectedProvider = normalizeProvider(provider);
  const token = randomBytes(TOKEN_BYTES).toString("base64url");
  const configPath = providerConfigPath(env);

  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url ?? "/", `http://${normalizedHost}`);
      if (url.searchParams.get("token") !== token) {
        sendText(response, 403, "Forbidden");
        return;
      }

      if (request.method === "POST") {
        await saveRequestValues(request, env, selectedProvider);
        response.writeHead(303, {
          Location: `/?token=${encodeURIComponent(token)}&saved=1`
        });
        response.end();
        return;
      }

      sendHtml(response, renderForm({
        token,
        configPath,
        saved: url.searchParams.has("saved"),
        selectedProvider
      }));
    } catch (error) {
      sendText(response, 500, sanitizeError(error));
    }
  });

  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, normalizedHost, resolve);
  });
  server.unref?.();

  const address = server.address();
  const actualPort = typeof address === "object" && address ? address.port : port;

  return {
    url: `http://${normalizedHost}:${actualPort}/?token=${encodeURIComponent(token)}`,
    host: normalizedHost,
    port: actualPort,
    provider: selectedProvider,
    config_path: configPath,
    stop: () => new Promise((resolve) => server.close(resolve))
  };
}

function normalizeHost(host) {
  const normalized = String(host || DEFAULT_HOST).trim().toLowerCase();
  if (!ALLOWED_HOSTS.has(normalized)) {
    throw new Error("host must be 127.0.0.1 or localhost");
  }
  return normalized === "localhost" ? DEFAULT_HOST : normalized;
}

function normalizeProvider(provider) {
  if (!provider) {
    return undefined;
  }

  const normalized = String(provider).trim().toLowerCase().replaceAll("-", "_");
  const aliases = {
    s2: "semantic_scholar",
    semanticscholar: "semantic_scholar"
  };
  const providerId = aliases[normalized] ?? normalized;
  if (!providerFieldAliases()[providerId]) {
    throw new Error(`unsupported provider: ${provider}`);
  }
  return providerId;
}

async function saveRequestValues(request, env, selectedProvider) {
  const body = await readBody(request);
  const values = new URLSearchParams(body);
  for (const [provider, fields] of providerEntries(selectedProvider)) {
    for (const field of Object.keys(fields)) {
      const value = String(values.get(`${provider}.${field}`) ?? "").trim();
      if (value) {
        saveProviderValue({ provider, field, value, env });
      }
    }
  }
}

function readBody(request) {
  const chunks = [];
  return new Promise((resolve, reject) => {
    request.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    request.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    request.on("error", reject);
  });
}

function providerEntries(selectedProvider) {
  const aliases = providerFieldAliases();
  return selectedProvider ? [[selectedProvider, aliases[selectedProvider]]] : Object.entries(aliases);
}

function renderForm({ token, configPath, saved, selectedProvider }) {
  const fields = providerEntries(selectedProvider).map(([provider, providerFields]) => {
    const rows = Object.keys(providerFields).map((field) => {
      const name = `${provider}.${field}`;
      return [
        "<label>",
        `<span>${escapeHtml(provider)} / ${escapeHtml(field)}</span>`,
        `<input name="${escapeHtml(name)}" type="password" autocomplete="off" />`,
        "</label>"
      ].join("");
    }).join("");
    return `<fieldset><legend>${escapeHtml(provider)}</legend>${rows}</fieldset>`;
  }).join("");
  const savedMessage = saved ? "<p class=\"saved\">Saved.</p>" : "";

  return [
    "<!doctype html>",
    "<html><head><meta charset=\"utf-8\" />",
    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />",
    "<title>Qiongli MCP Provider Configuration</title>",
    "<style>",
    "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:760px}",
    "fieldset{border:1px solid #d0d7de;border-radius:8px;margin:0 0 16px;padding:16px}",
    "label{display:grid;grid-template-columns:220px 1fr;gap:12px;align-items:center;margin:10px 0}",
    "input{font:inherit;padding:8px;border:1px solid #8c959f;border-radius:6px}",
    "button{font:inherit;padding:8px 12px;border:1px solid #57606a;border-radius:6px;background:#f6f8fa}",
    ".saved{color:#116329;font-weight:600}",
    ".path{color:#57606a;font-size:13px}",
    "@media(max-width:640px){label{grid-template-columns:1fr}}",
    "</style></head><body>",
    "<h1>Qiongli MCP Provider Configuration</h1>",
    `<p class="path">Config path: ${escapeHtml(configPath)}</p>`,
    savedMessage,
    `<form method="post" action="/save?token=${escapeHtml(token)}">`,
    fields,
    "<button type=\"submit\">Save</button>",
    "</form></body></html>"
  ].join("");
}

function sendHtml(response, body) {
  const payload = Buffer.from(body, "utf8");
  response.writeHead(200, {
    "Content-Type": "text/html; charset=utf-8",
    "Content-Length": String(payload.length)
  });
  response.end(payload);
}

function sendText(response, status, body) {
  const payload = Buffer.from(body, "utf8");
  response.writeHead(status, {
    "Content-Type": "text/plain; charset=utf-8",
    "Content-Length": String(payload.length)
  });
  response.end(payload);
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;");
}

function sanitizeError(error) {
  return String(error?.message ?? error ?? "configuration wizard failed")
    .replace(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, "[redacted-email]");
}
