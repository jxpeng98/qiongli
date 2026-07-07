# Qiongli Zotero Companion

The Qiongli Zotero Companion is a thin Zotero Desktop extension that lets
Qiongli treat Zotero as a local reference database. It is not a standalone MCP
server and it does not replace OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv
discovery.

The companion registers Qiongli-specific endpoints on Zotero Desktop's local
connector server:

- `GET /qiongli/ping`
- `POST /qiongli/search`
- `POST /qiongli/upsertItems`
- `GET /qiongli/collections`

## Attachment Metadata

`POST /qiongli/search` returns compact Zotero item records plus attachment
summaries when local attachments exist. Attachment summaries include the Zotero
attachment key, file name, MIME type, link mode, Zotero select URI, URL, and
`local_file_available`.

Raw local file paths are omitted by default. Send `include_attachment_paths:
true` only when a local resolver explicitly needs paths for controlled full-text
retrieval. Qiongli treats Zotero attachment data as local verification evidence;
provider or native search URLs remain candidate-only until `retrieval_manifest.csv`
records a retrieved or unresolved status.

## Project Collections

`POST /qiongli/upsertItems` accepts `collection_path` values such as
`Qiongli/platform-governance`. During dry runs, the companion reports the target
collection path without mutating Zotero. During explicit writes, it creates any
missing nested collections and adds created, updated, or unchanged duplicate
items to the target collection.

The companion does not infer project names from chat context. Qiongli's MCPB
resolves that context first, either from an explicit collection path or from
project-title fields, and sends one collection path to the companion.

## Reading Notes

`POST /qiongli/upsertItems` also accepts per-item `qiongli_notes`. During dry
runs, the companion reports planned note writes. During explicit writes, each
note is created as a Zotero child note under the reference item.

Qiongli reading notes are not written into `abstractNote` or `extra`; those
fields remain reserved for paper abstracts and compact provenance metadata.

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
