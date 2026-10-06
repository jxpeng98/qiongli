# Set up literature providers

The Plugin's MCP connection and an online literature account are separate steps.
Once the Host exposes Qiongli tools, call `qiongli_config_status` and
`qiongli_literature_status` to see which providers are configured and available.
Installing another MCP package is unnecessary.

## Configure a provider

Ask the Host to call `qiongli_configure_provider` for the provider you need.
Open the returned local setup URL and enter the credential there, keeping keys
out of the conversation. `qiongli_open_config_wizard` is a compatibility alias.
`qiongli_save_provider_config` also supports an explicitly requested field write;
its secret value is not returned in the result.

| Provider | Fields used by Qiongli 2 |
|---|---|
| OpenAlex (`openalex`) | `api_key` to activate; optional `email` |
| Semantic Scholar (`semantic_scholar`) | `api_key` to activate |
| Crossref (`crossref`) | `email` to activate |
| PubMed (`pubmed`) | `api_key` to activate |
| arXiv (`arxiv`) | No credential; available unless disabled |

These are Qiongli's configuration requirements, not a statement that each service
requires credentials for every possible API request. Lite and Full share the
provider contract in `content/mcp-contracts/provider-config.schema.json`.
After saving, check `qiongli_literature_status` again. Configured credentials do
not prove that a request will succeed; search results report service failures
and partial results separately.

## Plan and run a search

`qiongli_search_plan` helps the active agent choose between configured providers,
the Host's native search and supplied sources. Its `search_execution_mode`
can describe `provider_connected`, `native_only`, `hybrid_search` or
`strategy_only`, depending on the tools and inputs supplied to the plan.
The separate `provider_capability_mode` is either `provider_connected` or
`strategy_only`. `native_search_queries` are suggestions for the Host, not completed searches.

Use `qiongli_literature_search` for a bounded provider search. MCP servers do not
invoke Host-native search; the active agent uses its own available tools for that
part. If neither route is available, keep the search strategy and report the gap.
An empty provider result is not proof that no relevant papers exist.

For local references, use the separate [Zotero tools](mcp-zotero-integration.md).
For review methods and coverage limits, see [rigorous literature search](rigorous-literature-search.md).
The Python `RESEARCH_MCP_<PROVIDER>_CMD` adapter configuration belongs to 1.x and
is not the setup path for this native runtime.
