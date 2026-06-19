# Qiongli Zotero Companion

The Qiongli Zotero companion is a thin Zotero Desktop extension that lets
Qiongli treat Zotero as a local reference database. It is not a standalone MCP
server and it does not replace OpenAlex, Semantic Scholar, Crossref, or PubMed
discovery.

The companion registers Qiongli-specific endpoints on Zotero Desktop's local
connector server:

- `GET /qiongli/ping`
- `POST /qiongli/search`
- `POST /qiongli/upsertItems`
- `GET /qiongli/collections`

Qiongli's MCPB probes these endpoints through `qiongli_zotero_status`, performs
dry-run writes through `qiongli_zotero_upsert_references`, and falls back to
`references.json`, `references.ris`, and `bibliography.bib` when the companion is
not installed or Zotero is not running.

This package currently contains the endpoint contract and testable bridge helper
logic. Packaging as a user-installable `.xpi` is a follow-up release step.
