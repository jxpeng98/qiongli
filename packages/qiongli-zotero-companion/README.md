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

Companion `0.3.0` implements endpoint contract `2`. Qiongli clients must probe
`GET /qiongli/ping` and reject any other endpoint version before search or
write operations. An older live Companion is an update-required state, not a
successful connection.

## Search Contract

`POST /qiongli/search` accepts bounded DOI, title, citekey, creator, year, tag,
and collection-path filters. It returns at most 200 compact items with
creators, tags, collection paths, note summaries, and attachment summaries.
Local attachment paths remain omitted unless a controlled local resolver
explicitly requests them.

## Write Approval

Every `POST /qiongli/upsertItems` mutation uses a two-step, plan-bound approval:

1. Send the exact payload with `dry_run: true` and review the returned plan.
2. Within five minutes, resend the unchanged payload with `dry_run: false`,
   `write_intent: "apply"`, and the returned `write_approval.receipt`.

Receipts are one-shot, expire after five minutes, and are invalid if the item,
collection, note, tag, or update plan changes. A changed or expired plan must
be dry-run again. A direct write request never mutates Zotero and returns
`approval_required`. Batches are limited to 100 items.

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

The builder emits both `qiongli-zotero-companion-*.xpi` and
`qiongli-zotero-companion.manifest.json`. The XPI uses a fixed entry order,
timestamp, compression mode, and file metadata, while the canonical artifact
manifest binds its version, supported Zotero range, endpoint version, size,
SHA-256, and complete source inventory. Rebuilding unchanged sources therefore
produces identical bytes.

Release automation also supplies the immutable Qiongli release identity:

```bash
python3 scripts/build_zotero_companion.py \
  --dist-dir dist \
  --release-tag v2.0.0-alpha.2 \
  --repo jxpeng98/qiongli
```

That explicit release form additionally emits
`qiongli-zotero-companion-updates.json`, which binds the Companion version and
supported Zotero range to the versioned release XPI URL and its SHA-256. It is
uploaded beside the XPI. Zotero reads the public manifest through the
`releases/latest` URL, so automatic updates intentionally follow the latest
stable Qiongli release; prerelease Companion updates remain available through
Qiongli's bundled install handoff until stable advances.

The generated extension targets the bootstrapped Zotero Desktop plugin model
used by Zotero 8 through Zotero 9.0.x and has been tested with Zotero 9.0.4. It
can be installed through Zotero Desktop's add-on manager. The extension
manifest includes the Zotero `update_url` metadata required by Zotero's add-on
manager and the release job publishes the referenced Mozilla-style JSON
update manifest. Direct local writes require this companion extension; without it,
Qiongli still generates import files that can be imported manually.

Qiongli desktop packages contain the same verified XPI and artifact manifest.
The App may stage and reveal that product-controlled artifact, but Zotero owns
the installation confirmation and profile mutation. The presence of an XPI or
a Qiongli staging receipt is never treated as activation evidence.
