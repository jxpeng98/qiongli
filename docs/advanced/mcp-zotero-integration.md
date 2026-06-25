# Zotero Integration: Local Reference Database

Qiongli treats Zotero as a local reference database, not as a replacement for
OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv discovery. The normal workflow is
to search and enrich references through Qiongli providers, then save selected
records into Zotero Desktop through the Qiongli Zotero companion.

This local-first path does not require a Zotero Web API key or Zotero cloud sync.
If local Zotero is unavailable, Qiongli still generates import files:
`references.json`, `references.ris`, and `bibliography.bib`.

## Components

| Component | Role |
| --- | --- |
| Qiongli literature MCPB | Normalizes records, maps metadata, deduplicates, exposes Zotero tools, and generates import files. |
| Qiongli Zotero companion | A thin Zotero Desktop plugin that registers `/qiongli/*` local connector endpoints. |
| Zotero Desktop | Stores the local reference library, collections, tags, and user-curated metadata. |

The companion lives in `packages/qiongli-zotero-companion/`. It is a companion
plugin, not a standalone MCP server.

Direct local writes require this Qiongli companion plugin to be installed in
Zotero Desktop. No third-party Zotero plugin is required. Without the companion,
Qiongli still works in import-file mode.

Build the installable extension from the repository root:

```bash
python3 scripts/build_zotero_companion.py --dist-dir dist
```

Install the generated `qiongli-zotero-companion-*.xpi` in Zotero Desktop's
add-on manager, then restart Zotero.

## Local Status Check

Run:

```json
{ "tool": "qiongli_zotero_status", "arguments": {} }
```

The tool checks:

1. Zotero Desktop's connector server at `http://127.0.0.1:23119/connector/ping`.
2. The Qiongli companion endpoint at `http://127.0.0.1:23119/qiongli/ping`.
3. Import-file fallback availability.

Possible states:

- `ok`: Zotero Desktop and the Qiongli Zotero companion are available.
- `companion_missing`: Zotero Desktop is running, but the companion plugin is not installed or not loaded.
- `fallback_only`: Zotero Desktop is not reachable; use generated import files.
- `disabled`: local Zotero mode is disabled in config.

## Opt-In Local Source Search

`qiongli_literature_search` does not search Zotero by default. Add
`include_zotero: true` when you want Zotero to act as an additional local
reference source:

```json
{
  "tool": "qiongli_literature_search",
  "arguments": {
    "query": "platform governance",
    "include_zotero": true,
    "zotero_tag": "project:platform-governance"
  }
}
```

Local-only Zotero records return `provider: "zotero"` and
`source_type: "local_reference_database"`. External provider records can include
`local_zotero_match` when the DOI or title/year already exists in Zotero.

## Saving Search Results

Search first:

```json
{
  "tool": "qiongli_literature_search",
  "arguments": {
    "query": "platform governance systematic review",
    "search_mode": "review",
    "per_provider_limit": 50
  }
}
```

Then dry-run a Zotero write:

```json
{
  "tool": "qiongli_zotero_upsert_references",
  "arguments": {
    "records": [
      {
        "title": "Platform Governance in Practice",
        "authors": ["Smith, Alex"],
        "year": 2024,
        "doi": "10.1000/platform-governance",
        "venue": "Organization Science",
        "provider": "openalex",
        "source_id": "W123"
      }
    ],
    "collection_path": "Qiongli/platform-governance/To Screen",
    "tags": ["project:platform-governance", "status:to-screen"]
  }
}
```

Dry run is the default. To write, set `dry_run: false` explicitly. The bridge
matches existing Zotero items by DOI first, then title/year fallback. By default
it fills blank Zotero fields, adds identifiers, tags, and collection membership,
and avoids overwriting user-curated title, authors, date, publication title, or
abstract.

DOI-bearing writes use Crossref registry metadata by default before the Zotero
payload is sent. Crossref verification fills blank fields only; it does not
replace human review. New or updated candidates receive `qiongli:imported` and
`qiongli:needs-review`. Records verified through Crossref receive
`qiongli:crossref-verified`; material title or year conflicts receive
`qiongli:metadata-conflict` and expose details under
`verification.crossref.conflicts`.

## Import File Fallback

When the companion is unavailable, generate files:

```json
{
  "tool": "qiongli_zotero_export_import_files",
  "arguments": {
    "records": [
      {
        "title": "Fallback Paper",
        "authors": ["Smith, Alex"],
        "year": 2024,
        "doi": "10.1000/fallback"
      }
    ]
  }
}
```

The output includes:

- `references.json` for Zotero CSL-JSON import.
- `references.ris` for Zotero, EndNote, and Mendeley.
- `bibliography.bib` for BibTeX workflows.
- `zotero-import-report.md` with counts, Crossref verification summary, and
  fallback instructions.

## Configuration

Local mode uses loopback-only connector URLs.

```bash
QIONGLI_ZOTERO_LOCAL_ENABLED=true
QIONGLI_ZOTERO_CONNECTOR_URL=http://127.0.0.1:23119
QIONGLI_ZOTERO_WRITE_POLICY=explicit
QIONGLI_ZOTERO_UPDATE_POLICY=fill_blank
QIONGLI_ZOTERO_DEFAULT_COLLECTION_PATH="Qiongli/[topic]/To Screen"
QIONGLI_ZOTERO_DEFAULT_REVIEW_TAGS="qiongli:imported,qiongli:needs-review"
QIONGLI_ZOTERO_CROSSREF_VERIFICATION_ENABLED=true
```

`QIONGLI_ZOTERO_CONNECTOR_URL` must point to `127.0.0.1`, `localhost`, or `::1`.
Non-loopback URLs are rejected.

## Web API Mode

Zotero Web API supports writes with an API key that has write access. That mode
is useful for future cloud-sync workflows, but it is not the default Qiongli
integration path. The default path is local Zotero Desktop plus the Qiongli
Zotero companion, with import-file fallback when local write is unavailable.
