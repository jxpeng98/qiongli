# Local Zotero Bridge Design

## Goal

Make Zotero a local-first reference database for Qiongli literature workflows.
Qiongli should keep doing search, deduplication, metadata enrichment, and audit
export, but it should also be able to read from and write to a user's local
Zotero Desktop library. When local Zotero is unavailable, Qiongli must still
produce Zotero-compatible import files.

## Non-Goals

- Do not make Zotero replace OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv as
  Qiongli's discovery layer.
- Do not require Zotero cloud sync or Zotero Web API write access for the default
  flow.
- Do not depend on a community Zotero MCP server as the core implementation.
- Do not implement full attachment download or PDF storage in the first slice.
- Do not make Qiongli edit arbitrary Zotero items without explicit user action.

## External API Boundary

Zotero Web API supports write requests with an API key that has write access, and
it supports item, collection, and tag writes. That path is useful later, but it is
not the default for this feature because the desired behavior is local-first.

Zotero Desktop exposes a local connector HTTP server on `127.0.0.1:23119` with
default endpoints such as `/connector/saveItems` and `/connector/ping`. The
official docs also describe registering custom connector endpoints from a Zotero
extension with chrome privileges. This design uses that extension path for
reliable local read and write operations.

Useful references:

- https://www.zotero.org/support/dev/web_api/v3/write_requests
- https://www.zotero.org/support/dev/client_coding/connector_http_server
- https://www.zotero.org/support/dev/translators/coding

## Recommended Approach

Build a local-first Zotero bridge with two cooperating pieces:

1. A Qiongli MCPB storage adapter inside `packages/qiongli-literature-mcpb`.
2. A lightweight Zotero Desktop companion extension that exposes Qiongli-specific
   local HTTP endpoints through Zotero's connector server.

The MCPB remains responsible for normalized records, enrichment, conflict policy,
export files, and user-facing MCP tools. The Zotero companion extension remains
responsible for local library access: searching existing items, creating or
updating items, creating collections, adding tags, and returning Zotero item keys.

The import-file fallback is part of the same contract. If the local companion is
not installed, Zotero is not running, or the companion refuses a write, Qiongli
generates `references.json`, `references.ris`, and `bibliography.bib` for manual
import instead of dropping the references.

## Architecture

### Qiongli MCPB Tools

Add four user-facing tools to the literature MCPB:

- `qiongli_zotero_status`
  - Probes `http://127.0.0.1:23119/connector/ping`.
  - Probes `http://127.0.0.1:23119/qiongli/ping`.
  - Reports whether the default connector and Qiongli companion are available.
  - Does not expose local paths or user library details unless Zotero returns
    them explicitly through the companion endpoint.

- `qiongli_zotero_search`
  - Searches local Zotero by DOI, title, citekey, creator, year, tag, and
    collection.
  - Uses the companion endpoint when present.
  - Returns normalized Qiongli records plus Zotero identifiers:
    `library_id`, `library_type`, `item_key`, `select_uri`, `collections`, and
    `tags`.

- `qiongli_zotero_upsert_references`
  - Accepts Qiongli normalized records or a `qiongli_literature_search` result
    payload.
  - Optionally enriches records before write by reusing provider lookups already
    available in the MCPB.
  - Deduplicates in this order: DOI, PMID, arXiv ID, exact normalized title plus
    year, then Zotero item key if supplied.
  - Writes records through the companion endpoint.
  - Returns per-record status: `created`, `updated`, `unchanged`, `skipped`,
    `conflict`, or `failed`.

- `qiongli_zotero_export_import_files`
  - Converts the same normalized records into CSL-JSON, RIS, and BibTeX.
  - Returns the files as MCP text artifacts when the client supports it, or writes
    them to a project artifact directory when a `project_root` is supplied.
  - This tool is usable even without Zotero installed.

The existing `qiongli_literature_search` response format should remain stable.
Zotero writes should be explicit; search should not silently mutate Zotero.

### Zotero Companion Extension

Create a small installable Zotero extension source tree, tentatively:

`packages/qiongli-zotero-companion/`

The extension registers local endpoints:

- `GET /qiongli/ping`
  - Returns companion version, supported endpoint versions, and basic readiness.

- `POST /qiongli/search`
  - Accepts a structured query object.
  - Searches local Zotero items using Zotero Desktop APIs.
  - Returns a compact JSON list of matching items.

- `POST /qiongli/upsertItems`
  - Accepts Qiongli's local Zotero item payload.
  - Creates or updates Zotero items inside the selected library.
  - Creates collections when requested and permitted.
  - Adds tags and an optional child note containing Qiongli provenance.
  - Returns item keys and per-record decisions.

- `GET /qiongli/collections`
  - Returns collection keys and paths for collection selection and dry runs.

The endpoint namespace is deliberately Qiongli-specific. It avoids overloading
Zotero's default connector behavior and makes capability detection simple.

## Data Model

The bridge uses a canonical `ReferenceRecord` compatible with current MCPB search
normalization:

- `title`
- `authors`
- `year`
- `doi`
- `url`
- `abstract`
- `venue`
- `document_type`
- `citation_count`
- `reference_count`
- `citations`
- `references`
- `provider`
- `source_id`

Zotero-specific output fields are added only after local search or write:

- `zotero.item_key`
- `zotero.library_id`
- `zotero.library_type`
- `zotero.select_uri`
- `zotero.collections`
- `zotero.tags`
- `zotero.write_status`

The upsert payload also carries:

- `target_library`: `user` by default, with group library support later.
- `collection_path`: optional, for example `Qiongli/[topic]/To Screen`.
- `tags`: generated from project, status, provider, and review phase.
- `note`: optional child note with Qiongli provenance and enrichment trace.
- `dry_run`: default `true` for first manual calls unless the user explicitly
  requests a write.

## Zotero Item Mapping

Map Qiongli records to Zotero item JSON before sending them to the companion:

- `journal-article`, `JournalArticle`, `article`, and PubMed journal types map
  to `journalArticle`.
- `proceedings-article`, `Conference`, and conference-like values map to
  `conferencePaper`.
- `book`, `book-chapter`, `preprint`, `report`, and `webpage` map to the closest
  Zotero item type.
- Unknown types default to `journalArticle` only when a venue or DOI exists;
  otherwise default to `document`.

Creator mapping should preserve raw author strings but split conservative
`Family, Given` names into Zotero creator fields. Ambiguous names remain as
single-field creators to avoid destructive parsing.

Field mapping:

- `title` -> `title`
- `authors` -> `creators`
- `year` -> `date`
- `doi` -> `DOI`
- `url` -> `url`
- `abstract` -> `abstractNote`
- `venue` -> `publicationTitle` or `conferenceName`
- provider/source metadata -> `extra` and optional child note

## Enrichment And Deduplication

Qiongli enriches before Zotero write. The source priority should match the
existing metadata registry policy:

1. OpenAlex for core title, authors, year, venue, and open access metadata.
2. Crossref for publisher, volume, issue, pages, and DOI landing URL.
3. PubMed for biomedical metadata where PMID or MeSH terms exist.
4. Semantic Scholar for citation counts and broad discovery metadata.
5. Local Zotero for existing user-curated fields, tags, collections, and notes.

Local Zotero values are not overwritten by default when they appear user-edited.
The first implementation uses a conservative update policy:

- Fill blank Zotero fields from enriched Qiongli metadata.
- Add missing identifiers, tags, and collection membership.
- Add or update a Qiongli provenance child note.
- Do not rewrite non-empty title, creators, date, publication title, or abstract
  unless the call sets `update_policy: "prefer_enriched"`.

Every upsert result includes a merge trace so the user can see which record was
created, updated, skipped, or left unchanged.

## Import File Fallback

Qiongli must always be able to produce importable artifacts:

- `references.json` as CSL-JSON for Zotero and citation processors.
- `references.ris` for Zotero, EndNote, and Mendeley.
- `bibliography.bib` for BibTeX and existing Qiongli workflows.
- `zotero-import-report.md` summarizing counts, skipped records, missing fields,
  and import instructions.

The fallback is used when:

- Zotero Desktop is not running.
- The connector server is unreachable.
- The Qiongli companion extension is not installed.
- The companion returns a capability mismatch.
- The user explicitly asks for files only.

## Configuration

Add local configuration fields to the shared provider config and MCPB user config:

- `zotero.local_enabled`: default `true`
- `zotero.connector_url`: default `http://127.0.0.1:23119`
- `zotero.default_collection_path`: optional
- `zotero.write_policy`: `dry_run`, `explicit`, or `allow`
- `zotero.update_policy`: `fill_blank`, `prefer_zotero`, or `prefer_enriched`

Do not require a Zotero API key for this local mode. A future Web API adapter may
reuse Zotero API keys, but it should be a separate provider mode.

## Security

The MCPB should only connect to loopback hosts by default. It must reject
non-loopback `connector_url` values unless a future explicit unsafe override is
added.

The companion endpoint must reject unsupported methods and content types. Mutating
requests should require JSON and should support `dry_run`. The first write in a
new session should be explicit from the MCP client; search results must not
automatically write to Zotero.

The companion should not expose full local file paths, attachment paths, or note
contents unless a tool explicitly asks for them in a later feature.

## Error Handling

Common failures should return actionable diagnostics:

- `zotero_not_running`: connector ping failed.
- `companion_missing`: default connector works but `/qiongli/ping` failed.
- `companion_version_unsupported`: endpoint version mismatch.
- `write_denied`: companion refused a mutating operation.
- `duplicate_conflict`: multiple local Zotero candidates match the same record.
- `invalid_item`: record cannot be mapped to a Zotero item.
- `partial_write`: some records succeeded and others failed.

When local write fails, the response should include an import-file fallback plan.

## Testing

MCPB tests:

- Tool declarations include the four Zotero tools.
- Status checks distinguish connector unavailable, connector only, and companion
  available states using mocked `fetch`.
- Upsert builds the expected companion payload from normalized search records.
- Upsert defaults to dry run unless write intent is explicit.
- Deduplication prefers DOI and handles title/year fallback.
- Import fallback produces valid CSL-JSON, RIS, and BibTeX strings.
- Non-loopback connector URLs are rejected.

Companion tests:

- Endpoint registration works in a Zotero test harness or mocked Zotero runtime.
- Search returns compact item matches by DOI and title.
- Upsert creates a new item when no match exists.
- Upsert fills blank fields without overwriting non-empty user fields.
- Collection path creation and tag assignment are idempotent.
- Dry run returns planned operations without changing the library.

Integration smoke:

- With Zotero absent, MCPB reports fallback-only status and exports files.
- With Zotero running but companion absent, MCPB reports install guidance and
  exports files.
- With companion installed, MCPB can dry-run and then write one enriched record,
  returning a Zotero select URI.

## Documentation

Update both English and Chinese advanced docs:

- Replace the current framing that presents Zotero mainly as an external search
  MCP override.
- Explain Zotero's role as a local reference database.
- Document companion installation, status check, dry run, write, and fallback
  import-file workflows.
- Keep Web API write mode documented as a future or optional cloud-sync path,
  not the default.

Update `reference-manager-bridge` to say it supports three modes:

1. Local Zotero sync through the Qiongli companion extension.
2. Import-file generation.
3. Future or optional Zotero Web API sync.

## Rollout

First implementation slice:

1. Add the MCPB Zotero tool declarations and local HTTP adapter.
2. Add conversion helpers for Zotero item JSON, CSL-JSON, RIS, and BibTeX.
3. Add mocked MCPB tests for status, dry-run upsert, dedupe, and export fallback.
4. Add the companion extension source with endpoint stubs and a minimal local
   item search/upsert implementation.
5. Update docs and reference-manager-bridge wording.

Second slice:

1. Package the Zotero companion extension for user installation.
2. Add a local install verification script.
3. Add Python orchestrator wrappers that call the same local bridge contract.
4. Add optional Web API sync mode for users who want Zotero cloud writes.

## Acceptance Criteria

- A user can run `qiongli_zotero_status` and see whether local Zotero, the
  companion extension, and fallback export are available.
- A user can search online with `qiongli_literature_search`, then call
  `qiongli_zotero_upsert_references` to dry-run and explicitly write selected
  records to local Zotero.
- Duplicate records are matched before writes and are not blindly duplicated.
- The bridge fills missing metadata but does not overwrite user-curated Zotero
  fields by default.
- If Zotero local write is unavailable, Qiongli still generates importable
  Zotero-compatible files.
- The feature is documented as storage/sync, not as a replacement search source.
