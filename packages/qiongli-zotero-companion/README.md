# Qiongli Zotero Companion

The Qiongli Zotero Companion is a thin Zotero Desktop extension that lets
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

Build the user-installable Zotero extension from the repository root:

```bash
python3 scripts/build_zotero_companion.py --dist-dir dist
```

The generated `qiongli-zotero-companion-*.xpi` targets the bootstrapped Zotero
Desktop plugin model used by Zotero 8 through Zotero 9.0.x and has been tested
with Zotero 9.0.4. It can be installed through Zotero Desktop's add-on manager.
The manifest includes the Zotero `update_url` metadata required by Zotero's
add-on manager. Direct local writes require this companion extension; without
it, Qiongli still generates import files that can be imported manually.
