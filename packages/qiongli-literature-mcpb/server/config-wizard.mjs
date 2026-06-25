import { createServer } from "node:http";
import { randomBytes } from "node:crypto";
import {
  providerAccessGuidance,
  providerConfigPath,
  providerFieldAliases,
  saveProviderValue
} from "./config.mjs";

const DEFAULT_HOST = "127.0.0.1";
const ALLOWED_HOSTS = new Set(["127.0.0.1", "localhost"]);
const TOKEN_BYTES = 18;

export async function startConfigWizard({ host = DEFAULT_HOST, port = 0, provider, env = process.env } = {}) {
  const normalizedHost = normalizeHost(host);
  const selectedProvider = normalizeProvider(provider);
  const token = randomBytes(TOKEN_BYTES).toString("base64url");
  const configPath = providerConfigPath(env);
  let closeTimer;
  let completedResolve;
  const completed = new Promise((resolve) => {
    completedResolve = resolve;
  });

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
          Location: `/saved?token=${encodeURIComponent(token)}`
        });
        response.end();
        scheduleClose(1500);
        return;
      }

      if (url.pathname === "/saved") {
        sendHtml(response, renderSavedPage({ configPath }));
        scheduleClose(100);
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
  let closeStarted = false;
  const close = () => new Promise((resolve) => {
    if (closeStarted) {
      resolve();
      return;
    }
    closeStarted = true;
    if (closeTimer) {
      clearTimeout(closeTimer);
    }
    server.close(() => {
      completedResolve?.();
      resolve();
    });
  });

  function scheduleClose(delayMs) {
    if (closeStarted) {
      return;
    }
    if (closeTimer) {
      clearTimeout(closeTimer);
    }
    closeTimer = setTimeout(() => {
      close();
    }, delayMs);
    closeTimer.unref?.();
  }

  const result = {
    url: `http://${normalizedHost}:${actualPort}/?token=${encodeURIComponent(token)}`,
    host: normalizedHost,
    port: actualPort,
    provider: selectedProvider,
    config_path: configPath,
    stop: close
  };
  Object.defineProperty(result, "completed", {
    enumerable: false,
    value: completed
  });
  return result;
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
  const fields = providerFieldAliases()[providerId];
  if (!fields || Object.keys(fields).length === 0) {
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
  const entries = selectedProvider ? [[selectedProvider, aliases[selectedProvider]]] : Object.entries(aliases);
  return entries.filter(([_provider, fields]) => Object.keys(fields).length > 0);
}

function renderForm({ token, configPath, saved, selectedProvider }) {
  const fields = providerEntries(selectedProvider).map(([provider, providerFields]) => {
    const rows = Object.keys(providerFields).map((field) => {
      const name = `${provider}.${field}`;
      const id = `field-${provider}-${field}`;
      const previewId = `preview-${provider}-${field}`;
      return [
        "<div class=\"field-row\">",
        "<div>",
        `<label for="${escapeHtml(id)}">${escapeHtml(provider)} / ${escapeHtml(field)}</label>`,
        `<p class="hint">${escapeHtml(fieldHelp(provider, field))}</p>`,
        "</div>",
        "<div>",
        "<div class=\"input-row\">",
        [
          `<input id="${escapeHtml(id)}"`,
          `name="${escapeHtml(name)}"`,
          "type=\"password\"",
          "autocomplete=\"off\"",
          "spellcheck=\"false\"",
          `aria-describedby="${escapeHtml(previewId)}"`,
          `data-secret-input="${escapeHtml(name)}" />`
        ].join(" "),
        `<button type="button" data-toggle-for="${escapeHtml(name)}" aria-controls="${escapeHtml(id)}" aria-pressed="false">Show</button>`,
        "</div>",
        `<p class="preview" id="${escapeHtml(previewId)}">Preview: <code data-preview-for="${escapeHtml(name)}" class="is-empty">empty</code></p>`,
        "</div>",
        "</div>"
      ].join("");
    }).join("");
    return `<fieldset><legend>${escapeHtml(provider)}</legend>${rows}</fieldset>`;
  }).join("");
  const savedMessage = saved
    ? "<p class=\"saved\">Saved. Run <code>qiongli_config_status</code> to confirm redacted provider status.</p>"
    : "";
  const guidance = renderAccessGuidance();

  return [
    "<!doctype html>",
    "<html><head><meta charset=\"utf-8\" />",
    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />",
    "<title>Qiongli MCP Provider Configuration</title>",
    "<style>",
    "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:860px;color:#24292f}",
    "h1{font-size:28px;margin:0 0 8px}",
    "h2{font-size:16px;margin:0 0 8px}",
    "p{line-height:1.45}",
    ".intro{color:#57606a;margin:0 0 20px}",
    ".notice{background:#f6f8fa;border:1px solid #d0d7de;border-radius:8px;margin:20px 0;padding:16px}",
    ".notice ul{margin:8px 0 0;padding-left:22px}",
    ".notice li{margin:6px 0}",
    ".guidance-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin:20px 0}",
    ".guidance-card{border:1px solid #d0d7de;border-radius:8px;padding:14px}",
    ".guidance-card h3{font-size:15px;margin:0 0 6px}",
    ".guidance-card p{font-size:13px;margin:6px 0}",
    ".guidance-card ol{font-size:13px;margin:8px 0 0;padding-left:20px}",
    ".guidance-card li{margin:5px 0}",
    ".resource-links{display:flex;flex-wrap:wrap;gap:8px}",
    "fieldset{border:1px solid #d0d7de;border-radius:8px;margin:0 0 16px;padding:16px}",
    "legend{font-weight:700;padding:0 6px}",
    ".field-row{display:grid;grid-template-columns:240px 1fr;gap:16px;align-items:start;margin:14px 0}",
    "label{display:block;font-weight:600;margin:2px 0 4px}",
    ".hint{color:#57606a;font-size:13px;margin:0}",
    ".input-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px}",
    "input{font:inherit;padding:8px;border:1px solid #8c959f;border-radius:6px}",
    "input:focus{border-color:#0969da;box-shadow:0 0 0 3px rgba(9,105,218,.15);outline:none}",
    "button{font:inherit;padding:8px 12px;border:1px solid #57606a;border-radius:6px;background:#f6f8fa}",
    "button[disabled]{color:#6e7781;border-color:#d0d7de}",
    ".saved{color:#116329;font-weight:600}",
    ".path{color:#57606a;font-size:13px}",
    ".preview{color:#57606a;font-size:13px;margin:6px 0 0}",
    ".preview code{background:#f6f8fa;border:1px solid #d0d7de;border-radius:6px;color:#24292f;padding:2px 6px;word-break:break-all}",
    ".preview code.is-empty{color:#6e7781}",
    "@media(max-width:760px){body{margin:20px}.guidance-grid{grid-template-columns:1fr}.field-row{grid-template-columns:1fr}.input-row{grid-template-columns:1fr}}",
    "</style></head><body>",
    "<h1>Qiongli MCP Provider Configuration</h1>",
    "<p class=\"intro\">Use this local setup page for Qiongli provider credentials across Codex, Claude Desktop, and other MCP-capable clients.</p>",
    "<section class=\"notice\" aria-label=\"Security notes\">",
    "<h2>Before you save</h2>",
    "<ul>",
    "<li><strong>Keys stay on this machine.</strong> The MCP server writes only to the local shared Qiongli provider config.</li>",
    "<li><strong>Do not paste API keys into chat.</strong> Configure them here so secrets do not enter the conversation transcript.</li>",
    "<li>After saving, ask the client to run <code>qiongli_config_status</code>; it returns only redacted status.</li>",
    "</ul>",
    "</section>",
    guidance,
    `<p class="path">Config path: ${escapeHtml(configPath)}</p>`,
    savedMessage,
    `<form method="post" action="/save?token=${escapeHtml(token)}">`,
    fields,
    "<button type=\"submit\">Save</button>",
    "</form>",
    "<script>",
    "(() => {",
    "  const findByAttr = (selector, attr, value) => Array.from(document.querySelectorAll(selector)).find((node) => node.getAttribute(attr) === value);",
    "  const mask = (value) => {",
    "    if (!value) return 'empty';",
    "    if (value.length <= 8) return '*'.repeat(value.length);",
    "    return `${value.slice(0, 4)}...${value.slice(-4)}`;",
    "  };",
    "  document.querySelectorAll('[data-secret-input]').forEach((input) => {",
    "    const name = input.getAttribute('data-secret-input');",
    "    const preview = findByAttr('[data-preview-for]', 'data-preview-for', name);",
    "    const toggle = findByAttr('[data-toggle-for]', 'data-toggle-for', name);",
    "    let revealed = false;",
    "    const syncPreview = () => {",
    "      if (!preview) return;",
    "      preview.textContent = revealed ? (input.value || 'empty') : mask(input.value);",
    "      preview.classList.toggle('is-empty', !input.value);",
    "    };",
    "    input.addEventListener('input', syncPreview);",
    "    toggle?.addEventListener('click', () => {",
    "      revealed = !revealed;",
    "      input.type = revealed ? 'text' : 'password';",
    "      toggle.textContent = revealed ? 'Hide' : 'Show';",
    "      toggle.setAttribute('aria-pressed', revealed ? 'true' : 'false');",
    "      syncPreview();",
    "      input.focus();",
    "    });",
    "    syncPreview();",
    "  });",
    "  document.querySelector('form')?.addEventListener('submit', (event) => {",
    "    const form = event.currentTarget;",
    "    const submit = form.querySelector('button[type=\"submit\"]');",
    "    form.setAttribute('aria-busy', 'true');",
    "    if (submit) {",
    "      submit.textContent = 'Saving...';",
    "      submit.disabled = true;",
    "    }",
    "  });",
    "})();",
    "</script>",
    "</body></html>"
  ].join("");
}

function renderSavedPage({ configPath }) {
  return [
    "<!doctype html>",
    "<html><head><meta charset=\"utf-8\" />",
    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />",
    "<title>Qiongli MCP Provider Configuration Saved</title>",
    "<style>",
    "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:720px;color:#24292f}",
    "h1{font-size:28px;margin:0 0 8px}",
    "p{line-height:1.45}",
    ".saved{color:#116329;font-weight:700}",
    ".path{color:#57606a;font-size:13px}",
    "code{background:#f6f8fa;border:1px solid #d0d7de;border-radius:6px;color:#24292f;padding:2px 6px;word-break:break-all}",
    "</style></head><body>",
    "<h1>Saved</h1>",
    "<p class=\"saved\">Provider configuration was saved locally.</p>",
    "<p>You can close this page. If this was opened from the CLI, the waiting process will continue automatically.</p>",
    `<p class="path">Config path: <code>${escapeHtml(configPath)}</code></p>`,
    "<script>setTimeout(() => { try { window.close(); } catch (_) {} }, 1200);</script>",
    "</body></html>"
  ].join("");
}

function renderAccessGuidance() {
  const cards = Object.values(providerAccessGuidance()).map((entry) => {
    const links = [
      entry.apply_url
        ? `<a href="${escapeHtml(entry.apply_url)}" target="_blank" rel="noreferrer">Apply or configure</a>`
        : "",
      entry.docs_url
        ? `<a href="${escapeHtml(entry.docs_url)}" target="_blank" rel="noreferrer">Official docs</a>`
        : ""
    ].filter(Boolean).join("");
    const steps = entry.steps.map((step) => `<li>${escapeHtml(step)}</li>`).join("");

    return [
      "<article class=\"guidance-card\">",
      `<h3>${escapeHtml(entry.title)}</h3>`,
      `<p>${escapeHtml(entry.summary)}</p>`,
      `<p>Config field: <code>${escapeHtml(entry.config_field)}</code></p>`,
      `<ol>${steps}</ol>`,
      links ? `<p class="resource-links">${links}</p>` : "",
      "</article>"
    ].join("");
  }).join("");

  return [
    "<section aria-label=\"Provider access guidance\">",
    "<h2>How to get provider access</h2>",
    "<p class=\"intro\">Use the links below to request or prepare provider credentials before saving them here.</p>",
    `<div class="guidance-grid">${cards}</div>`,
    "</section>"
  ].join("");
}

function fieldHelp(provider, field) {
  if (provider === "openalex" && field === "api_key") {
    return "OpenAlex API key from openalex.org/settings/api. Stored only in the local Qiongli provider config.";
  }
  if (provider === "openalex" && field === "email") {
    return "Optional contact email included as mailto for OpenAlex requests.";
  }
  if (provider === "semantic_scholar" && field === "api_key") {
    return "Semantic Scholar API key received by email and used only by the local MCP server.";
  }
  if (provider === "crossref" && field === "email") {
    return "Email for Crossref polite access; no public API key is required.";
  }
  if (provider === "pubmed" && field === "api_key") {
    return "NCBI API key generated from your NCBI account.";
  }
  if (field === "email") {
    return "Provider contact email for rate limits or attribution.";
  }
  return "Provider credential used only by the local MCP server.";
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
