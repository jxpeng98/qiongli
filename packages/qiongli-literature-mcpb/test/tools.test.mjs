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

test("literature search tool exposes extended search controls", () => {
  const searchTool = TOOL_DECLARATIONS.find((tool) => tool.name === "qiongli_literature_search");
  const properties = searchTool.inputSchema.properties;

  for (const property of [
    "per_provider_limit",
    "perProviderLimit",
    "total_limit",
    "totalLimit",
    "search_depth",
    "searchDepth",
    "include_citations",
    "includeCitations",
    "include_references",
    "includeReferences",
    "document_types",
    "documentTypes",
    "venue_filter",
    "venueFilter",
    "query_variants",
    "queryVariants"
  ]) {
    assert.ok(properties[property], `${property} schema is missing`);
  }

  assert.deepEqual(properties.search_depth.enum, ["quick", "standard", "review", "deep"]);
  assert.deepEqual(properties.searchDepth.enum, ["quick", "standard", "review", "deep"]);
  assert.equal(properties.document_types.items.type, "string");
  assert.equal(properties.documentTypes.items.type, "string");
  assert.equal(properties.query_variants.items.type, "string");
  assert.equal(properties.queryVariants.items.type, "string");
});

test("handleConfigStatus suggests the platform-neutral setup tool when provider secrets are missing", () => {
  const status = handleConfigStatus({
    env: {
      QIONGLI_CONFIG_HOME: path.join(os.tmpdir(), "qiongli-missing-config")
    }
  });
  const serialized = JSON.stringify(status);

  assert.equal(status.providers.openalex.fields.api_key, "missing");
  assert.equal(status.providers.semantic_scholar.fields.api_key, "missing");
  assert.equal(status.providers.crossref.fields.email, "missing");
  assert.equal(status.providers.pubmed.fields.api_key, "missing");
  assert.deepEqual(status.missing, [
    "openalex.api_key",
    "semantic_scholar.api_key",
    "crossref.email",
    "pubmed.api_key"
  ]);
  assert.equal(status.provider_access_guidance.openalex.config_field, "openalex.api_key");
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
      provider: "openalex"
    },
    message: "Run qiongli_configure_provider to open a local setup page. Do not paste API keys in chat."
  });
  assert.equal(serialized.includes("api_key"), true);
  assert.equal(serialized.includes("secret-key"), false);
});

test("handleStatus redacts configured secrets", () => {
  const status = handleStatus({
    env: {
      QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
      QIONGLI_MCPB_OPENALEX_EMAIL: "person@example.com",
      QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key",
      QIONGLI_MCPB_CROSSREF_EMAIL: "crossref@example.com",
      QIONGLI_MCPB_PUBMED_API_KEY: "pubmed-secret-key"
    }
  });

  const serialized = JSON.stringify(status);
  assert.equal(status.status, "ok");
  assert.equal(status.capability_mode, "provider_connected");
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(status.providers.crossref, "configured");
  assert.equal(status.providers.pubmed, "configured");
  assert.equal(serialized.includes("openalex-secret-key"), false);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("secret-key"), false);
  assert.equal(serialized.includes("crossref@example.com"), false);
  assert.equal(serialized.includes("pubmed-secret-key"), false);
});

test("status responses include provider capability registry", () => {
  const status = handleStatus({
    env: {
      QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
      QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
    }
  });

  assert.equal(status.provider_capabilities.openalex.status, "implemented");
  assert.equal(status.provider_capabilities.semantic_scholar.status, "implemented");
  assert.equal(status.provider_capabilities.crossref.status, "implemented");
  assert.equal(status.provider_capabilities.pubmed.status, "implemented");
  assert.equal(status.provider_capabilities.openalex.capabilities.includes("document_type_filter"), true);
  assert.equal(status.provider_capabilities.semantic_scholar.capabilities.includes("publication_type_metadata"), true);
  assert.equal(status.provider_capabilities.crossref.capabilities.includes("reference_metadata"), true);
  assert.equal(status.provider_capabilities.pubmed.capabilities.includes("biomedical_topic_search"), true);
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
        "openalex.api_key": "openalex-secret-key",
        "openalex.email": "person@example.com",
        "semantic_scholar.api_key": "secret-key"
      }),
      redirect: "manual"
    });
    await wizard.completed;

    const configPath = path.join(configHome, "providers.json");
    const config = JSON.parse(await readFile(configPath, "utf8"));
    const serialized = JSON.stringify(wizard);

    assert.equal(response.status, 303);
    assert.equal(wizard.host, "127.0.0.1");
    assert.equal(wizard.config_path, configPath);
    assert.equal(config.providers.openalex.api_key, "openalex-secret-key");
    assert.equal(config.providers.openalex.email, "person@example.com");
    assert.equal(config.providers.semantic_scholar.api_key, "secret-key");
    assert.equal(serialized.includes("openalex-secret-key"), false);
    assert.equal(serialized.includes("person@example.com"), false);
    assert.equal(serialized.includes("secret-key"), false);
  } finally {
    await wizard.stop();
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleOpenConfigWizard serves guidance, OpenAlex key input, and saving state", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-wizard-ui-"));
  const wizard = await handleOpenConfigWizard(
    { provider: "openalex" },
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
    assert.equal(html.includes("data-preview-for=\"openalex.api_key\""), true);
    assert.equal(html.includes("type=\"button\" data-toggle-for=\"openalex.api_key\""), true);
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
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
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
  assert.equal(calls.length, 4);
  assert.deepEqual(response.diagnostics.providers, [
    {
      provider: "openalex",
      status: "success",
      result_count: 1,
      request_count: 1,
      attempts: 1,
      error: null
    },
    {
      provider: "semantic_scholar",
      status: "failed",
      result_count: 0,
      request_count: 1,
      attempts: 3,
      error: "semantic_scholar HTTP 429"
    }
  ]);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("secret-key"), false);
});

test("handleSearch fans out to configured Crossref and PubMed providers", async () => {
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    if (requestedUrl.hostname === "api.openalex.org") {
      return {
        ok: true,
        status: 200,
        async json() {
          return { results: [{ id: "https://openalex.org/W1", title: "OpenAlex Result" }] };
        }
      };
    }

    if (requestedUrl.hostname === "api.semanticscholar.org") {
      return {
        ok: true,
        status: 200,
        async json() {
          return { data: [{ paperId: "S1", title: "Semantic Result" }] };
        }
      };
    }

    if (requestedUrl.hostname === "api.crossref.org") {
      return {
        ok: true,
        status: 200,
        async json() {
          return {
            message: {
              items: [{ DOI: "10.5555/crossref", title: ["Crossref Result"] }]
            }
          };
        }
      };
    }

    assert.equal(requestedUrl.hostname, "eutils.ncbi.nlm.nih.gov");
    if (requestedUrl.pathname.endsWith("/esearch.fcgi")) {
      return {
        ok: true,
        status: 200,
        async json() {
          return { esearchresult: { idlist: ["123"] } };
        }
      };
    }

    return {
      ok: true,
      status: 200,
      async json() {
        return {
          result: {
            uids: ["123"],
            123: {
              uid: "123",
              title: "PubMed Result"
            }
          }
        };
      }
    };
  };

  const response = await handleSearch(
    { query: "provider fanout", per_provider_limit: 2 },
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
        QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key",
        QIONGLI_MCPB_CROSSREF_EMAIL: "crossref@example.com",
        QIONGLI_MCPB_PUBMED_API_KEY: "pubmed-secret-key"
      },
      fetchImpl
    }
  );

  assert.deepEqual(response.providers.attempted, [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed"
  ]);
  assert.deepEqual(response.providers.failed, []);
  assert.equal(calls.some((url) => url.hostname === "api.crossref.org"), true);
  assert.equal(calls.some((url) => url.hostname === "eutils.ncbi.nlm.nih.gov"), true);
  assert.equal(response.results.some((result) => result.provider === "crossref"), true);
  assert.equal(response.results.some((result) => result.provider === "pubmed"), true);
});

test("handleSearch uses review-mode default limit for literature reviews", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-review-default-"));
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
          return { results: [] };
        }
      };
    }

    assert.equal(requestedUrl.hostname, "api.semanticscholar.org");
    assert.equal(requestedUrl.searchParams.get("limit"), "50");
    return {
      ok: true,
      status: 200,
      async json() {
        return { data: [] };
      }
    };
  };

  try {
    const response = await handleSearch(
      { query: "social media mental health", search_mode: "review" },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
          QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key",
          QIONGLI_MCPB_DEFAULT_LIMIT: "10"
        },
        fetchImpl
      }
    );

    assert.equal(response.search_mode, "review");
    assert.equal(calls.length, 2);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch keeps provider pages capped for explicit review limits", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-review-explicit-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    if (requestedUrl.hostname === "api.openalex.org") {
      assert.equal(requestedUrl.searchParams.get("per-page"), "100");
      return {
        ok: true,
        status: 200,
        async json() {
          return { results: [] };
        }
      };
    }

    assert.equal(requestedUrl.hostname, "api.semanticscholar.org");
    assert.equal(requestedUrl.searchParams.get("limit"), "100");
    return {
      ok: true,
      status: 200,
      async json() {
        return { data: [] };
      }
    };
  };

  try {
    const response = await handleSearch(
      { query: "social media mental health", search_mode: "systematic_review", limit: 120 },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
          QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.search_mode, "review");
    assert.equal(response.search_options.per_provider_limit, 120);
    assert.equal(calls.length, 2);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch separates per-provider and total result limits", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-advanced-limits-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    if (requestedUrl.hostname === "api.openalex.org") {
      assert.equal(requestedUrl.searchParams.get("per-page"), "3");
      return {
        ok: true,
        status: 200,
        async json() {
          return {
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Governance One",
                publication_year: 2024,
                type: "journal-article"
              },
              {
                id: "https://openalex.org/W2",
                title: "Governance Two",
                publication_year: 2023,
                type: "book-chapter"
              }
            ]
          };
        }
      };
    }

    assert.equal(requestedUrl.hostname, "api.semanticscholar.org");
    assert.equal(requestedUrl.searchParams.get("limit"), "3");
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          data: [
            {
              paperId: "S1",
              title: "Governance Three",
              year: 2022,
              publicationTypes: ["JournalArticle"]
            },
            {
              paperId: "S2",
              title: "Governance Four",
              year: 2021,
              publicationTypes: ["Review"]
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      { query: "governance", per_provider_limit: 3, total_limit: 2 },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
          QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(calls.length, 2);
    assert.equal(response.search_options.per_provider_limit, 3);
    assert.equal(response.search_options.total_limit, 2);
    assert.equal(response.results.length, 2);
    assert.deepEqual(
      response.results.map((result) => result.title),
      ["Governance One", "Governance Two"]
    );
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch fans out explicit query variants with query diagnostics", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-query-variants-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);
    assert.equal(requestedUrl.hostname, "api.openalex.org");
    assert.equal(requestedUrl.searchParams.get("per-page"), "2");

    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        return {
          results: [
            {
              id: `https://openalex.org/W${calls.length}`,
              title: `Result for ${requestedUrl.searchParams.get("search")}`
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      {
        query: "older adults conversational agents",
        query_variants: [
          "older people chatbots",
          "home health conversational agents"
        ],
        per_provider_limit: 6
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.deepEqual(
      calls.map((url) => url.searchParams.get("search")),
      [
        "older adults conversational agents",
        "older people chatbots",
        "home health conversational agents"
      ]
    );
    assert.equal(response.search_options.per_provider_limit, 6);
    assert.equal(response.search_options.per_query_provider_limit, 2);
    assert.equal(response.search_options.query_count, 3);
    assert.deepEqual(response.search_plan.queries, [
      {
        query_id: "q1",
        query: "older adults conversational agents",
        source: "primary",
        rationale: "primary query"
      },
      {
        query_id: "q2",
        query: "older people chatbots",
        source: "explicit_variant",
        rationale: "user supplied query variant"
      },
      {
        query_id: "q3",
        query: "home health conversational agents",
        source: "explicit_variant",
        rationale: "user supplied query variant"
      }
    ]);
    assert.deepEqual(response.diagnostics.providers, [
      {
        provider: "openalex",
        status: "success",
        result_count: 3,
        request_count: 3,
        attempts: 3,
        error: null
      }
    ]);
    assert.deepEqual(
      response.diagnostics.queries.map((entry) => ({
        query_id: entry.query_id,
        provider: entry.provider,
        result_count: entry.result_count,
        status: entry.status
      })),
      [
        { query_id: "q1", provider: "openalex", result_count: 1, status: "success" },
        { query_id: "q2", provider: "openalex", result_count: 1, status: "success" },
        { query_id: "q3", provider: "openalex", result_count: 1, status: "success" }
      ]
    );
    assert.equal(response.results.length, 3);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch adds automatic query variants for deep searches", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-auto-query-variants-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);
    const callIndex = calls.length;
    assert.equal(requestedUrl.hostname, "api.openalex.org");
    assert.equal(requestedUrl.searchParams.get("per-page"), "3");

    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        return {
          results: [
            {
              id: `https://openalex.org/A${callIndex}`,
              title: `Auto Result ${callIndex}`
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      {
        query: "social media mental health",
        search_depth: "deep",
        per_provider_limit: 9
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.deepEqual(
      calls.map((url) => url.searchParams.get("search")),
      [
        "social media mental health",
        "social media mental health review",
        "social media mental health systematic review"
      ]
    );
    assert.equal(response.search_plan.mode, "auto_deep");
    assert.equal(response.search_options.per_query_provider_limit, 3);
    assert.equal(response.search_options.query_count, 3);
    assert.deepEqual(
      response.search_plan.queries.map((query) => query.source),
      ["primary", "auto_variant", "auto_variant"]
    );
    assert.equal(response.diagnostics.queries.length, 3);
    assert.equal(response.results.length, 3);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch uses finance/econ routing for deep searches", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-finance-routing-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);
    const search = requestedUrl.searchParams.get("search");
    assert.equal(requestedUrl.hostname, "api.openalex.org");
    assert.equal(requestedUrl.searchParams.get("per-page"), "3");

    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        if (search?.includes("working paper")) {
          return {
            results: [
              {
                id: "https://openalex.org/W-working",
                title: "Corporate Finance Working Paper",
                type: "working-paper",
                primary_location: {
                  source: {
                    display_name: "NBER Working Paper"
                  }
                }
              }
            ]
          };
        }

        if (search === "asset pricing corporate finance") {
          return {
            results: [
              {
                id: "https://openalex.org/W-published",
                doi: "https://doi.org/10.1000/finance-published",
                title: "Asset Pricing Corporate Finance Study",
                type: "journal-article",
                primary_location: {
                  source: {
                    display_name: "Journal of Finance"
                  }
                }
              }
            ]
          };
        }

        return { results: [] };
      }
    };
  };

  try {
    const response = await handleSearch(
      {
        query: "asset pricing corporate finance",
        search_depth: "deep",
        per_provider_limit: 12
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.deepEqual(
      calls.map((url) => url.searchParams.get("search")),
      [
        "asset pricing corporate finance",
        "asset pricing corporate finance working paper",
        "asset pricing corporate finance JEL G12",
        "asset pricing corporate finance review"
      ]
    );
    assert.equal(response.search_plan.domain, "finance_economics");
    assert.deepEqual(response.search_plan.domain_terms, ["asset pricing", "corporate finance"]);
    assert.equal(response.search_options.query_count, 4);
    assert.equal(response.search_options.per_query_provider_limit, 3);
    assert.deepEqual(
      response.search_plan.queries.map((query) => query.source),
      ["primary", "domain_variant", "domain_variant", "domain_variant"]
    );
    assert.equal(response.diagnostics.domain, "finance_economics");
    assert.deepEqual(response.diagnostics.field_term_coverage, {
      covered: true,
      matched_terms: ["asset pricing", "corporate finance"]
    });
    assert.deepEqual(response.diagnostics.working_paper_coverage, {
      covered: true,
      result_count: 1
    });
    assert.deepEqual(response.diagnostics.published_version_coverage, {
      covered: true,
      result_count: 1
    });
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch computes finance/econ coverage before total limit truncation", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-finance-coverage-"));
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    headers: new Headers(),
    async json() {
      return {
        results: [
          {
            id: "https://openalex.org/W-published",
            doi: "https://doi.org/10.1000/asset-pricing-published",
            title: "Asset Pricing Study",
            type: "journal-article",
            primary_location: {
              source: {
                display_name: "Journal of Finance"
              }
            }
          },
          {
            id: "https://openalex.org/W-working",
            title: "Asset Pricing Working Paper",
            type: "working-paper",
            primary_location: {
              source: {
                display_name: "NBER Working Paper"
              }
            }
          }
        ]
      };
    }
  });

  try {
    const response = await handleSearch(
      {
        query: "asset pricing",
        search_depth: "deep",
        query_variants: [],
        per_provider_limit: 10,
        total_limit: 1
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.results.length, 1);
    assert.equal(response.diagnostics.filtered_result_count, 2);
    assert.deepEqual(response.diagnostics.working_paper_coverage, {
      covered: true,
      result_count: 1
    });
    assert.deepEqual(response.diagnostics.published_version_coverage, {
      covered: true,
      result_count: 1
    });
    assert.equal(response.warnings.includes("missing_working_paper_coverage"), false);
    assert.equal(response.warnings.includes("missing_published_version_coverage"), false);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch warns when finance/econ deep search misses version coverage", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-finance-coverage-warnings-"));
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    headers: new Headers(),
    async json() {
      return {
        results: [
          {
            id: "https://openalex.org/W-untyped",
            title: "Asset Pricing Notes"
          }
        ]
      };
    }
  });

  try {
    const response = await handleSearch(
      {
        query: "asset pricing",
        search_depth: "deep",
        query_variants: [],
        per_provider_limit: 10
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.diagnostics.domain, "finance_economics");
    assert.equal(response.diagnostics.working_paper_coverage.covered, false);
    assert.equal(response.diagnostics.published_version_coverage.covered, false);
    assert.equal(response.warnings.includes("missing_working_paper_coverage"), true);
    assert.equal(response.warnings.includes("missing_published_version_coverage"), true);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch returns structured diagnostics for deep paginated searches", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-deep-diagnostics-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);
    assert.equal(requestedUrl.hostname, "api.openalex.org");
    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        if (calls.length === 1) {
          return {
            meta: {
              next_cursor: "next-deep-cursor"
            },
            results: [
              {
                id: "https://openalex.org/W1",
                title: "Deep Result One"
              }
            ]
          };
        }

        return {
          meta: {},
          results: [
            {
              id: "https://openalex.org/W2",
              title: "Deep Result Two"
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      {
        query: "deep diagnostics",
        search_depth: "deep",
        per_provider_limit: 150,
        query_variants: []
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.search_options.per_provider_limit, 150);
    assert.equal(calls.length, 2);
    assert.equal(response.diagnostics.raw_result_count, 2);
    assert.equal(response.diagnostics.deduped_result_count, 2);
    assert.equal(response.diagnostics.filtered_result_count, 2);
    assert.equal(response.diagnostics.returned_result_count, 2);
    assert.deepEqual(response.diagnostics.providers, [
      {
        provider: "openalex",
        status: "success",
        result_count: 2,
        request_count: 2,
        attempts: 2,
        error: null
      }
    ]);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch applies venue and document type filters", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-search-filters-"));
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    assert.equal(requestedUrl.hostname, "api.openalex.org");
    assert.equal(
      requestedUrl.searchParams.get("filter"),
      "type:journal-article"
    );

    return {
      ok: true,
      status: 200,
      async json() {
        return {
          results: [
            {
              id: "https://openalex.org/W-match",
              title: "Matching Paper",
              publication_year: 2024,
              type: "journal-article",
              primary_location: {
                source: {
                  display_name: "Journal of Tests"
                }
              }
            },
            {
              id: "https://openalex.org/W-venue",
              title: "Wrong Venue",
              publication_year: 2024,
              type: "journal-article",
              primary_location: {
                source: {
                  display_name: "Conference of Tests"
                }
              }
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      {
        query: "filter test",
        document_types: ["journal-article"],
        venue_filter: "Journal of Tests",
        total_limit: 10
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.deepEqual(response.results.map((result) => result.title), ["Matching Paper"]);
    assert.equal(response.results[0].document_type, "journal-article");
    assert.deepEqual(response.search_options.filters.document_types, ["journal-article"]);
    assert.equal(response.search_options.filters.venue_filter, "Journal of Tests");
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch warns when review-mode search returns too few results", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-review-threshold-"));
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    async json() {
      return {
        results: [
          {
            id: "https://openalex.org/W1",
            title: "Sparse Review Result",
            publication_year: 2024
          }
        ]
      };
    }
  });

  try {
    const response = await handleSearch(
      { query: "rare topic", search_mode: "review" },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.search_options.search_depth, "review");
    assert.equal(response.search_options.minimum_result_threshold, 25);
    assert.equal(response.warnings.includes("insufficient_review_results"), true);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch reports limited citation and reference expansion warnings", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-expansion-warnings-"));
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    async json() {
      return { results: [] };
    }
  });

  try {
    const response = await handleSearch(
      {
        query: "citation graph topic",
        include_citations: true,
        include_references: true
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(response.search_options.include_citations, true);
    assert.equal(response.search_options.include_references, true);
    assert.equal(response.warnings.includes("citation_expansion_limited"), true);
    assert.equal(response.warnings.includes("reference_expansion_limited"), true);
    assert.equal(response.warnings.includes("citation_expansion_not_available"), false);
    assert.equal(response.warnings.includes("reference_expansion_not_available"), false);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleSearch reranks exact title matches and limits title-mode output", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-title-search-"));
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    assert.equal(requestedUrl.hostname, "api.openalex.org");
    assert.equal(requestedUrl.searchParams.get("per-page"), "10");

    return {
      ok: true,
      status: 200,
      async json() {
        return {
          results: [
            {
              id: "https://openalex.org/W-near",
              title: "Attention Mechanisms in Neural Translation",
              publication_year: 2018
            },
            {
              id: "https://openalex.org/W-exact",
              title: "Attention Is All You Need",
              publication_year: 2017,
              doi: "https://doi.org/10.5555/exact-title"
            }
          ]
        };
      }
    };
  };

  try {
    const response = await handleSearch(
      { query: "Attention Is All You Need", search_mode: "title", limit: 1 },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(calls.length, 1);
    assert.deepEqual(
      response.results.map((result) => result.title),
      ["Attention Is All You Need"]
    );
    assert.equal(response.results[0].doi, "10.5555/exact-title");
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});

test("handleExportEvidence without query does not call fetch", async () => {
  let fetchCalls = 0;
  const evidence = await handleExportEvidence(
    {},
    {
      env: {
        QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key",
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

test("handleExportEvidence with query returns an auditable search snapshot", async () => {
  const configHome = await mkdtemp(path.join(os.tmpdir(), "qiongli-mcpb-export-evidence-"));
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    headers: new Headers(),
    async json() {
      return {
        results: [
          {
            id: "https://openalex.org/W-export",
            title: "Export Evidence Paper",
            doi: "https://doi.org/10.1000/export-evidence",
            publication_year: 2024
          }
        ]
      };
    }
  });

  try {
    const evidence = await handleExportEvidence(
      {
        query: "export evidence",
        search_depth: "deep",
        query_variants: [],
        total_limit: 1
      },
      {
        env: {
          QIONGLI_CONFIG_HOME: configHome,
          QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret-key"
        },
        fetchImpl
      }
    );

    assert.equal(evidence.status, "ok");
    assert.equal(evidence.result_count, 1);
    assert.equal(evidence.search_plan.mode, "explicit");
    assert.equal(evidence.search_options.search_depth, "deep");
    assert.equal(evidence.diagnostics.raw_result_count, 1);
    assert.equal(evidence.provider_capabilities.openalex.status, "implemented");
    assert.deepEqual(
      evidence.results.map((result) => result.doi),
      ["10.1000/export-evidence"]
    );
    assert.equal(JSON.stringify(evidence).includes("openalex-secret-key"), false);
  } finally {
    await rm(configHome, { recursive: true, force: true });
  }
});
