import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {
  TOOL_DECLARATIONS,
  handleExportEvidence,
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
      "qiongli_save_provider_config",
      "qiongli_literature_search",
      "qiongli_literature_export_evidence"
    ]
  );
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
    assert.equal(config.providers.semantic_scholar.enabled, true);
    assert.equal(config.providers.semantic_scholar.api_key, "secret-key");
    assert.equal(serialized.includes("secret-key"), false);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
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
