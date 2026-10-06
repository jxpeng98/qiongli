# Use Zotero as your local reference library

Qiongli searches online literature services for candidate references, while Zotero
keeps your local library. The Qiongli Zotero Companion connects the native MCP
tools to a running Zotero Desktop over loopback. This local path needs neither a
Zotero Web API key nor cloud sync.

## Install the Companion in Zotero

Install `qiongli-zotero-companion-*.xpi` from the matching
[Qiongli release](https://github.com/jxpeng98/qiongli/releases) through Zotero's
add-on manager, then restart Zotero. Companion 0.3.1 declares support for Zotero 8
through 10.0.x, including 10.0.2. Installing a Qiongli Host Plugin does not install
an extension inside Zotero.

The Companion source is `packages/qiongli-zotero-companion/`. Maintainers can build
its XPI with `python3 scripts/build_zotero_companion.py --dist-dir dist`; ordinary
users can use the release asset without a source checkout.

## Check and search

Both native Lite and Full expose `qiongli_zotero_status`,
`qiongli_zotero_search`, `qiongli_zotero_upsert_references` and
`qiongli_zotero_export_import_files`. Search and direct writes need the Companion;
exporting import-file contents does not.

First ask the Host to call `qiongli_zotero_status`. If the Companion is unavailable,
check that Zotero is open and the extension is enabled. A running Connector alone
does not prove that the Companion is ready. Then search an authorized part of your
library, for example:

```json
{"tool":"qiongli_zotero_search","arguments":{"title":"platform governance","limit":10}}
```

The native search accepts criteria such as `doi`, `title`, `year`, `citekey`,
`creator`, `tag` or `collection_path`. At least one criterion is required.
Use this tool for local references; the native literature-search API does not
accept the old `include_zotero` switch.

## Preview before saving

Pass the selected reference objects as `items` to
`qiongli_zotero_upsert_references`. Its default `dry_run: true` previews the change;
`update_policy: "fill_blank"` preserves existing populated fields. Review the
specific items, collection and changes before authorizing a write.

To apply that exact preview, call the tool again with the same planned content,
`dry_run: false`, `write_intent: "apply"` and the returned `dry_run_receipt`.
A changed or expired plan needs a fresh preview. Do not substitute a fabricated
receipt or treat a successful search as permission to save.

If direct access is unavailable, `qiongli_zotero_export_import_files` accepts
`records` and returns import-file contents: `references.json`, `references.ris`,
`bibliography.bib` and `zotero-import-report.md`. Returning those contents does not
save them to disk or import them into Zotero; save approved files and import them
through Zotero separately.

The native CLI accepts `QIONGLI_ZOTERO_CONNECTOR_URL` (normally `http://127.0.0.1:23119`). Remote Connector
URLs are rejected. This guide does not promise a native Zotero Web API write path.
