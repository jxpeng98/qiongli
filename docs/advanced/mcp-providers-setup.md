# Set up literature providers

The Plugin's MCP connection and an online literature account are separate steps.
Once the Host exposes Qiongli tools, call `qiongli_config_status` and
`qiongli_literature_status` to see which providers are configured and available.
Installing another MCP package is unnecessary.

## Configure a provider

The unified entry below belongs to the native source increment after 2.5.3. Check
`qiongli provider --help` before using it: the tagged 2.5.3 build has no fresh-key CLI
entry and its native MCP configuration tools return unavailable. Updating the CLI
and refreshing the Plugin are separate steps because each carries an executable.

```sh
qiongli provider setup
qiongli provider setup openalex
qiongli provider setup pubmed --no-open
qiongli provider status
qiongli provider test openalex --confirm-network-request
```

Setup opens a tokenized local page. With `--no-open`, use the displayed link while
keeping the terminal process running. Select one service, enter its fields, review
its enablement and check the confirmation box before saving. Blank fields keep
stored values; removal is explicit. Removing a key disables that service. The
link expires after ten minutes or one successful save. Setup itself does not
contact literature services. `status` and its `doctor` alias are offline;
`test` requires the explicit network flag and returns bounded connectivity
information, including successful requests with zero matches, without paper data.

In a Host, ask for `qiongli_configure_provider` with the desired provider, then
open its returned local link. `qiongli_open_config_wizard` is an alias using the
same page. For arXiv, omit the provider argument and select it on the page;
the existing MCP provider argument enum remains unchanged. Native MCP never opens
a browser. Enter keys only on the page; native
`qiongli_save_provider_config` returns setup-required guidance without writing
raw values from the conversation. Status refreshes in the same native MCP session
after a save or another settings writer increments the configuration revision.
This entry serves the CLI and its Lite/Full MCP profiles, including the bundled
Full Plugin. The separate standalone Lite package still reports setup unavailable;
check the actual runtime response before expecting a setup page.

On macOS, keys use the existing Keychain adapter. Ordinary
`~/.config/qiongli/v2/settings.json` stores opaque references, service switches and
public contact fields. `QIONGLI_CONFIG_HOME` changes the shared configuration root;
Hosts using the same user/root read the same provider settings. Windows/Linux do
not yet have a native credential adapter: secret writes fail clearly, while
Crossref contact settings and arXiv enablement remain configurable. No plaintext
file or API-key environment-variable fallback is enabled. A committed save with
cleanup-required is shown as such; retain that warning instead of assuming that
previous credential cleanup completed. Model credentials remain Host-owned.

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
