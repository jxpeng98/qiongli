import test from "node:test";
import assert from "node:assert/strict";
import { searchOpenAlex } from "../server/providers/openalex.mjs";
import { searchSemanticScholar } from "../server/providers/semantic-scholar.mjs";
import { dedupeResults, normalizeResult } from "../server/normalize.mjs";

test("searchOpenAlex normalizes records and reconstructs inverted abstracts", async () => {
  let requestedUrl;
  const fetchImpl = async (url) => {
    requestedUrl = new URL(url);
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          results: [
            {
              id: "https://openalex.org/W123",
              doi: "https://doi.org/10.1000/Example",
              title: "OpenAlex Paper",
              publication_year: 2024,
              primary_location: {
                source: {
                  display_name: "Journal of Tests"
                },
                landing_page_url: "https://example.test/paper"
              },
              authorships: [
                {
                  author: {
                    display_name: "Ada Lovelace"
                  }
                },
                {
                  author: {
                    display_name: "Grace Hopper"
                  }
                }
              ],
              abstract_inverted_index: {
                Results: [0],
                are: [1],
                normalized: [2]
              }
            }
          ]
        };
      }
    };
  };

  const response = await searchOpenAlex({
    query: "test query",
    limit: 7,
    email: "person@example.com",
    apiKey: "openalex-secret-key",
    fromYear: 2020,
    toYear: 2024,
    fetchImpl
  });

  assert.equal(response.provider, "openalex");
  assert.equal(response.error, null);
  assert.equal(requestedUrl.origin, "https://api.openalex.org");
  assert.equal(requestedUrl.pathname, "/works");
  assert.equal(requestedUrl.searchParams.get("search"), "test query");
  assert.equal(requestedUrl.searchParams.get("per-page"), "7");
  assert.equal(requestedUrl.searchParams.get("mailto"), "person@example.com");
  assert.equal(requestedUrl.searchParams.get("api_key"), "openalex-secret-key");
  assert.equal(requestedUrl.searchParams.get("filter"), "from_publication_date:2020-01-01,to_publication_date:2024-12-31");
  assert.deepEqual(response.results, [
    {
      title: "OpenAlex Paper",
      authors: ["Ada Lovelace", "Grace Hopper"],
      year: 2024,
      doi: "10.1000/Example",
      url: "https://example.test/paper",
      abstract: "Results are normalized",
      venue: "Journal of Tests",
      provider: "openalex",
      source_id: "W123"
    }
  ]);
});

test("searchSemanticScholar sends optional API key header and normalizes results", async () => {
  const calls = [];
  const fetchImpl = async (url, options = {}) => {
    calls.push({ url: new URL(url), options });
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          data: [
            {
              paperId: "abc123",
              title: "Semantic Scholar Paper",
              year: 2023,
              authors: [{ name: "Katherine Johnson" }],
              abstract: "Useful abstract",
              url: "https://semanticscholar.org/paper/abc123",
              venue: "Conference of Tests",
              externalIds: {
                DOI: "https://doi.org/10.2000/Semantic"
              }
            }
          ]
        };
      }
    };
  };

  const response = await searchSemanticScholar({
    query: "semantic query",
    limit: 3,
    apiKey: "secret-api-key",
    fromYear: 2021,
    toYear: 2023,
    fetchImpl
  });

  assert.equal(response.provider, "semantic_scholar");
  assert.equal(response.error, null);
  assert.equal(calls[0].url.origin, "https://api.semanticscholar.org");
  assert.equal(calls[0].url.pathname, "/graph/v1/paper/search");
  assert.equal(calls[0].url.searchParams.get("query"), "semantic query");
  assert.equal(calls[0].url.searchParams.get("limit"), "3");
  assert.equal(calls[0].url.searchParams.get("year"), "2021-2023");
  assert.equal(calls[0].url.searchParams.get("fields"), "paperId,title,year,authors,abstract,url,venue,externalIds");
  assert.equal(calls[0].options.headers["x-api-key"], "secret-api-key");
  assert.deepEqual(response.results, [
    {
      title: "Semantic Scholar Paper",
      authors: ["Katherine Johnson"],
      year: 2023,
      doi: "10.2000/Semantic",
      url: "https://semanticscholar.org/paper/abc123",
      abstract: "Useful abstract",
      venue: "Conference of Tests",
      provider: "semantic_scholar",
      source_id: "abc123"
    }
  ]);
});

test("searchSemanticScholar omits blank API key header", async () => {
  let observedHeaders;
  const fetchImpl = async (_url, options = {}) => {
    observedHeaders = options.headers;
    return {
      ok: true,
      status: 200,
      async json() {
        return { data: [] };
      }
    };
  };

  await searchSemanticScholar({
    query: "semantic query",
    limit: 1,
    apiKey: "   ",
    fetchImpl
  });

  assert.deepEqual(observedHeaders, {});
});

test("provider HTTP failures return sanitized error payloads", async () => {
  const fetchImpl = async () => ({
    ok: false,
    status: 429,
    statusText: "Too Many Requests: secret-api-key",
    async text() {
      return "quota exceeded for secret-api-key";
    }
  });

  const response = await searchSemanticScholar({
    query: "semantic query",
    limit: 1,
    apiKey: "secret-api-key",
    fetchImpl
  });

  assert.equal(response.provider, "semantic_scholar");
  assert.deepEqual(response.results, []);
  assert.match(response.error, /HTTP 429/);
  assert.equal(response.error.includes("secret-api-key"), false);
});

test("normalizeResult normalizes DOI and preserves missing metadata as nulls and arrays", () => {
  assert.deepEqual(
    normalizeResult({
      title: "",
      authors: null,
      year: undefined,
      doi: "https://doi.org/10.3000/Normalize",
      url: "",
      abstract: undefined,
      venue: "",
      provider: "test_provider",
      source_id: "source-1"
    }),
    {
      title: null,
      authors: [],
      year: null,
      doi: "10.3000/Normalize",
      url: null,
      abstract: null,
      venue: null,
      provider: "test_provider",
      source_id: "source-1"
    }
  );
});

test("dedupeResults dedupes by DOI before falling back to provider source and title year", () => {
  const results = dedupeResults([
    normalizeResult({ title: "A", year: 2024, doi: "https://doi.org/10.4000/Dupe", provider: "openalex", source_id: "W1" }),
    normalizeResult({ title: "A duplicate", year: 2024, doi: "10.4000/Dupe", provider: "semantic_scholar", source_id: "S1" }),
    normalizeResult({ title: "B", year: 2024, provider: "openalex", source_id: "W2" }),
    normalizeResult({ title: "B", year: 2024, provider: "openalex", source_id: "W2" }),
    normalizeResult({ title: "Same Title", year: 2022, provider: "openalex", source_id: null }),
    normalizeResult({ title: " Same  Title ", year: 2022, provider: "openalex", source_id: null })
  ]);

  assert.deepEqual(
    results.map((result) => [result.title, result.provider, result.source_id, result.doi]),
    [
      ["A", "openalex", "W1", "10.4000/Dupe"],
      ["B", "openalex", "W2", null],
      ["Same Title", "openalex", null, null]
    ]
  );
});
