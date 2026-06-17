import test from "node:test";
import assert from "node:assert/strict";
import { searchCrossref } from "../server/providers/crossref.mjs";
import { searchOpenAlex } from "../server/providers/openalex.mjs";
import { searchPubMed } from "../server/providers/pubmed.mjs";
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
              type: "journal-article",
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
    documentTypes: ["journal-article"],
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
  assert.equal(requestedUrl.searchParams.get("filter"), "from_publication_date:2020-01-01,to_publication_date:2024-12-31,type:journal-article");
  assert.deepEqual(response.results, [
    {
      title: "OpenAlex Paper",
      authors: ["Ada Lovelace", "Grace Hopper"],
      year: 2024,
      doi: "10.1000/Example",
      url: "https://example.test/paper",
      abstract: "Results are normalized",
      venue: "Journal of Tests",
      document_type: "journal-article",
      citation_count: null,
      reference_count: null,
      citations: [],
      references: [],
      provider: "openalex",
      source_id: "W123"
    }
  ]);
});

test("searchOpenAlex resolves DOI queries through the singleton work endpoint", async () => {
  let requestedUrl;
  const fetchImpl = async (url) => {
    requestedUrl = new URL(url);
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          id: "https://openalex.org/W999",
          doi: "https://doi.org/10.5555/Exact",
          title: "Exact DOI Paper",
          publication_year: 2022
        };
      }
    };
  };

  const response = await searchOpenAlex({
    query: "https://doi.org/10.5555/Exact",
    limit: 7,
    apiKey: "openalex-secret-key",
    fetchImpl
  });

  assert.equal(requestedUrl.origin, "https://api.openalex.org");
  assert.equal(decodeURIComponent(requestedUrl.pathname), "/works/doi:10.5555/Exact");
  assert.equal(requestedUrl.searchParams.has("search"), false);
  assert.equal(requestedUrl.searchParams.has("per-page"), false);
  assert.equal(requestedUrl.searchParams.get("api_key"), "openalex-secret-key");
  assert.deepEqual(response.results.map((result) => result.title), ["Exact DOI Paper"]);
  assert.equal(response.results[0].doi, "10.5555/Exact");
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
              publicationTypes: ["JournalArticle"],
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
  assert.equal(calls[0].url.searchParams.get("fields"), "paperId,title,year,authors,abstract,url,venue,publicationTypes,externalIds");
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
      document_type: "JournalArticle",
      citation_count: null,
      reference_count: null,
      citations: [],
      references: [],
      provider: "semantic_scholar",
      source_id: "abc123"
    }
  ]);
});

test("searchSemanticScholar runs title-match before regular search in title mode", async () => {
  const calls = [];
  const fetchImpl = async (url, options = {}) => {
    const requestedUrl = new URL(url);
    calls.push({ url: requestedUrl, options });

    if (requestedUrl.pathname.endsWith("/paper/search/match")) {
      return {
        ok: true,
        status: 200,
        async json() {
          return {
            data: [
              {
                paperId: "exact123",
                title: "Attention Is All You Need",
                year: 2017,
                authors: [{ name: "Ashish Vaswani" }],
                externalIds: {
                  DOI: "10.5555/title-match"
                }
              }
            ]
          };
        }
      };
    }

    return {
      ok: true,
      status: 200,
      async json() {
        return {
          data: [
            {
              paperId: "broad456",
              title: "Attention Mechanisms for Search",
              year: 2018,
              authors: [{ name: "Example Author" }]
            }
          ]
        };
      }
    };
  };

  const response = await searchSemanticScholar({
    query: "Attention Is All You Need",
    searchMode: "title",
    limit: 2,
    apiKey: "secret-api-key",
    fetchImpl
  });

  assert.equal(calls.length, 2);
  assert.equal(calls[0].url.pathname, "/graph/v1/paper/search/match");
  assert.equal(calls[0].url.searchParams.get("query"), "Attention Is All You Need");
  assert.equal(calls[0].url.searchParams.get("fields"), "paperId,title,year,authors,abstract,url,venue,publicationTypes,externalIds");
  assert.equal(calls[0].options.headers["x-api-key"], "secret-api-key");
  assert.equal(calls[1].url.pathname, "/graph/v1/paper/search");
  assert.deepEqual(
    response.results.map((result) => result.title),
    ["Attention Is All You Need", "Attention Mechanisms for Search"]
  );
});

test("searchSemanticScholar requests citation and reference fields for expansion", async () => {
  let requestedUrl;
  const fetchImpl = async (url, options = {}) => {
    requestedUrl = new URL(url);
    assert.equal(options.headers["x-api-key"], "secret-api-key");
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          data: [
            {
              paperId: "seed123",
              title: "Seed Paper",
              year: 2024,
              citationCount: 11,
              referenceCount: 22,
              citations: [
                {
                  paperId: "cited-by-1",
                  title: "Citing Paper",
                  year: 2025,
                  authors: [{ name: "Citing Author" }],
                  externalIds: {
                    DOI: "10.6000/citing"
                  },
                  url: "https://semanticscholar.org/paper/cited-by-1"
                }
              ],
              references: [
                {
                  paperId: "ref-1",
                  title: "Referenced Semantic Paper",
                  year: 2020,
                  authors: [{ name: "Reference Author" }],
                  externalIds: {
                    DOI: "10.6000/reference"
                  },
                  url: "https://semanticscholar.org/paper/ref-1"
                }
              ]
            }
          ]
        };
      }
    };
  };

  const response = await searchSemanticScholar({
    query: "seed query",
    limit: 1,
    apiKey: "secret-api-key",
    includeCitations: true,
    includeReferences: true,
    fetchImpl
  });

  const fields = requestedUrl.searchParams.get("fields");
  assert.equal(fields.includes("citationCount"), true);
  assert.equal(fields.includes("referenceCount"), true);
  assert.equal(fields.includes("citations.paperId"), true);
  assert.equal(fields.includes("references.paperId"), true);
  assert.deepEqual(response.results[0].citations, [
    {
      title: "Citing Paper",
      authors: ["Citing Author"],
      year: 2025,
      doi: "10.6000/citing",
      url: "https://semanticscholar.org/paper/cited-by-1",
      provider: "semantic_scholar",
      source_id: "cited-by-1"
    }
  ]);
  assert.deepEqual(response.results[0].references, [
    {
      title: "Referenced Semantic Paper",
      authors: ["Reference Author"],
      year: 2020,
      doi: "10.6000/reference",
      url: "https://semanticscholar.org/paper/ref-1",
      provider: "semantic_scholar",
      source_id: "ref-1"
    }
  ]);
  assert.equal(response.results[0].citation_count, 11);
  assert.equal(response.results[0].reference_count, 22);
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

test("searchCrossref sends polite email and normalizes bibliographic results", async () => {
  let requestedUrl;
  const fetchImpl = async (url) => {
    requestedUrl = new URL(url);
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          message: {
            items: [
              {
                DOI: "10.4000/Crossref",
                title: ["Crossref Paper"],
                author: [
                  {
                    given: "Ada",
                    family: "Lovelace"
                  }
                ],
                "published-print": {
                  "date-parts": [[2024, 1, 2]]
                },
                URL: "https://doi.org/10.4000/Crossref",
                abstract: "Crossref abstract",
                "container-title": ["Journal of Crossref Tests"],
                type: "journal-article",
                "is-referenced-by-count": 7,
                "reference-count": 1,
                reference: [
                  {
                    DOI: "10.4000/Reference",
                    "article-title": "Referenced Paper",
                    author: "Grace Hopper",
                    year: "2020"
                  }
                ]
              }
            ]
          }
        };
      }
    };
  };

  const response = await searchCrossref({
    query: "crossref query",
    limit: 5,
    email: "crossref@example.com",
    fromYear: 2020,
    toYear: 2024,
    documentTypes: ["journal-article"],
    fetchImpl
  });

  assert.equal(response.provider, "crossref");
  assert.equal(response.error, null);
  assert.equal(requestedUrl.origin, "https://api.crossref.org");
  assert.equal(requestedUrl.pathname, "/works");
  assert.equal(requestedUrl.searchParams.get("query.bibliographic"), "crossref query");
  assert.equal(requestedUrl.searchParams.get("rows"), "5");
  assert.equal(requestedUrl.searchParams.get("mailto"), "crossref@example.com");
  assert.equal(requestedUrl.searchParams.get("filter"), "from-pub-date:2020-01-01,until-pub-date:2024-12-31,type:journal-article");
  assert.deepEqual(response.results, [
    {
      title: "Crossref Paper",
      authors: ["Ada Lovelace"],
      year: 2024,
      doi: "10.4000/Crossref",
      url: "https://doi.org/10.4000/Crossref",
      abstract: "Crossref abstract",
      venue: "Journal of Crossref Tests",
      document_type: "journal-article",
      citation_count: 7,
      reference_count: 1,
      citations: [],
      references: [
        {
          title: "Referenced Paper",
          authors: ["Grace Hopper"],
          year: 2020,
          doi: "10.4000/Reference",
          url: null,
          provider: "crossref",
          source_id: null
        }
      ],
      provider: "crossref",
      source_id: "10.4000/Crossref"
    }
  ]);
});

test("searchCrossref resolves DOI queries through singleton work endpoint", async () => {
  let requestedUrl;
  const fetchImpl = async (url) => {
    requestedUrl = new URL(url);
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          message: {
            DOI: "10.4000/Exact",
            title: ["Exact Crossref Paper"],
            issued: {
              "date-parts": [[2022]]
            }
          }
        };
      }
    };
  };

  const response = await searchCrossref({
    query: "https://doi.org/10.4000/Exact",
    limit: 5,
    email: "crossref@example.com",
    fetchImpl
  });

  assert.equal(requestedUrl.origin, "https://api.crossref.org");
  assert.equal(decodeURIComponent(requestedUrl.pathname), "/works/10.4000/Exact");
  assert.equal(requestedUrl.searchParams.has("query.bibliographic"), false);
  assert.equal(requestedUrl.searchParams.has("rows"), false);
  assert.deepEqual(response.results.map((result) => result.title), ["Exact Crossref Paper"]);
});

test("searchPubMed uses ESearch and ESummary and normalizes records", async () => {
  const calls = [];
  const fetchImpl = async (url) => {
    const requestedUrl = new URL(url);
    calls.push(requestedUrl);

    if (requestedUrl.pathname.endsWith("/esearch.fcgi")) {
      return {
        ok: true,
        status: 200,
        async json() {
          return {
            esearchresult: {
              idlist: ["12345"]
            }
          };
        }
      };
    }

    assert.equal(requestedUrl.pathname.endsWith("/esummary.fcgi"), true);
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          result: {
            uids: ["12345"],
            12345: {
              uid: "12345",
              title: "PubMed Paper",
              pubdate: "2024 Jan",
              fulljournalname: "Journal of PubMed Tests",
              authors: [{ name: "Katherine Johnson" }],
              articleids: [
                {
                  idtype: "doi",
                  value: "10.5000/PubMed"
                }
              ],
              pubtype: ["Journal Article"]
            }
          }
        };
      }
    };
  };

  const response = await searchPubMed({
    query: "pubmed query",
    limit: 4,
    apiKey: "pubmed-secret-key",
    fromYear: 2021,
    toYear: 2024,
    fetchImpl
  });

  assert.equal(response.provider, "pubmed");
  assert.equal(response.error, null);
  assert.equal(calls[0].origin, "https://eutils.ncbi.nlm.nih.gov");
  assert.equal(calls[0].pathname, "/entrez/eutils/esearch.fcgi");
  assert.equal(calls[0].searchParams.get("db"), "pubmed");
  assert.equal(calls[0].searchParams.get("term"), "pubmed query");
  assert.equal(calls[0].searchParams.get("retmax"), "4");
  assert.equal(calls[0].searchParams.get("mindate"), "2021");
  assert.equal(calls[0].searchParams.get("maxdate"), "2024");
  assert.equal(calls[0].searchParams.get("datetype"), "pdat");
  assert.equal(calls[0].searchParams.get("api_key"), "pubmed-secret-key");
  assert.equal(calls[1].pathname, "/entrez/eutils/esummary.fcgi");
  assert.equal(calls[1].searchParams.get("id"), "12345");
  assert.deepEqual(response.results, [
    {
      title: "PubMed Paper",
      authors: ["Katherine Johnson"],
      year: 2024,
      doi: "10.5000/PubMed",
      url: "https://pubmed.ncbi.nlm.nih.gov/12345/",
      abstract: null,
      venue: "Journal of PubMed Tests",
      document_type: "Journal Article",
      citation_count: null,
      reference_count: null,
      citations: [],
      references: [],
      provider: "pubmed",
      source_id: "12345"
    }
  ]);
});

test("searchPubMed translates DOI queries into PubMed DOI field terms", async () => {
  let requestedUrl;
  const fetchImpl = async (url) => {
    requestedUrl = new URL(url);
    return {
      ok: true,
      status: 200,
      async json() {
        if (requestedUrl.pathname.endsWith("/esearch.fcgi")) {
          return {
            esearchresult: {
              idlist: []
            }
          };
        }
        return { result: { uids: [] } };
      }
    };
  };

  const response = await searchPubMed({
    query: "https://doi.org/10.5000/Exact",
    limit: 1,
    apiKey: "pubmed-secret-key",
    fetchImpl
  });

  assert.equal(requestedUrl.pathname, "/entrez/eutils/esearch.fcgi");
  assert.equal(requestedUrl.searchParams.get("term"), "10.5000/Exact[doi]");
  assert.deepEqual(response.results, []);
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
      document_type: " ",
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
      document_type: null,
      citation_count: null,
      reference_count: null,
      citations: [],
      references: [],
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
