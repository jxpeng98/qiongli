import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {
  TOOL_DECLARATIONS,
  handleConfigStatus,
  handleExportEvidence,
  handleOpenConfigWizard,
  handleSearch,
  handleSaveProviderConfig,
  handleStatus
} from "../server/index.mjs";

test("tool declarations match manifest tool names", () => {
  assert.deepEqual(
    TOOL_DECLARATIONS.map((tool) => tool.name),
    [
      "qiongli_literature_status",
      "qiongli_config_status",
      "qiongli_configure_provider",
      "qiongli_save_provider_config",
      "qiongli_open_config_wizard",
      "qiongli_literature_search",
      "qiongli_literature_export_evidence"
    ]
  );
});

test("handleConfigStatus suggests the platform-neutral setup tool when provider secrets are missing", () => {
  const status = handleConfigStatus({
    env: {
      QIONGLI_CONFIG_HOME: path.join(os.tmpdir(), "qiongli-missing-config")
    }
  });
  const serialized = JSON.stringify(status);

  assert.equal(status.providers.semantic_scholar.fields.api_key, "missing");
  assert.deepEqual(status.missing, ["semantic_scholar.api_key"]);
  assert.equal(status.provider_access_guidance.semantic_scholar.config_field, "semantic_scholar.api_key");
  assert.equal(status.provider_access_guidance.semantic_scholar.apply_url, "https://www.semanticscholar.org/product/api");
  assert.equal(status.provider_access_guidance.openalex.apply_url, "https://openalex.org/settings/api");
  assert.equal(status.provider_access_guidance.pubmed.apply_url, "https://support.nlm.nih.gov/kbArticle/?pn=KA-05317");
  assert.equal(
    status.provider_access_guidance.crossref.docs_url,
    "https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication/"
  );
  assert.deepEqual(status.next_action, {
    tool: "qiongli_configure_provider",
    args: {
      provider: "semantic_scholar"
    },
    message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
  });
  assert.equal(serialized.includes("api_key"), true);
  assert.equal(serialized.includes("secret-key"), false);
});

test("handleStatus redacts configured secrets", () => {
  const status = handleStatus({
    env: {
      QIONGLI_MCPB_OPENALEX_EMAIL: "person@example.com",
      QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
    }
  });

  const serialized = JSON.stringify(status);
  assert.equal(status.status, "ok");
  assert.equal(status.capability_mode, "provider_connected");
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("secret-key"), false);
});

test("handleSaveProviderConfig writes shared provider config without echoing secrets", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-config-"));
  try {
    const response = await handleSaveProviderConfig(
      { provider: "semantic-scholar", field: "api-key", value: "secret-key" },
      { env: { QIONGLI_CONFIG_HOME: configHome } }
    );
    const configPath = path.join(configHome, "providers.json");
    const config = JSON.parse(await readFile(configPath, "utf8"));
    const serialized = JSON.stringify(response);

    assert.equal(response.status, "saved");
    assert.equal(response.provider, "semantic_scholar");
    assert.equal(response.field, "api_key");
    assert.equal(response.warning, "api_key was saved from chat input. Prefer qiongli_configure_provider so provider secrets do not enter chat history.");
    assert.equal(config.providers.semantic_scholar.enabled, true);
    assert.equal(config.providers.semantic_scholar.api_key, "secret-key");
    assert.equal(serialized.includes("secret-key"), false);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleOpenConfigWizard returns local setup URL and saves provider config", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-wizard-"));
  const wizard = await handleOpenConfigWizard({}, { env: { QIONGLI_CONFIG_HOME: configHome } });
  try {
    const response = await fetch(wizard.url, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded"
      },
      body: new URLSearchParams({
        "openalex.email": "person@example.com",
        "semantic_scholar.api_key": "secret-key"
      }),
      redirect: "manual"
    });

    const configPath = path.join(configHome, "providers.json");
    const config = JSON.parse(await readFile(configPath, "utf8"));
    const serialized = JSON.stringify(wizard);

    assert.equal(response.status, 303);
    assert.equal(wizard.host, "127.0.0.1");
    assert.equal(wizard.config_path, configPath);
    assert.equal(config.providers.openalex.email, "person@example.com");
    assert.equal(config.providers.semantic_scholar.api_key, "secret-key");
    assert.equal(serialized.includes("person@example.com"), false);
    assert.equal(serialized.includes("secret-key"), false);
  } finally {
    await wizard.stop();
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleOpenConfigWizard serves guidance, input preview, and saving state", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-wizard-ui-"));
  const wizard = await handleOpenConfigWizard(
    { provider: "semantic_scholar" },
    { env: { QIONGLI_CONFIG_HOME: configHome } }
  );
  try {
    const response = await fetch(wizard.url);
    const html = await response.text();

    assert.equal(response.status, 200);
    assert.equal(html.includes("Keys stay on this machine"), true);
    assert.equal(html.includes("Do not paste API keys into chat"), true);
    assert.equal(html.includes("How to get provider access"), true);
    assert.equal(html.includes("Semantic Scholar API key"), true);
    assert.equal(html.includes("https://www.semanticscholar.org/product/api"), true);
    assert.equal(html.includes("OpenAlex API key"), true);
    assert.equal(html.includes("https://openalex.org/settings/api"), true);
    assert.equal(html.includes("NCBI API key"), true);
    assert.equal(html.includes("https://support.nlm.nih.gov/kbArticle/?pn=KA-05317"), true);
    assert.equal(html.includes("Crossref polite access"), true);
    assert.equal(html.includes("Saving..."), true);
    assert.equal(html.includes("data-preview-for=\"semantic_scholar.api_key\""), true);
    assert.equal(html.includes("type=\"button\" data-toggle-for=\"semantic_scholar.api_key\""), true);
    assert.equal(html.includes("qiongli_config_status"), true);
    assert.equal(html.includes("secret-key"), false);
  } finally {
    await wizard.stop();
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleOpenConfigWizard rejects non-local hosts", async () => {
  await assert.rejects(
    () => handleOpenConfigWizard({ host: "0.0.0.0" }),
    /host must be 127\.0\.0\.1 or localhost/
  );
});

test("handleSearch rejects blank query with sanitized error", async () => {
  await assert.rejects(
    () => handleSearch({ query: "   " }, {
      env: {
        QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
      }
    }),
    (error) => {
      assert.equal(error.message, "query is required");
      assert.equal(error.message.includes("secret-key"), false);
      return true;
    }
  );
});

test("handleSearch aggregates successful and failed providers with warnings", async () => {
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    if (requestedUrl.hostname === "api.openalex.org") {
      assert.equal(requestedUrl.searchParams.get("per-page"), "50");
      return {
        ok: true,
        status: 200,
        async json() {
          return {
            results: [
              {
                id: "https://openalex.org/W1",
                doi: "https://doi.org/10.1234/dupe",
                title: "Shared Result",
                publication_year: 2024
              }
            ]
          };
        }
      };
    }

    assert.equal(requestedUrl.hostname, "api.semanticscholar.org");
    assert.equal(requestedUrl.searchParams.get("limit"), "50");
    return {
      ok: false,
      status: 429,
      async json() {
        return {};
      }
    };
  };

  const response = await handleSearch(
    { query: "literature review", limit: 500 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_EMAIL: "person@example.com",
        QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key",
        QIONGLI_MCPB_DEFAULT_LIMIT: "3"
      },
      fetchImpl
    }
  );

  const serialized = JSON.stringify(response);
  assert.equal(response.status, "ok");
  assert.equal(response.capability_mode, "provider_connected");
  assert.deepEqual(response.providers.attempted, ["openalex", "semantic_scholar"]);
  assert.deepEqual(response.providers.successful, ["openalex"]);
  assert.deepEqual(response.providers.failed, ["semantic_scholar"]);
  assert.deepEqual(response.warnings, ["single_successful_provider", "partial_provider_failure"]);
  assert.equal(response.results.length, 1);
  assert.equal(response.results[0].doi, "10.1234/dupe");
  assert.equal(calls.length, 2);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("secret-key"), false);
});

test("handleExportEvidence without query does not call fetch", async () => {
  let fetchCalls = 0;
  const evidence = await handleExportEvidence(
    {},
    {
      env: {
        QIONGLI_MCPB_OPENALEX_EMAIL: "person@example.com",
        QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
      },
      fetchImpl: async () => {
        fetchCalls += 1;
        throw new Error("fetch should not be called");
      }
    }
  );

  assert.equal(fetchCalls, 0);
  assert.equal(evidence.status, "ok");
  assert.equal(evidence.capability_mode, "provider_connected");
  assert.deepEqual(evidence.providers.openalex, "configured");
  assert.deepEqual(evidence.providers.semantic_scholar, "configured");
  assert.equal(evidence.result_count, 0);
});
