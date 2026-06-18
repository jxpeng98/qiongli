# Finance/Economics Data MCP Boundary

Qiongli's literature MCPB stays focused on scholarly search, provider setup, search plans, and evidence diagnostics. Finance and economics workflows also need live data discovery, but those APIs have different rate limits, result shapes, and citation requirements. Keep them in a separate data MCP surface.

## First Data MCP Surface

Proposed tools:

- `qiongli_finance_data_status`: reports configured data providers and redacted credential status.
- `qiongli_finance_data_search`: searches or resolves finance/economics data series, filings, and source metadata.

Initial sources:

- FRED / ALFRED: macroeconomic and financial time series, releases, vintages, and observations. Use the official FRED API at `https://fred.stlouisfed.org/docs/api/fred/`.
- SEC EDGAR JSON APIs: company submissions, filing metadata, and XBRL company facts. Use the official SEC API boundary at `https://www.sec.gov/search-filings/edgar-application-programming-interfaces`.

## Literature MCPB Finance/Econ Behavior

The literature MCPB supports finance/economics through domain routing, not data retrieval:

- Deep finance/econ searches add working-paper, JEL, and review query variants.
- Search diagnostics report `field_term_coverage`, `working_paper_coverage`, and `published_version_coverage` from the filtered pre-limit result set, so `total_limit` does not hide advisory coverage.
- Working-paper and published-version coverage are advisory diagnostics, not a full version-resolution database.
- Discipline routing lives in reusable domain profiles rather than one-off MCP branching.

## RePEc And NBER Boundary

RePEc and NBER should start as metadata enrichment and version-linking surfaces, not primary search providers:

- RePEc is useful for handles, series metadata, author/institution metadata, and IDEAS/EconPapers-style bibliographic enrichment.
- NBER is useful for working-paper awareness and published-version linking.
- Neither should be treated as an unrestricted full-text source in the literature MCPB.

## Community Expansion Pattern

Future disciplines should add a discipline profile rather than forking MCP logic. In the MCPB server this means extending the domain-profile layer and reusing the shared search fanout and diagnostics path:

1. Domain detector terms and query variants.
2. Domain diagnostics such as field-term coverage or source-type coverage.
3. Optional data MCP sources when the discipline has live data APIs.
4. Metadata enrichment adapters when a source is useful for authority records but not broad search.
