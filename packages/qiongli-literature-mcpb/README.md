# Qiongli Literature Provider MCPB

This package is the Claude Desktop MCPB for Qiongli literature provider access. It contains a zero-dependency Node stdio MCP server, so users do not need to install the `qiongli` CLI or run npm before installing the MCPB.

Pair it with a manual Desktop skill ZIP when you are installing Qiongli without Claude Code or Codex plugin marketplaces. Upload a `qiongli-claude-desktop-skill-*.zip` skill first, then install this MCPB when the same Desktop workspace needs literature MCP tools such as `qiongli_literature_search`, `qiongli_config_status`, `qiongli_configure_provider`, and `qiongli_save_provider_config`.

This MCPB does not launch orchestrator agents. If the Desktop or coding client also needs the full CLI MCP server, local agent runtime, or orchestration tools such as `qiongli_orchestrator_route` and `qiongli_task_run`, install the Python or npm Qiongli CLI and configure the full CLI MCP server separately:

For Codex-style desktop clients, use the Codex plugin bundle when available. For Claude Code, Cursor-style clients, or any client that can launch a local stdio MCP command and needs the full Python-backed tool set, use the unified CLI server:

```bash
qiongli mcp serve --transport stdio
qiongli mcp config example --target codex --json
qiongli mcp config example --target hermes --json
```

The bundled MCPB server and the full CLI MCP server both read the shared provider config. The MCPB also accepts Claude Desktop user configuration values directly from the extension settings.

## Search Precision

`qiongli_literature_search` defaults to broad topic search. For known-item lookup, pass a DOI directly or set `search_mode`:

```json
{ "query": "10.5555/example", "limit": 1 }
{ "query": "Attention Is All You Need", "search_mode": "title", "limit": 1 }
{ "query": "social media mental health", "search_mode": "review", "limit": 150 }
{ "query": "climate governance", "per_provider_limit": 50, "total_limit": 75 }
{ "query": "older adults conversational agents", "query_variants": ["older people chatbots", "home health conversational agents"], "per_provider_limit": 90 }
{ "query": "public health", "document_types": ["journal-article"], "venue_filter": "Lancet" }
```

DOI queries use provider singleton lookup where available. Title mode asks Semantic Scholar for a title match before regular search, requests a wider provider page, then ranks merged results by title similarity before applying the final limit.

For general topic searches, omitted limits default to 25 results per provider. For literature reviews, use `search_mode: "review"` or `search_mode: "systematic_review"`. Review mode defaults to 50 results per provider when `limit` is omitted and accepts explicit limits up to 200 per provider.

`limit` remains the backward-compatible per-provider limit for topic and review searches. Use `per_provider_limit` when you want that intent to be explicit, and use `total_limit` to cap the merged, deduplicated result list returned to the MCP client. Both snake_case and camelCase aliases are accepted.

Advanced controls include:

- `search_depth`: `quick`, `standard`, `review`, or `deep`. Review and deep searches return `insufficient_review_results` when the merged result set is below the review threshold.
- `search_depth: "deep"` defaults to 200 results per provider, uses provider pagination instead of stopping at the first provider page, and automatically searches the primary query plus conservative review and systematic-review variants.
- Finance/economics deep searches use a domain profile for field-aware variants around working papers, JEL terms, and reviews. Search diagnostics include `field_term_coverage`, `working_paper_coverage`, and `published_version_coverage`; coverage is computed before `total_limit` truncates returned results.
- `query_variants`: adds explicit alternate queries to the same call. The MCPB splits the per-provider budget across the primary query and variants, returns the auditable `search_plan`, and records each query/provider attempt in `diagnostics.queries`. Pass an empty array to disable automatic deep-search variants.
- `document_types`: filters OpenAlex and Crossref at request time and filters merged provider results after normalization. Semantic Scholar publication types are normalized from `publicationTypes`, and PubMed publication types are normalized from ESummary.
- `venue_filter`: filters merged results by venue text.
- `include_citations` and `include_references`: request limited citation/reference metadata when providers expose it. The MCPB reports `citation_expansion_limited` or `reference_expansion_limited` because this is metadata expansion, not a full citation graph crawler.

Search responses include `search_plan` and `diagnostics` with raw, deduplicated, filtered, coverage-basis, and returned result counts plus per-provider and per-query status, result count, request count, retry attempts, and sanitized error messages. Search and status responses also include `provider_capabilities`, which marks OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv as implemented providers. `qiongli_literature_export_evidence` returns the same search plan, options, diagnostics, capabilities, warnings, result count, and normalized result snapshot for audit handoff. Crossref needs `crossref.email` for polite access. PubMed needs `pubmed.api_key` to enable the bundled E-Utilities provider. arXiv is enabled without credentials.

## Local Zotero Reference Database

The MCPB can use Zotero Desktop as a local reference database through the
Qiongli Zotero companion extension in `packages/qiongli-zotero-companion/`. This
does not replace OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv as discovery
providers. Qiongli still searches and enriches references, then selected records
can be dry-run or written into local Zotero.

Available tools:

- `qiongli_zotero_status`: checks Zotero Desktop's local connector, the Qiongli
  Zotero companion, and import-file fallback availability.
- `qiongli_zotero_search`: searches the local Zotero library through the
  companion by DOI, title, creator, year, tag, or collection path.
- `qiongli_zotero_upsert_references`: maps normalized Qiongli references to
  Zotero items, deduplicates by DOI or title/year, defaults to dry run, and
  writes only when `dry_run: false` is explicit.
- `qiongli_zotero_export_import_files`: generates `references.json`,
  `references.ris`, `bibliography.bib`, and `zotero-import-report.md` without
  contacting Zotero.

### Opt-in Zotero source search

`qiongli_literature_search` does not search Zotero by default. Pass
`include_zotero: true` to include the local Zotero library as an additional
reference source. Local-only records return `provider: "zotero"` and external
records can include `local_zotero_match` when the DOI or title/year already
exists in Zotero.

### Crossref verification before Zotero writes

DOI-bearing imports use Crossref DOI registry metadata by default to fill blank
fields before writing to Zotero. Crossref metadata is not human verification, so
new or updated items still receive `qiongli:needs-review`. Conflicts between
incoming metadata and Crossref registry metadata add `qiongli:metadata-conflict`
and are returned in `verification.crossref.conflicts`.

Local mode uses the loopback connector URL `http://127.0.0.1:23119` by default.
Non-loopback connector URLs are rejected. If Zotero Desktop or the companion is
not available, use the generated import files for manual Zotero import.

## Local Claude Desktop Install

Build or package this directory as a Claude Desktop `.mcpb` extension, then install it through Claude Desktop's extension settings. The manifest declares user configuration fields for:

- OpenAlex API key
- OpenAlex email
- Semantic Scholar API key
- Crossref polite access email
- NCBI / PubMed API key
- Default result limit

Claude Desktop injects these values into the local Node MCP server environment when the extension runs. The server can also open a local setup page through `qiongli_configure_provider` or save explicit provider values through `qiongli_save_provider_config` into the shared local provider config. `qiongli_open_config_wizard` remains as a compatibility alias for older instructions. Do not store provider credentials in the Qiongli Desktop skill ZIP or commit local secrets into this package.

## Development

Run the syntax check:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Start the stdio server:

```bash
npm --prefix packages/qiongli-literature-mcpb start
```
